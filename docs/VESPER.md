# Vesper interaction lighting

Vesper takes its name from evening light. It gives interactive surfaces a localized white highlight, a continuous reflective rim, and optional accents for movement and activation. Lighting is off by default.

## Use it in an application

Enable lighting for one control, inherit it for Temivu buttons and switches in an egui context, or override it on any of those controls. A context default can be set when configuring the app and changed in response to a user preference.

```rust
use temivu::{Button, LightQuality, Switch, Theme, Vesper};

fn actions(ui: &mut egui::Ui, theme: &Theme, enabled: &mut bool) {
    let mut light = Vesper::expressive();
    light.quality = LightQuality::Low;
    light.effects.particles = true;

    Vesper::set_context(ui.ctx(), Some(light));
    Button::new("Continue", theme).show(ui);
    Button::new("Cancel", theme).lighting(None).show(ui);
    Switch::new(enabled, theme)
        .label("Automatic updates")
        .lighting(Some(Vesper::subtle()))
        .show(ui);
}
```

Call `Vesper::set_context(ctx, None)` to remove the inherited style. Explicit widget overrides apply independently. `Vesper::subtle()` is the default preset. `Vesper::expressive()` increases reach and enables spring tracking and a short trailing highlight. The individual `LightEffects` flags select pointer follow, edge light, scattering, flow, particles, and depth. Set `color` to override the neutral light, or leave it at `None` to let the light adapt to the surface. The style also exposes intensity, opacity, radius, softness, propagation speed, attenuation, inertia, and elevation.

For custom widgets, paint the material between the background and foreground. Supply the same rectangle and corners used for the background. `paint_on` also takes the actual fill so the light can distinguish a dark primary action from a pale field in the same theme. `paint` uses the theme card as its fill hint.

```rust
use temivu::{Theme, Vesper};

fn lit_surface(ui: &mut egui::Ui, theme: &Theme) {
    let (rect, response) =
        ui.allocate_exact_size(egui::vec2(180.0, 48.0), egui::Sense::click_and_drag());
    let corners = egui::CornerRadius::same(24);
    ui.painter().rect_filled(rect, corners, theme.secondary);
    Vesper::subtle().paint_on(ui.painter(), theme, &response, rect, corners, theme.secondary);
    ui.painter().text(
        rect.center(),
        egui::Align2::CENTER_CENTER,
        "Drag here",
        egui::FontId::proportional(15.0),
        theme.secondary_foreground,
    );
}
```

Use `paint_at` for a steady light at an explicit point. Repeated calls can layer additional light sources, with one mesh per call. The interactive API combines a tracked light and one activation burst in the same mesh. The Showcase’s **Overview → Vesper lighting** panel has a drag surface, lit and plain buttons, a lit switch, and a 24-surface load preview.

## Share light between nearby objects

Set `spill` on each participating material. Interactive objects emit their pointer light, and passive objects can receive light through `Vesper::receive` or `EffectSurface`. Each receiver picks the strongest nearby emitter on the same egui layer and paints inside its own shape. The UI can expose a separate preference by setting `spill` to `None` while keeping local lighting enabled.

```rust
use temivu::{EffectSurface, LightSpill, Theme, Vesper};

fn adjacent_objects(ui: &mut egui::Ui, theme: &Theme) {
    let shared = Vesper {
        spill: Some(LightSpill::default()),
        ..Vesper::expressive()
    };
    ui.horizontal(|ui| {
        EffectSurface::new(theme)
            .lighting(Some(shared))
            .show(ui, |ui| ui.label("Receiving surface"));
        temivu::Button::new("Hold here", theme)
            .lighting(Some(shared))
            .show(ui);
    });
}
```

## Design references

