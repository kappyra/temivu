# Temivu motion system

Temivu uses shared motion tracks for interactive feedback, geometry, and status.
The track chooses how a property changes. A widget owns its input, geometry,
painting, and accessibility. This lets a drag follow the pointer precisely while
its release transfers velocity to a spring.

## Use a track

Use one stable egui ID per property. Reuse the same ID and value type while that
property exists. A new track starts at its explicit initial value. Pass the
target as the initial value if first paint should appear settled.

```rust
use temivu::{Theme, motion::{MotionSpec, MotionTrack, SpringPreset}};

fn marker(ui: &egui::Ui, theme: &Theme, origin: egui::Pos2, target: egui::Pos2) {
    let track = MotionTrack::new(ui.ctx(), ui.make_persistent_id("marker position"), theme.motion);
    let spec = MotionSpec::Spring(SpringPreset::Smooth.spring(theme.motion));
    let pose = track.sample(origin, target, spec);
    ui.painter().circle_filled(pose.value, 6.0, theme.primary);
}
```

The built-in values are `f32`, `f64`, `Vec2`, `Pos2`, `Rect`, and
`Color32`. Use `f32` for opacity and `Vec2` for dimensions. Color interpolation
uses linear-light premultiplied channels. Rectangles stay ordered. A
`MotionValue<N>` implementation can add another numeric representation. Use
`MotionSample::running` to detect settling and `velocity` for release or
coordination. `inspect` samples without scheduling a frame. `reset::<N>()`
forgets the property. Clamp bounded values such as opacity at the paint site.

`follow(value, velocity)` records a direct gesture pose. On release, sample
a spring toward a detent or `MotionSpec::Decay` toward the point returned by
`Decay::destination`. Clamping the projected destination limits travel and
reduces release speed accordingly. Use `rubber_band` only for visual overflow.
Leave hit targets at their logical positions. Existing `spring_*`,
`animate_*` and Droplet helpers keep their signatures. Code that passed
ratios above one to the legacy spring helpers now gets actual overdamping.

## Rhythm and accessibility

| Role | Treatment |
| --- | --- |
| Feedback | Fast ease-out for opacity. Responsive spring for press and release. |
| State | Normal theme duration for color and indicators. Prefer a spring when a target can change mid-gesture. |
| Navigation | Slow emphasized timing for an explicit transition. Direct page changes remain suitable for the Showcase. |
| Spatial | Droplet or Bloom for connected geometry. A spring also drives floating placement. |
| Attention | Normal ease-in-out for a short emphasis. Preserve legible labels and focus indicators. |
| Status | Determinate interpolation or a clocked progress cycle. Sleep during invisible portions of a cycle. |
| Decorative | Slow ease-in-out as a reusable role. Give motion a reason and limit its duration. |

The default theme's fast, normal, and slow durations are 110, 220, and 330 ms.
For spatial motion, **Soft** is restrained, **Snappy** is short, **Bouncy**
has deliberate overshoot, **Smooth** is critically damped, **Gentle** is
overdamped, and **Responsive** preserves the standard interaction spring.
The nominal response time controls natural frequency. It is distinct from
measured settling time. Hosts set `MotionSettings::reduced_motion` from user
preferences. Disabled motion, reduced motion, and zero egui animation time
snap tracks and provide a static progress pose. Immediate gesture input
stays available.

## Physics, timing, and storage

For offset \(y=x-x_0\), natural frequency is \(\omega=\sqrt{k/m}\) and damping
ratio is \(\zeta=c/(2m\omega)\). The underdamped solver uses a decaying sine and
cosine with frequency \(\omega\sqrt{1-\zeta^2}\). The critical solution uses
\((A+Bt)e^{-\omega t}\). Overdamping uses two real negative roots. It evaluates
their difference with `exp_m1` and computes the slow root from the reciprocal
to avoid subtracting two large close numbers. A single set of coefficients
advances all channels. The original segment is sampled at elapsed time from
egui, so dropped frames do not accumulate integration error. Retargeting
samples the old segment at the event time and gives a new spring the
resulting position and velocity. A tween starts at the same position and uses
the velocity defined by its easing. The decay model uses
\(x(t)=x_0+(v_0/\lambda)(1-e^{-\lambda t})\) with friction \(\lambda>0\).

The cache is temporary egui context data, scoped by viewport, ID, and channel
count. Tracks update only while sampled. No thread or app-wide frame loop is
required. Resting tracks request no repaint. After a widget has been absent
for more than 60 seconds, its own viewport can reclaim its track and reinitialize
it on return. Adjacent frames separated by idle wall time retain their state.
The cache sweeps opportunistically and caps each channel count at 16,384
tracks. At capacity, new tracks snap. Use clipping and virtualization for long
lists. Layout sizes and hit testing remain controlled by the widget.

## Widget and effect inventory

