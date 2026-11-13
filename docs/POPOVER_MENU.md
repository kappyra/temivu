# Popover menu (experimental)

Anchors a menu to a trigger and reuses the preliminary `Popover` for outside-click dismissal and focus restoration. Draw every row through `MenuItems::item` so the menu can expose the menu and menu-item roles and focus the first enabled item when it opens.

```rust
use temivu::{Item, PopoverMenu, Theme};

fn example(ui: &mut egui::Ui, theme: &Theme, open: &mut bool) {
let anchor = ui.button("Actions");
if anchor.clicked() {
    *open = !*open;
}
PopoverMenu::new(egui::Id::unique("actions-menu"), "Actions", open).show(&anchor, |ui, menu| {
    if menu.item(ui, Item::new("Rename", theme)).clicked() {
        ui.close();
    }
    let _ = menu.item(ui, Item::new("Delete", theme).enabled(false));
});
}
```

The menu opens when the caller sets `open`, or when its trigger reports a click. When the trigger has keyboard focus, opening focuses the first enabled item; disabled items are skipped and cannot be activated. Escape, an outside click or `ui.close()` dismisses the menu and updates `open`. Escape restores focus to the trigger only when the menu owned focus, matching the popover's guard against stealing unrelated focus. Pass a stable `egui::Id` and give every item a stable `Item::id` if the menu's content can change, otherwise focus follows the row's position.

The default content width is 200 points; Forui's reference range is 150 to 250, but intrinsic sizing, scrolling, submenus, hover-to-focus, separators or groups, checkable items, animation, RTL and screen-reader verification remain open. See `COVERAGE.md` for current limitations.
