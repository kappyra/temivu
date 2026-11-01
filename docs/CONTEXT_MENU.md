# Context menu (experimental)

Attach `context_menu` to a clickable, focusable egui response. Return the first enabled menu item so a keyboard user enters the menu when it opens. Use `ui.close()` after an action that should dismiss the menu.

```rust
use temivu::context_menu;

fn example(ui: &mut egui::Ui, copied: &mut bool) {
let target = ui.button("Document");
context_menu(&target, |ui| {
    let copy = ui.button("Copy");
    if copy.clicked() {
        *copied = true;
        ui.close();
    }
    copy
});
}
```

A secondary click opens the menu at the pointer. When the target has keyboard focus, Shift+F10 opens it and focuses the returned item. Escape closes it; keyboard activation of a menu action returns focus to the target when the menu owned focus. Provide a stable ID for the target if its position in the UI can change, and ensure the returned item is enabled and focusable.

This is a preliminary API. Long press, the menu key, the macOS shortcut, nested menus, custom styles, screen-reader behavior, and complete Forui visual parity remain open. egui itself surrenders a focused widget on Escape even when no menu is open; the menu does not override that global behavior. See `COVERAGE.md` for current limitations.
