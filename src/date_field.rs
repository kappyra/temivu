//! Preliminary port of Forui's date field: a validated date text input with a
//! calendar button that opens the crate's [`crate::Calendar`] in a popover.
//!
//! The field owns validated date text in caller-owned [`DateFieldState`]. Text
//! is parsed every frame with a configurable parser (the default accepts ISO
//! `YYYY-MM-DD`), bounds are enforced, and selecting a day in the popover
//! writes the formatted text back. There is no wall-clock access; `today` is
//! caller-supplied. See `docs/DATE_FIELD.md`.
//!
//! ```
//! # fn example(ui: &mut egui::Ui) {
//! use temivu::{DateField, DateFieldState, Theme};
//!
//! let theme = Theme::light();
//! let mut state = DateFieldState::new();
//! state.set_text("2024-07-15");
//! let output = DateField::new(&mut state, &theme, "Due date").show(ui);
//! assert_eq!(
//!     output.value.map(|date| date.to_string()).as_deref(),
//!     Some("2024-07-15")
//! );
//! # }
//! ```

use crate::Theme;
use crate::calendar::{Calendar, CalendarState, Date};
use crate::label::{FieldLabel, LabelLayout};
use crate::localization::Messages;
use crate::popover::Popover;
use egui::{Id, Key, Response, Ui, Vec2};

/// A date parser: maps input text to a date, or to a localized/parsed error.
pub type DateParser<'a> = dyn Fn(&str) -> Result<Date, DateParseError> + 'a;
/// A date formatter used to write a selected date back into the input text.
pub type DateFormatter<'a> = dyn Fn(Date) -> String + 'a;

/// Why a date string could not be parsed.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum DateParseError {
    /// The text was empty or only whitespace.
    Empty,
    /// The text did not match the expected shape.
    Format,
    /// The text named a calendar date that does not exist.
    InvalidDate,
    /// A custom parser supplied this message; shown verbatim.
    Custom(String),
}

impl std::fmt::Display for DateParseError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Empty => write!(f, "the date is empty"),
            Self::Format => write!(f, "the date must use the YYYY-MM-DD form"),
            Self::InvalidDate => write!(f, "the date does not exist"),
            Self::Custom(message) => write!(f, "{message}"),
        }
    }
}

impl std::error::Error for DateParseError {}

/// Parse the default ISO `YYYY-MM-DD` date form, ignoring surrounding
/// whitespace. Month and day may be one or two digits.
pub fn parse_iso_date(text: &str) -> Result<Date, DateParseError> {
    let text = text.trim();
    if text.is_empty() {
        return Err(DateParseError::Empty);
    }
    let mut parts = text.split('-');
    let year = parts.next().unwrap_or_default();
    let month = parts.next().ok_or(DateParseError::Format)?;
    let day = parts.next().ok_or(DateParseError::Format)?;
    if parts.next().is_some() {
        return Err(DateParseError::Format);
    }
    let year = parse_year(year)?;
    let month = parse_part(month, 2)?;
    let day = parse_part(day, 2)?;
    let month = u8::try_from(month).map_err(|_| DateParseError::InvalidDate)?;
    let day = u8::try_from(day).map_err(|_| DateParseError::InvalidDate)?;
    Date::new(year as i32, month, day).map_err(|_| DateParseError::InvalidDate)
}

/// Format a date as the default ISO `YYYY-MM-DD` form.
pub fn format_iso_date(date: Date) -> String {
    date.to_string()
}

fn parse_year(part: &str) -> Result<u16, DateParseError> {
    if part.len() != 4 || !part.bytes().all(|byte| byte.is_ascii_digit()) {
        return Err(DateParseError::Format);
    }
    part.parse::<u16>().map_err(|_| DateParseError::Format)
}

fn parse_part(part: &str, max_digits: usize) -> Result<u16, DateParseError> {
    if part.is_empty() || part.len() > max_digits || !part.bytes().all(|byte| byte.is_ascii_digit())
    {
        return Err(DateParseError::Format);
    }
    part.parse::<u16>().map_err(|_| DateParseError::Format)
}

