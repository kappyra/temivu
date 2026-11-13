//! Shared motion clocks, physics, presets, and rendering helpers.
#![doc = include_str!("../docs/MOTION.md")]
use crate::theme::{Easing, MotionSettings, Theme};
use egui::{Context, Id};

mod physics;
mod track;

pub use physics::{Decay, Spring, rubber_band, tracking_alpha};
pub use track::{MotionSample, MotionSpec, MotionTrack, MotionValue, Tween};

fn spring_id(id: Id) -> Id {
    id.with("temivu-spring")
}
fn tween_id(id: Id) -> Id {
    id.with("temivu-tween")
}

/// Motion roles define rhythm without coupling an effect to a particular widget.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum MotionRole {
    Feedback,
    State,
    Navigation,
    Attention,
    Spatial,
    Status,
    Decorative,
}

impl MotionRole {
    pub fn tween(self, settings: MotionSettings) -> MotionSpec {
        let (duration, easing) = match self {
            Self::Feedback => (settings.fast, Easing::EaseOut),
            Self::State | Self::Status => (settings.normal, settings.easing),
            Self::Navigation | Self::Spatial => (settings.slow, Easing::Emphasized),
            Self::Attention => (settings.normal, Easing::EaseInOut),
            Self::Decorative => (settings.slow, Easing::EaseInOut),
        };
        MotionSpec::Tween(Tween { duration, easing })
    }
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum SpringPreset {
    Soft,
    Snappy,
    Bouncy,
    Smooth,
    Gentle,
    #[default]
    Responsive,
}

impl SpringPreset {
    pub fn spring(self, settings: MotionSettings) -> Spring {
        let (pace, damping) = match self {
            Self::Soft => (1.12, 0.86),
            Self::Snappy => (0.72, 0.92),
            Self::Bouncy => (1.00, 0.48),
            Self::Smooth => (1.00, 1.00),
            Self::Gentle => (1.30, 1.13),
            Self::Responsive => (1.00, 0.72),
        };
        Spring::response(settings.normal * pace, damping)
    }
}

/// Phase of a visible, repeating effect. During the inactive tail, the
/// renderer sleeps until the next cycle instead of ticking every frame.
pub fn cycle(
    ctx: &Context,
    settings: MotionSettings,
    period: f64,
    active_fraction: f64,
) -> Option<f32> {
    if !settings.animations_enabled(ctx)
        || !period.is_finite()
        || !(0.016..=3600.0).contains(&period)
        || !active_fraction.is_finite()
        || !(0.0..=1.0).contains(&active_fraction)
        || active_fraction == 0.0
    {
        return None;
    }
    let now = ctx.input(|input| input.time);
    if !now.is_finite() {
        return None;
    }
    let elapsed = now.rem_euclid(period);
    let phase = elapsed / period;
    let wait = if phase < active_fraction {
        0.016
    } else {
        period - elapsed
    };
    ctx.request_repaint_after(std::time::Duration::from_secs_f64(wait));
    Some(phase as f32)
}

/// Damped interaction spring with a small spatial overshoot.
/// Retargeting preserves velocity; `theme.motion.normal` controls settling speed.
/// The first frame starts at the target; reduced motion snaps immediately.
pub fn spring_bool(ctx: &Context, id: Id, target: bool, theme: &Theme) -> f32 {
    let target = if target { 1.0 } else { 0.0 };
    spring_f32(ctx, id, target, target, theme)
}

/// Velocity-preserving spring with an explicit initial value for entrances.
pub fn spring_f32(ctx: &Context, id: Id, initial: f32, target: f32, theme: &Theme) -> f32 {
    spring_f32_with_damping(ctx, id, initial, target, 0.72, theme)
}

/// Animate a layout size using the same clock and damping on both axes.
/// The first frame uses the requested size. Keep the ID stable as the size
/// changes and clamp the result to the parent constraints before laying it out.
pub fn spring_size(ctx: &Context, id: Id, target: egui::Vec2, theme: &Theme) -> egui::Vec2 {
    let dimension = |v: f32| if v.is_finite() { v.max(0.0) } else { 0.0 };
    let target = egui::vec2(dimension(target.x), dimension(target.y));
    MotionTrack::new(ctx, spring_id(id), theme.motion)
        .sample(
            target,
            target,
            MotionSpec::Spring(Spring::response(theme.motion.normal, 0.86)),
        )
        .value
        .max(egui::Vec2::ZERO)
}

/// Scale paint around a fixed point. With egui's `Ui::with_visual_transform`
/// this leaves layout and hit areas stable, even while the paint scales.
pub fn scale_around(pivot: egui::Pos2, scale: f32) -> egui::emath::TSTransform {
    let scale = if scale.is_finite() && scale > 0.0 {
        scale
    } else {
        1.0
    };
    egui::emath::TSTransform {
        scaling: scale,
        translation: pivot.to_vec2() * (1.0 - scale),
    }
}

/// A spring with custom damping using the theme duration.
pub fn spring_f32_with_damping(
    ctx: &Context,
    id: Id,
    initial: f32,
    target: f32,
    damping: f32,
    theme: &Theme,
) -> f32 {
    spring_f32_with_settings(ctx, id, initial, target, damping, theme.motion)
}

/// A spring with custom damping and duration that respects reduced motion.
pub fn spring_f32_with_settings(
    ctx: &Context,
    id: Id,
    initial: f32,
    target: f32,
    damping: f32,
    settings: MotionSettings,
) -> f32 {
    MotionTrack::new(ctx, spring_id(id), settings)
        .sample(
            initial,
            target,
            MotionSpec::Spring(Spring::response(settings.normal, damping)),
        )
        .value
}

/// Animate a float with the theme duration and easing. The first call starts
/// at `initial`; subsequent targets start at the current pose.
pub fn animate_f32(ctx: &Context, id: Id, initial: f32, target: f32, theme: &Theme) -> f32 {
    if !initial.is_finite() || !target.is_finite() {
        reset(ctx, id);
        // Preserve the legacy helper's invalid-input return convention.
        return target;
    }
    MotionTrack::new(ctx, tween_id(id), theme.motion)
        .sample(initial, target, MotionRole::State.tween(theme.motion))
        .value
}

/// Animate a boolean towards `target`, returning progress in `0.0..=1.0`.
///
/// The first call jumps to `target`, matching egui's `Context::animate_bool`; a
/// widget that should fade in on first appearance uses [`animate_f32`] with an
/// explicit `initial` value instead.
pub fn animate_bool(ctx: &Context, id: Id, target: bool, theme: &Theme) -> f32 {
    let target = if target { 1.0 } else { 0.0 };
    animate_f32(ctx, id, target, target, theme)
}

/// Forget the stored animation for `id`.
///
/// A float animation starts again from its initial value and a boolean
/// animation starts at its target.
pub fn reset(ctx: &Context, id: Id) {
    track::forget::<1>(ctx, tween_id(id));
    track::forget::<1>(ctx, spring_id(id));
    track::forget::<2>(ctx, spring_id(id));
}

/// Shared spatial feedback for controls. A quick tap still gets enough time
/// to deform the surface before the same spring carries it back.
pub(crate) fn press(ctx: &Context, id: Id, held: bool, clicked: bool, theme: &Theme) -> f32 {
    let spring = id.with("press-spring");
    let pulse = id.with("press-pulse");
    let now = ctx.input(|input| input.time);
    if !theme.motion.animations_enabled(ctx) {
        ctx.data_mut(|data| data.remove::<f64>(pulse));
        return spring_bool(ctx, spring, held, theme);
    }
    let previous = MotionTrack::new(ctx, spring_id(spring), theme.motion).inspect::<f32, 1>();
    if clicked && previous.is_none_or(|state| state.value < 0.60) {
        let duration = 0.048 * f64::from(theme.motion.normal_scale().clamp(0.2, 10.0));
        ctx.data_mut(|data| data.insert_temp(pulse, now + duration));
    }
    let release = ctx.data(|data| data.get_temp::<f64>(pulse));
    let pulsing = release.is_some_and(|until| now < until);
    if pulsing {
        ctx.request_repaint_after(std::time::Duration::from_secs_f64(
            (release.unwrap() - now).max(0.0),
        ));
    } else if release.is_some() {
        ctx.data_mut(|data| data.remove::<f64>(pulse));
    }
    spring_f32_with_damping(
        ctx,
        spring,
        0.0,
        if held || pulsing { 1.0 } else { 0.0 },
        0.69,
        theme,
    )
    .clamp(-0.13, 1.06)
}

/// Convert the shared press phase into its visual scale.
pub fn press_scale(amount: f32) -> f32 {
    1.0 - 0.064 * amount
}

fn fractional_press_radius(rest: f32, limit: f32, amount: f32) -> f32 {
    let target = if rest == 0.0 {
        0.0
    } else if rest <= (limit * 0.57).min(12.0) {
        limit * 0.9
    } else {
        (rest * 0.57).min(14.0)
    };
    egui::lerp(rest..=target, amount.clamp(0.0, 1.0))
}

/// Use egui's own rounded-rectangle path with subpixel corners while pressed.
pub(crate) fn press_shape(rect: egui::epaint::RectShape, amount: f32) -> egui::Shape {
    if amount <= 0.001 || rect.corner_radius == egui::CornerRadius::ZERO {
        return rect.into();
    }
    rounded_shape(
        rect.rect,
        press_corners(rect.rect, rect.corner_radius, amount),
        rect.fill,
        rect.stroke,
        rect.stroke_kind,
    )
}

/// Interpolate the corner radii of a custom control using the shared press phase.
pub fn press_corners(
    rect: egui::Rect,
    corners: egui::CornerRadius,
    amount: f32,
) -> egui::epaint::CornerRadiusF32 {
    let limit = (rect.size().min_elem() * 0.5).max(0.0);
    let radius = |r| fractional_press_radius(f32::from(r).min(limit), limit, amount);
    egui::epaint::CornerRadiusF32 {
        nw: radius(corners.nw),
        ne: radius(corners.ne),
        sw: radius(corners.sw),
        se: radius(corners.se),
    }
}

pub(crate) fn rounded_shape(
    rect: egui::Rect,
    corners: egui::epaint::CornerRadiusF32,
    fill: egui::Color32,
    stroke: egui::Stroke,
    kind: egui::StrokeKind,
) -> egui::Shape {
    use egui::epaint::{PathShape, PathStroke, RoundedRect, tessellator::path};
    let mut points = Vec::new();
    path::rounded_rectangle(&mut points, RoundedRect::new(rect, corners));
    PathShape::convex_polygon(points, fill, PathStroke::from(stroke).with_kind(kind)).into()
}

/// Publish painted bounds while retaining egui's full hit area.
pub(crate) fn remember_anchor(response: &egui::Response, rect: egui::Rect) {
    response
        .ctx
        .data_mut(|data| data.insert_temp(response.id.with("temivu-visual-anchor"), rect));
}

#[derive(Clone, Copy, PartialEq, Eq)]
pub(crate) struct AnchorMaterial {
    pub fill: egui::Color32,
    pub border: egui::Color32,
}

impl AnchorMaterial {
    pub fn from_shape(shape: &egui::Shape, bounds: egui::Rect) -> Option<Self> {
        let egui::Shape::Rect(rect) = shape else {
            return None;
        };
        let delta = (rect.rect.size() - bounds.size()).abs();
        (rect.rect.center().distance(bounds.center()) < 0.5
            && delta.max_elem() < 5.0
            && rect.fill.a() > 0
            && rect.brush.is_none())
        .then_some(Self {
            fill: rect.fill,
            border: rect.stroke.color,
        })
    }
}

pub(crate) fn remember_anchor_material(
    response: &egui::Response,
    material: Option<AnchorMaterial>,
) {
    let key = response.id.with("temivu-anchor-material");
    let pass = response.ctx.cumulative_pass_nr();
    response
        .ctx
        .data_mut(|data| data.insert_temp(key, (pass, material)));
}

pub(crate) fn anchor_material(response: &egui::Response) -> Option<AnchorMaterial> {
    let ctx = &response.ctx;
    let pass = ctx.cumulative_pass_nr();
    let painted = ctx.data(|data| {
        data.get_temp::<(u64, Option<AnchorMaterial>)>(response.id.with("temivu-anchor-material"))
    });
    if let Some((painted_pass, material)) = painted
        && painted_pass == pass
    {
        return material;
    }

    // Ordinary egui buttons may also anchor a Temivu popup. Look only
    // at recent rectangles that match the trigger, never at text.
    ctx.graphics(|graphics| {
        let entries = graphics.get(response.layer_id)?.all_entries();
        let skip = entries.len().saturating_sub(32);
        entries
            .skip(skip)
            .filter_map(|entry| AnchorMaterial::from_shape(&entry.shape, response.rect))
            .last()
    })
}

pub(crate) fn anchor_rect(response: &egui::Response) -> egui::Rect {
    let rect = response
        .ctx
        .data(|data| data.get_temp::<egui::Rect>(response.id.with("temivu-visual-anchor")))
        .unwrap_or(response.rect);
    response
        .ctx
        .layer_transform_to_global(response.layer_id)
        .map_or(rect, |transform| transform * rect)
}

#[cfg(test)]
mod tests {
    use super::press_corners;

    #[test]
    fn press_rounds_compact_corners_and_softens_capsules() {
        let rect = egui::Rect::from_min_size(egui::Pos2::ZERO, egui::vec2(120.0, 40.0));
        for (radius, grows) in [(7, true), (20, false)] {
            let resting = f32::from(radius);
            let halfway = press_corners(rect, radius.into(), 0.5).nw;
            let pressed = press_corners(rect, radius.into(), 1.0).nw;
            assert!((pressed > resting) == grows);
            assert_eq!(halfway, egui::lerp(resting..=pressed, 0.5));
            assert!(pressed > 0.0 && pressed <= rect.height() * 0.5);
        }
    }
}
