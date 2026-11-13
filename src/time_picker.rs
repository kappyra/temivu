//! Preliminary port of Forui's time picker: hour/minute(/second) wheels built
//! on the picker module with a caller-owned, validated [`Time`].
//!
//! 12-hour periods, locale-dependent time formats, typed digit entry, per-wheel
//! intervals and animation remain open. See `docs/TIME_PICKER.md`.
//!
//! ```
//! # fn example(ui: &mut egui::Ui, time: &mut temivu::Time) {
//! use temivu::{Theme, TimePicker};
//!
//! let output = TimePicker::new(time, &Theme::light()).show(ui);
//! if output.changed {
//!     let _current = time.formatted();
//! }
//! # }
//! ```

use crate::Theme;
use crate::picker::{Picker, PickerItem};
use crate::tokens::Sizes;
use egui::{Id, Response, Ui};
use std::sync::OnceLock;

const HOUR_LABELS: [&str; 24] = [
    "00", "01", "02", "03", "04", "05", "06", "07", "08", "09", "10", "11", "12", "13", "14", "15",
    "16", "17", "18", "19", "20", "21", "22", "23",
];

const MINUTE_LABELS: [&str; 60] = [
    "00", "01", "02", "03", "04", "05", "06", "07", "08", "09", "10", "11", "12", "13", "14", "15",
    "16", "17", "18", "19", "20", "21", "22", "23", "24", "25", "26", "27", "28", "29", "30", "31",
    "32", "33", "34", "35", "36", "37", "38", "39", "40", "41", "42", "43", "44", "45", "46", "47",
    "48", "49", "50", "51", "52", "53", "54", "55", "56", "57", "58", "59",
];

const SECOND_LABELS: [&str; 60] = MINUTE_LABELS;

/// Why a time value could not be constructed.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum TimeError {
    /// The hour is outside `0..=23`.
    InvalidHour(u8),
    /// The minute is outside `0..=59`.
    InvalidMinute(u8),
    /// The second is outside `0..=59`.
    InvalidSecond(u8),
}

impl std::fmt::Display for TimeError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::InvalidHour(hour) => write!(f, "hour {hour} is outside 0..=23"),
            Self::InvalidMinute(minute) => write!(f, "minute {minute} is outside 0..=59"),
            Self::InvalidSecond(second) => write!(f, "second {second} is outside 0..=59"),
        }
    }
}

impl std::error::Error for TimeError {}

/// Why a time string could not be parsed.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum TimeParseError {
    /// The text was empty or only whitespace.
    Empty,
    /// The text had an hours part but no minutes part.
    MissingMinutes,
    /// The text had more than three colon-separated parts.
    TooManyParts,
    /// A part was not one or two ASCII digits.
    Malformed,
    /// The hours part was outside `0..=23`.
    InvalidHour,
    /// The minutes part was outside `0..=59`.
    InvalidMinute,
    /// The seconds part was outside `0..=59`.
    InvalidSecond,
}

impl std::fmt::Display for TimeParseError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Empty => write!(f, "the time is empty"),
            Self::MissingMinutes => write!(f, "minutes are required"),
            Self::TooManyParts => write!(f, "too many time parts"),
            Self::Malformed => write!(f, "hours, minutes and seconds must be 1-2 digits"),
            Self::InvalidHour => write!(f, "hours must be between 0 and 23"),
            Self::InvalidMinute => write!(f, "minutes must be between 0 and 59"),
            Self::InvalidSecond => write!(f, "seconds must be between 0 and 59"),
        }
    }
}

impl std::error::Error for TimeParseError {}

/// A validated wall-clock time without a date or time zone.
///
/// Construction rejects out-of-range hours, minutes and seconds, so a `Time`
/// value is always representable.
///
/// ```
/// use temivu::Time;
///
/// let time = Time::new(9, 30).expect("a valid time");
/// assert_eq!(time.formatted(), "09:30");
/// assert!(Time::new(24, 0).is_err());
/// assert!(Time::new(0, 60).is_err());
/// ```
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Time {
    hour: u8,
    minute: u8,
    second: u8,
}

