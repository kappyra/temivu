//! Reversible shared-element motion between measured egui surfaces.
mod paint;

use crate::Theme;
use crate::motion::{MotionSpec, MotionTrack, MotionValue, SpringPreset};
use egui::{Color32, Context, Id, Painter, Rect, Response, epaint::CornerRadiusF32};

pub use paint::SharedElementSnapshot;

/// One end of a transition, in viewport coordinates.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct SharedElementEndpoint {
    pub rect: Rect,
    pub corners: CornerRadiusF32,
    pub fill: Color32,
}

impl SharedElementEndpoint {
    pub fn new(rect: Rect, corners: impl Into<CornerRadiusF32>, fill: Color32) -> Self {
        Self {
            rect,
            corners: corners.into(),
            fill,
        }
    }

    /// Read a widget's bounds and account for transforms on its egui layer.
    pub fn from_response(
        response: &Response,
        corners: impl Into<CornerRadiusF32>,
        fill: Color32,
    ) -> Self {
        let transform = response
            .ctx
            .layer_transform_to_global(response.layer_id)
            .unwrap_or_default();
        Self::new(
            transform.mul_rect(response.rect),
            corners.into() * transform.scaling,
            fill,
        )
    }

    fn channels(self, progress: f64) -> Option<Channels> {
        if !self.rect.is_finite() || !self.rect.is_positive() || !self.rect.size().is_finite() {
            return None;
        }
        let center = self.rect.center();
        if !center.is_finite() {
            return None;
        }
        let c = bounded_corners(self.corners, self.rect);
        let values = [
            center.x,
            center.y,
            self.rect.width(),
            self.rect.height(),
            c.nw,
            c.ne,
            c.se,
            c.sw,
        ];
        Some(Channels([
            progress,
            values[0].into(),
            values[1].into(),
            values[2].into(),
            values[3].into(),
            values[4].into(),
            values[5].into(),
            values[6].into(),
            values[7].into(),
        ]))
    }
}

/// Which live endpoint should take over when motion settles.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum SharedElementSide {
    #[default]
    Source,
    Destination,
}

/// Sample either an animated destination or a directly controlled gesture.
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum SharedElementDrive {
    SettleTo(SharedElementSide),
    /// Fraction along the path and velocity in fractions per second.
    Follow {
        progress: f32,
        velocity: f32,
    },
}

/// How the two contents exchange visibility inside the moving surface.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum SharedElementFade {
    #[default]
    Through,
    Cross,
}

/// Uniform scale to preserve the aspect of text, icons and images.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum SharedElementFit {
    #[default]
    Contain,
    Cover,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct SharedElementStyle {
    pub spring: SpringPreset,
    pub fade: SharedElementFade,
    pub fit: SharedElementFit,
    /// Color at the destination. The scrim is transparent at the source.
    pub scrim: Color32,
}

impl Default for SharedElementStyle {
    fn default() -> Self {
        Self {
            spring: SpringPreset::Smooth,
            fade: SharedElementFade::Through,
            fit: SharedElementFit::Contain,
            scrim: Color32::TRANSPARENT,
        }
    }
}

/// Geometry and compositing values for the current frame.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct SharedElementPose {
    pub rect: Rect,
    pub corners: CornerRadiusF32,
    pub fill: Color32,
    pub progress: f32,
    /// Progress units per second. Positive motion travels toward the destination.
    pub velocity: f32,
    /// Opacity for source and destination content, respectively.
    pub opacity: [f32; 2],
    /// Present when the caller can replace the traveling paint with a live widget.
    pub settled: Option<SharedElementSide>,
}

/// A viewport-local track addressed by an application-owned stable ID.
///
/// Sample every frame from the same ID. A gesture can take over at the last
/// visible progress, and a subsequent spring inherits its release velocity.
/// Use separate IDs for transitions that can run at the same time.
#[derive(Clone, Copy, Debug)]
pub struct SharedElementTransition {
    id: Id,
    style: SharedElementStyle,
}

impl SharedElementTransition {
    pub fn new(id: Id) -> Self {
        Self {
            id,
            style: SharedElementStyle::default(),
        }
    }

    pub fn style(mut self, style: SharedElementStyle) -> Self {
        self.style = style;
        self
    }

