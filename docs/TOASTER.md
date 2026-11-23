# Notifications

Use `Toast` for an inline message or `Toaster` for a foreground stack. Arrival uses Droplet. On dismissal or expiry, the card, text and icon soften and fade in place over 200 ms. The list releases the row's space near the end of the fade. Reduced motion skips both transitions.

```rust
use temivu::{Theme, ToastEntry, Toaster, ToasterState};

fn notifications(ui: &mut egui::Ui, theme: &Theme, queue: &mut ToasterState) {
    if ui.button("Save").clicked() {
        queue.push(
            ToastEntry::new(egui::Id::unique("saved"), "Changes saved")
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

Set `Toast::fade_out` or `ToastEntry::fade_out` with a `FadeOut` value to tune the exit for an inline card or an entry in the stack and center. Set `blur_radius` to `0.0` for an opacity-only transition or `duration` to `Duration::ZERO` for immediate removal.

Use the same exit on an arbitrary egui widget through `EffectSurface`. Keep its `FadeOutState` alongside the widget's state and keep calling `show_with_fade` while the widget is visible or `fade.has_snapshot()` returns true.

```rust
use std::time::Duration;
use temivu::{EffectSurface, FadeOut, FadeOutState, Glass, Theme, Vesper};

fn connection(ui: &mut egui::Ui, theme: &Theme, visible: bool, fade: &mut FadeOutState) {
    EffectSurface::new(theme)
        .id_salt("connection-status")
        .glass(Some(Glass::soft_prism()))
        .lighting(Some(Vesper::subtle()))
        .fade_out(FadeOut {
            duration: Duration::from_millis(200),
            blur_radius: 4.0,
        })
        .show_with_fade(ui, fade, visible, |ui| ui.label("Connected"));
}
```

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
- The tray-to-center transition uses Droplet. A row dismissed inside the center
  uses the same fade as the preview card.
