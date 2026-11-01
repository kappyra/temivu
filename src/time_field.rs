//! Preliminary port of Forui's time field: a validated time-of-day text input.
//!
//! The input is free-form text backed by a caller-owned [`TimeFieldState`]. The
//! parse result is validated on every frame with [`Time::parse`], which accepts
//! `H:MM`, `HH:MM`, `H:MM:SS` and `HH:MM:SS`. There is no picker button or
//! segmented arrow adjustment yet: the separate picker module is being ported
//! elsewhere. See `docs/TIME_FIELD.md`.
//!
//! ```
//! # fn example(ui: &mut egui::Ui) {
//! use temivu::{TimeField, TimeFieldState, Theme};
//!
//! let theme = Theme::light();
//! let mut state = TimeFieldState::new();
//! if state.set_text("09:30") {
//!     // `state.value()` is now `Some(Time { hour: 9, minute: 30, second: 0 })`
//! }
//! let output = TimeField::new(&mut state, &theme, "Start time").show(ui);
//! if output.changed {
//!     let _ = state.value();
//! }
//! # }
//! ```

use crate::Theme;
use crate::localization::Messages;
use crate::text_field::TextField;
use egui::{Id, Response, Ui};

pub use crate::time_picker::{Time, TimeError, TimeParseError};

/// Caller-owned time field state: the input text and its current parsed value.
///
/// The text is the source of truth. The widget re-parses it every frame and
/// writes the result to [`Self::value`].
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct TimeFieldState {
    text: String,
    value: Option<Time>,
}

impl TimeFieldState {
    /// Create an empty time field.
    pub fn new() -> Self {
        Self::default()
    }

    /// Create a field with `text`, parsing the value immediately.
    pub fn with_text(text: impl Into<String>) -> Self {
        let mut state = Self {
            text: text.into(),
            value: None,
        };
        state.sync_value();
        state
    }

    /// The current input text.
    pub fn text(&self) -> &str {
        &self.text
    }

    /// The parsed value of the current text, or `None` when it is empty or
    /// invalid.
    pub fn value(&self) -> Option<Time> {
        self.value
    }

    /// Replace the input text and re-parse the value; returns whether the value
    /// changed.
    pub fn set_text(&mut self, text: impl Into<String>) -> bool {
        self.text = text.into();
        self.sync_value()
    }

    /// Clear the text and value; returns whether the value changed.
    pub fn clear(&mut self) -> bool {
        self.set_text("")
    }

    fn sync_value(&mut self) -> bool {
        let value = Time::parse(&self.text).ok();
        let changed = self.value != value;
        self.value = value;
        changed
    }
}

/// What [`TimeField::show`] reports for the current frame.
pub struct TimeFieldResponse {
    /// The underlying text edit's response.
    pub response: Response,
    /// Whether the text edited this frame changed the value or the text.
    pub changed: bool,
    /// The current parsed value.
    pub value: Option<Time>,
    /// The validation message shown for unparsable text.
    pub error: Option<String>,
}

/// A time-of-day text field bound to caller-owned [`TimeFieldState`].
pub struct TimeField<'a> {
    state: &'a mut TimeFieldState,
    theme: &'a Theme,
    label: &'a str,
    hint: Option<&'a str>,
    description: Option<&'a str>,
    invalid_message: Option<&'a str>,
    messages: Messages,
    enabled: bool,
    id: Option<Id>,
}

impl<'a> TimeField<'a> {
    /// Create a time field with the English hint and validation message.
    pub fn new(state: &'a mut TimeFieldState, theme: &'a Theme, label: &'a str) -> Self {
        Self {
            state,
            theme,
            label,
            hint: None,
            description: None,
            invalid_message: None,
            messages: Messages::for_locale("en"),
            enabled: true,
            id: None,
        }
    }

    /// Override the hint text. Defaults to the localized `timeFieldHint`.
    pub fn hint(mut self, hint: &'a str) -> Self {
        self.hint = Some(hint);
        self
    }

    /// Supporting text shown below the input.
    pub fn description(mut self, description: &'a str) -> Self {
        self.description = Some(description);
        self
    }

    /// Override the validation message shown for unparsable input.
    pub fn invalid_message(mut self, message: &'a str) -> Self {
        self.invalid_message = Some(message);
        self
    }

    /// Use a localized message set for the hint and validation message.
    pub fn messages(mut self, messages: Messages) -> Self {
        self.messages = messages;
        self
    }

    /// Disable the input.
    pub fn enabled(mut self, enabled: bool) -> Self {
        self.enabled = enabled;
        self
    }

    /// Give the input a stable, globally unique ID across layout changes.
    pub fn id(mut self, id: Id) -> Self {
        self.id = Some(id);
        self
    }

