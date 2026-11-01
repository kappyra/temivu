# Date field (experimental)

A validated date text input with a calendar button that opens the crate's
`Calendar` in a `Popover`. The field is a Forui-style form field: label,
description, validation message and invalid state come from `FieldLabel`.

```rust
use temivu::{Date, DateField, DateFieldState, Theme};

fn example(ui: &mut egui::Ui, state: &mut DateFieldState) {
    let theme = Theme::light();
    state.set_text("2024-07-15");
    let output = DateField::new(state, &theme, "Due date")
        .bounds(
            Date::new(2024, 1, 1).expect("a valid date"),
            Date::new(2024, 12, 31).expect("a valid date"),
        )
        .today(Date::new(2024, 7, 15).expect("a valid date"))
        .show(ui);
    if let Some(date) = output.value {
        let _selected = date;
    }
}
```

## State, parsing and bounds

`DateFieldState` owns the input text, the parsed value and the popover's open
flag. The text is the source of truth: every frame the widget parses it, writes
the result to `DateFieldState::value` (`None` when the text is empty, invalid or
out of bounds) and returns the message in `DateFieldResponse::error`. The
default parser accepts ISO `YYYY-MM-DD`, with one- or two-digit month and day;
`DateField::parser` and `DateField::formatter` replace it, and
`DateParseError::Custom` is shown verbatim. `DateField::bounds` enforces
inclusive minimum and maximum dates both for typed text and for the popover
calendar, and `DateField::initial_month` chooses which month the calendar opens
on when the value is empty. Library code never reads the wall clock: pass
`DateField::today` for the today highlight.

Selecting a day in the popover formats it with `DateField::formatter`, closes the
popover and returns keyboard focus to the calendar button. Escape closes the
popover. `DateField::messages` localizes the hint and invalid-date message from
the crate's `Messages` set, and `DateFieldLabels` overrides the invalid, range
and button labels.

## Accessibility

`FieldLabel` associates the visible label with the text input, appends the
description and error to its AccessKit description and sets the invalid state
when validation fails. The text input exposes the current text as its value, and
the calendar button is a `Button` whose accessible name defaults to
"Choose date". The popover is a labelled `Group` containing the standard
`Calendar` grid and its day cells.

## Known gaps

Segment-based editing (selecting day/month/year parts and adjusting them with the
arrow keys) is not ported; arrows move the text caret only. Locale-specific date
patterns, time zones, form integration and upstream visual parity remain open.
The calendar button uses a provisional emoji glyph. See `COVERAGE.md`.
