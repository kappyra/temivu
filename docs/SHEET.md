# Bottom sheet

Use a bottom sheet for a focused task such as choosing filters. The sheet is a modal with a spring-driven entrance and exit. It follows a downward drag on its handle and springs back when released early.

```rust
use temivu::{Sheet, Theme};

fn filters(ui: &mut egui::Ui, theme: &Theme, open: &mut bool) {
    Sheet::new(egui::Id::new("filters"), "Filters", theme)
        .description("Choose which items are shown.")
        .show(ui.ctx(), open, |ui| {
            if ui.button("Apply filters").clicked() {
                ui.close();
            }
        });
}
```

Both layouts float above the bottom edge and use the same radius on all four corners. The touch layout has narrow side margins and a dismiss action below the content when height permits. The desktop layout uses a wider panel with the title, description and dismiss action in its header. Long content scrolls between the fixed controls. Use `Sheet::input_mode` to select a layout explicitly. The default maximum width is 560 points.

The close button, downward drag, Escape, a backdrop click when enabled, and `ui.close()` dismiss the sheet. Use a stable ID and call `show` every frame so the exit can finish and previous keyboard focus can be restored. Reduced motion skips the automatic transition.
