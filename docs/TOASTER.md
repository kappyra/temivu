# Notifications

Use `Toast` for an inline message or `Toaster` for a foreground stack. Both use the theme's surface and typography tokens. Cards resize and fade on arrival and dismissal using the same spring. Reduced motion skips the transition.

```rust
use temivu::{Theme, ToastEntry, Toaster, ToasterState};

fn notifications(ui: &mut egui::Ui, theme: &Theme, queue: &mut ToasterState) {
    if ui.button("Save").clicked() {
        queue.push(
            ToastEntry::new(egui::Id::new("saved"), "Changes saved")
                .description("Your draft is up to date."),
        );
    }
    if queue.needs_render() {
        Toaster::new(queue, theme, "Dismiss").show(ui);
    }
}
```

- Wide desktop windows use the bottom trailing corner. Compact windows and touch layouts use the top center. Override the position with `Toaster::anchor` or `ToasterState::show_with`. Tall stacks scroll within the viewport.
- The visible limit defaults to three. Timers start when an entry first appears and restart after hover or focus. Use `timeout(None)` for a persistent notification. Reusing an ID replaces its entry.
- The dismiss button uses the supplied label for assistive technology. Each live card exposes a polite alert, and the closing animation contains no interactive controls.
- Use `needs_render()` when skipping an idle stack or dropping an inline `ToastState`. `is_empty()` and `is_visible()` reflect the logical state immediately after dismissal.