impl Time {
    /// Midnight, `00:00:00`.
    pub const MIDNIGHT: Self = Self {
        hour: 0,
        minute: 0,
        second: 0,
    };

    /// Construct a time from an hour and a minute; the second is zero.
    pub const fn new(hour: u8, minute: u8) -> Result<Self, TimeError> {
        if hour > 23 {
            return Err(TimeError::InvalidHour(hour));
        }
        if minute > 59 {
            return Err(TimeError::InvalidMinute(minute));
        }
        Ok(Self {
            hour,
            minute,
            second: 0,
        })
    }

    /// Construct a time including seconds.
    pub const fn with_seconds(hour: u8, minute: u8, second: u8) -> Result<Self, TimeError> {
        if hour > 23 {
            return Err(TimeError::InvalidHour(hour));
        }
        if minute > 59 {
            return Err(TimeError::InvalidMinute(minute));
        }
        if second > 59 {
            return Err(TimeError::InvalidSecond(second));
        }
        Ok(Self {
            hour,
            minute,
            second,
        })
    }

    /// The hour, in `0..=23`.
    pub const fn hour(self) -> u8 {
        self.hour
    }

    /// The minute, in `0..=59`.
    pub const fn minute(self) -> u8 {
        self.minute
    }

    /// The second, in `0..=59`.
    pub const fn second(self) -> u8 {
        self.second
    }

    /// Return a copy with a different hour, rejecting an out-of-range value.
    pub fn with_hour(self, hour: u8) -> Result<Self, TimeError> {
        Self::with_seconds(hour, self.minute, self.second)
    }

    /// Return a copy with a different minute, rejecting an out-of-range value.
    pub fn with_minute(self, minute: u8) -> Result<Self, TimeError> {
        Self::with_seconds(self.hour, minute, self.second)
    }

    /// Return a copy with a different second, rejecting an out-of-range value.
    pub fn with_second(self, second: u8) -> Result<Self, TimeError> {
        Self::with_seconds(self.hour, self.minute, second)
    }

    /// Seconds since midnight.
    pub const fn seconds_since_midnight(self) -> u32 {
        self.hour as u32 * 3600 + self.minute as u32 * 60 + self.second as u32
    }

    /// Format as `HH:MM`, or `HH:MM:SS` when the second is non-zero.
    pub fn formatted(self) -> String {
        format_time(self, false)
    }

    /// Parse `H:MM`, `HH:MM`, `H:MM:SS` or `HH:MM:SS`, ignoring surrounding
    /// whitespace.
    pub fn parse(text: &str) -> Result<Self, TimeParseError> {
        let text = text.trim();
        if text.is_empty() {
            return Err(TimeParseError::Empty);
        }
        let mut parts = text.split(':');
        let hour = parse_time_component(parts.next().unwrap_or_default())?;
        let minute = parse_time_component(parts.next().ok_or(TimeParseError::MissingMinutes)?)?;
        let second = match parts.next() {
            Some(part) => parse_time_component(part)?,
            None => 0,
        };
        if parts.next().is_some() {
            return Err(TimeParseError::TooManyParts);
        }
        Self::with_seconds(hour, minute, second).map_err(|error| match error {
            TimeError::InvalidHour(_) => TimeParseError::InvalidHour,
            TimeError::InvalidMinute(_) => TimeParseError::InvalidMinute,
            TimeError::InvalidSecond(_) => TimeParseError::InvalidSecond,
        })
    }

    /// Format as `HH:MM`.
    pub fn format_hm(self) -> String {
        format!("{:02}:{:02}", self.hour, self.minute)
    }

    /// Format as `HH:MM:SS`.
    pub fn format_hms(self) -> String {
        format!("{:02}:{:02}:{:02}", self.hour, self.minute, self.second)
    }
}

