# Floating surfaces and responsive geometry

The interface groups controls, transient menus, panels, and content by role. Use a capsule for narrow interactive controls. Menus and panels use tighter corners and enough inset for wrapped text. Maintain the same surface through an interaction so that the fill, outline, and light follow the visible geometry.

## Build a custom surface

Use `EffectSurface` for arbitrary egui content. Set the frame to control
padding, corners, stroke, fill, and elevation. Both glass and lighting are
optional and can be set per surface. `SurfaceShape` supplies a size-aware
corner radius when you want to share the library's defaults.

```rust
use temivu::{EffectSurface, FloatingMotion, Glass, LightSpill, SurfaceShape, Theme, Vesper};

fn toolbar(ctx: &egui::Context, theme: &Theme, open: bool, spread: bool) {
    let frame = egui::Frame::new()
        .corner_radius(SurfaceShape::Control.radius(egui::vec2(180.0, 52.0)))
        .fill(theme.card)
        .stroke(egui::Stroke::new(1.0, theme.border))
        .inner_margin(8);
    let light = Vesper {
        spill: spread.then(LightSpill::default),
        ..Vesper::expressive()
    };
    FloatingMotion::lift().show(
        ctx,
        egui::Id::new("floating-toolbar"),
        (egui::Align2::CENTER_BOTTOM, egui::vec2(0.0, -16.0)),
        open,
        EffectSurface::new(theme)
            .frame(frame)
            .elevated()
            .glass(Glass::context(ctx))
            .lighting(Some(light)),
        |ui| temivu::Button::new("New item", theme).show(ui),
    );
}
```

Use `EffectSurface::show` for an inline container. `FloatingMotion::lift`
moves a foreground area into its anchored position. `FloatingMotion::flight`
takes a custom entrance offset. Call it every frame, even when closing.
The area follows the safe content bounds and disables its content's input
during exit. The shared spring respects reduced motion.

Live blur requires a `GlassRenderer` supplied by the host. The frame falls back
to a legible tint without it. Custom-painted shapes can use `Glass::paint`
and `Vesper::paint_on` directly. For a source-connected overlay use `Popover`
or `Dialog` with a custom frame, or paint the same geometry with
[`DropletAnchor`](LIQUID_MOTION.md).

## Adaptivity and reference notes

- Phone layout uses the safe content rectangle and retains a margin between
  floating controls and the screen edge. Narrow navigation can reduce its
  visual labels while retaining accessible names.
- Navigation uses a scrollable side drawer with grouped destination surfaces
  and a compact quick menu with flat rows.
- The localization preview groups locale selection and translated controls,
  provides a compact pager, and embeds small fallback font subsets for its
  Japanese, Simplified Chinese, and Arabic samples.

The implementation draws on the video demonstrations of floating controls,
anchored menus, responsive layout, and concentrated interaction highlights.
The supplied phone recording was reviewed locally. Its fixed menu contour
and independently scrolling rows guided the menu layout. [Apple WWDC25](https://developer.apple.com/videos/play/wwdc2025/219/) · [Apple WWDC26](https://developer.apple.com/videos/play/wwdc2026/278/) · [HarmonyOS 7](https://consumer.huawei.com/cn/harmonyos-7/)
