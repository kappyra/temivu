# Select tile group (experimental)

A labelled group of selectable tiles with multi-selection stored in a caller-owned `Vec` of stable tile IDs.

```rust
use temivu::{SelectTile, SelectTileGroup, Theme};

fn example(ui: &mut egui::Ui, theme: &Theme, selected: &mut Vec<egui::Id>) {
let tiles = [
    SelectTile::new(egui::Id::new("wifi"), "Wi-Fi").description("Wireless networks"),
    SelectTile::new(egui::Id::new("bluetooth"), "Bluetooth"),
    SelectTile::new(egui::Id::new("nfc"), "NFC").enabled(false),
];
let group = SelectTileGroup::new(
    egui::Id::new("radios"),
    "Radios",
    theme,
    &tiles,
    selected,
)
.description("Choose any that apply")
.show(ui);
if group.changed {
    // `selected` already contains the updated selection.
}
}
```

A click, Enter or Space toggles the focused tile's membership in `selected`; the group reports the activated tile and whether the selection changed this frame. Passing the same vector every frame keeps selection attached to stable tile IDs across reordering. The container exposes the list role with its label, size and description; each tile exposes the list-item role with position, set size, selected state and description. Disabled tiles stay visible and cannot be toggled, and a disabled group (or a disabled parent UI) disables every tile. Keyboard: Up/Down move focus between enabled tiles, Home/End jump to the first/last enabled tile and Enter/Space toggle the focused tile.

Unsupported or unverified: the upstream `FSelectTileGroup` is a form field with scrolling, dividers, intrinsic width, long press, swipe gestures, animation, error transition and form validation. Those, plus real icons, checkmark slots, RTL, screen-reader testing and upstream visual parity remain open. See `COVERAGE.md` for the current limitations.
