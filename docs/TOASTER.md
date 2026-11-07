# Notifications

Use `Toast` for an inline message or `Toaster` for a foreground stack. Both use the theme's surface and typography tokens. Both notification layouts use Droplet for arrival and dismissal. Reduced motion skips the transition.

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

- Touch cards use a stacked layout with a status marker, full-width message and labelled dismiss action. Desktop cards use a status rail and a compact close icon. Use `Toast::input_mode` to select a layout explicitly.
- The queue sits at the bottom trailing corner on desktop and at the top center on touch layouts. Override the position with `Toaster::anchor` or `ToasterState::show_with`. Tall stacks scroll within the viewport.
- The visible limit defaults to three. Timers start when an entry first appears and restart after hover or focus. Use `timeout(None)` for a persistent notification. Reusing an ID replaces its entry.
- The dismiss button uses the supplied label for assistive technology. Each live card exposes a polite alert, and the closing animation contains no interactive controls.
- Use `needs_render()` when skipping an idle stack or dropping an inline `ToastState`. `is_empty()` and `is_visible()` reflect the logical state immediately after dismissal.

For a notification center, render a trigger and keep an `open` flag in app state.
Set it on activation and call `ToasterState::show_center` every frame with the
trigger response and `NotificationCenterLabels`. Hide the floating preview while
the center is open. The modal lists active and pending entries in a scrollable
panel, pauses preview timers, and offers **Clear all** when at least two
notifications remain.

`ToastState::status()` reports `Visible`, `Closing`, or `Closed`.
`ToasterState::closing_count()` counts preview cards still animating out.

Use `Toaster::show_with_center` to attach actions directly below the stacked
preview cards. The footer appears with two or more queued notifications and
shows a `+N` count when more entries are waiting beyond the preview limit.
**View all notifications** opens the center from its own button, and **Clear all**
dismisses the entire queue, including entries outside the preview limit.

## Notification layout

The preview is a single floating tray with a quiet border and one shadow. Each
notice has the same layout as the center list: status icon, wrapping message,
and a centered dismiss button. Severity is conveyed by the icon and its color.
The body copy uses the regular theme text colors.

- Desktop: a narrow tray at the trailing bottom edge. Stack notices vertically
  and group **Clear all** and **View all notifications** in the tray footer.
- Phone: center the tray near the top with safe side insets. The message wraps
  next to the icon, and the dismiss button has a 44-point target. Footer actions
  wrap onto another line when the available width requires it.
- Notification center: use a title, count badge, and close control above the
  list. Size rows to their messages and scroll only the list when it reaches the
  height limit. Keep **Clear all** below the list so it remains reachable.
- Continue using the same Droplet transitions for toast arrival, dismissal,
  and the transition from the tray to the center.
