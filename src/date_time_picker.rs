//! Preliminary port of Forui's date-time picker: a caller-owned date and time
//! composed from the calendar grid and the time picker wheels.
//!
//! The calendar and time picker keep their own caller-owned state; this widget
//! forwards date bounds, accessible labels and the enabled flag, and reports a
//! combined validated [`DateTime`] value. Layout is a fixed vertical stack
//! (calendar above the time wheels).
//!
//! Upstream's interval options (`dayInterval`/`hourInterval`/`minuteInterval`),
//! "Today" default date labels, 12-hour periods, locale-dependent date formats
//! and animation remain open. See `docs/DATE_TIME_PICKER.md`.
//!
//! ```
//! # fn example(ui: &mut egui::Ui, state: &mut temivu::DateTimePickerState) {
//! use temivu::{DateTimePicker, Theme};
//!
//! let output = DateTimePicker::new(state, &Theme::light()).show(ui);
//! if let Some(selected) = output.selected {
//!     let _value = selected.formatted();
//! }
//! # }
//! ```

use crate::Theme;
use crate::calendar::{Calendar, CalendarLabels, CalendarResponse, CalendarState, Date, Weekday};
use crate::time_picker::{Time, TimePicker, TimePickerLabels, TimePickerResponse};
use crate::tokens::Sizes;
use egui::{Id, Response, Ui};

/// A validated date and time without a time zone.
///
/// Both fields are already validated by [`Date`] and [`Time`], so every
/// `DateTime` is representable.
///
/// ```
/// use temivu::{Date, DateTime, Time};
///
/// let date = Date::new(2024, 7, 15).expect("a valid date");
/// let time = Time::new(9, 30).expect("a valid time");
/// let value = DateTime::new(date, time);
/// assert_eq!(value.formatted(), "Monday, July 15, 2024, 09:30");
/// ```
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct DateTime {
    date: Date,
    time: Time,
}

impl DateTime {
    /// Combine a validated date and time.
    pub const fn new(date: Date, time: Time) -> Self {
        Self { date, time }
    }

    /// The date part.
    pub const fn date(self) -> Date {
        self.date
    }

    /// The time part.
    pub const fn time(self) -> Time {
        self.time
    }

    /// Replace the date part.
    pub const fn with_date(self, date: Date) -> Self {
        Self {
            date,
            time: self.time,
        }
    }

    /// Replace the time part.
    pub const fn with_time(self, time: Time) -> Self {
        Self {
            date: self.date,
            time,
        }
    }

    /// Format as `"Monday, July 15, 2024, 09:30"` using English names.
    ///
    /// Localized formatting is not implemented; callers format values
    /// themselves when a locale is required.
    pub fn formatted(self) -> String {
        format!(
            "{}, {} {}, {}, {}",
            self.date.weekday().name(),
            self.date.month_name(),
            self.date.day(),
            self.date.year(),
            self.time.formatted()
        )
    }
}

/// Caller-owned state for a [`DateTimePicker`]: the calendar state and the time.
///
/// The selected date lives in [`CalendarState`] and the time in this struct, so
/// a single selection produces one [`DateTime`].
#[derive(Clone, Debug)]
pub struct DateTimePickerState {
    calendar: CalendarState,
    time: Time,
}

impl DateTimePickerState {
    /// Create state showing `visible_month` with an optional initial selection.
    ///
    /// A `selected` value also sets the time; without one the time starts at
    /// [`Time::MIDNIGHT`].
    pub fn new(visible_month: Date, selected: Option<DateTime>) -> Self {
        Self {
            calendar: CalendarState::new(visible_month, selected.map(DateTime::date)),
            time: selected.map_or(Time::MIDNIGHT, DateTime::time),
        }
    }

    /// Create state showing the selected date's month.
    pub fn from_selected(selected: DateTime) -> Self {
        Self::new(selected.date(), Some(selected))
    }

    /// The calendar's caller-owned state.
    pub fn calendar(&self) -> &CalendarState {
        &self.calendar
    }

    /// The calendar's caller-owned state, for external navigation.
    pub fn calendar_mut(&mut self) -> &mut CalendarState {
        &mut self.calendar
    }

    /// The caller-owned time.
    pub fn time(&self) -> Time {
        self.time
    }

    /// Replace the time; returns whether it changed.
    pub fn set_time(&mut self, time: Time) -> bool {
        if self.time == time {
            return false;
        }
        self.time = time;
        true
    }

    /// The currently selected date and time, if a date is selected.
    pub fn selected(&self) -> Option<DateTime> {
        self.calendar
            .selected()
            .map(|date| DateTime::new(date, self.time))
    }

