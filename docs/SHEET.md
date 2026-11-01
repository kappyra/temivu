# Sheet (experimental)

A caller-owned modal bottom sheet. It blocks pointer and keyboard access to the rest of the UI, moves focus into the sheet when it opens and restores the previous focus after a normal dismissal.

```rust
use temivu::{Sheet, Theme};

fn example(ui: &mut egui::Ui, theme: &Theme, open: &mut bool) {
Sheet::new(egui::Id::new("filters-sheet"), "Filters", theme).show(ui.ctx(), open, |ui| {
    if ui.button("Apply filters").clicked() {
        ui.close();
    }
});
}
```

`open` is updated by Escape, the header's `Close` button, a backdrop click when `barrier_dismissible` is enabled (the default), and `ui.close()` calls inside the content. Only the topmost sheet handles Escape, and an open popup takes precedence. The sheet requests focus on opening and restores the previously focused widget after it disappears; call `show` every frame, including while closed, so focus can be restored. Pass a stable `egui::Id`.

The sheet is clamped to the window with an 8-point outer margin and a 16-point padding; `max_width` defaults to 560 points. Its height follows the content, so callers should provide a scroll area for long content. Drag-to-dismiss, entrance and exit animation, other sides, safe areas, keyboard insets, route semantics, localization, screen-reader verification and upstream visual parity remain open. See `COVERAGE.md` for current limitations.
