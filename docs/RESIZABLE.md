# Resizable (experimental)

A caller-owned two-pane split. The divider is draggable and keyboard-resizable, panes are clamped to minimum extents and the split is derived from a fraction so it scales deterministically when the window changes size.

```rust
use temivu::{Resizable, ResizableAxis, Theme};

fn example(ui: &mut egui::Ui, theme: &Theme, fraction: &mut f32) {
Resizable::new(
    egui::Id::unique("editor-split"),
    ResizableAxis::Horizontal,
    fraction,
    theme,
)
.min_extents(120.0, 160.0)
.show(
    ui,
    |ui| {
        ui.label("Outline");
    },
    |ui| {
        ui.label("Preview");
    },
);
}
```

`fraction` is the first pane's share of the space available to both panes, excluding the divider; it is clamped to the minimum extents on every frame and updated while dragging or when the focused divider receives an arrow key on the main axis. The divider is focusable and reports the slider role with its label (default `Resize panes`) and a value of both pane extents in points. Give it a stable `egui::Id`.

Only two panes are supported, and panes must be finite; the split is clamped to the visible area when it is used inside a scroll area. RTL, fixed-pixel panes, cascading resize rules, touch-sized hit regions, haptic feedback, focus-outline parity, animation, screen-reader verification and upstream visual parity remain open. See `COVERAGE.md` for current limitations.
