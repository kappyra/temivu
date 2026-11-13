use std::collections::HashMap;

use egui::{Color32, Context, Id, Pos2, Rect, Rgba, Vec2, ViewportId};

use super::{Decay, Easing, MotionSettings, Spring};

/// A value represented by independent numeric channels. Channel order is also
/// the order used by `MotionSample::velocity`.
pub trait MotionValue<const N: usize>: Copy {
    fn into_channels(self) -> [f64; N];
    fn from_channels(channels: [f64; N]) -> Self;
}

impl MotionValue<1> for f32 {
    fn into_channels(self) -> [f64; 1] {
        [f64::from(self)]
    }
    fn from_channels([value]: [f64; 1]) -> Self {
        finite_f32(value)
    }
}

impl MotionValue<1> for f64 {
    fn into_channels(self) -> [f64; 1] {
        [self]
    }
    fn from_channels([value]: [f64; 1]) -> Self {
        value
    }
}

impl MotionValue<2> for Vec2 {
    fn into_channels(self) -> [f64; 2] {
        [self.x.into(), self.y.into()]
    }
    fn from_channels([x, y]: [f64; 2]) -> Self {
        Self::new(finite_f32(x), finite_f32(y))
    }
}

impl MotionValue<2> for Pos2 {
    fn into_channels(self) -> [f64; 2] {
        self.to_vec2().into_channels()
    }
    fn from_channels(channels: [f64; 2]) -> Self {
        Vec2::from_channels(channels).to_pos2()
    }
}

impl MotionValue<4> for Rect {
    fn into_channels(self) -> [f64; 4] {
        [
            self.min.x.into(),
            self.min.y.into(),
            self.max.x.into(),
            self.max.y.into(),
        ]
    }
    fn from_channels([x0, y0, x1, y1]: [f64; 4]) -> Self {
        let min = Pos2::from_channels([x0, y0]);
        let max = Pos2::from_channels([x1, y1]).max(min);
        Self::from_min_max(min, max)
    }
}

impl MotionValue<4> for Color32 {
    fn into_channels(self) -> [f64; 4] {
        Rgba::from(self).to_array().map(f64::from)
    }
    fn from_channels([r, g, b, a]: [f64; 4]) -> Self {
        let a = a.clamp(0.0, 1.0);
        let component = |v: f64| v.clamp(0.0, 1.0) as f32;
        Rgba::from_rgba_premultiplied(component(r), component(g), component(b), a as f32).into()
    }
}

fn finite_f32(v: f64) -> f32 {
    v.clamp(-(f32::MAX as f64), f32::MAX as f64) as f32
}

/// A fixed-duration interpolation. Duration is in seconds.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Tween {
    pub duration: f32,
    pub easing: Easing,
}

/// Use the same track when switching between gesture following, a spring,
/// a tween, and momentum. Springs inherit the preceding velocity.
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum MotionSpec {
    Spring(Spring),
    Tween(Tween),
    Decay(Decay),
}

/// Current animated pose. A settled animation has zero velocity.
/// A gesture sample can carry velocity while `running` is false.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct MotionSample<T, const N: usize> {
    pub value: T,
    pub velocity: [f64; N],
    pub running: bool,
}

#[derive(Clone, Copy)]
struct State<const N: usize> {
    initial: [f64; N],
    velocity: [f64; N],
    target: [f64; N],
    started: f64,
    seen: f64,
    spec: Option<MotionSpec>,
    following: bool,
}

impl<const N: usize> State<N> {
    fn rest(value: [f64; N], now: f64) -> Self {
        Self {
            initial: value,
            velocity: [0.0; N],
            target: value,
            started: now,
            seen: now,
            spec: None,
            following: false,
        }
    }

    fn expired(self, now: f64, previous_frame: f64) -> bool {
        // A single slow or idle frame does not mean the widget disappeared.
        now - self.seen > IDLE_TTL && previous_frame - self.seen > 0.001
    }

