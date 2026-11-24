# Glass surfaces

Enable **Glass surfaces** on the Showcase Overview to choose the material used
by navigation and overlays. **Glass studio** compares four presets over one
photograph. Use **Move backdrop** to see the same scene pan behind all four
surfaces. Drag a sample to move its lens, or focus it and use the arrow keys.

- **Frost** diffuses fine details under an even matte tint and a fine outline.
- **Clear Frost** keeps more of the backdrop's color and silhouette under a
  lighter tint, inspired by translucent Android panels. Both frost presets
  use even diffusion. [Android window blurs](https://source.android.com/docs/core/display/window-blurs) · [Android 17 compositor source](https://android.googlesource.com/platform/frameworks/native/+/android17-release/libs/renderengine/skia/SkiaRenderEngine.cpp)
- **Soft Prism** softens high-contrast detail across the face and uses a gently
  refracting bevel. Showcase selects this preset when glass is first enabled.
- **Prism** keeps the clearer center and stronger refraction. Both prism
  presets add a thin, directional reflection over a shaded inner edge. The
  sharp and diffuse samples come from the same frame. [iOS lensing](https://developer.apple.com/videos/play/wwdc2025/219/?time=89) · [iOS 27 material refinements](https://developer.apple.com/videos/play/wwdc2026/102/?time=1363)

**Glass tint** controls the strength of the surface color. **Optical settings**
adjust blur, refraction and rim visibility for the chosen preset. High contrast
uses opaque surfaces.

The Showcase OpenGL renderer captures the visible region drawn so far in
each frame, with padding for blur and refraction. When the region permits it,
the copy starts at half resolution. Odd bounds and drivers that reject
scaled copies use an exact shader reduction with the same sampling grid.
Two separable Gaussian passes run at the selected lower resolution. The
renderer reuses three textures, grows storage in blocks, and samples sharp
and blurred detail in screen coordinates as the backdrop scrolls or the
surface moves. Foreground text and controls draw on top of the glass.

The floating navigation adds a soft scroll-edge fade through the viewport edge
and uses a wider vertical blur. During a Droplet transition, the optical mask
follows the animated bounds and corners. The panel starts with its regular solid
fill, then blends into glass over the filtered backdrop. Closing restores the
solid fill before releasing the filter.

```rust
use temivu::{Glass, GlassTuning};

fn configure_glass(ctx: &egui::Context) {
    let glass = Glass::soft_prism()
        .with_tint(0.24)
        .with_tuning(GlassTuning { blur: 1.6, refraction: 0.55, rim: 0.4 });
    Glass::set_context(ctx, Some(glass));
}
```

Use `Glass::frost()`, `Glass::clear_frost()`, `Glass::soft_prism()` and
`Glass::prism()` on individual surfaces. Clear Frost uses `GlassMode::Frost`
with its own tint and tuning, so an existing renderer can handle it through
the same callback. Nest glass surfaces to sample the outer material beneath
the inner one.

```rust
use temivu::{Glass, GlassProfile, Theme};

fn mixer(ui: &mut egui::Ui, theme: &Theme) {
    let panel = egui::Frame::new()
        .fill(theme.card)
        .inner_margin(16)
        .corner_radius(20);
    let control = egui::Frame::new()
        .fill(theme.card)
        .inner_margin(10)
        .corner_radius(16);
    Glass::frost().show(ui, theme, panel, GlassProfile::Floating, |ui| {
        Glass::prism().show(ui, theme, control, GlassProfile::Floating, |ui| {
            ui.label("Volume");
        });
        Glass::soft_prism().show(ui, theme, control, GlassProfile::Floating, |ui| {
            ui.label("Balance");
        });
    });
}
```

Install a `GlassRenderer` to sample the live scene. Pass `None` to
`Glass::set_context` to turn glass off. In a renderer adapter, use
`region.with_paint_bounds(info.viewport)` to follow egui's final callback
bounds when an overlay translates or scales. Without a live adapter, floating
transitions use the base fill and stationary navigation uses a denser
translucent surface. When you supply a `GlassBackdrop` texture to
`Glass::paint`, its filtering comes from the supplied texture. Refraction,
tint and rim settings still apply to that texture-based fallback.

For custom glass widgets, combine `.glass(Some(material))` and `.lighting(Some(style))`
on `EffectSurface`. For manually painted glass, use `Vesper::paint_on_surface`
with `LightSurface::Glass` after the glass and before text or icons.