/// Caller-owned date field state: the input text, its parsed value and whether
/// the calendar popover is open.
///
/// The text is the source of truth. The widget re-parses it every frame and
/// writes the result to [`Self::value`]; choosing a day in the popover writes
/// both fields.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct DateFieldState {
    text: String,
    value: Option<Date>,
    open: bool,
}

impl DateFieldState {
    /// Create an empty date field.
    pub fn new() -> Self {
        Self::default()
    }

    /// The current input text.
    pub fn text(&self) -> &str {
        &self.text
    }

    /// The parsed value of the current text, or `None` when it is empty,
    /// invalid or out of bounds.
    pub fn value(&self) -> Option<Date> {
        self.value
    }

    /// Whether the calendar popover is open.
    pub fn is_open(&self) -> bool {
        self.open
    }

    /// Replace the input text without parsing it; the widget re-parses on the
    /// next frame.
    pub fn set_text(&mut self, text: impl Into<String>) {
        self.text = text.into();
    }

    /// Replace the parsed value without touching the text.
    pub fn set_value(&mut self, value: Option<Date>) {
        self.value = value;
    }

    /// Open or close the calendar popover.
    pub fn set_open(&mut self, open: bool) {
        self.open = open;
    }

    /// Clear the text, value and popover state.
    pub fn clear(&mut self) {
        self.text.clear();
        self.value = None;
        self.open = false;
    }

    /// Set `text` from `value` using the ISO form; useful for tests and for
    /// callers that keep a date without custom formatting.
    pub fn set_date(&mut self, value: Option<Date>) {
        self.text = value.map(format_iso_date).unwrap_or_default();
        self.value = value;
        self.open = false;
    }
}

/// English defaults for the date field's validation and button labels.
#[derive(Clone, Copy)]
pub struct DateFieldLabels<'a> {
    /// Message shown when the text cannot be parsed. `None` uses the localized
    /// `dateFieldInvalidDateError` message from [`DateField::messages`].
    pub invalid_date: Option<&'a str>,
    /// Message shown when the date is outside the configured bounds.
    pub out_of_range: &'a str,
    /// Accessible name of the calendar button and its popover.
    pub calendar_button: &'a str,
}

impl<'a> Default for DateFieldLabels<'a> {
    fn default() -> Self {
        DateFieldLabels::english()
    }
}

impl DateFieldLabels<'static> {
    /// The English label set used when the caller supplies none.
    pub fn english() -> Self {
        Self {
            invalid_date: None,
            out_of_range: "Date is outside the allowed range.",
            calendar_button: "Choose date",
        }
    }
}

/// What [`DateField::show`] reports for the current frame.
pub struct DateFieldResponse {
    /// The text edit's response.
    pub response: Response,
    /// Whether the text, value or popover state changed this frame.
    pub changed: bool,
    /// The current parsed value.
    pub value: Option<Date>,
    /// The validation message shown for the current text.
    pub error: Option<String>,
    /// Whether the calendar popover is open.
    pub open: bool,
}

/// A date text field with an attached calendar popover.
pub struct DateField<'a> {
    state: &'a mut DateFieldState,
    theme: &'a Theme,
    label: &'a str,
    hint: Option<&'a str>,
    description: Option<&'a str>,
    labels: DateFieldLabels<'a>,
    messages: Messages,
    parser: &'a DateParser<'a>,
    formatter: &'a DateFormatter<'a>,
    min_date: Date,
    max_date: Date,
    today: Option<Date>,
    initial_month: Option<Date>,
    liquid: Option<crate::liquid::LiquidStyle>,
    enabled: bool,
    id: Option<Id>,
}

impl<'a> DateField<'a> {
    /// Create a date field using the ISO parser/formatter and English labels.
    pub fn new(state: &'a mut DateFieldState, theme: &'a Theme, label: &'a str) -> Self {
        Self {
            state,
            theme,
            label,
            hint: None,
            description: None,
            labels: DateFieldLabels::english(),
            messages: Messages::for_locale("en"),
            parser: &parse_iso_date,
            formatter: &format_iso_date,
            min_date: Calendar::default_min_date(),
            max_date: Calendar::default_max_date(),
            today: None,
            initial_month: None,
            liquid: None,
            enabled: true,
            id: None,
        }
    }