    fn evaluate(self, now: f64) -> ([f64; N], [f64; N], bool) {
        let elapsed = (now - self.started).max(0.0);
        if self.spec.is_some() && self.initial == self.target && self.velocity == [0.0; N] {
            return (self.target, [0.0; N], false);
        }
        let mut value = self.initial;
        let mut velocity = self.velocity;
        let (amount, speed) = match self.spec {
            None => return (value, velocity, false),
            Some(MotionSpec::Spring(spring)) => {
                let kernel = spring.kernel(elapsed);
                for i in 0..N {
                    (value[i], velocity[i]) = kernel.apply(value[i], velocity[i], self.target[i]);
                }
                return self.finish(value, velocity);
            }
            Some(MotionSpec::Tween(tween)) => {
                if !tween.duration.is_finite()
                    || tween.duration <= 0.0
                    || elapsed >= f64::from(tween.duration)
                {
                    return (self.target, [0.0; N], false);
                }
                let phase = (elapsed / f64::from(tween.duration)) as f32;
                (
                    f64::from(tween.easing.apply(phase)),
                    slope(tween.easing, phase) / f64::from(tween.duration),
                )
            }
            Some(MotionSpec::Decay(decay)) => decay.fraction(elapsed),
        };
        for i in 0..N {
            value[i] = self.initial[i] * (1.0 - amount) + self.target[i] * amount;
            velocity[i] = (self.target[i] - self.initial[i]) * speed;
            if !value[i].is_finite() || !velocity[i].is_finite() {
                return (self.target, [0.0; N], false);
            }
        }
        if matches!(self.spec, Some(MotionSpec::Tween(_))) {
            (value, velocity, true)
        } else {
            self.finish(value, velocity)
        }
    }

    fn finish(self, value: [f64; N], velocity: [f64; N]) -> ([f64; N], [f64; N], bool) {
        if (0..N).all(|i| (value[i] - self.target[i]).abs() < 0.001 && velocity[i].abs() < 0.01) {
            (self.target, [0.0; N], false)
        } else {
            (value, velocity, true)
        }
    }
}

fn slope(easing: Easing, t: f32) -> f64 {
    let t = f64::from(t);
    match easing {
        Easing::Linear => 1.0,
        Easing::EaseIn => 2.0 * t,
        Easing::EaseOut => 2.0 * (1.0 - t),
        Easing::EaseInOut => 4.0 * t.min(1.0 - t),
        Easing::Emphasized => 4.0 * (1.0 - t).powi(3),
        Easing::Spring => 64.0 * t * (-8.0 * t).exp() / (1.0 - 9.0 * (-8.0_f64).exp()),
    }
}

const IDLE_TTL: f64 = 60.0;
const TRACK_LIMIT: usize = 16_384;

#[derive(Clone)]
struct Cache<const N: usize> {
    states: HashMap<(ViewportId, Id), State<N>>,
    prune_at: (ViewportId, f64),
}

impl<const N: usize> Default for Cache<N> {
    fn default() -> Self {
        Self {
            states: HashMap::new(),
            prune_at: (ViewportId::ROOT, 0.0),
        }
    }
}

impl<const N: usize> Cache<N> {
    fn prune(&mut self, viewport: ViewportId, now: f64, previous_frame: f64) {
        let (last_viewport, next) = self.prune_at;
        if now >= next || (viewport == last_viewport && now < next - 8.0) {
            self.states.retain(|(owner, _), state| {
                if *owner == viewport {
                    !state.expired(now, previous_frame)
                } else {
                    now - state.seen <= 600.0
                }
            });
            self.prune_at = (viewport, now + 8.0);
        }
    }
}

const CACHE: Id = Id::NULL;

/// A short-lived handle to a channel in egui's temporary data. Construct it
/// while rendering a widget. Give each animated property its own stable ID.
#[derive(Clone, Copy)]
pub struct MotionTrack<'a> {
    ctx: &'a Context,
    id: Id,
    settings: MotionSettings,
}

impl<'a> MotionTrack<'a> {
    pub fn new(ctx: &'a Context, id: Id, settings: MotionSettings) -> Self {
        Self { ctx, id, settings }
    }

