# Vesper interaction lighting

Vesper takes its name from evening light. It gives interactive surfaces a localized tint, a continuous rim highlight, and optional accents for movement and activation. Lighting is off by default.

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

Call `Vesper::set_context(ctx, None)` to remove the inherited style. Explicit widget overrides apply independently. `Vesper::subtle()` is the default preset. `Vesper::expressive()` increases reach and enables spring tracking and surface flow. The individual `LightEffects` flags select pointer follow, edge light, scattering, flow, particles, and depth. The style exposes tint, intensity, opacity, radius, softness, propagation speed, attenuation, inertia, and elevation.

For custom widgets, paint the material between the background and foreground. Supply the same rectangle and corners used for the background. Use the response returned by the widget to share its pointer and keyboard interaction.

```rust
use temivu::{Theme, Vesper};

fn lit_surface(ui: &mut egui::Ui, theme: &Theme) {
    let (rect, response) =
        ui.allocate_exact_size(egui::vec2(180.0, 48.0), egui::Sense::click_and_drag());
    let corners = egui::CornerRadius::same(24);
    ui.painter().rect_filled(rect, corners, theme.secondary);
    Vesper::subtle().paint(ui.painter(), theme, &response, rect, corners);
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

## Design references

Huawei’s design guide describes spatiality and immersive interaction. Its HarmonyOS 7 page shows finger-following light, boundary illumination, particles and spatial cards. [HarmonyOS design guide](https://developer.huawei.com/consumer/cn/doc/best-practices/bpta-spatiality-immersive/) · [HarmonyOS 7](https://consumer.huawei.com/cn/harmonyos-7/)

Frame comparisons of four official clips informed the following decisions. Times describe the clips and do not measure device response.

| Visual reference | Observed behavior | Vesper interpretation |
| --- | --- | --- |
| [Brightness capsule](https://consumer.huawei.com/content/dam/huawei-cbg-site/cn/mkt/harmonyos-7/video/hero/highlights-1.mp4) | A compact burst appears around the interaction by roughly 0.4 s. Illumination follows the changing fill and reverses with the gesture. | Track normalized local coordinates and emit a short burst on activation. |
| [Icon and folder](https://consumer.huawei.com/content/dam/huawei-cbg-site/cn/mkt/harmonyos-7/video/hero/highlights-2.mp4) | The bright part of the rim moves toward the displaced icon and relaxes after it returns. | Project the response onto a continuous rounded rim and keep its position synchronized with press geometry. |
| [Spatial weather card](https://consumer.huawei.com/content/dam/huawei-cbg-site/cn/mkt/harmonyos-7/video/hero/highlights-3.mp4) | Card motion and diffuse neighboring colors suggest depth while the foreground stays sharp. | Use a restrained directional depth accent and paint content above the light. |
| [Quick Settings](https://consumer.huawei.com/content/dam/huawei-cbg-site/cn/mkt/harmonyos-7/video/hero/highlights-4.mp4) | Brief local flashes accompany activation. A softer tint follows while labels and control outlines remain clear. | Give press bursts and sustained light separate envelopes. |

Apple’s material session and SwiftUI documentation describe separating functional controls from content and applying effects to custom surfaces. Its spring session shows why maintaining velocity helps transitions stay continuous when interrupted. Vesper uses Temivu’s existing spring and press geometry. [Material session](https://developer.apple.com/videos/play/wwdc2025/219/) · [SwiftUI custom surfaces](https://developer.apple.com/documentation/swiftui/applying-liquid-glass-to-custom-views) · [Spring animation session](https://developer.apple.com/videos/play/wwdc2023/10158/?time=1117)

MacStories’ iOS 27 screenshots offered a separate comparison for material contrast and highlight placement. The motion observations above come from the official Huawei clips. Independent HarmonyOS video streams were unavailable for frame comparison. [MacStories design review](https://www.macstories.net/stories/ios-and-ipados-27-review/2/)

A bounded distance falloff was selected after comparing physically based attenuation and UI rendering options. The egui mesh path carries vertex colors through the existing renderer. Backend-specific callbacks and framebuffer passes would add integration cost for this effect. [Filament rendering reference](https://google.github.io/filament/Filament.html) · [egui mesh](https://docs.rs/egui/0.36.2/egui/struct.Mesh.html) · [egui paint callback](https://docs.rs/egui/0.36.2/egui/struct.PaintCallback.html)

## Rendering and motion

The interaction state is keyed by egui’s widget ID. Pointer positions are normalized so resizing a surface preserves the light’s relative location. Direct tracking follows the input immediately. Smooth tracking uses a time-based lag. Spring tracking reuses Temivu’s spring. Release and leaving the surface decay the light, and reduced motion makes the response immediate and removes the animated burst and flow.

The renderer builds one bounded vertex-color mesh for each active surface. Its interior grid adds samples near the sources, its rim follows the rounded outline, and particles are reconstructed analytically from time and initial velocity. The mesh uses two pre-sized vectors and bounds particle counts to four, six, or eight per burst at Low, Balanced, and High. Unlit controls skip mesh generation. Repaints requested by the effect stop when motion settles. Button and switch integration puts the mesh behind foreground content, with the button mesh sharing the press deformation and the switch mesh remaining on the track as the thumb moves.

## Verification and measurements

The targeted suite covers rounded geometry and corner coverage on wide and tall surfaces, the idle allocation path, pointer tracking and release, reduced motion, widget overrides, and Showcase interactions at 390 points. The visual check used eframe’s Linux renderer. It caught a gap between the lighting mesh and the rounded corner on larger surfaces. The renderer now samples the corner arcs directly.

The table shows median CPU time to generate meshes with all effects active at Balanced quality on an Intel Core i7-1165G7. Three runs of 500 batches each used the actual renderer compiled with `rustc -O` and linked against existing development dependencies. The fixture generated 382 vertices per active widget. The unlit path emits no lighting mesh.

| Active widgets | Median per batch | Observed range |
| ---: | ---: | ---: |
| 1 | 19.6 µs | 19.4–19.9 µs |
| 32 | 0.60 ms | 0.59–0.60 ms |
| 128 | 2.43 ms | 2.38–2.52 ms |

Run the same fixture under Cargo’s release profile with `cargo test --release --lib lighting::render::tests::measure_mesh_generation -- --ignored --nocapture`. These measurements cover mesh construction only. GPU time, full frame time, energy use, and Android device performance were outside the measurement. The signed Android package was verified for both bundled architectures. Runtime testing was performed on Linux.

## Scope and trade-offs

The public APIs, rounded rendering, response tracking, and quality settings are implemented. Flow and particles are optional expressive accents. The depth treatment is directional shading on each surface. Automatic device-based quality selection, shared lighting between adjacent surfaces, and a 3D material model are possible extensions. Applications can select Low explicitly for tighter rendering budgets.

The light is drawn through egui’s ordinary mesh path without adding dependencies or requiring a glass material. Dense interfaces, high-DPI rendering, and custom light colors can change the cost and perceived contrast. Use the Showcase’s stress preview to compare quality settings on target devices.