    /// Override the text hint. Defaults to the localized `dateFieldHint`.
    pub fn hint(mut self, hint: &'a str) -> Self {
        self.hint = Some(hint);
        self
    }

    /// Supporting text shown below the input.
    pub fn description(mut self, description: &'a str) -> Self {
        self.description = Some(description);
        self
    }

    /// Replace the validation and button label defaults.
    pub fn labels(mut self, labels: DateFieldLabels<'a>) -> Self {
        self.labels = labels;
        self
    }

    /// Use a localized message set for the hint and invalid-date message.
    pub fn messages(mut self, messages: Messages) -> Self {
        self.messages = messages;
        self
    }

    /// Use a custom parser for the input text.
    pub fn parser(mut self, parser: &'a DateParser<'a>) -> Self {
        self.parser = parser;
        self
    }

    /// Use a custom formatter when the popover writes a selected date back.
    pub fn formatter(mut self, formatter: &'a DateFormatter<'a>) -> Self {
        self.formatter = formatter;
        self
    }

    /// Set the inclusive selectable date bounds. The argument order does not
    /// matter: the earlier date is the minimum.
    pub fn bounds(mut self, min: Date, max: Date) -> Self {
        self.min_date = min.min(max);
        self.max_date = max.max(min);
        self
    }

    /// Set the caller-supplied "today" used for the calendar's today highlight.
    pub fn today(mut self, today: Date) -> Self {
        self.today = Some(today);
        self
    }

    /// The month the calendar opens on when the value is empty. Defaults to
    /// `today`'s month and then to the minimum month.
    pub fn initial_month(mut self, month: Date) -> Self {
        self.initial_month = Some(month);
        self
    }

    /// Override the context's popup animation preset.
    pub fn liquid(mut self, motion: crate::liquid::LiquidStyle) -> Self {
        self.liquid = Some(motion);
        self
    }

    /// Disable the field and its button.
    pub fn enabled(mut self, enabled: bool) -> Self {
        self.enabled = enabled;
        self
    }

    /// Give the field a stable, globally unique ID across layout changes.
    pub fn id(mut self, id: Id) -> Self {
        self.id = Some(id);
        self
    }

    /// Draw the field and return its response.
    pub fn show(self, ui: &mut Ui) -> DateFieldResponse {
        let id = self
            .id
            .unwrap_or_else(|| ui.next_auto_id().with("temivu-date-field"));
        ui.scope_builder(egui::UiBuilder::new().id(id), |ui| self.show_inner(ui, id))
            .inner
    }