impl Default for Time {
    fn default() -> Self {
        Self::MIDNIGHT
    }
}

impl std::fmt::Display for Time {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(&format_time(*self, false))
    }
}

fn parse_time_component(part: &str) -> Result<u8, TimeParseError> {
    if part.is_empty() || part.len() > 2 || !part.bytes().all(|byte| byte.is_ascii_digit()) {
        return Err(TimeParseError::Malformed);
    }
    part.parse::<u8>().map_err(|_| TimeParseError::Malformed)
}

fn format_time(time: Time, with_seconds: bool) -> String {
    if with_seconds || time.second != 0 {
        format!("{:02}:{:02}:{:02}", time.hour, time.minute, time.second)
    } else {
        format!("{:02}:{:02}", time.hour, time.minute)
    }
}

fn item_id(unit: &'static str, index: usize) -> Id {
    Id::unique(("temivu-time-picker-item", unit, index))
}

fn hours() -> &'static [PickerItem<'static>] {
    static ITEMS: OnceLock<Vec<PickerItem<'static>>> = OnceLock::new();
    ITEMS.get_or_init(|| {
        (0..HOUR_LABELS.len())
            .map(|index| PickerItem::new(item_id("hour", index), HOUR_LABELS[index]))
            .collect()
    })
}

fn minutes() -> &'static [PickerItem<'static>] {
    static ITEMS: OnceLock<Vec<PickerItem<'static>>> = OnceLock::new();
    ITEMS.get_or_init(|| {
        (0..MINUTE_LABELS.len())
            .map(|index| PickerItem::new(item_id("minute", index), MINUTE_LABELS[index]))
            .collect()
    })
}

fn seconds() -> &'static [PickerItem<'static>] {
    static ITEMS: OnceLock<Vec<PickerItem<'static>>> = OnceLock::new();
    ITEMS.get_or_init(|| {
        (0..SECOND_LABELS.len())
            .map(|index| PickerItem::new(item_id("second", index), SECOND_LABELS[index]))
            .collect()
    })
}

/// Accessible names for a [`TimePicker`]'s wheels and group.
#[derive(Clone, Copy, Debug)]
pub struct TimePickerLabels<'a> {
    /// The group's accessible name.
    pub group: &'a str,
    /// The hour wheel's accessible name.
    pub hour: &'a str,
    /// The minute wheel's accessible name.
    pub minute: &'a str,
    /// The second wheel's accessible name.
    pub second: &'a str,
}

impl TimePickerLabels<'static> {
    /// The English defaults used when no labels are supplied.
    pub const fn english() -> Self {
        Self {
            group: "Time",
            hour: "Hour",
            minute: "Minute",
            second: "Second",
        }
    }
}

/// What [`TimePicker::show`] reports for the current frame.
pub struct TimePickerResponse {
    /// The group's response; it does not hold keyboard focus itself.
    pub response: Response,
    /// The hour wheel's response.
    pub hour: Response,
    /// The minute wheel's response.
    pub minute: Response,
    /// The second wheel's response, when seconds are shown.
    pub second: Option<Response>,
    /// Whether the caller-owned time changed this frame.
    pub changed: bool,
}

/// Presentation used to edit the same caller-owned time.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum TimePickerLayout {
    /// Scrollable wheels with keyboard navigation.
    #[default]
    Wheels,
    /// Numeric fields with separate touch-friendly decrement and increment controls.
    Stepper,
}

/// An hour/minute(/second) wheel picker bound to caller-owned [`Time`] state.
///
/// ```
/// # use temivu::Time;
/// # fn example(ui: &mut egui::Ui, time: &mut Time) {
/// use temivu::{Theme, TimePicker};
/// let output = TimePicker::new(time, &Theme::light())
///     .seconds(true)
///     .show(ui);
/// if output.changed {
///     let _value = time.formatted();
/// }
/// # }
/// ```
pub struct TimePicker<'a> {
    time: &'a mut Time,
    theme: &'a Theme,
    labels: TimePickerLabels<'a>,
    seconds: bool,
    sizes: Sizes,
    enabled: bool,
    id: Option<Id>,
    layout: TimePickerLayout,
}