| Components | Behavior, cost, and decision |
| --- | --- |
| Button, Radio, Checkbox, Switch, Item, Tile | Retain shared press, color, and selection motion. A few scalar tracks plus the normal widget paint. Preserve keyboard activation and stable hit areas. |
| Tabs, floating navigation, navigation menus, sidebar, pagination, calendar, line calendar, select tile group | Keep shared selection springs and paint-only position changes where already implemented. Keyboard navigation updates selection immediately. |
| Autocomplete, Select, MultiSelect, DateField, DateTimePicker, TimeField, Popover, PopoverMenu, Dialog, Sheet, FloatingMotion, PageTransition | Retain existing anchored geometry and spring timing through the new solver. Droplet uses adaptive rounded distance-field contour painting during joins and ordinary rounded paint after settling. Logical dismissal stays immediate. |
| Accordion, Toaster, Toast | Add an opacity reveal to open accordion content and use a fast shared tween for queue actions. Keep toast lifetimes as timers and retain existing Droplet entrance/exit. Collapsed accordion content is not evaluated. |
| TextField, OTP, Search | Preserve focus, selection, and direct editing. TextField and Search already use shared motion. OTP remains immediate for input clarity. |
| Slider, RangeSlider, Resizable, Picker, TimePicker, ScrollArea | Preserve direct drag values. Keep native scroll inertia and wheel snapping. A second physics system would fight egui's input ownership. The Motion Lab demonstrates optional decay and edge resistance for custom drags. |
| Progress and CircularProgress | Share the same cycle clock and scheduling. Determinate bar motion uses the existing tween. Determinate ring keeps exact progress. |
| Tooltip, ContextMenu, link preview | Keep native tooltip timing, context-menu input rules, and the existing Droplet link preview. Focus and Escape remain authoritative. |
| Vesper, Frost, Prism, EffectSurface | Vesper uses a single two-channel spring and shared exponential tracking. Local light meshes and glass backdrop passes are independent of animation state. The existing renderer and its quality limits remain in place. |
| VideoPlayer | Preserve exact seek position and media time. Existing chrome visibility and focus tweens use the shared engine. Playback scheduling stays with the player. |
| Alert, Avatar, Badge, Breadcrumb, Card, Divider, Header, Label, Scaffold, SelectGroup, SelectMenuTile | Keep static or structural rendering. Embedded interactive controls provide their own feedback. |
| Motion gallery controls | Existing search, quantity, direction, toggle, and navigation examples retain their shared transitions. Motion Lab adds presets, custom coefficients, live reversal, drag release, decay, resistance, and detents. |

Use **Widgets → Find a widget → Motion lab** to inspect the regimes. Open it
alongside **Switch**, **Segments**, **Spring resize**, and **Action menu**
in the gallery to compare interaction and spatial motion.

Ripple, stagger, and shimmer can be built from bounded opacity, delayed tracks,
and the cycle clock when a component has a concrete need. No widget currently
requires independent ripple shaders, animated clipping masks for every accordion
row, or continuous parallax. Those would add paint, input, or accessibility
complexity. Prefer GPU profiling on real Android hardware before expanding the
live glass or SDF rendering budget.

The choice of springs for interruptible movement and tweens for fixed timing
follows independently documented interaction principles. Repaint scheduling
uses the host's egui integration. [Apple gesture continuity](https://developer.apple.com/videos/play/wwdc2024/10145/?time=130) · [Compose animation specifications](https://developer.android.com/develop/ui/compose/animation/customize) · [egui Context](https://docs.rs/egui/0.36.2/egui/struct.Context.html) · [W3C reduced motion guidance](https://www.w3.org/WAI/WCAG22/Understanding/animation-from-interactions)

## Verification and measurements

Run `cargo bench --locked --bench motion` to reproduce the headless CPU
comparison. The harness uses seven batches per scenario, each with 120
warm-up frames and 480 measured frames. Input timestamps advance at 120 Hz.
Active scalar tracks reverse every 24 frames. Idle tracks start at rest.
The same harness was compiled against tag `v0.1.3` and the new implementation.
The values below are medians across three alternating runs, pinned to one
logical CPU on an Intel Core i7-1165G7 with Rust 1.99.1.

| Scalar tracks | Workload | 0.1.3 (µs/frame) | 0.1.4 (µs/frame) |
| ---: | --- | ---: | ---: |
| 0 | egui frame only | 8.63 | 9.29 |
| 1 | active | 7.07 | 8.96 |
| 128 | idle | 17.71 | 32.93 |
| 128 | active | 62.60 | 71.09 |
| 1024 | idle | 65.96 | 229.89 |
| 1024 | active | 408.76 | 557.21 |

Small scenarios are close to the empty-frame measurement. Large batches
take more CPU time than the legacy scalar-only helper. The 1,024-track
cases add roughly 0.15–0.16 ms to this workload. This measures track sampling
and headless egui frame overhead. It excludes widget painting, GPU time,
native frame presentation, energy consumption, and phone performance.
Avoid polling off-screen tracks in large lists.

Regression tests cover all damping regimes, the near-critical limit,
elapsed-time consistency, velocity on retarget, gesture-to-decay and
decay-to-spring handoff, color interpolation, stale state, reduced motion,
and the existing widget interactions. The three optional CPU probes for
Vesper mesh generation, Droplet geometry, and Showcase theme switching
were also exercised. Native GPU profiling and real-device motion review
remain separate from these measurements.