    /// Advance the old segment to the event time, then apply the new target.
    /// A new track starts at `initial`. To seed velocity, call `follow` first.
    pub fn sample<T: MotionValue<N>, const N: usize>(
        self,
        initial: T,
        target: T,
        spec: MotionSpec,
    ) -> MotionSample<T, N> {
        let (key, now, previous_frame) = self.key_and_time();
        let initial = initial
            .into_channels()
            .map(|v| if v.is_finite() { v } else { 0.0 });
        let target = target.into_channels();
        let valid = target.iter().all(|v| v.is_finite());
        let (value, velocity, running) = self.ctx.data_mut(|data| {
            let cache = data.get_temp_mut_or_default::<Cache<N>>(CACHE);
            cache.prune(key.0, now, previous_frame);
            if !valid {
                let value = cache
                    .states
                    .remove(&key)
                    .filter(|s| !s.expired(now, previous_frame))
                    .map_or(initial, |s| s.evaluate(now).0);
                return (value, [0.0; N], false);
            }
            let full = cache.states.len() >= TRACK_LIMIT;
            let slot = match cache.states.entry(key) {
                std::collections::hash_map::Entry::Occupied(entry) => entry.into_mut(),
                std::collections::hash_map::Entry::Vacant(entry) if !full => {
                    entry.insert(State::rest(initial, now))
                }
                _ => return (target, [0.0; N], false),
            };
            if slot.expired(now, previous_frame) {
                *slot = State::rest(initial, now);
            }
            if slot.spec.is_none() && !slot.following && slot.target == target {
                slot.seen = now;
                return (target, [0.0; N], false);
            }
            let mut state = *slot;
            // Rebase if a host resets its clock. Keep the elapsed time and
            // allow the very next frame to advance on the new clock.
            if now < state.seen {
                state.started -= state.seen - now;
            }
            let (value, velocity, running) = state.evaluate(now);
            let result = if state.target != target
                || state.following
                || state.spec.is_some_and(|s| s != spec)
            {
                state = State::rest(value, now);
                state.target = target;
                state.spec = Some(spec);
                if matches!(spec, MotionSpec::Spring(_)) {
                    state.velocity = velocity;
                }
                state.evaluate(now)
            } else {
                (value, velocity, running)
            };
            let (value, velocity, running) = result;
            if !running {
                state = State::rest(value, now);
            }
            state.seen = now;
            *slot = state;
            (value, velocity, running)
        });
        if running {
            if !self.settings.animations_enabled(self.ctx) {
                self.ctx.data_mut(|data| {
                    let cache = data.get_temp_mut_or_default::<Cache<N>>(CACHE);
                    cache.states.insert(key, State::rest(target, now));
                });
                return MotionSample {
                    value: T::from_channels(target),
                    velocity: [0.0; N],
                    running: false,
                };
            }
            self.ctx.request_repaint();
        }
        MotionSample {
            value: T::from_channels(value),
            velocity,
            running,
        }
    }

    /// Record a directly controlled gesture pose and release velocity without
    /// requesting repaint. The next spring can continue from this sample.
    pub fn follow<T: MotionValue<N>, const N: usize>(self, value: T, velocity: [f64; N]) {
        let (key, now, previous_frame) = self.key_and_time();
        let value = value.into_channels();
        if !value.iter().all(|v| v.is_finite()) {
            return;
        }
        self.ctx.data_mut(|data| {
            let cache = data.get_temp_mut_or_default::<Cache<N>>(CACHE);
            cache.prune(key.0, now, previous_frame);
            if cache.states.contains_key(&key) || cache.states.len() < TRACK_LIMIT {
                let mut state = State::rest(value, now);
                state.following = true;
                state.velocity = velocity.map(|v| if v.is_finite() { v } else { 0.0 });
                cache.states.insert(key, state);
            }
        });
    }

    /// Read the current pose without advancing state or requesting repaint.
    pub fn inspect<T: MotionValue<N>, const N: usize>(self) -> Option<MotionSample<T, N>> {
        let (key, now, previous_frame) = self.key_and_time();
        let (value, velocity, running) = self.ctx.data(|data| {
            let cache = data
                .get_temp_raw(egui::util::id_type_map::RawKey::new::<Cache<N>>(CACHE))?
                .downcast_ref::<Cache<N>>()?;
            let state = cache
                .states
                .get(&key)
                .filter(|s| !s.expired(now, previous_frame))?;
            Some(state.evaluate(now.max(state.seen)))
        })?;
        Some(MotionSample {
            value: T::from_channels(value),
            velocity,
            running,
        })
    }

    /// Forget this property for the given value type/channel count.
    pub fn reset<const N: usize>(self) {
        forget::<N>(self.ctx, self.id);
    }

    fn key_and_time(self) -> ((ViewportId, Id), f64, f64) {
        self.ctx.input(|input| {
            let now = if input.time.is_finite() {
                input.time
            } else {
                0.0
            };
            let previous = if input.unstable_dt.is_finite() {
                now - f64::from(input.unstable_dt.max(0.0))
            } else {
                now
            };
            ((input.raw.viewport_id, self.id), now, previous)
        })
    }
}

pub(super) fn forget<const N: usize>(ctx: &Context, id: Id) {
    let key = (ctx.viewport_id(), id);
    ctx.data_mut(|data| {
        if let Some(cache) = data
            .get_temp_raw_mut(egui::util::id_type_map::RawKey::new::<Cache<N>>(CACHE))
            .and_then(|cache| cache.downcast_mut::<Cache<N>>())
        {
            cache.states.remove(&key);
        }
    });
}