impl<'a> TimePicker<'a> {
    /// Create a time picker for the caller-owned `time`.
    pub fn new(time: &'a mut Time, theme: &'a Theme) -> Self {
        Self {
            time,
            theme,
            labels: TimePickerLabels::english(),
            seconds: false,
            sizes: Sizes::desktop(),
            enabled: true,
            id: None,
            layout: TimePickerLayout::Wheels,
        }
    }

    /// Show a third wheel for seconds. Defaults to `false`.
    pub fn seconds(mut self, seconds: bool) -> Self {
        self.seconds = seconds;
        self
    }

    /// Replace the English accessible names.
    pub fn labels(mut self, labels: TimePickerLabels<'a>) -> Self {
        self.labels = labels;
        self
    }

    /// The responsive size tokens used for the wheel rows.
    pub fn sizes(mut self, sizes: Sizes) -> Self {
        self.sizes = sizes;
        self
    }

    /// Disable the whole time picker.
    pub fn enabled(mut self, enabled: bool) -> Self {
        self.enabled = enabled;
        self
    }

    /// Give the time picker a stable, globally unique ID across layout changes.
    pub fn id(mut self, id: Id) -> Self {
        self.id = Some(id);
        self
    }

    /// Choose the wheel or stepper presentation.
    pub fn layout(mut self, layout: TimePickerLayout) -> Self {
        self.layout = layout;
        self
    }

