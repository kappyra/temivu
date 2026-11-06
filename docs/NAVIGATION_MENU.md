# Navigation menus

`NavigationMenu` reuses `NavigationItem` and caller-owned selection. Use one destination model for the bottom bar, a burger menu and a floating island.

```rust
# fn example(ui: &mut egui::Ui, open: &mut bool, selected: &mut Option<egui::Id>) {
use temivu::{Button, MenuPresentation, NavigationItem, NavigationMenu, Theme};
let theme = Theme::light();
let items = [NavigationItem::new(egui::Id::new("home"), "Home")];
let trigger = Button::new("Menu", &theme).show(ui);
NavigationMenu::new(egui::Id::new("main-menu"), "Navigation", open, &theme)
    .presentation(MenuPresentation::Adaptive)
    .show(&trigger, &items, selected);
# }
```

Call `show` every frame, including when closed. The trigger toggles the menu. Selection updates the supplied `Option<Id>`, closes the menu and returns the selected ID. Destination IDs and the menu ID must be unique and stable. Disabled items stay visible and cannot be selected.

- `FullScreen`: a padded modal surface filling the viewport with an outer margin.
- `Floating`: a rounded, shadowed island anchored to the trigger; `.width(...)` sets its preferred surface width, capped to available screen space.
- `Drawer`: a rounded left-edge modal drawer with a sliding spring, a dimmed backdrop and scrollable destinations.
- `Adaptive`: drawer below 640 logical points and floating otherwise.

`Close`, Escape and selection dismiss the menu. Outside clicks dismiss through egui's modal/popover handling. Keyboard Up/Down skips disabled destinations, Tab traverses controls and focus returns to the trigger after dismissal. egui supplies touch/pointer input. The drawer expands from the edge and the floating surface expands from its trigger. Both use Droplet by default and replay paint while closing. Reduced motion snaps both.

`NavigationMenu::frame(egui::Frame)` overrides padding, corners, shadow, border and fill without introducing a separate styling language. The same egui implementation is used across targets. Tests cover drawer, full-screen and floating presentations in a narrow viewport, selection, disabled semantics and Escape/focus restoration. Native mobile deployment and assistive-device testing remain separate work.

Demo: `cargo run --features demo --example showcase -- --menu` or `--island`; combine with `--mobile` for a narrow preview.

Design references: [shadcn dropdown menus](https://ui.shadcn.com/docs/components/base/dropdown-menu) for restrained surfaces and spacing; [Google Material switch implementation](https://android.googlesource.com/platform/frameworks/support/+/285557c5dd9c6a2c1422860b10a3ea3b5358b0fc/compose/material3/material3/src/commonMain/kotlin/androidx/compose/material3/Switch.kt) for separate thumb-size and position transitions. The Rust paint and spring implementation is local.

See [liquid motion](LIQUID_MOTION.md) for the transition implementation and its limits. The showcase's left hamburger always opens the drawer; Quick menu opens the island.

`NavigationMenu::liquid(LiquidStyle::droplet())` overrides the context preset; without an override it inherits `LiquidStyle::set_context_default`. Drawers keep their edge alignment. See [Droplet motion](LIQUID_MOTION.md).
