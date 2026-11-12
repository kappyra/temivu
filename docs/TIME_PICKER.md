# Time picker (experimental)

Time controls bound to a caller-owned validated `Time` value. The default
presentation uses [`Picker`](PICKER.md) wheels. The stepper presentation adds
editable numeric fields and separate increment and decrement buttons.

```rust
use temivu::{Theme, Time, TimePicker, TimePickerLayout};

fn example(ui: &mut egui::Ui, time: &mut Time) {
    let output = TimePicker::new(time, &Theme::light())
        .seconds(true)
        .layout(TimePickerLayout::Stepper)
        .show(ui);
    if output.changed {
        let _current = time.formatted();
    }
}
```

## Time values

`Time` is a validated wall-clock time with no date, time zone or dependency on
a date crate. `Time::new(hour, minute)` and `Time::with_seconds(hour, minute,
second)` reject hours outside `0..=23` and minutes/seconds outside `0..=59`;
`with_hour`, `with_minute` and `with_second` return a validated copy. Helpers
provide `MIDNIGHT`, `hour`, `minute`, `second`, `seconds_since_midnight` and
`formatted` (`"HH:MM"`, or `"HH:MM:SS"` when the second is non-zero). The type
derives ordering and hashing for caller-side comparisons.

## Interaction

Each wheel is a `Picker`. Up/Down move the value by one step and clamp at the
wheel ends (00 and 23, 00 and 59), matching upstream's non-looping wheels.
Left/Right move keyboard focus between the hour, minute and seconds wheels.
Pointer presses select the pressed row. The wheels lock the arrow keys while
focused so egui's spatial navigation does not steal them.

`TimePicker::seconds(true)` adds a seconds control to either presentation. The
stepper arranges its units in columns when space allows and stacks them in a
narrow container. Both layouts preserve the caller's second value when it is
hidden. `TimePicker::enabled(false)` disables all controls.

## Accessibility

The component is a `Group` node named by `TimePickerLabels::group` (default
`"Time"`) whose value is the current formatted time. Each wheel is a `ListBox`
named `"Hour"`, `"Minute"` and `"Second"` with the current value in its value
slot, and rows expose the listbox-option role with selected/disabled state.
`TimePickerLabels` replaces all four names. The stepper uses the same names
for its numeric fields and adds the operation symbol to each button's name.
The wheel separators are hidden from the accessibility tree.

## Known gaps

12-hour AM/PM periods, locale-dependent formats (`HH:mm` versus `h:mm a`),
per-wheel intervals (`hourInterval`/`minuteInterval`),
wheel animation, haptics, reduced-motion behavior, localized wheel labels,
screen-reader testing on a real platform and upstream visual parity remain open.
See `COVERAGE.md`.

## Formatting note

`Time::formatted()` (and `Display`) print `HH:MM` for a zero-seconds value and `HH:MM:SS` otherwise; `format_hm` and `format_hms` always use the requested precision. Parsing accepts `H:MM`, `HH:MM`, `H:MM:SS` and `HH:MM:SS`, so a second-precision value round-trips through `format_hms`.