    /// Draw the time controls and return their responses.
    pub fn show(self, ui: &mut Ui) -> TimePickerResponse {
        let Self {
            time,
            theme,
            labels,
            seconds: show_seconds,
            sizes,
            enabled,
            id,
            layout,
        } = self;
        let id = id.unwrap_or_else(|| ui.next_auto_id().with("temivu-time-picker"));
        ui.scope_builder(egui::UiBuilder::new().scope_id(id), |ui| {
            if layout == TimePickerLayout::Stepper {
                let before = *time;
                let mut values = [time.hour(), time.minute(), time.second()];
                let units = [labels.hour, labels.minute, labels.second];
                let count = if show_seconds { 3 } else { 2 };
                let inner = ui.scope(|ui| {
                    let mut draw = |ui: &mut Ui, index: usize| {
                        stepper(
                            ui,
                            theme,
                            units[index],
                            &mut values[index],
                            if index == 0 { 23 } else { 59 },
                            enabled,
                        )
                    };
                    if ui.available_width() < count as f32 * 70.0 {
                        let hour = draw(ui, 0);
                        let minute = draw(ui, 1);
                        let second = show_seconds.then(|| draw(ui, 2));
                        (hour, minute, second)
                    } else {
                        ui.columns(count, |columns| {
                            let hour = draw(&mut columns[0], 0);
                            let minute = draw(&mut columns[1], 1);
                            let second = show_seconds.then(|| draw(&mut columns[2], 2));
                            (hour, minute, second)
                        })
                    }
                });
                *time = Time {
                    hour: values[0],
                    minute: values[1],
                    second: values[2],
                };
                return time_picker_response(
                    ui,
                    inner,
                    labels,
                    *time,
                    show_seconds,
                    enabled,
                    *time != before,
                );
            }
            // A picker's focusable wheel response uses its own ID with the
            // "wheel" salt; the IDs below are the picker IDs.
            let hour_id = id.with("hour");
            let minute_id = id.with("minute");
            let second_id = id.with("second");
            let hour_focus = hour_id.with("wheel");
            let minute_focus = minute_id.with("wheel");
            let second_focus = second_id.with("wheel");
            let hour_focused = ui.memory(|memory| memory.has_focus(hour_focus));
            let minute_focused = ui.memory(|memory| memory.has_focus(minute_focus));
            let second_focused = show_seconds && ui.memory(|memory| memory.has_focus(second_focus));

            // Left/Right move focus between wheels, matching upstream. The
            // wheels lock the arrow keys, so the move is handled here and the
            // keys are consumed before the pickers read their own input.
            let mut focus_target = None;
            if enabled && (hour_focused || minute_focused || second_focused) {
                let (left, right) = ui.input(|input| {
                    (
                        input.key_pressed(egui::Key::ArrowLeft),
                        input.key_pressed(egui::Key::ArrowRight),
                    )
                });
                if left || right {
                    ui.input_mut(|input| {
                        input.consume_key(egui::Modifiers::NONE, egui::Key::ArrowLeft);
                        input.consume_key(egui::Modifiers::NONE, egui::Key::ArrowRight);
                    });
                    focus_target = if right {
                        match (hour_focused, minute_focused) {
                            (true, _) => Some(minute_focus),
                            (_, true) if show_seconds => Some(second_focus),
                            _ => None,
                        }
                    } else {
                        match (hour_focused, minute_focused, second_focused) {
                            (true, _, _) => None,
                            (_, true, _) => Some(hour_focus),
                            (_, _, true) => Some(minute_focus),
                            _ => None,
                        }
                    };
                }
            }

            let wheel_count = if show_seconds { 3 } else { 2 };
            let gaps = (wheel_count - 1) as f32 * 8.0;
            let wheel_width =
                ((ui.available_width() - gaps) / wheel_count as f32).clamp(28.0, 88.0);

            let mut hour_selection = Some(item_id("hour", time.hour() as usize));
            let mut minute_selection = Some(item_id("minute", time.minute() as usize));
            let mut second_selection = Some(item_id("second", time.second() as usize));
            let before = *time;
            let inner = ui.horizontal(|ui| {
                let hour = Picker::new(&mut hour_selection, hours(), theme)
                    .label(labels.hour)
                    .id(hour_id)
                    .sizes(sizes)
                    .width(wheel_width)
                    .enabled(enabled)
                    .show(ui);
                separator(ui);
                let minute = Picker::new(&mut minute_selection, minutes(), theme)
                    .label(labels.minute)
                    .id(minute_id)
                    .sizes(sizes)
                    .width(wheel_width)
                    .enabled(enabled)
                    .show(ui);
                let second = show_seconds.then(|| {
                    separator(ui);
                    Picker::new(&mut second_selection, seconds(), theme)
                        .label(labels.second)
                        .id(second_id)
                        .sizes(sizes)
                        .width(wheel_width)
                        .enabled(enabled)
                        .show(ui)
                });
                (
                    hour.response,
                    minute.response,
                    second.map(|second| second.response),
                )
            });
            let mut changed = false;

            if let Some(hour) = index_of(hours(), &hour_selection)
                && hour != time.hour() as usize
                && let Ok(updated) = time.with_hour(hour as u8)
            {
                *time = updated;
                changed = true;
            }
            if let Some(minute) = index_of(minutes(), &minute_selection)
                && minute != time.minute() as usize
                && let Ok(updated) = time.with_minute(minute as u8)
            {
                *time = updated;
                changed = true;
            }
            if show_seconds
                && let Some(second) = index_of(seconds(), &second_selection)
                && second != time.second() as usize
                && let Ok(updated) = time.with_second(second as u8)
            {
                *time = updated;
                changed = true;
            }
            if *time != before {
                changed = true;
            }
            if let Some(target) = focus_target {
                ui.memory_mut(|memory| memory.request_focus(target));
                ui.ctx().request_repaint();
            }
            time_picker_response(ui, inner, labels, *time, show_seconds, enabled, changed)
        })
        .inner
    }
}

