# Date-time picker (experimental)

A caller-owned date and time composed from the existing calendar grid
([CALENDAR.md](CALENDAR.md)) and the time picker wheels
([TIME_PICKER.md](TIME_PICKER.md)), ported from Forui's `FDateTimePicker` as a
fixed vertical stack: the calendar above the time wheels.

```rust
use temivu::{Date, DateTimePicker, DateTimePickerState, Theme};

fn example(ui: &mut egui::Ui, state: &mut DateTimePickerState) {
    let output = DateTimePicker::new(state, &Theme::light())
        .bounds(
            Date::new(2024, 1, 1).expect("a valid date"),
            Date::new(2024, 12, 31).expect("a valid date"),
        )
        .show(ui);
    if let Some(selected) = output.selected {
        let _value = selected.formatted();
    }
}
```

## Value and state

`DateTime` combines an already-validated `Date` and `Time`, so every value is
representable; it derives ordering and hashing and formats as
`"Monday, July 15, 2024, 09:30"` (English). `DateTimePickerState` owns the
calendar's visible month, selected date and focused date plus the time. A single
selection produces one `DateTime`; `set_selected(None)` clears the date and
keeps the time. Helpers: `new`, `from_selected`, `calendar`, `calendar_mut`,
`time`, `set_time`, `selected`, `set_selected`, `visible_month`.

## Layout, bounds and focus

The calendar and the time wheels are laid out vertically. `bounds` forwards
inclusive date bounds to the calendar (out-of-range days and month navigation
are disabled), `selectable` forwards a disabled-date predicate, `today` sets the
calendar's "today" marker, `first_day_of_week`, `sizes`, `seconds` and `enabled`
are forwarded to the child widgets, and `labels` replaces the English group,
calendar and time-picker names.

Each child keeps its own keyboard focus and stable IDs (`<id>/calendar` and
`<id>/time`), so tabbing between the grid and the wheels and selecting from
either does not steal focus from the other. The component itself only reports a
`Group` node and never requests focus.

## Accessibility

The component is a `Group` node named by `DateTimePickerLabels::group` (default
`"Date and time"`) whose value is the formatted selection, or
`DateTimePickerLabels::empty_value` when no date is selected. The calendar and
time pickers expose their own heading, grid/column-header/grid-cell and
listbox/listbox-option nodes; the group reports the disabled state when the
component is disabled.

## Known gaps

The upstream single date wheel, day/hour/minute intervals, localized date and
time formats, AM/PM periods, "Today" default date labels in the date wheel,
month/year picker sub-grids, swipe and wheel animation, haptics, reduced-motion
behaviour, screen-reader testing on a real platform and upstream visual parity
remain open. See `COVERAGE.md`.