    /// Draw the field and return its response.
    pub fn show(self, ui: &mut Ui) -> TimeFieldResponse {
        let Self {
            state,
            theme,
            label,
            hint,
            description,
            invalid_message,
            messages,
            enabled,
            id,
        } = self;
        let hint = hint.unwrap_or(messages.time_field_hint);
        let invalid_message = invalid_message.unwrap_or("Invalid time.");
        let field_id = id.unwrap_or_else(|| ui.next_auto_id().with("temivu-time-field"));

        let before = state.value;
        let parsed = Time::parse(state.text()).ok();
        state.value = parsed;
        let error = match (state.text().trim().is_empty(), parsed) {
            (true, _) => None,
            (false, Some(_)) => None,
            (false, None) => Some(invalid_message.to_owned()),
        };
        let validator = |text: &str| {
            let text = text.trim();
            if text.is_empty() || Time::parse(text).is_ok() {
                None
            } else {
                Some(invalid_message.to_owned())
            }
        };

        let mut field = TextField::new(&mut state.text, theme, label)
            .hint(hint)
            .validator(&validator)
            .enabled(enabled)
            .id(field_id.with("text"));
        if let Some(description) = description {
            field = field.description(description);
        }
        let output = field.show(ui);
        let changed = output.response.changed() || state.value != before;
        TimeFieldResponse {
            response: output.response,
            changed,
            value: state.value,
            error,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_full_and_short_forms() {
        assert_eq!(Time::parse("00:00").ok(), Time::with_seconds(0, 0, 0).ok());
        assert_eq!(Time::parse("9:05").ok(), Time::with_seconds(9, 5, 0).ok());
        assert_eq!(Time::parse("09:05").ok(), Time::with_seconds(9, 5, 0).ok());
        assert_eq!(
            Time::parse("23:59").ok(),
            Time::with_seconds(23, 59, 0).ok()
        );
        assert_eq!(
            Time::parse("23:59:59").ok(),
            Time::with_seconds(23, 59, 59).ok()
        );
        assert_eq!(
            Time::parse(" 7:8:9 ").ok(),
            Time::with_seconds(7, 8, 9).ok()
        );
    }

    #[test]
    fn rejects_invalid_input() {
        assert_eq!(Time::parse(""), Err(TimeParseError::Empty));
        assert_eq!(Time::parse("   "), Err(TimeParseError::Empty));
        assert_eq!(Time::parse("12"), Err(TimeParseError::MissingMinutes));
        assert_eq!(
            Time::parse("12:30:45:00"),
            Err(TimeParseError::TooManyParts)
        );
        assert_eq!(Time::parse("12:3a"), Err(TimeParseError::Malformed));
        assert_eq!(Time::parse("123:00"), Err(TimeParseError::Malformed));
        assert_eq!(Time::parse("24:00"), Err(TimeParseError::InvalidHour));
        assert_eq!(Time::parse("12:60"), Err(TimeParseError::InvalidMinute));
        assert_eq!(Time::parse("12:30:60"), Err(TimeParseError::InvalidSecond));
        assert_eq!(Time::parse("12:"), Err(TimeParseError::Malformed));
        assert_eq!(Time::parse(":30"), Err(TimeParseError::Malformed));
    }

    #[test]
    fn construction_validates_components() {
        assert!(Time::with_seconds(23, 59, 59).is_ok());
        assert_eq!(
            Time::with_seconds(24, 0, 0),
            Err(TimeError::InvalidHour(24))
        );
        assert_eq!(
            Time::with_seconds(0, 60, 0),
            Err(TimeError::InvalidMinute(60))
        );
        assert_eq!(
            Time::with_seconds(0, 0, 60),
            Err(TimeError::InvalidSecond(60))
        );
    }

    #[test]
    fn formats_with_one_and_two_second_precision() {
        let time = Time::with_seconds(9, 5, 7).expect("test time is valid");
        assert_eq!(time.format_hm(), "09:05");
        assert_eq!(time.format_hms(), "09:05:07");
        assert_eq!(time.to_string(), "09:05:07");
        assert_eq!(time.hour(), 9);
        assert_eq!(time.minute(), 5);
        assert_eq!(time.second(), 7);
    }

    #[test]
    fn state_syncs_value_from_text() {
        let mut state = TimeFieldState::new();
        assert_eq!(state.value(), None);
        assert!(state.set_text("08:15"));
        assert_eq!(state.value(), Time::with_seconds(8, 15, 0).ok());
        assert!(!state.set_text("08:15"));
        assert!(state.set_text("not a time"));
        assert_eq!(state.value(), None);
        assert!(!state.set_text("still not a time"));
        assert!(!state.clear());
        assert_eq!(state.text(), "");
        assert_eq!(state.value(), None);
        let state = TimeFieldState::with_text("17:45:30");
        assert_eq!(state.value(), Time::with_seconds(17, 45, 30).ok());
    }
}
