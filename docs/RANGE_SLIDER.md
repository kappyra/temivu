# Range slider

`RangeSlider` is the preliminary two-thumb form of Forui's range slider
controls. Values are normalized to `0.0..=1.0` and owned by the caller.

```rust
use temivu::{RangeSlider, Theme};

fn example(ui: &mut egui::Ui, value: &mut (f64, f64), theme: &Theme) {
    RangeSlider::new(value, theme)
        .expect("ordered values in zero..=one")
        .label("Price range")
        .step(0.05)
        .expect("valid step")
        .show(ui);
}
```

## Behaviour

- The track is painted with the theme's secondary fill and primary selection
  span; both thumbs are circles with a card fill and primary stroke, sized for
  desktop and the touch scale. The default orientation is horizontal;
  `.vertical(true)` lays the track out from top (maximum) to bottom (minimum) in
  a fixed-width column (40 pt desktop, 48 pt touch) whose height is clamped to
  64–240 pt of the available height, with drag, track clicks and keyboard using
  the same normalized value mapping (Up/Right increase). Labels that do not fit
  the narrow column are truncated (by characters, so grapheme clusters or ZWJ
  emoji can be split) with an ellipsis instead of overlapping neighbouring
  widgets. Clicks on the label strip above the track are ignored. Parents
  shorter than 64 pt still overflow the vertical minimum, and a horizontal row
  where no height is available uses the minimum height; give the slider a
  parent with at least the minimum size.
- Pointer: dragging either thumb moves it; the thumbs cannot cross (the start
  clamps to the end and vice versa). Clicking the track moves the nearer thumb to
  the click position.
- Keyboard: both thumbs are focusable sliders. Arrow Left/Right and Down/Up
  move the focused thumb by the step (0.05 by default) — both axes are
  focus-locked so arrows never move focus — Home sets it to the minimum and End
  to the maximum, with the same ordering clamp.
- Accessibility: two `Slider` nodes ("{label} start" / "{label} end") with
  normalized numeric values, min/max 0..1 and disabled state; assistive-
  technology increment/decrement actions are applied with the same step as the
  arrow keys, the focused thumb paints a focus ring, arrow keys are focus-locked
  so they adjust the value without moving focus, and disabled thumbs are not
  focusable or clickable.
- An explicit stable ID can be supplied with `.id(...)`; the default ID is
  derived from the label and the widget's position, so same-label sliders in one
  parent stay independent. Supply an explicit ID when the application needs to
  address the widget across layout changes.
- The caller's `(start, end)` tuple is written every frame with the current
  values; the response only reports `changed()` when a value actually changes.
  Invalid input (NaN, out of range, start > end) is rejected by `new`.

## Known gaps

- Arbitrary ranges (the widget is normalized), marks and tick labels, pointer
  drag on the whole span, tooltips, touch gesture tuning, RTL mirroring and
  upstream visual parity remain open.
- Dragging snaps the thumb centre to the pointer, so grabbing the edge of a thumb
  can jump it slightly. When `start == end` the two thumbs fully overlap and the
  start thumb is only reachable by keyboard (Tab).
- Screen-reader announcements of the two thumbs have not been tested against a
  real screen reader, and the focus ring is a plain primary-coloured circle
  rather than the upstream focus treatment.