fn time_picker_response(
    ui: &Ui,
    inner: egui::InnerResponse<(Response, Response, Option<Response>)>,
    labels: TimePickerLabels<'_>,
    time: Time,
    show_seconds: bool,
    enabled: bool,
    changed: bool,
) -> TimePickerResponse {
    let mut response = inner.response;
    let (hour, minute, second) = inner.inner;
    ui.ctx().accesskit_node_builder(response.id, |node| {
        node.set_role(egui::accesskit::Role::Group);
        node.set_label(labels.group);
        node.set_value(format_time(time, show_seconds));
        node.set_bounds(accesskit_rect(response.rect));
        if !enabled {
            node.set_disabled();
        }
    });
    if changed {
        response.mark_changed();
    }
    TimePickerResponse {
        response,
        hour,
        minute,
        second,
        changed,
    }
}

fn stepper(
    ui: &mut Ui,
    theme: &Theme,
    unit: &str,
    value: &mut u8,
    max: u8,
    enabled: bool,
) -> Response {
    ui.vertical_centered(|ui| {
        ui.label(
            egui::RichText::new(unit)
                .size(theme.typography.caption.size)
                .color(theme.muted_foreground),
        );
        let input = if theme.touch {
            crate::InputMode::Touch
        } else {
            crate::InputMode::Desktop
        };
        let change = |ui: &mut Ui, symbol: &str, can_change| {
            let response = crate::Button::new(symbol, theme)
                .variant(crate::ButtonVariant::Secondary)
                .size(crate::ButtonSize::Small)
                .input_mode(input)
                .enabled(enabled && can_change)
                .show(ui);
            ui.ctx().accesskit_node_builder(response.id, |node| {
                node.set_label(format!("{unit} {symbol}"));
            });
            response.clicked()
        };
        if change(ui, "+", *value < max) {
            *value += 1;
        }
        let response = ui
            .scope(|ui| {
                ui.spacing_mut().interact_size =
                    egui::vec2(48.0, if theme.touch { 44.0 } else { 36.0 });
                ui.add_enabled(
                    enabled,
                    egui::DragValue::new(value)
                        .range(0..=max)
                        .custom_formatter(|n, _| format!("{n:02.0}")),
                )
            })
            .inner;
        ui.ctx()
            .accesskit_node_builder(response.id, |node| node.set_label(unit));
        if change(ui, "−", *value > 0) {
            *value -= 1;
        }
        response
    })
    .inner
}

fn separator(ui: &mut Ui) {
    let response = ui.label(":");
    ui.ctx()
        .accesskit_node_builder(response.id, |node| node.set_hidden());
}

fn index_of(items: &[PickerItem<'_>], selected: &Option<Id>) -> Option<usize> {
    selected.and_then(|selected| items.iter().position(|item| item.id == selected))
}

fn accesskit_rect(rect: egui::Rect) -> egui::accesskit::Rect {
    egui::accesskit::Rect {
        x0: f64::from(rect.min.x),
        y0: f64::from(rect.min.y),
        x1: f64::from(rect.max.x),
        y1: f64::from(rect.max.y),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn time_validation_rejects_out_of_range_fields() {
        assert_eq!(Time::new(24, 0), Err(TimeError::InvalidHour(24)));
        assert_eq!(Time::new(0, 60), Err(TimeError::InvalidMinute(60)));
        assert_eq!(
            Time::with_seconds(0, 0, 60),
            Err(TimeError::InvalidSecond(60))
        );
        let time = Time::new(23, 59).expect("valid");
        assert_eq!(time.with_hour(24), Err(TimeError::InvalidHour(24)));
        assert_eq!(time.with_minute(60), Err(TimeError::InvalidMinute(60)));
    }

    #[test]
    fn time_formatting_and_ordering() {
        assert_eq!(Time::new(9, 30).expect("valid").formatted(), "09:30");
        assert_eq!(
            Time::with_seconds(9, 30, 5).expect("valid").formatted(),
            "09:30:05"
        );
        assert_eq!(
            Time::with_seconds(0, 0, 5)
                .expect("valid")
                .seconds_since_midnight(),
            5
        );
        assert!(Time::new(9, 30).expect("valid") < Time::new(10, 0).expect("valid"));
    }
}
