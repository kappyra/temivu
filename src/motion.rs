//! Reduced-motion-aware animation helpers for boolean and floating-point
//! values.
//!
//! The helper keeps its animation state in egui's temporary data, keyed by an
//! [`Id`], so widgets do not need an extra animation field. It reads the clock
//! from `Context::input(|input| input.time)` and therefore works with
//! `egui_kittest` tests that drive time explicitly. There is no async code and
//! no threading: a repaint is requested only while a value is actually changing,
//! so an idle UI does not repaint.
//!
//! An animation snaps straight to its target when any of these is true:
//!
//! * [`Theme::motion`]'s `enabled` flag is `false`,
//! * [`Theme::motion`]'s `reduced_motion` flag is `true`, or
//! * egui's active style has `animation_time == 0.0` (in egui 0.36 that style
//!   is `Context::global_style()`; older releases call it `Context::style()`).
use crate::theme::{Easing, Theme};
use egui::{Context, Id};

#[derive(Clone, Copy)]
struct SpringState {
    value: f32,
    velocity: f32,
    target: f32,
    time: f64,
    duration: f32,
    damping: f32,
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

pub(crate) fn spring_f32_with_damping(
    ctx: &Context,
    id: Id,
    initial: f32,
    target: f32,
    damping: f32,
    theme: &Theme,
) -> f32 {
    if !initial.is_finite() || !target.is_finite() {
        return target;
    }
    let duration = if theme.motion.normal.is_finite() {
        theme.motion.normal.clamp(0.04, 2.0)
    } else {
        0.2
    };
    let damping = if damping.is_finite() {
        damping.clamp(0.1, 0.99)
    } else {
        0.72
    };
    let stored = ctx.data(|data| data.get_temp::<SpringState>(id));
    if stored.is_some_and(|state| {
        state.target == target && state.value == target && state.velocity == 0.0
    }) {
        return target;
    }
    let now = ctx.input(|input| input.time);
    let mut state = stored.unwrap_or(SpringState {
        value: initial,
        velocity: 0.0,
        target,
        time: now,
        duration,
        damping,
    });
    if !theme.motion.animations_enabled(ctx) {
        state.value = target;
        state.velocity = 0.0;
    } else {
        // This is an analytic solution, so it remains stable over a long
        // frame. Do not stretch animations after backgrounding or slow swaps.
        let dt = ((now - state.time) as f32).clamp(0.0, 10.0);
        // Integrate the elapsed interval using the parameters that were in
        // effect during it, including when a faster exit retargets the spring.
        let omega = 5.2 / state.duration;
        let offset = state.value - state.target;
        let damping = state.damping;
        let attenuation = damping * omega;
        let frequency = omega * (1.0 - damping * damping).sqrt();
        let coefficient = (state.velocity + attenuation * offset) / frequency;
        let (sin, cos) = (frequency * dt).sin_cos();
        let decay = (-attenuation * dt).exp();
        state.value = state.target + decay * (offset * cos + coefficient * sin);
        state.velocity = decay
            * ((coefficient * frequency - attenuation * offset) * cos
                - (offset * frequency + attenuation * coefficient) * sin);
        if (state.value - target).abs() < 0.001 && state.velocity.abs() < 0.01 {
            state.value = target;
            state.velocity = 0.0;
        } else {
            ctx.request_repaint();
        }
    }
    state.target = target;
    state.time = now;
    state.duration = duration;
    state.damping = damping;
    ctx.data_mut(|data| data.insert_temp(id, state));
    state.value
}

#[derive(Clone, Copy, Debug, Default, PartialEq)]
struct Animation {
    from: f32,
    target: f32,
    start: f64,
    duration: f32,
}

impl Animation {
    /// Seconds since the animation started. The delta is reduced to `f32` so it
    /// compares exactly against the `f32` duration instead of accumulating the
    /// wider representation of an `f32` constant.
    fn elapsed(&self, now: f64) -> f32 {
        (now - self.start) as f32
    }

    fn value(&self, now: f64, easing: Easing) -> f32 {
        if !self.duration.is_finite() || self.duration <= 0.0 {
            return self.target;
        }
        let progress = self.elapsed(now) / self.duration;
        if !progress.is_finite() {
            return self.target;
        }
        self.from + (self.target - self.from) * easing.apply(progress)
    }

