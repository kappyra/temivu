# Toaster

`ToasterState` is the caller-owned overlay queue for toasts; `Toaster` renders it
as a stacked foreground area anchored to the top-right corner. It complements the
single inline [`Toast`](TOAST.md) widget (see `COVERAGE.md` for the module).

```rust
use temivu::{ToastEntry, Toaster, ToasterState, Theme};

fn example(ui: &mut egui::Ui, state: &mut ToasterState, theme: &Theme) {
    state.push(
        ToastEntry::new(egui::Id::new("saved"), "Changes saved")
            .description("Your draft is now up to date.")
            .timeout(None),
    );
    Toaster::new(state, theme, "Dismiss").show(ui);
}
```

## Behaviour

- **Queue:** `push` inserts an entry; re-pushing an existing ID replaces and
  restarts it. `dismiss(id)`, `clear`, `len`, `is_empty` and `contains` manage the
  queue. The default limit is three visible toasts (clamped to 1–8); entries
  beyond the limit wait until a visible toast is dismissed or expires.
- **Timers:** each visible entry uses `ToastState`, so timed entries expire,
  hover or dismiss-button focus pauses the timer, and `timeout(None)` keeps an
  entry until it is dismissed. Expired entries are pruned after rendering.
- **Layout:** the overlay is a foreground `Area` anchored bottom-end on desktop
  and top-centre on touch (matching `FToasterStyle.toastAlignment`) with a
  360-point maximum width, 16-point offset and 8-point spacing. Override it with
  `Toaster::anchor`, and give each toaster under one parent its own
  `Toaster::id`. The area is interactable so its dismiss buttons can be focused
  and activated; a polite alert node per toast is announced.
- **Variants and icons:** `ToastEntry::variant` selects the upstream primary or
  destructive style; the destructive title and description use the
  contrast-checked destructive colour. `ToastEntry::icon` adds a leading text
  glyph as an approximation of the upstream vector icon. The glyph is hidden
  from the accessibility tree (decorative), so assistive technology only sees the
  toast's alert label.
- **Animation and names:** each entry renders inside its own ID scope, so every
  stacked toast fades in independently, and each dismiss control gets a unique
  accessible name derived from its toast title.
- **Accessibility:** each toast keeps the inline toast's `Alert` role, polite
  live region, label (title plus description) and 32-point dismiss control.

## Known gaps

- Positioning presets beyond the default anchor, stacking order, expand/collapse
  hover behaviour, swipe/tap-to-dismiss gestures, exit animations, custom
  per-entry styles and duplicate announcement handling are not implemented.
- The queued (not yet visible) entries do not start their timers until they are
  rendered; very long queues and hover-pause across the whole stack are
  unverified. The overlay reports the number of toasts rendered in the last
  frame.
- Screen-reader behaviour and upstream visual parity remain open.