    fn show_inner(self, ui: &mut Ui, id: Id) -> DateFieldResponse {
        let Self {
            state,
            theme,
            label,
            hint,
            description,
            labels,
            messages,
            parser,
            formatter,
            min_date,
            max_date,
            today,
            initial_month,
            enabled,
            liquid,
            ..
        } = self;
        let hint = hint.unwrap_or(messages.date_field_hint);
        let invalid_message = labels
            .invalid_date
            .unwrap_or(messages.date_field_invalid_date_error);
        let before_value = state.value;
        let before_open = state.open;

        // Parse the text each frame: text is the source of truth.
        let (parsed, error) = match state.text.trim() {
            "" => (None, None),
            text => match parser(text) {
                Ok(date) if date >= min_date && date <= max_date => (Some(date), None),
                Ok(_) => (None, Some(labels.out_of_range.to_owned())),
                Err(DateParseError::Custom(message)) => (None, Some(message)),
                Err(_) => (None, Some(invalid_message.to_owned())),
            },
        };
        state.value = parsed;

        let height = theme.sizes.field.lg;
        let button_size = Vec2::splat(height);
        let spacing = ui.spacing().item_spacing.x;
        let edit_id = id.with("text");
        let popover_id = id.with("calendar-popover");
        let calendar_key = id.with("calendar-state");
        let mut activated = None;
        let mut calendar_button: Option<Response> = None;

        let mut field_label = FieldLabel::new(theme, LabelLayout::Vertical).label(label);
        if let Some(description) = description {
            field_label = field_label.description(description);
        }
        if let Some(error) = error.as_deref() {
            field_label = field_label.error(error);
        }
        let response = field_label.show(ui, |ui| {
            ui.horizontal(|ui| {
                let available = ui.available_width();
                let edit_width = (available - button_size.x - spacing).max(0.0);
                let edit = crate::text_field::show_native(
                    ui,
                    theme,
                    enabled,
                    egui::TextEdit::singleline(&mut state.text)
                        .hint_text(hint)
                        .id(edit_id)
                        .desired_width(edit_width),
                );
                let button = crate::button::press_native(ui, theme, |ui| {
                    let button = ui.add_enabled(
                        enabled,
                        egui::Button::new("")
                            .fill(theme.card)
                            .stroke(egui::Stroke::new(1.0, theme.border))
                            .min_size(button_size)
                            .corner_radius(255),
                    );
                    if ui.is_rect_visible(button.rect) {
                        let icon =
                            egui::Rect::from_center_size(button.rect.center(), Vec2::splat(16.0));
                        let color = if enabled {
                            theme.foreground
                        } else {
                            theme.muted_foreground
                        };
                        let stroke = egui::Stroke::new(1.5, color);
                        ui.painter().rect_stroke(
                            icon.shrink(1.0),
                            3,
                            stroke,
                            egui::StrokeKind::Inside,
                        );
                        ui.painter().line_segment(
                            [
                                egui::pos2(icon.left() + 2.0, icon.top() + 5.0),
                                egui::pos2(icon.right() - 2.0, icon.top() + 5.0),
                            ],
                            stroke,
                        );
                    }
                    button
                });
                ui.ctx().accesskit_node_builder(button.id, |node| {
                    node.set_label(labels.calendar_button);
                });
                if button.clicked() {
                    state.open = !state.open;
                }
                calendar_button = Some(button.clone());
                if !enabled {
                    state.open = false;
                }
                if enabled {
                    let mut calendar_state = ui
                        .data(|data| data.get_temp::<CalendarState>(calendar_key))
                        .unwrap_or_else(|| {
                            let initial =
                                state.value.or(initial_month).or(today).unwrap_or(min_date);
                            CalendarState::new(initial, state.value)
                        });
                    calendar_state.set_selected(state.value);
                    if !before_open {
                        let anchor = state.value.or(initial_month).or(today).unwrap_or(min_date);
                        calendar_state.set_visible_month(anchor);
                        calendar_state.set_focused(state.value);
                    }
                    let mut popover_open = state.open;
                    let max_height = (ui.ctx().content_rect().height() - 72.0).max(1.0);
                    Popover::new(popover_id, labels.calendar_button, &mut popover_open)
                        .theme(theme)
                        .liquid(liquid.unwrap_or_else(|| {
                            crate::liquid::LiquidStyle::context_default(ui.ctx())
                        }))
                        .focus_on_open(true)
                        .width((theme.sizes.calendar.max(40.0) * 7.0 + 32.0).min(340.0))
                        .frame(crate::surface::popup_frame(theme, 16))
                        .show(&button, |ui| {
                            egui::ScrollArea::vertical()
                                .max_height(max_height)
                                .show(ui, |ui| {
                                    let mut calendar = Calendar::new(&mut calendar_state, theme)
                                        .bounds(min_date, max_date)
                                        .toggleable(false);
                                    if let Some(today) = today {
                                        calendar = calendar.today(today);
                                    }
                                    let output = calendar.show(ui);
                                    if let Some(date) = output.activated {
                                        activated = Some(date);
                                        calendar_state.set_selected(Some(date));
                                    }
                                    output.response
                                })
                                .inner
                        });
                    state.open = popover_open;
                    ui.data_mut(|data| data.insert_temp(calendar_key, calendar_state));
                }
                edit
            })
            .inner
        });

        if let Some(date) = activated {
            state.value = Some(date);
            state.text = formatter(date);
            state.open = false;
            if let Some(button) = &calendar_button {
                button.request_focus();
            }
        }
        if state.open && ui.input(|input| input.key_pressed(Key::Escape)) {
            state.open = false;
            if let Some(button) = &calendar_button {
                button.request_focus();
            }
        }

        let changed =
            response.changed() || state.value != before_value || state.open != before_open;
        DateFieldResponse {
            response,
            changed,
            value: state.value,
            error,
            open: state.open,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn date(year: i32, month: u8, day: u8) -> Date {
        Date::new(year, month, day).expect("test date is valid")
    }

    #[test]
    fn parses_iso_dates() {
        assert_eq!(parse_iso_date("2024-07-15"), Ok(date(2024, 7, 15)));
        assert_eq!(parse_iso_date(" 2024-7-5 "), Ok(date(2024, 7, 5)));
        assert_eq!(parse_iso_date("0001-01-01"), Ok(date(1, 1, 1)));
        assert_eq!(parse_iso_date("9999-12-31"), Ok(date(9999, 12, 31)));
    }

    #[test]
    fn rejects_invalid_iso_dates() {
        assert_eq!(parse_iso_date(""), Err(DateParseError::Empty));
        assert_eq!(parse_iso_date("   "), Err(DateParseError::Empty));
        assert_eq!(parse_iso_date("2024-07"), Err(DateParseError::Format));
        assert_eq!(parse_iso_date("2024-07-15-01"), Err(DateParseError::Format));
        assert_eq!(parse_iso_date("24-07-15"), Err(DateParseError::Format));
        assert_eq!(
            parse_iso_date("2024-13-01"),
            Err(DateParseError::InvalidDate)
        );
        assert_eq!(
            parse_iso_date("2023-02-29"),
            Err(DateParseError::InvalidDate)
        );
        assert_eq!(
            parse_iso_date("2024-00-10"),
            Err(DateParseError::InvalidDate)
        );
        assert_eq!(
            parse_iso_date("2024-01-00"),
            Err(DateParseError::InvalidDate)
        );
        assert_eq!(parse_iso_date("2024/07/15"), Err(DateParseError::Format));
        assert_eq!(parse_iso_date("2024-0a-15"), Err(DateParseError::Format));
        assert_eq!(
            parse_iso_date("0000-01-01"),
            Err(DateParseError::InvalidDate)
        );
    }

    #[test]
    fn formats_iso_dates() {
        assert_eq!(format_iso_date(date(2024, 7, 5)), "2024-07-05");
        assert_eq!(format_iso_date(date(1, 1, 1)), "0001-01-01");
    }

    #[test]
    fn state_tracks_text_value_and_open() {
        let mut state = DateFieldState::new();
        assert_eq!(state.text(), "");
        assert_eq!(state.value(), None);
        assert!(!state.is_open());
        state.set_text("2024-07-15");
        assert_eq!(state.text(), "2024-07-15");
        state.set_value(Some(date(2024, 7, 16)));
        assert_eq!(state.value(), Some(date(2024, 7, 16)));
        state.set_open(true);
        assert!(state.is_open());
        state.set_date(Some(date(2024, 8, 1)));
        assert_eq!(state.text(), "2024-08-01");
        assert_eq!(state.value(), Some(date(2024, 8, 1)));
        assert!(!state.is_open());
        state.clear();
        assert_eq!(state.text(), "");
        assert_eq!(state.value(), None);
        state.set_date(None);
        assert_eq!(state.text(), "");
    }

    #[test]
    fn custom_parser_errors_are_exposed() {
        let parser = |text: &str| {
            if text == "today" {
                Ok(date(2024, 7, 15))
            } else {
                Err(DateParseError::Custom("use 'today'".to_owned()))
            }
        };
        assert_eq!(parser("today"), Ok(date(2024, 7, 15)));
        assert_eq!(
            parser("tomorrow"),
            Err(DateParseError::Custom("use 'today'".to_owned()))
        );
        assert_eq!(DateParseError::Custom("x".to_owned()).to_string(), "x");
    }

    #[test]
    fn labels_have_english_defaults() {
        let labels = DateFieldLabels::english();
        assert_eq!(labels.invalid_date, None);
        assert_eq!(labels.out_of_range, "Date is outside the allowed range.");
        assert_eq!(labels.calendar_button, "Choose date");
    }
}