    /// Replace the selection and its time; returns whether anything changed.
    ///
    /// Selecting `None` clears the date but keeps the time.
    pub fn set_selected(&mut self, selected: Option<DateTime>) -> bool {
        let mut changed = self.calendar.set_selected(selected.map(DateTime::date));
        if let Some(selected) = selected {
            changed |= self.set_time(selected.time());
        }
        changed
    }

    /// The visible calendar month, normalized to its first day.
    pub fn visible_month(&self) -> Date {
        self.calendar.visible_month()
    }
}

/// Accessible names for a [`DateTimePicker`] and its two child pickers.
#[derive(Clone, Copy)]
pub struct DateTimePickerLabels<'a> {
    /// The group's accessible name.
    pub group: &'a str,
    /// The group's accessible value while no date is selected.
    pub empty_value: &'a str,
    /// The calendar's accessible and visible labels.
    pub calendar: CalendarLabels<'a>,
    /// The time picker's accessible and visible labels.
    pub time: TimePickerLabels<'a>,
    /// Builds the group's accessible value for a selected date and time.
    pub value: &'a dyn Fn(DateTime) -> String,
}

impl DateTimePickerLabels<'static> {
    /// The English defaults used when no labels are supplied.
    pub fn english() -> Self {
        Self {
            group: "Date and time",
            empty_value: "No date selected",
            calendar: CalendarLabels::english(),
            time: TimePickerLabels::english(),
            value: &english_value,
        }
    }
}

fn english_value(value: DateTime) -> String {
    value.formatted()
}

/// What [`DateTimePicker::show`] reports for the current frame.
pub struct DateTimePickerResponse {
    /// The group's response; it does not hold keyboard focus itself.
    pub response: Response,
    /// The combined selection after this frame.
    pub selected: Option<DateTime>,
    /// Whether the date, the time or a child widget changed this frame.
    pub changed: bool,
    /// The calendar's own response.
    pub calendar: CalendarResponse,
    /// The time picker's own response.
    pub time: TimePickerResponse,
}

/// A calendar and time picker bound to caller-owned [`DateTimePickerState`].
///
/// Both child widgets keep their own keyboard focus and stable IDs; the
/// calendar uses `id.with("calendar")` and the time wheels `id.with("time")`, so
/// focus survives layout changes. The component itself does not steal focus.
pub struct DateTimePicker<'a> {
    state: &'a mut DateTimePickerState,
    theme: &'a Theme,
    labels: DateTimePickerLabels<'a>,
    bounds: Option<(Date, Date)>,
    today: Option<Date>,
    first_day_of_week: Weekday,
    selectable: Option<&'a dyn Fn(Date) -> bool>,
    sizes: Sizes,
    seconds: bool,
    enabled: bool,
    id: Option<Id>,
}

impl<'a> DateTimePicker<'a> {
    /// Create a date-time picker for the caller-owned state and theme.
    pub fn new(state: &'a mut DateTimePickerState, theme: &'a Theme) -> Self {
        Self {
            state,
            theme,
            labels: DateTimePickerLabels::english(),
            bounds: None,
            today: None,
            first_day_of_week: Weekday::Monday,
            selectable: None,
            sizes: Sizes::desktop(),
            seconds: false,
            enabled: true,
            id: None,
        }
    }

    /// Replace the English labels.
    pub fn labels(mut self, labels: DateTimePickerLabels<'a>) -> Self {
        self.labels = labels;
        self
    }

    /// Set inclusive selectable date bounds, forwarded to the calendar.
    ///
    /// The argument order does not matter: the earlier date is the minimum.
    pub fn bounds(mut self, min: Date, max: Date) -> Self {
        self.bounds = Some((min.min(max), max.max(min)));
        self
    }

    /// Set the caller-supplied "today" used by the calendar.
    ///
    /// Library code never reads the wall clock.
    pub fn today(mut self, today: Date) -> Self {
        self.today = Some(today);
        self
    }

    /// Set the calendar's first day of the week. Defaults to Monday.
    pub fn first_day_of_week(mut self, weekday: Weekday) -> Self {
        self.first_day_of_week = weekday;
        self
    }

    /// Disable individual dates through a caller predicate, forwarded to the
    /// calendar.
    pub fn selectable(mut self, selectable: &'a dyn Fn(Date) -> bool) -> Self {
        self.selectable = Some(selectable);
        self
    }

    /// Show a seconds wheel. Defaults to `false`.
    pub fn seconds(mut self, seconds: bool) -> Self {
        self.seconds = seconds;
        self
    }

