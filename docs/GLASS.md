# Glass surfaces

Enable **Glass surfaces** on the Showcase Overview to choose the material used
by navigation and overlays. **Glass studio** puts Frost and the selected Prism
preset over the same scene. Drag either sample, or focus it and use the arrow
keys, to see how it changes the background while moving.

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

The native renderer captures the visible region beneath each surface with a
small sampling margin. It reuses two textures of at most 512 by 512 pixels.
A pair of reduced-resolution Gaussian passes diffuses the region when blur
is enabled. Prism displaces a single sample along the normal of the current
rounded edge. A white highlight and a subtle opposite edge shadow define the
bevel. As the glass moves, the renderer samples the scene in screen coordinates
and draws text and controls on top.

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

Install a `GlassRenderer` to sample the live scene. Pass `None` to
`Glass::set_context` to turn glass off. Without a live adapter, floating
transitions use the base fill and stationary navigation uses a denser
translucent surface. When you supply a `GlassBackdrop` texture to
`Glass::paint`, its filtering comes from the supplied texture. Refraction,
tint and rim settings still apply to that texture-based fallback.
