# Droplet motion

Droplet is Temivu's default for opening, closing, and resizing surfaces. Pages,
dialogs, popovers, drawers, sheets, notifications and inline resizing share its
spring. Compact surfaces grow from their trigger. Anchored popovers can form a
liquid neck as they separate or rejoin.

The default nominal spring duration is 374 ms for opening and 307 ms for
closing. Reduced motion snaps to the destination. The older Bloom preset remains
available as an explicit library override for floating widgets.

## Apple references

Apple's [custom-view documentation](https://developer.apple.com/documentation/swiftui/applying-liquid-glass-to-custom-views) describes shared containers, stable effect identity and spacing-dependent merging/separation. [matchedGeometry](https://developer.apple.com/documentation/swiftui/glasseffecttransition/matchedgeometry) derives an appearing/disappearing shape from a nearby shape and describes separate content scale/offset behavior. [Meet Liquid Glass](https://developer.apple.com/videos/play/wwdc2025/219/) demonstrates the interaction, including a menu bubble opening from its control. These public mechanics inform Droplet's movement; no different, independently verified iOS 27 algorithm is claimed here.

The frame review used the roughly 30 fps WWDC25 recordings. In the Landmarks opening around 20:44.2, an elongated silhouette appears before the individual badges become readable. In the closing around 21:12.6, the symbols fade while the silhouette is still retracting. The UIKit merge around 24:31.7 begins where two moving contours meet. The menu around 5:08 in Meet Liquid Glass grows along the opening direction before settling into its full width. These timings identify sections of the recordings. Temivu uses its own spring and phase mapping for popup and group transitions. [Landmarks opening](https://developer.apple.com/videos/play/wwdc2025/323/?time=1244) · [Landmarks closing](https://developer.apple.com/videos/play/wwdc2025/323/?time=1272) · [UIKit merging](https://developer.apple.com/videos/play/wwdc2025/284/?time=1471) · [Menu opening](https://developer.apple.com/videos/play/wwdc2025/219/?time=307)

The independent [GetStream menu recording and SwiftUI implementation](https://github.com/GetStream/awesome-liquid-glass#liquid-glass-menu-liquidglassmenuswift) shows the same separation between panel movement and content. [JuniperPhoton’s iPhone testing](https://juniperphoton.substack.com/p/adopting-liquid-glass-experiences) identified a corner-shape snap during the system menu’s handoff in iOS 26.0 and different behavior in 26.1. [MacStories’ iOS 27 review](https://www.macstories.net/stories/ios-and-ipados-27-review/2/) documents improved context-menu edge contrast. Temivu keeps a temporary contour as the two surfaces separate so the outline remains visible in a monochrome theme.

The local iOS 27 recordings show three distinct poses. Compact menus grow out of the trigger's own bounds and take its place. Tall menus lengthen along the opening direction before reaching their final width, with the labels becoming readable later. Centered alerts enter with a short scale and fade in place. Temivu uses the menu recordings to shape anchored motion and gives centered dialogs a wider expansion.

The source contains an analytic velocity-preserving spring, not a fluid simulator. New targets begin at the input event, so idle time cannot skip the next opening. Long frames with an unchanged target still advance by elapsed time. Reduced motion, zero egui animation duration or disabled motion snaps to the final state. Settled transitions stop requesting repaints.

## Configure floating widgets

`Popover`, `Select`, `MultiSelect`, `Autocomplete`, `DateField`, `Dialog` and
`NavigationMenu` use Droplet by default. For those widgets, pass a
`LiquidStyle` through `.liquid(...)` or set a context-wide preset with
`LiquidStyle::set_context_default`. The style controls the gap, blend radius,
spring duration and damping. Duration scales with `Theme::motion.normal`.

`Popover::frame` and `Dialog::frame` accept an egui frame. The default gap is
eight logical points. Compact command menus take the trigger's place as they
expand.

## Geometry and content

Bloom uses an ordinary egui frame. Droplet starts with a seed inside the trigger and grows into a single contour with the trigger before separating. The paint over the old trigger border follows its rounded shape and feathers inward to preserve the label. Neighboring mesh segments share the same field normals for antialiasing. The panel's shadow develops near separation. Adaptive tiles skip the regions unaffected by the union and refine the outline. Once the source field is out of range, egui draws the panel with the same fractional bounds and radii.

Droplet moves the contents with the panel and fits them within its rounded outline. The label opacity follows both the spring phase and the available space. Bloom keeps text near its final size and clips the faded contents to the expanding frame. Droplet reveals text after the first stretch and fades it early during collapse. Native popup layout remains at its final bounds. The trigger uses stable layout bounds throughout the transition. Pointer and touch activation set the horizontal origin for opening and closing. Keyboard activation starts in the center. Logical close immediately removes live controls while cached paint completes the exit.

First/reopened Area sizing passes request a same-frame egui discard so invisible sizing shapes cannot overwrite the exit snapshot. Settled frame bounds already include padding; painting does not add it twice.

A centered dialog expands from a small rounded surface near the viewport center and reveals its content as it grows. A dialog with `Dialog::source_response(&response)` follows the Droplet spring in both directions. With a separated trigger, it uses a liquid neck. If the final window overlaps its trigger, one silhouette expands from the trigger bounds and retracts into them. Both paths move and reveal the dialog contents with the surface. Drawers and bottom sheets expand from the edge they are anchored to. Toasts expand from a small rounded seed at the bottom of the card. `Dialog::area` reuses egui positioning. Call these controls every frame, including when closed.

## Review and limits

In Showcase, click the left, middle or right of **Preview Droplet from any
point** to choose where the popup grows from. Use **Widgets → Motion → Spring
resize** for inline resizing. Showcase pages switch immediately.
Use **Reduced motion** on Overview to compare the accessible snap behavior.

Popup width is viewport-bounded; egui chooses an available side of the trigger. Calendar width includes its padding and the drawer has bounded width and scrolling. The development profile optimizes egui/epaint for responsive layout/tessellation; test executables strip debug symbols to avoid temporary-space exhaustion. No new renderer dependency or unsafe code is added.

Tests cover above/below mirrored Droplet geometry, finite/bounded meshes, source/final bounds, retained text size, inherited defaults and explicit overrides, mode selection/preview, closing callbacks, selection/focus and idle spring timing. These are functional and geometry checks, not a guarantee of native display FPS, exact Apple parity or mobile/browser deployment performance. `--mobile` is a desktop viewport preview. `--no-vsync` remains an explicit workaround for presentation stalls observed in this unfocused X11 session; normal vsync is the default.

## Combine several surfaces

Use a shared spring phase for all members of a Droplet group. Two to eight rounded surfaces can join into one frame and separate again. Keep the group's ID and source bounds stable while the animation runs.

```rust
use temivu::{Theme, liquid::{self, DropletPart, LiquidStyle}};

fn paint_group(
    ui: &egui::Ui,
    theme: &Theme,
    parts: &[DropletPart],
    target: egui::Rect,
    merged: bool,
) {
    let style = LiquidStyle::droplet();
    let phase = liquid::droplet_group_progress(
        ui.ctx(), ui.make_persistent_id("view-options"), merged, style, theme,
    );
    let frame = egui::Frame::new().fill(theme.muted).corner_radius(28);
    liquid::paint_droplet_group(ui.painter(), parts, target, phase, frame, style);
}
```

Use `droplet_part_rect` with that same phase to position member contents. The group paints one shared fill and border. The caller owns hit targets, accessibility and the handoff between individual and combined controls. See **Overlays → Droplet group** in Showcase.

## Resize content with Droplet

Use `liquid::DropletResize` for a panel or custom widget. Use the same instance
for layout and paint on a given frame so the outline follows the size spring.

```rust
use temivu::{Theme, liquid::DropletResize};

fn preview(ui: &mut egui::Ui, theme: &Theme, expanded: bool) {
    let available = ui.available_size();
    let resize = DropletResize::new(
        ui.ctx(),
        ui.make_persistent_id("preview"),
        egui::vec2(160.0, 80.0).min(available),
        egui::vec2(280.0, 140.0).min(available),
        expanded,
        theme,
    );
    let (rect, _) = ui.allocate_exact_size(resize.size, egui::Sense::hover());
    resize.paint(ui.painter(), rect, egui::Frame::new().fill(theme.muted).corner_radius(20));
}
```

`motion::spring_size` remains available for layout-only changes. Press
feedback, direct drag tracking and continuous indicators use their existing
interaction motion.