    fn finished(&self, now: f64) -> bool {
        !self.duration.is_finite() || self.duration <= 0.0 || self.elapsed(now) >= self.duration
    }
}

/// Animate a floating-point value towards `target` and return its current
/// value.
///
/// `initial` is used only on the first call for an `id`; later calls continue
/// from the stored value, so callers can pass a constant `initial` every frame.
/// Passing the previous value is not required. The first call jumps directly to
/// `target` when animations are disabled.
///
/// A repaint is requested only while the value is between `initial` and
/// `target`. The stored state is normalized when the animation settles, so the
/// next target change starts exactly at the settled value.
pub fn animate_f32(ctx: &Context, id: Id, initial: f32, target: f32, theme: &Theme) -> f32 {
    if !initial.is_finite() || !target.is_finite() {
        reset(ctx, id);
        return target;
    }
    let now = ctx.input(|input| input.time);
    let enabled = theme.motion.animations_enabled(ctx);
    let duration = theme.motion.normal;
    let previous = ctx.data_mut(|data| data.get_temp::<Animation>(id));
    let animation = match previous {
        Some(mut animation) => {
            if animation.target != target {
                // Continue from wherever the previous animation had reached.
                animation.from = animation.value(now, theme.motion.easing);
                animation.target = target;
                animation.start = now;
                animation.duration = duration;
            }
            animation
        }
        None => Animation {
            from: if enabled { initial } else { target },
            target,
            start: now,
            duration,
        },
    };
    // `from == target` means there is nothing to animate (for example the first
    // frame of `animate_bool`), so settle without scheduling a repaint.
    if !enabled || animation.from == target || animation.finished(now) {
        // Normalize so a later change starts from the settled target.
        ctx.data_mut(|data| {
            data.insert_temp(
                id,
                Animation {
                    from: target,
                    target,
                    start: now,
                    duration,
                },
            );
        });
        target
    } else {
        ctx.data_mut(|data| data.insert_temp(id, animation));
        ctx.request_repaint();
        animation.value(now, theme.motion.easing)
    }
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
    ctx.data_mut(|data| {
        data.remove_temp::<Animation>(id);
        data.remove::<SpringState>(id);
    });
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
    let previous = ctx.data(|data| data.get_temp::<SpringState>(spring));
    if clicked && previous.is_none_or(|state| state.value < 0.60) {
        ctx.data_mut(|data| data.insert_temp(pulse, now + 0.048));
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

pub(crate) fn press_scale(amount: f32) -> f32 {
    1.0 - 0.064 * amount
}

fn fractional_press_radius(rest: f32, amount: f32) -> f32 {
    let compressed = (rest * 0.44).clamp(0.0, 13.0);
    egui::lerp(rest..=compressed, amount.clamp(0.0, 1.0))
}

/// Use egui's own rounded-rectangle path with subpixel corners while pressed.
pub(crate) fn press_shape(rect: egui::epaint::RectShape, amount: f32) -> egui::Shape {
    use egui::epaint::CornerRadiusF32;
    if amount <= 0.001 || rect.corner_radius == egui::CornerRadius::ZERO {
        return rect.into();
    }
    let limit = (rect.rect.size().min_elem() * 0.5).max(0.0);
    let radius = |r| fractional_press_radius(f32::from(r).min(limit), amount);
    rounded_shape(
        rect.rect,
        CornerRadiusF32 {
            nw: radius(rect.corner_radius.nw),
            ne: radius(rect.corner_radius.ne),
            sw: radius(rect.corner_radius.sw),
            se: radius(rect.corner_radius.se),
        },
        rect.fill,
        rect.stroke,
        rect.stroke_kind,
    )
}

pub(crate) fn rounded_shape(
    rect: egui::Rect,
    corners: egui::epaint::CornerRadiusF32,
    fill: egui::Color32,
    stroke: egui::Stroke,
    kind: egui::StrokeKind,
) -> egui::Shape {
    use egui::epaint::{PathShape, PathStroke, tessellator::path};
    let mut points = Vec::new();
    path::rounded_rectangle(&mut points, rect, corners);
    PathShape::convex_polygon(points, fill, PathStroke::from(stroke).with_kind(kind)).into()
}

/// Publish painted bounds while retaining egui's full hit area.
pub(crate) fn remember_anchor(response: &egui::Response, rect: egui::Rect) {
    response
        .ctx
        .data_mut(|data| data.insert_temp(response.id.with("temivu-visual-anchor"), rect));
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
