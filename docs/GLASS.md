# Glass surfaces

Enable **Glass surfaces** on the Showcase Overview to choose the material used
by navigation and overlays. **Glass studio** shows Frost, Soft Prism and Prism
over matching crops of the same generated street photograph. Pull and release
a sample to compare the filters in motion. Focus a sample and use the arrow
keys to change its resting position.

- **Frost** uses even diffusion and a translucent matte tint, with a fine
  outline. The volume controls and other translucent panels in Android 17
  are visual references for this material. [Android window blur guidance](https://source.android.com/docs/core/display/window-blurs) · [Android 17 panel screenshots](https://9to5google.com/2026/05/19/android-17-beta-1-blur/)
- **Soft Prism** uses stronger diffusion and gentler refraction along the
  bevel. Showcase selects this preset when glass is first enabled.
- **Prism** keeps a clearer center and makes the polished bevel more visible.
  It is the library's default material. [Apple's lensing design reference](https://developer.apple.com/videos/play/wwdc2025/219/)

**Glass tint** controls the strength of the surface color. **Optical settings**
adjust blur, refraction and rim visibility for the chosen preset. High contrast
uses opaque surfaces.

The Showcase's OpenGL renderer captures the visible region beneath each
surface at framebuffer resolution, with a margin for blur and refraction. It
reuses one capture texture and two work textures. When the blur radius calls
for a lower resolution, successive 2:1 reductions smooth the region before
two separable Gaussian passes. The filter samples neighboring texels along
each axis. Prism bends the result along the normal of the current rounded
edge and adds a narrow reflective rim. As the glass moves, the renderer
samples the scene at its current screen position. Text and controls remain
sharp. [Arm's multiscale filtering notes](https://developer.arm.com/cfs-file/__key/communityserver-blogs-components-weblogsfiles/00-00-00-20-66/siggraph2015_2D00_mmg_2D00_marius_2D00_notes.pdf)

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

For per-surface materials, use `Glass::frost()`, `Glass::soft_prism()` and
`Glass::prism()` in the same frame. Each surface has its own tint and
optical tuning. To layer materials, nest the surfaces so the live renderer
can sample the outer material beneath the inner one.

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
`Glass::set_context` to turn glass off. Without a live adapter, floating
transitions use the base fill and stationary navigation uses a denser
translucent surface. When you supply a `GlassBackdrop` texture to
`Glass::paint`, its filtering comes from the supplied texture. Refraction,
tint and rim settings still apply to that texture-based fallback.
