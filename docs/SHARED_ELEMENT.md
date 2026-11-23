# Shared element transitions

Use `SharedElementTransition` when a card, button, image or other widget should appear to become a dialog, popover, image viewer or panel. Open **Showcase → Overlays → Shared element** to try all four destinations. Toggle while the surface is moving, or drag it and release to settle toward either endpoint.

## References and design

| System | Implementation and behavior | Temivu choice |
| --- | --- | --- |
| SwiftUI Zoom | The Wishlist sample pairs the trip's stable ID and namespace on a card and its navigation destination. The source view supplies position, size and corner radius. Zoom supports grabbing after the transition has started. [Wishlist sample](https://developer.apple.com/documentation/swiftui/wishlist-planning-travel-in-a-swiftui-app) · [WWDC24 zoom demo](https://developer.apple.com/videos/play/wwdc2024/10145/?time=127) | One stable egui ID per transition and viewport. The application supplies measured endpoints rather than navigation stack indices. |
| SwiftUI Matched Geometry and springs | Matched Geometry links view frames in window coordinates. The view's own transition defines its opacity and rendering. A retargeted SwiftUI spring inherits velocity, including from a gesture. [Matched Geometry API](https://developer.apple.com/documentation/swiftui/view/matchedgeometryeffect(id:in:properties:anchor:issource:)) · [WWDC23 spring handoff](https://developer.apple.com/videos/play/wwdc2023/10158/) | Use a viewport-local `MotionTrack` for progress, center, size and four corner radii. Evaluate the previous segment at the event time before assigning a destination. Opacity follows the resulting progress in either direction. |
| Material Container Transform | The Android implementation captures bounds and shape from start and end views and paints a drawable overlay. Fit modes maintain aspect and mask excess content. Separate thresholds control scale, mask, corners and opacity. The shape evaluator builds rounded start and end paths and clips their union. The Catalog demo wires opening and return transitions around a shared view name. [Motion guide](https://github.com/material-components/material-components-android/blob/master/docs/theming/Motion.md#container-transform) · [Mask](https://github.com/material-components/material-components-android/blob/master/lib/java/com/google/android/material/transition/MaskEvaluator.java) · [Fit](https://github.com/material-components/material-components-android/blob/master/lib/java/com/google/android/material/transition/FitModeEvaluators.java) · [Fade](https://github.com/material-components/material-components-android/blob/master/lib/java/com/google/android/material/transition/FadeModeEvaluators.java) · [Catalog demo](https://github.com/material-components/material-components-android/blob/master/catalog/java/io/material/catalog/transition/TransitionContainerTransformDemoFragment.java) | Paint one interpolated rounded mask and uniformly fit each cached endpoint into it. Choose crossfade or fade through the container. Reuse the same snapshot on both sides for a photo or icon that stays fully visible. |
| Compose shared bounds | Compose offers an overlay, bounds transforms, clipping, and both scaling and remeasurement. Scaling avoids text wrapping changes during travel. [Compose shared elements](https://developer.android.com/develop/ui/compose/animation/shared-elements/customize) | Lay out text at each endpoint and reuse its tessellation. The application's live UI takes over when the motion settles. |

Apple publishes sample code and API behavior for these transitions. Its framework renderer is proprietary. Temivu's geometry, mask and motion policy are implemented with egui and Temivu primitives.

## Integrate a transition

```rust
use temivu::{
    SharedElementDrive, SharedElementEndpoint, SharedElementSide, SharedElementSnapshot,
    SharedElementTransition, Theme,
};

fn travel(
    ui: &egui::Ui,
    theme: &Theme,
    source: &egui::Response,
    destination: egui::Rect,
    target: SharedElementSide,
    snapshots: [&SharedElementSnapshot; 2],
) -> Option<SharedElementSide> {
    let transition = SharedElementTransition::new(source.id.with("details"));
    let from = SharedElementEndpoint::from_response(source, 16.0, theme.card);
    let to = SharedElementEndpoint::new(destination, 24.0, theme.card);
    let pose = transition.sample(
        ui.ctx(), theme, from, to, SharedElementDrive::SettleTo(target),
    )?;
    if pose.settled.is_none() {
        let layer = egui::LayerId::new(egui::Order::Foreground, source.id.with("travel"));
        let painter = ui.ctx().layer_painter(layer);
        transition.paint(&painter, pose, snapshots.map(Some));
    }
    pose.settled
}
```

1. Keep the ID stable and supply both valid endpoint bounds every frame. Use `SharedElementEndpoint::from_response` for a measured Card or anchor. Retain the last valid measurement if a source scrolls out while the view is open.
2. Prepare `SharedElementSnapshot::from_shapes` once for paint-only content, or use `SharedElementSnapshot::capture` around a live card or frame on the capture frame. Recapture when the content changes. In Dialog or Popover, capture inside the surface's own UI layer. The snapshot contains regular egui meshes. Keep referenced textures alive for the flight.
3. During travel, draw the overlay in viewport coordinates on an untransformed layer. Keep layout placeholders where needed. Assign gestures to a dedicated `Response`. Keep live controls and focus in the endpoint that owns them. A custom host can draw its own material using `SharedElementPose` and choose the paint layer and outer clip.
4. Once settled at the destination, show the live Dialog, Popover, panel or viewer at the measured bounds. If using a widget that supplies its own entrance, configure its `LiquidStyle` with `enabled: false` for this handoff. Keep sampling while returning to the card.
5. To drag, initialize progress from the displayed pose and add normalized pointer displacement. Pass progress and velocity in fractions per second to `Follow`. On release, submit the final gesture sample and then use `SettleTo` with either side. Keyboard and programmatic reversal can switch `SettleTo` directly. Reduced motion settles those spring requests immediately.

The motion uses one closed-form spring solve per sample, shared by nine numeric channels. An in-flight layout resize retargets those channels at their current position and velocity. The renderer preserves texture coordinates through a convex rounded clip and a separate viewport scissor. It shares cached tessellation across frames, transforms indexed meshes directly when they fit inside the mask, and uses stack buffers for boundary triangles. Paint-only snapshots do not create input regions or accessibility nodes.

The integration tests cover reversal at the same timestamp, handoff from a held gesture, moving destinations, elapsed-time invariance, reduced motion, real Card capture, the rounded clip and a shared image's opacity. The Showcase test exercises reversal and the image viewer on a narrow viewport.
