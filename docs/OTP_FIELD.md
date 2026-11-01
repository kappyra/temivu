# OTP field (experimental)

A one-time password input ported from Forui's `FOtpField`. It renders a
caller-owned `String` as fixed-width boxes backed by one focusable input group.

```rust
use temivu::{OtpField, Theme};

fn example(ui: &mut egui::Ui, code: &mut String) {
    let theme = Theme::light();
    let output = OtpField::new(code, &theme, "One-time password")
        .length(6)
        .show(ui);
    if output.completed {
        let _submitted = code.as_str();
    }
}
```

## State and input

`OtpField::new` binds a caller-owned `String`; the field normalizes it to ASCII
digits and truncates it to the configured length (`length`, default 6). Typing
advances the caret automatically, a single typed digit replaces the focused box
so a mistyped code can be corrected, and longer text (a paste or an operating
system autofill) is inserted at the caret and truncated to the first `length`
digits, matching upstream truncation. Backspace deletes the digit before the
caret and steps back; Delete removes the digit under the caret; Left/Right move
the caret; Home/End jump to the ends. Clicking a box focuses the field and
places the caret on that box. `OtpFieldResponse::changed` reports a frame's
edit and `completed` reports a full-length code.

## Accessibility

The group is one focusable `TextInput` node whose accessible name is the visible
label and whose value is the current code; `OtpField::invalid` / `OtpField::error`
set the invalid state and `OtpField::enabled(false)` sets disabled and blocks
input. Every box is a `TextInput` node named from `OtpLabels::digit`
(`"Digit 1 of 6"` by default) whose value is the box's digit, so the per-box
labels and the group label are both queryable. Provide `OtpField::id` when more
than one field can appear in the same layout, or when layout order can change.

## Known gaps

Backspace removes the digit at the current caret position rather than
re-running platform text-editing conventions; upstream comparison for
mid-string editing is still open.

The caret does not blink (this keeps the widget repaint-free while idle),
dividers and grouped boxes, platform keyboard appearance and cursor settings,
form integration and validation, IME composition, and upstream visual parity
remain open. See `COVERAGE.md`.
