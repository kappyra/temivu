# Time field (experimental)

A validated time-of-day text input. The field reuses the crate's `TextField`
layout, so its label, hint, description and validation message match other form
fields.

```rust
use temivu::{Theme, TimeField, TimeFieldState};

fn example(ui: &mut egui::Ui, state: &mut TimeFieldState) {
    let theme = Theme::light();
    state.set_text("09:30");
    let output = TimeField::new(state, &theme, "Start time").show(ui);
    if output.changed {
        let _value = state.value();
    }
}
```

## State and parsing

`TimeFieldState` owns the input text and its parsed `Option<Time>` value. The
text is the source of truth: the widget parses it every frame, writes the result
to `TimeFieldState::value` (`None` when the text is empty or invalid) and shows
the validation message. `Time::parse` accepts `H:MM`, `HH:MM`, `H:MM:SS` and
`HH:MM:SS` with 1-2 digit parts, ignoring surrounding whitespace, and validates
`0..=23` hours and `0..=59` minutes and seconds. `TimeField::hint`,
`TimeField::description`, `TimeField::invalid_message` and `TimeField::messages`
override the localized defaults. A caller that keeps the text without the widget
can use `TimeFieldState::set_text`, `set_text` re-parses and returns whether the
value changed.

## Accessibility

The `TextField` association supplies the accessible label, the text edit's value
is the input text, and invalid input sets the AccessKit invalid state plus the
description. The localized `timeFieldHint` is the placeholder text.

## Known gaps

There is no time picker button: the separate picker/time-picker modules are being
ported elsewhere, so this field only supports keyboard entry. Segmented arrow-key
adjustment, 12-hour clocks with AM/PM, locale-specific separators, form
integration and upstream visual parity remain open. The invalid-time message is
not part of the generated localization set and defaults to English
("Invalid time."); override it with `TimeField::invalid_message`. See
`COVERAGE.md`.
