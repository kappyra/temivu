# Select menu tile (experimental)

A tile-shaped, selectable menu row for menus and popovers: a full-width, 48-point row with an optional interim icon glyph, a title, an optional description and caller-owned selected/disabled state. It exposes the menu-item role and the selected state.

```rust
use temivu::{PopoverMenu, SelectMenuTile, Theme};

fn example(ui: &mut egui::Ui, theme: &Theme, open: &mut bool, selected: &mut Option<egui::Id>) {
let anchor = ui.button("Density");
if anchor.clicked() {
    *open = !*open;
}
PopoverMenu::new(egui::Id::unique("density-menu"), "Density menu", open).show(&anchor, |ui, menu| {
    if menu
        .select_menu_tile(
            ui,
            SelectMenuTile::new("Compact", theme)
                .id(egui::Id::unique("density-compact"))
                .selected(*selected == Some(egui::Id::unique("density-compact"))),
        )
        .clicked()
    {
        *selected = Some(egui::Id::unique("density-compact"));
        ui.close();
    }
    let _ = menu.select_menu_tile(
        ui,
        SelectMenuTile::new("Comfortable", theme)
            .id(egui::Id::unique("density-comfortable"))
            .enabled(false),
    );
});
}
```

Activating the row is reported through its response; the caller owns the selection and dismisses the enclosing menu with `ui.close()`. Disabled rows cannot be activated and expose the disabled state to access tools. Draw rows through `MenuItems::select_menu_tile` so the menu still focuses its first enabled row when it opens; pass a stable `SelectMenuTile::id` if the menu's content can change so focus stays with the same row.

Unsupported or unverified: the upstream `FSelectMenuTile` is a tile that opens a popover containing a select tile group. That popover orchestration, the menu scroll area, dividers, intrinsic sizing, real icons, checkmark slots, long press, hover-to-focus, animation, RTL, screen-reader testing and upstream visual parity remain open. See `COVERAGE.md` for the current limitations.
