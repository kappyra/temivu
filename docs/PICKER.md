# Picker (experimental)

A wheel-style picker ported from Forui's `FPicker`/`FPickerWheel`: a vertical
scroll area over caller-owned items with a fixed centre selection band. The
selection is caller-owned; the library does not read the wall clock or allocate
the item list.

```rust
use temivu::{Picker, PickerItem, Theme};

fn example(ui: &mut egui::Ui, selected: &mut Option<egui::Id>) {
    let theme = Theme::light();
    let items = [
        PickerItem::new(egui::Id::unique("small"), "Small"),
        PickerItem::new(egui::Id::unique("medium"), "Medium"),
        PickerItem::new(egui::Id::unique("large"), "Large"),
    ];
    let output = Picker::new(selected, &items, &theme)
        .label("Size")
        .visible_rows(5)
        .show(ui);
    if output.changed {
        let _chosen = output.selected;
    }
}
```

## State and identity

`Picker::new(&mut selected, &items, &theme)` binds a caller-owned
`Option<egui::Id>` selection and a caller-owned `&[PickerItem]`. Every item needs
a stable, unique ID so the selection band, keyboard focus and accessibility row
nodes stay attached to the same value when rows are inserted or reordered.
`Picker::id` pins the wheel's own ID; the default derives it from call order and
therefore moves when a preceding conditional widget appears.

## Interaction

- A pointer press on a row selects it and moves keyboard focus to the wheel.
- The mouse wheel or a drag scrolls the wheel. When the wheel settles, the
  nearest enabled row becomes the selection and the offset snaps to a whole row.
- Up/Down move the selection by one enabled row (skipping disabled rows);
  Home/End move to the first/last enabled row. Navigation clamps by default;
  `Picker::looping(true)` wraps it.
- A caller-driven selection change scrolls the wheel to centre that row on the
  next frame, so external state stays visible.

Scrolling and snapping only run while something changes. A settled wheel does
not request repaints, and the component does not run an idle animation loop; the
regression test `wheel_scroll_selects_the_nearest_row_and_stops_repainting`
checks this.

## Accessibility

The wheel is a `ListBox` node under the configured accessible label, with the
current label in its value slot; every row is a `ListBoxOption` with its label,
selected state and disabled state. The wheel is focusable (Tab and click) and
draws a visible focus ring around the selection band. Disabled pickers remain
announced but expose the disabled state and ignore pointer and keyboard input.

## Known gaps

Looping animation, haptics, custom per-row content, per-widget styles, disabled
row styling parity, touch hit regions, reduced-motion behaviour, screen-reader
testing on a real platform and upstream visual parity remain open. See
`COVERAGE.md`.
