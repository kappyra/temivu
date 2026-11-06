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

On compact windows and touch layouts the sheet is bottom-aligned with narrow side insets. Wide windows get a centered, floating panel. The default maximum width is 560 points. Long content scrolls within the sheet while the title and dismiss control remain visible.

The close button, downward drag, Escape, a backdrop click when enabled, and `ui.close()` dismiss the sheet. Use a stable ID and call `show` every frame so the exit can finish and previous keyboard focus can be restored. Reduced motion skips the automatic transition.