    /// Advance the existing segment to the current event time, then retarget.
    /// Pass the latest measured bounds to handle resizing while in flight.
    pub fn sample(
        self,
        ctx: &Context,
        theme: &Theme,
        source: SharedElementEndpoint,
        destination: SharedElementEndpoint,
        drive: SharedElementDrive,
    ) -> Option<SharedElementPose> {
        let from = source.channels(0.0)?;
        let to = destination.channels(1.0)?;
        let track = self.track(ctx, theme);
        let (channels, velocity, settled) = match drive {
            SharedElementDrive::Follow { progress, velocity } => {
                let progress = if progress.is_finite() {
                    f64::from(progress.clamp(0.0, 1.0))
                } else {
                    track
                        .inspect::<Channels, 9>()
                        .map_or(0.0, |s| s.value.0[0].clamp(0.0, 1.0))
                };
                let mut velocity = if velocity.is_finite() {
                    f64::from(velocity.clamp(-10.0, 10.0))
                } else {
                    0.0
                };
                if (progress <= 0.0 && velocity < 0.0) || (progress >= 1.0 && velocity > 0.0) {
                    velocity = 0.0;
                }
                let phases = std::array::from_fn(|i| to.0[i] - from.0[i]);
                let channels = Channels(std::array::from_fn(|i| from.0[i] + progress * phases[i]));
                track.follow(channels, phases.map(|delta| velocity * delta));
                (channels, velocity, None)
            }
            SharedElementDrive::SettleTo(side) => {
                let target = if side == SharedElementSide::Source {
                    from
                } else {
                    to
                };
                let spec = MotionSpec::Spring(self.style.spring.spring(theme.motion));
                let sample = track.sample_with_rest(from, target, spec, |current, target| {
                    current[0] == target[0]
                });
                let near_rest = (0..9).all(|i| {
                    let (distance, speed) = if i == 0 { (0.001, 0.015) } else { (0.045, 1.1) };
                    (sample.value.0[i] - target.0[i]).abs() <= distance
                        && sample.velocity[i].abs() <= speed
                });
                if near_rest {
                    if sample.running {
                        track.follow(target, [0.0; 9]);
                    }
                    (target, 0.0, Some(side))
                } else {
                    (sample.value, sample.velocity[0], None)
                }
            }
        };
        Some(self.pose(channels, velocity, settled, source.fill, destination.fill))
    }

    /// Forget progress and velocity, for example when an ID is reused for new content.
    pub fn reset(self, ctx: &Context, theme: &Theme) {
        self.track(ctx, theme).reset::<9>();
    }

    /// Paint a snapshot of each side in the traveling rounded surface.
    ///
    /// The painter determines z order and the outer clip. Only allocate or
    /// recapture snapshots when the contents change. Keep input and focus in
    /// the live endpoint UIs, and use a separate response to handle scrubbing.
    /// Pass the same snapshot twice to keep one shared image or icon at full opacity.
    pub fn paint(
        self,
        painter: &Painter,
        pose: SharedElementPose,
        content: [Option<&SharedElementSnapshot>; 2],
    ) {
        paint::paint(painter, self.style, pose, content);
    }

    fn track<'a>(self, ctx: &'a Context, theme: &Theme) -> MotionTrack<'a> {
        MotionTrack::new(ctx, self.id.with("shared-element"), theme.motion)
    }

    fn pose(
        self,
        Channels([progress, x, y, width, height, nw, ne, se, sw]): Channels,
        velocity: f64,
        settled: Option<SharedElementSide>,
        from: Color32,
        to: Color32,
    ) -> SharedElementPose {
        let rect = Rect::from_center_size(
            egui::pos2(x as f32, y as f32),
            egui::vec2((width as f32).max(0.1), (height as f32).max(0.1)),
        );
        let corners = bounded_corners(
            CornerRadiusF32 {
                nw: nw as f32,
                ne: ne as f32,
                se: se as f32,
                sw: sw as f32,
            },
            rect,
        );
        let progress = (progress as f32).clamp(0.0, 1.0);
        let opacity = match self.style.fade {
            SharedElementFade::Through => [
                1.0 - blend(progress, 0.06, 0.42),
                blend(progress, 0.48, 0.88),
            ],
            SharedElementFade::Cross => {
                let t = blend(progress, 0.12, 0.84);
                [1.0 - t, t]
            }
        };
        SharedElementPose {
            rect,
            corners,
            fill: from.lerp_to_gamma(to, progress),
            progress,
            velocity: velocity as f32,
            opacity,
            settled,
        }
    }
}

#[derive(Clone, Copy, PartialEq)]
struct Channels([f64; 9]);

impl MotionValue<9> for Channels {
    fn into_channels(self) -> [f64; 9] {
        self.0
    }
    fn from_channels(samples: [f64; 9]) -> Self {
        Self(samples)
    }
}

fn bounded_corners(corners: CornerRadiusF32, rect: Rect) -> CornerRadiusF32 {
    let limit = rect.size().min_elem() / 2.0;
    let bound = |radius: f32| {
        if radius.is_finite() {
            radius.clamp(0.0, limit)
        } else {
            0.0
        }
    };
    CornerRadiusF32 {
        nw: bound(corners.nw),
        ne: bound(corners.ne),
        se: bound(corners.se),
        sw: bound(corners.sw),
    }
}

fn blend(progress: f32, start: f32, end: f32) -> f32 {
    let t = ((progress - start) / (end - start)).clamp(0.0, 1.0);
    t * t * (3.0 - 2.0 * t)
}