    /// The responsive size tokens forwarded to both child pickers.
    pub fn sizes(mut self, sizes: Sizes) -> Self {
        self.sizes = sizes;
        self
    }

    /// Disable the whole date-time picker.
    pub fn enabled(mut self, enabled: bool) -> Self {
        self.enabled = enabled;
        self
    }

    /// Give the component a stable, globally unique ID across layout changes.
    pub fn id(mut self, id: Id) -> Self {
        self.id = Some(id);
        self
    }

    /// Draw the calendar and the time wheels.
    pub fn show(self, ui: &mut Ui) -> DateTimePickerResponse {
        let id = self
            .id
            .unwrap_or_else(|| ui.next_auto_id().with("temivu-date-time-picker"));
        ui.scope_builder(egui::UiBuilder::new().scope_id(id), |ui| {
            self.show_inner(ui, id)
        })
        .inner
    }

    fn show_inner(self, ui: &mut Ui, id: Id) -> DateTimePickerResponse {
        let Self {
            state,
            theme,
            labels,
            bounds,
            today,
            first_day_of_week,
            selectable,
            sizes,
            seconds,
            enabled,
            ..
        } = self;
        let before = state.selected();
        let inner = ui.vertical(|ui| {
            let mut calendar = Calendar::new(&mut state.calendar, theme)
                .id(id.with("calendar"))
                .labels(labels.calendar)
                .first_day_of_week(first_day_of_week)
                .sizes(sizes)
                .enabled(enabled);
            if let Some((min, max)) = bounds {
                calendar = calendar.bounds(min, max);
            }
            if let Some(today) = today {
                calendar = calendar.today(today);
            }
            if let Some(selectable) = selectable {
                calendar = calendar.selectable(selectable);
            }
            let calendar_response = calendar.show(ui);
            let time_response = TimePicker::new(&mut state.time, theme)
                .id(id.with("time"))
                .labels(labels.time)
                .sizes(sizes)
                .seconds(seconds)
                .enabled(enabled)
                .show(ui);
            (calendar_response, time_response)
        });
        let (calendar, time) = inner.inner;
        let mut response = inner.response;
        let selected = state.selected();
        let changed =
            selected != before || calendar.changed || time.changed || time.response.changed();
        let value = selected.map_or_else(
            || labels.empty_value.to_owned(),
            |selected| (labels.value)(selected),
        );
        ui.ctx().accesskit_node_builder(response.id, |node| {
            node.set_role(egui::accesskit::Role::Group);
            node.set_label(labels.group);
            node.set_value(value);
            node.set_bounds(accesskit_rect(response.rect));
            if enabled {
                node.clear_disabled();
            } else {
                node.set_disabled();
            }
        });
        if changed {
            response.mark_changed();
        }
        DateTimePickerResponse {
            response,
            selected,
            changed,
            calendar,
            time,
        }
    }
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
    fn date_time_combines_and_formats_validated_parts() {
        let date = Date::new(2024, 7, 15).expect("valid date");
        let time = Time::new(9, 30).expect("valid time");
        let value = DateTime::new(date, time);
        assert_eq!(value.date(), date);
        assert_eq!(value.time(), time);
        assert_eq!(value.formatted(), "Monday, July 15, 2024, 09:30");
        assert_eq!(
            value
                .with_time(Time::new(10, 0).expect("valid time"))
                .time(),
            Time::new(10, 0).expect("valid time")
        );
        assert_eq!(
            DateTime::new(Date::new(2024, 7, 16).expect("valid date"), time),
            value.with_date(Date::new(2024, 7, 16).expect("valid date"))
        );
    }

    #[test]
    fn state_tracks_calendar_and_time_together() {
        let date = Date::new(2024, 7, 15).expect("valid date");
        let time = Time::new(9, 30).expect("valid time");
        let mut state = DateTimePickerState::new(date, Some(DateTime::new(date, time)));
        assert_eq!(state.selected(), Some(DateTime::new(date, time)));
        assert_eq!(state.visible_month(), date.first_of_month());
        assert!(state.set_time(Time::new(10, 0).expect("valid time")));
        assert_eq!(
            state.selected(),
            Some(DateTime::new(date, Time::new(10, 0).expect("valid time")))
        );
        assert!(!state.set_time(Time::new(10, 0).expect("valid time")));
        assert!(state.set_selected(None));
        assert_eq!(state.selected(), None);
        assert_eq!(
            state.time(),
            Time::new(10, 0).expect("valid time"),
            "clearing the date keeps the time"
        );
    }
}
