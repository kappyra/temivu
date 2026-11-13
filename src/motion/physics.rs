//! Closed-form motion in seconds and caller-defined units.
use super::MotionSettings;

/// A damped harmonic oscillator. Parameters are normalized to natural angular
/// frequency and damping ratio when constructed.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Spring {
    omega: f64,
    ratio: f64,
}

impl Spring {
    /// Construct from mass, stiffness, and viscous damping. Returns `None`
    /// for non-finite, non-positive, or excessively ill-conditioned settings.
    pub fn physical(mass: f64, stiffness: f64, damping: f64) -> Option<Self> {
        if ![mass, stiffness, damping]
            .into_iter()
            .all(|v| v.is_finite() && v > 0.0)
        {
            return None;
        }
        let omega = (stiffness / mass).sqrt();
        let ratio = (damping / mass) / (2.0 * omega);
        ((0.001..=10_000.0).contains(&omega) && (0.001..=10_000.0).contains(&ratio))
            .then_some(Self { omega, ratio })
    }

    /// Use a nominal response time and damping ratio. A ratio of one is
    /// critical. Higher ratios remove bounce and slow the return to rest.
    pub fn response(seconds: f32, ratio: f32) -> Self {
        let seconds = if seconds.is_finite() {
            seconds.clamp(0.04, 2.0)
        } else {
            MotionSettings::default().normal
        };
        let ratio = if ratio.is_finite() {
            ratio.clamp(0.05, 8.0)
        } else {
            0.72
        };
        Self {
            omega: 5.2 / f64::from(seconds),
            ratio: f64::from(ratio),
        }
    }

    pub fn damping_ratio(self) -> f64 {
        self.ratio
    }

    /// Evaluate an initial position and velocity at elapsed time. Velocity is
    /// measured in units per second. The target is constant over the interval.
    pub fn advance(self, position: f64, velocity: f64, target: f64, seconds: f64) -> (f64, f64) {
        let target = if target.is_finite() { target } else { 0.0 };
        if !position.is_finite() || !velocity.is_finite() {
            return (target, 0.0);
        }
        if seconds <= 0.0 {
            return (position, velocity);
        }
        if !seconds.is_finite() {
            return (target, 0.0);
        }
        self.kernel(seconds).apply(position, velocity, target)
    }

    pub(super) fn kernel(self, t: f64) -> Kernel {
        let w = self.omega;
        let z = self.ratio;
        if (z - 1.0).abs() < 1e-8 {
            let wt = w * t;
            let e = (-wt).exp();
            if e == 0.0 {
                return Kernel::REST;
            }
            return Kernel::new(e * (1.0 + wt), e * t, -e * w * wt, e * (1.0 - wt));
        }
        if z < 1.0 {
            let a = w * z;
            let frequency = w * ((1.0 - z) * (1.0 + z)).sqrt();
            let e = (-a * t).exp();
            if e == 0.0 {
                return Kernel::REST;
            }
            let (sin, cos) = (frequency * t).sin_cos();
            let s = sin / frequency;
            return Kernel::new(e * (cos + a * s), e * s, -e * w * w * s, e * (cos - a * s));
        }
        // The reciprocal avoids cancellation in the slow root at high damping.
        let q = z + ((z - 1.0) * (z + 1.0)).sqrt();
        let slow = -w / q;
        let fast = -w * q;
        let es = (slow * t).exp();
        let ef = (fast * t).exp();
        let b = es * ((fast - slow) * t).exp_m1() / (fast - slow);
        Kernel::new(es - slow * b, b, -w * w * b, ef + slow * b)
    }
}

// One kernel is shared across all channels in a track.
pub(super) struct Kernel {
    xx: f64,
    xv: f64,
    vx: f64,
    vv: f64,
}

impl Kernel {
    const REST: Self = Self::new(0.0, 0.0, 0.0, 0.0);

    const fn new(xx: f64, xv: f64, vx: f64, vv: f64) -> Self {
        Self { xx, xv, vx, vv }
    }

    pub(super) fn apply(&self, position: f64, velocity: f64, target: f64) -> (f64, f64) {
        let offset = position - target;
        let position = target + self.xx * offset + self.xv * velocity;
        let velocity = self.vx * offset + self.vv * velocity;
        if position.is_finite() && velocity.is_finite() {
            (position, velocity)
        } else {
            (target, 0.0)
        }
    }
}

/// Exponential momentum decay with a positive friction rate in inverse seconds.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Decay(f64);

impl Decay {
    pub fn new(friction: f64) -> Option<Self> {
        (0.001..=1000.0)
            .contains(&friction)
            .then_some(Self(friction))
    }

    /// Project a release to its resting coordinate. Clamp the result to the
    /// allowed range before starting a decay when movement must stay bounded.
    pub fn destination(self, position: f64, velocity: f64) -> f64 {
        let target = position + velocity / self.0;
        if target.is_finite() {
            target
        } else if position.is_finite() {
            position
        } else {
            0.0
        }
    }

    pub(super) fn fraction(self, seconds: f64) -> (f64, f64) {
        let exponent = -self.0 * seconds;
        (-exponent.exp_m1(), self.0 * exponent.exp())
    }
}

/// Apply bounded resistance outside an edge. The slope starts at one and
/// decreases as the absolute overflow approaches the given visual limit.
pub fn rubber_band(overflow: f32, limit: f32) -> f32 {
    if !overflow.is_finite() || !limit.is_finite() || limit <= 0.0 {
        return 0.0;
    }
    let distance = f64::from(overflow.abs());
    let limit = f64::from(limit);
    (limit * distance / (limit + distance)) as f32 * overflow.signum()
}

/// Time-correct exponential tracking for a moving input. A shorter time
/// constant keeps the visual closer to the pointer.
pub fn tracking_alpha(elapsed: f32, time_constant: f32) -> f32 {
    if elapsed.is_nan() || elapsed <= 0.0 {
        0.0
    } else if !time_constant.is_finite() || time_constant <= 0.0 {
        1.0
    } else {
        -(-elapsed / time_constant).exp_m1()
    }
}