Huawei’s design guide describes spatiality and immersive interaction. Its HarmonyOS 7 page shows finger-following light, boundary illumination, particles and spatial cards. [HarmonyOS design guide](https://developer.huawei.com/consumer/cn/doc/best-practices/bpta-spatiality-immersive/) · [HarmonyOS 7](https://consumer.huawei.com/cn/harmonyos-7/)

Frame comparisons of four official clips informed the following decisions. Times describe the clips and do not measure device response.

| Visual reference | Observed behavior | Vesper interpretation |
| --- | --- | --- |
| [Brightness capsule](https://consumer.huawei.com/content/dam/huawei-cbg-site/cn/mkt/harmonyos-7/video/hero/highlights-1.mp4) | A compact flash is visible by roughly 0.35 s. The brightest edge then follows the changing fill. | Track normalized local coordinates and emit a short white burst on activation. |
| [Icon and folder](https://consumer.huawei.com/content/dam/huawei-cbg-site/cn/mkt/harmonyos-7/video/hero/highlights-2.mp4) | The rim facing the moving icon brightens. Around 3.9 s the highlight is above the folder, and around 5.5 s it is to the left. | Weight the continuous rounded rim by proximity to the light and the direction of its normal. |
| [Spatial weather card](https://consumer.huawei.com/content/dam/huawei-cbg-site/cn/mkt/harmonyos-7/video/hero/highlights-3.mp4) | Card motion and diffuse neighboring colors suggest depth while the foreground stays sharp. | Use a restrained directional depth accent and paint content above the light. |
| [Quick Settings](https://consumer.huawei.com/content/dam/huawei-cbg-site/cn/mkt/harmonyos-7/video/hero/highlights-4.mp4) | White flashes at roughly 0.72 s and 4.20 s are distinct from the blue selected state and the wallpaper. | Keep the light neutral, fade it after release, and leave selection colors to the control. |

In Apple’s iOS 26 material session, the contact light appears beneath a fingertip and spreads into the silhouette. The material principles sequence shows the outline separating a pale control from pale content and a colored reflection on the outer bevel over a photo. The iOS 27 UIKit session shows a stable outline and sharp foreground icons in the floating bar while content scrolls behind it. Vesper uses a fast contact core, a softer wash, and a separate rim. [iOS 26 dynamics](https://developer.apple.com/videos/play/wwdc2025/219/?time=232) · [iOS 26 material principles](https://developer.apple.com/videos/play/wwdc2025/219/?time=631) · [iOS 27 navigation and scroll edges](https://developer.apple.com/videos/play/wwdc2026/278/?time=658)

MacStories’ iOS 27 comparisons show that a darker lip can separate a white specular edge from pale content. Microsoft’s Fluent documentation distinguishes hovering, pressing, and moving while pressed. The independent LiquidGlass project publishes side-by-side recordings and separates measured behavior from its own implementation choices. These references informed the narrow shadow on pale fills, the distinct hover and press strengths, and the separation of Vesper light from Prism backdrop refraction. [MacStories design review](https://www.macstories.net/stories/ios-and-ipados-27-review/2/) · [Fluent pointer animations](https://learn.microsoft.com/en-us/windows/apps/develop/composition/pointer-input-animations) · [LiquidGlass comparison project](https://github.com/shayann07/liquidglass)

A bounded distance falloff was selected after comparing physically based attenuation and UI rendering options. The egui mesh path carries vertex colors through the existing renderer. Backend-specific callbacks and framebuffer passes would add integration cost for this effect. [Filament rendering reference](https://google.github.io/filament/Filament.html) · [egui mesh](https://docs.rs/egui/0.36.2/egui/struct.Mesh.html) · [egui paint callback](https://docs.rs/egui/0.36.2/egui/struct.PaintCallback.html)

## Rendering and motion

The interaction state is keyed by egui’s widget ID. Pointer positions are normalized so resizing a surface preserves the light’s relative location. Direct tracking follows the input immediately. Smooth tracking uses a time-based lag and spring tracking reuses Temivu’s spring. The contact highlight follows a held mouse button or touch, including after egui recognizes a long press. A hover-only surface can observe the contact while its children retain their input behavior. On release, the light settles to the hover level or fades when the pointer leaves. Keyboard focus can show a centered cue. Reduced motion makes the response immediate and skips the animated burst and flow.

The renderer builds one bounded vertex-color mesh for each active surface. The core and expanding wash use polynomial falloff on squared distances. The grid adds samples near a narrow light on wide surfaces, and the beveled rim samples straight sections in proportion to their length. On pale fills, a shaded gradient and inner rim make the white contact core visible. Dark fills use the white highlight directly. The mesh uses two pre-sized vectors and bounds particle counts to four, six, or eight per burst at Low, Balanced, and High. The spill resolver uses the Low mesh density, checks at most 16 emitters and 256 receivers per pass, and requests no repaints of its own. Controls draw the lighting meshes behind their text and indicators.

## Verification and measurements

The targeted suite covers rounded geometry, composited contrast on pale and dark fills, stationary and moving contact during mouse and touch holds, release and focus behavior, reduced motion, spill clipping, and opt-out.

The table records the earlier gradient optimization on an Intel Core i7-1165G7 with all local effects active at Balanced quality. Three alternating runs per revision used 500 batches each. Each renderer was compiled with `rustc -O` and linked against the same development dependencies. The optimized renderer emitted 356 vertices per widget in this fixture, down from 382. The unlit path emits no lighting mesh.

| Active widgets | Original median per batch | Optimized median per batch |
| ---: | ---: | ---: |
| 1 | 30.0 µs | 19.0 µs |
| 32 | 0.94 ms | 0.68 ms |
| 128 | 3.82 ms | 2.54 ms |

Run the same fixture under Cargo’s release profile with `cargo test --release --lib lighting::render::tests::measure_mesh_generation -- --ignored --nocapture`. These measurements cover CPU mesh construction. They exclude GPU time, full frame time, energy use, and Android device performance.

## Scope and trade-offs

Flow and particles are optional expressive accents. The depth treatment is directional shading on each surface. Automatic device-based quality selection and a 3D material model are possible extensions. Applications can select Low explicitly for tighter rendering budgets.

The light is drawn through egui’s ordinary mesh path without adding dependencies or requiring a glass material. Dense interfaces, high-DPI rendering, and custom light colors can change the cost and perceived contrast. Use the Showcase’s stress preview to compare quality settings on target devices.
