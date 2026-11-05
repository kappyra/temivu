# Bloom and Droplet

Temivu has two independently selectable opening/closing animations. They use the application's normal shadcn surfaces; neither adds optical glass, refraction or lighting effects.

| Preset | Movement | Base timing |
| --- | --- | --- |
| **Bloom** | The original softly expanding bubble, growing from the whole trigger and collapsing back. | 320 ms nominal spring, 0.82 damping |
| **Droplet** | Compact command menus grow in place. Other anchored panels stretch from the trigger, narrow at the join and separate. | 340 ms nominal opening spring, 279 ms nominal exit, 0.77 damping |

Bloom remains the default. The curves are tuned for Temivu and its egui renderer.

## Apple references

Apple's [custom-view documentation](https://developer.apple.com/documentation/swiftui/applying-liquid-glass-to-custom-views) describes shared containers, stable effect identity and spacing-dependent merging/separation. [matchedGeometry](https://developer.apple.com/documentation/swiftui/glasseffecttransition/matchedgeometry) derives an appearing/disappearing shape from a nearby shape and describes separate content scale/offset behavior. [Meet Liquid Glass](https://developer.apple.com/videos/play/wwdc2025/219/) demonstrates the interaction, including a menu bubble opening from its control. These public mechanics inform Droplet's movement; no different, independently verified iOS 27 algorithm is claimed here.

The frame review used the roughly 30 fps WWDC25 recordings. In the Landmarks opening around 20:44.2, an elongated silhouette appears before the individual badges become readable. In the closing around 21:12.6, the symbols fade while the silhouette is still retracting. The UIKit merge around 24:31.7 begins where two moving contours meet. The menu around 5:08 in Meet Liquid Glass grows along the opening direction before settling into its full width. These timings identify sections of the recordings. Temivu uses its own spring and phase mapping for a single popup. [Landmarks opening](https://developer.apple.com/videos/play/wwdc2025/323/?time=1244) · [Landmarks closing](https://developer.apple.com/videos/play/wwdc2025/323/?time=1272) · [UIKit merging](https://developer.apple.com/videos/play/wwdc2025/284/?time=1471) · [Menu opening](https://developer.apple.com/videos/play/wwdc2025/219/?time=307)

The independent [GetStream menu recording and SwiftUI implementation](https://github.com/GetStream/awesome-liquid-glass#liquid-glass-menu-liquidglassmenuswift) shows the same separation between panel movement and content. [JuniperPhoton’s iPhone testing](https://juniperphoton.substack.com/p/adopting-liquid-glass-experiences) identified a corner-shape snap during the system menu’s handoff in iOS 26.0 and different behavior in 26.1. [MacStories’ iOS 27 review](https://www.macstories.net/stories/ios-and-ipados-27-review/2/) documents improved context-menu edge contrast. Temivu keeps a temporary contour as the two surfaces separate so the outline remains visible in a monochrome theme.

The local iOS 27 recordings show three distinct poses. Compact menus grow out of the trigger's own bounds and take its place. Tall menus lengthen along the opening direction before reaching their final width, with the labels becoming readable later. Centered alerts enter with a short scale and fade in place. Temivu uses the menu recordings to shape anchored motion and gives centered dialogs a wider expansion.

The source contains an analytic velocity-preserving spring, not a fluid simulator. New targets begin at the input event, so idle time cannot skip the next opening. Long frames with an unchanged target still advance by elapsed time. Reduced motion, zero egui animation duration or disabled motion snaps to the final state. Settled transitions stop requesting repaints.

## Choose a preset

Set the default for one egui context, normally during application setup or when the user's preference changes:

```rust
# fn setup(ctx: &egui::Context) {
use temivu::liquid::LiquidStyle;
LiquidStyle::droplet().set_context_default(ctx);
# }
```

Unconfigured floating controls inherit that setting. Explicit widget configuration takes precedence:

```rust
# fn example(ui: &mut egui::Ui, options: &[temivu::SelectOption<'_>], selected: &mut Option<egui::Id>) {
use temivu::{Select, liquid::LiquidStyle};
Select::new(egui::Id::new("color"), "Color", "Choose a color", options, selected)
    .liquid(LiquidStyle::bloom())
    .show(ui);
# }
```

`Popover`, `Select`, `MultiSelect`, `Autocomplete`, `DateField`, `Dialog` and `NavigationMenu` expose `.liquid(...)`. `LiquidStyle` also exposes enabled state, transition kind, gap, blend radius, duration and damping. Duration scales with `Theme::motion.normal`; numeric inputs are bounded and nonfinite values use safe defaults. `SurfaceTransition::Expand` and `Merge` remain compatibility constants for Bloom and Droplet.

`Popover::frame` and `Dialog::frame` reuse egui fill, padding, border, corners and shadow configuration. The settled gap is 12 logical points in Bloom and 8 in Droplet. Compact triggers in Droplet command menus sit under the opened panel so it can take their place.

## Geometry and content

Bloom uses an ordinary egui frame. Droplet starts with a seed inside the trigger and grows into a single contour with the trigger before separating. The paint over the old trigger border follows its rounded shape and feathers inward to preserve the label. Neighboring mesh segments share the same field normals for antialiasing. The panel's shadow develops near separation. Adaptive tiles skip the regions unaffected by the union and refine the outline. Once the source field is out of range, egui draws the panel with the same fractional bounds and radii.

Droplet moves the contents with the panel and fits them within its rounded outline. The label opacity follows both the spring phase and the available space. Bloom keeps text near its final size and clips the faded contents to the expanding frame. Droplet reveals text after the first stretch and fades it early during collapse. Native popup layout remains at its final bounds. The trigger uses stable layout bounds throughout the transition. Logical close immediately removes live controls while cached paint completes the exit.

First/reopened Area sizing passes request a same-frame egui discard so invisible sizing shapes cannot overwrite the exit snapshot. Settled frame bounds already include padding; painting does not add it twice.

A centered dialog expands from a small rounded surface near the viewport center and reveals its content as it grows. A `Dialog::source(rect)` outside its final surface can use the anchored motion. When the source already overlaps the final window, a zoom remains appropriate instead of drawing a neck through the window. Drawers retain their directional slide with the chosen response curve. `Dialog::area` reuses egui positioning. Call these controls every frame, including when closed.

## Review and limits

In the showcase's **Controls** page, select **Bloom** or **Droplet**, then press **Preview** and **Done**. The selected context preset also applies to the normal selects, autocomplete, calendar and floating menus. Start with `--controls --droplet` to preselect Droplet for review. Bloom remains available without altering its accepted movement. Keyboard and disabled/reduced-motion behavior use the same controls in both modes.

Popup width is viewport-bounded; egui chooses an available side of the trigger. Calendar width includes its padding and the drawer has bounded width and scrolling. The development profile optimizes egui/epaint for responsive layout/tessellation; test executables strip debug symbols to avoid temporary-space exhaustion. No new renderer dependency or unsafe code is added.

Tests cover above/below mirrored Droplet geometry, finite/bounded meshes, source/final bounds, retained text size, inherited defaults and explicit overrides, mode selection/preview, closing callbacks, selection/focus and idle spring timing. These are functional and geometry checks, not a guarantee of native display FPS, exact Apple parity or mobile/browser deployment performance. `--mobile` is a desktop viewport preview. `--no-vsync` remains an explicit workaround for presentation stalls observed in this unfocused X11 session; normal vsync is the default.
