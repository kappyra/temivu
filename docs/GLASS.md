# Glass surfaces

Glass starts off. Enable **Glass surfaces** on the Showcase Overview. **Glass
studio** compares the two materials over the same scene, while the selector
chooses which material to use for navigation and overlays.

- **Frost glassmorphism** uses a matte tint and even background diffusion.
- **Prism optical glass** uses a clearer center and a directional highlight.
  A continuous displacement bends the blurred background along the bevel.

**Glass tint** controls the strength of the surface color. High contrast uses
opaque surfaces.

The native renderer copies only the visible region beneath a floating surface,
with a small sampling margin. It reuses two textures of at most 512 by 512
pixels. Horizontal and vertical Gaussian passes filter that reduced-resolution
region before the full-resolution surface samples it. Prism displaces the
sample once using the contour normal. Text and controls are drawn on top.

The floating navigation adds a soft scroll-edge fade and uses a wider vertical
blur. During a Droplet transition, the optical mask follows the panel's
animated bounds and corners. The panel starts with its regular solid fill.
When the backdrop can be filtered, the fill blends into glass over the
filtered pixels. Closing restores the solid fill before releasing the filter.

To enable the material in an application, call
`Glass::set_context(ctx, Some(Glass::new(0.4).mode(GlassMode::Prism)))`.
Install a `GlassRenderer` for the host renderer to sample the live scene.
Use `Glass::set_context(ctx, None)` to turn glass off. Without a live adapter,
animated floating widgets keep their base fill, and the stationary navigation
uses a denser translucent surface. For an app-owned image, `Glass::paint` can
take a prefiltered texture through `GlassBackdrop`.
