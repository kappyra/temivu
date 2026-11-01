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
/// The next [`animate_f32`] call for this `id` starts again from its `initial`
/// value; the next [`animate_bool`] call jumps to its target.
pub fn reset(ctx: &Context, id: Id) {
    ctx.data_mut(|data| {
        data.remove_temp::<Animation>(id);
    });
}
