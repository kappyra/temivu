//! Optional interaction lighting. Vesper paints inside a widget's silhouette.
use crate::Theme;
use egui::{Color32, Context, CornerRadius, Id, Painter, Pos2, Rect, Response, Shape, Vec2};

mod render;

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum LightQuality {
    Low,
    #[default]
    Balanced,
    High,
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum LightTracking {
    Direct,
    #[default]
    Smooth,
    Spring,
}

/// Choose any combination of effects. Motion preferences still apply.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct LightEffects {
    pub follow: bool,
    pub edge: bool,
    pub scatter: bool,
    pub flow: bool,
    pub particles: bool,
    pub depth: bool,
}

impl LightEffects {
    fn any(self) -> bool {
        self.follow || self.edge || self.scatter || self.flow || self.particles || self.depth
    }
}

/// Lighting material for a control. Out-of-range numeric parameters are bounded at paint time.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Vesper {
    pub effects: LightEffects,
    pub quality: LightQuality,
    pub tracking: LightTracking,
    /// Defaults to a theme-aware color.
    pub color: Option<Color32>,
    pub intensity: f32,
    pub opacity: f32,
    /// Reach of the light, in logical points.
    pub radius: f32,
    pub softness: f32,
    /// Relative speed of the scattering wave and surface flow.
    pub propagation: f32,
    pub attenuation: f32,
    /// Strength of pointer lag.
    pub inertia: f32,
    pub elevation: f32,
}

impl Default for Vesper {
    fn default() -> Self {
        Self::subtle()
    }
}

#[derive(Clone, Copy)]
struct Sample {
    point: Pos2,
    energy: f32,
    pulse_point: Pos2,
    pulse: f32,
    velocity: Vec2,
    flow_phase: f32,
}

#[derive(Clone, Copy)]
struct Interaction {
    target: Pos2,
    position: Pos2,
    energy: f32,
    time: f64,
    held: bool,
    active: bool,
    pulse: Option<(f64, Pos2, Vec2)>,
    activated: Option<f64>,
    flow_until: f64,
    flow_phase: f32,
    velocity: Vec2,
    tracking: LightTracking,
}

impl Vesper {
    pub const fn subtle() -> Self {
        Self {
            effects: LightEffects {
                follow: true,
                edge: true,
                scatter: true,
                flow: false,
                particles: false,
                depth: true,
            },
            quality: LightQuality::Balanced,
            tracking: LightTracking::Smooth,
            color: None,
            intensity: 0.72,
            opacity: 0.34,
            radius: 92.0,
            softness: 0.67,
            propagation: 1.0,
            attenuation: 1.6,
            inertia: 0.32,
            elevation: 0.35,
        }
    }

    pub const fn expressive() -> Self {
        let mut style = Self::subtle();
        style.effects.flow = true;
        style.tracking = LightTracking::Spring;
        style.opacity = 0.47;
        style.radius = 132.0;
        style.elevation = 0.60;
        style
    }

    /// Set an inherited material for Temivu buttons and switches in this context.
    /// The default is disabled.
    pub fn set_context(ctx: &Context, style: Option<Self>) {
        let id = Id::new("temivu-vesper-default");
        if Self::context(ctx) == style {
            return;
        }
        ctx.data_mut(|data| {
            if let Some(style) = style {
                data.insert_temp(id, style);
            } else {
                data.remove::<Self>(id);
            }
        });
        ctx.request_repaint();
    }

    pub fn context(ctx: &Context) -> Option<Self> {
        ctx.data(|data| data.get_temp(Id::new("temivu-vesper-default")))
    }

    /// Paint lighting using this response as the interaction source.
    /// Draw after the surface and before its foreground content.
    pub fn paint(
        &self,
        painter: &Painter,
        theme: &Theme,
        response: &Response,
        rect: Rect,
        corners: CornerRadius,
    ) {
        if let Some(shape) = self.interactive_shape(painter, theme, response, rect, corners.into())
        {
            painter.add(shape);
        }
    }

    /// Paint a steady light at a caller-owned position, with no animation state.
    pub fn paint_at(
        &self,
        painter: &Painter,
        theme: &Theme,
        rect: Rect,
        corners: CornerRadius,
        point: Pos2,
        strength: f32,
    ) {
        let style = self.bounded();
        let energy = bounded(strength, 0.0, 0.0, 1.0);
        if energy <= 0.0 || !point.is_finite() || !style.can_paint(painter, rect) {
            return;
        }
        let sample = Sample {
            point,
            energy,
            pulse_point: point,
            pulse: 1.0,
            velocity: Vec2::ZERO,
            flow_phase: 0.0,
        };
        painter.add(style.shape(theme, rect, corners.into(), sample));
    }

    pub(crate) fn interactive_shape(
        &self,
        painter: &Painter,
        theme: &Theme,
        response: &Response,
        rect: Rect,
        corners: egui::epaint::CornerRadiusF32,
    ) -> Option<Shape> {
        if !response.enabled() || !self.can_paint(painter, rect) {
            return None;
        }
        let style = self.bounded();
        let sample = style.sample(response, theme, rect)?;
        Some(style.shape(theme, rect, corners, sample))
    }

    fn can_paint(&self, painter: &Painter, rect: Rect) -> bool {
        self.effects.any()
            && self.opacity.is_finite()
            && self.opacity > 0.0
            && self.intensity.is_finite()
            && self.intensity > 0.0
            && rect.is_finite()
            && rect.size().is_finite()
            && rect.size().min_elem() > 1.0
            && painter.is_visible()
            && painter.clip_rect().intersects(rect)
    }

    fn shape(
        &self,
        theme: &Theme,
        rect: Rect,
        corners: egui::epaint::CornerRadiusF32,
        sample: Sample,
    ) -> Shape {
        let color = self.color.unwrap_or_else(|| {
            if theme.dark {
                Color32::from_rgb(153, 206, 248)
            } else {
                Color32::from_rgb(57, 117, 170)
            }
        });
        Shape::mesh(render::mesh(rect, corners, self, color, sample))
    }

    fn bounded(self) -> Self {
        let defaults = Self::subtle();
        Self {
            intensity: bounded(self.intensity, defaults.intensity, 0.0, 1.0),
            opacity: bounded(self.opacity, defaults.opacity, 0.0, 1.0),
            radius: bounded(self.radius, defaults.radius, 12.0, 420.0),
            softness: bounded(self.softness, defaults.softness, 0.0, 1.0),
            propagation: bounded(self.propagation, defaults.propagation, 0.25, 4.0),
            attenuation: bounded(self.attenuation, defaults.attenuation, 0.5, 4.0),
            inertia: bounded(self.inertia, defaults.inertia, 0.0, 1.0),
            elevation: bounded(self.elevation, defaults.elevation, 0.0, 1.0),
            ..self
        }
    }

    fn sample(&self, response: &Response, theme: &Theme, rect: Rect) -> Option<Sample> {
        let ctx = &response.ctx;
        let id = response.id.with("vesper");
        let held = response.is_pointer_button_down_on() || response.dragged();
        let activated = response.clicked();
        let active = held || response.hovered() || response.has_focus();
        let now = ctx.input(|input| input.time);
        if !now.is_finite() {
            return None;
        }
        let previous = ctx
            .data(|data| data.get_temp::<Interaction>(id))
            .filter(|state| {
                (0.0..=1.0).contains(&(now - state.time)) && state.tracking == self.tracking
            });
        if !active && !activated && previous.is_none() {
            return None;
        }
        if previous.is_none() && self.tracking == LightTracking::Spring {
            crate::motion::reset(ctx, id.with("x"));
            crate::motion::reset(ctx, id.with("y"));
        }

        // Track normalized widget coordinates so a resize does not move the light
        // into an unrelated part of the next frame.
        let pointer = if held || activated {
            response
                .interact_pointer_pos()
                .or_else(|| response.hover_pos())
        } else {
            response.hover_pos()
        };
        let target = pointer.filter(|p| p.is_finite()).map_or_else(
            || {
                previous
                    .filter(|_| !active)
                    .map_or(Pos2::new(0.5, 0.5), |state| state.target)
            },
            |p| {
                ((p - rect.min) / rect.size())
                    .clamp(Vec2::ZERO, Vec2::splat(1.0))
                    .to_pos2()
            },
        );
        let target_energy = if held {
            1.0
        } else if active {
            0.65
        } else {
            0.0
        };
        let animated = theme.motion.animations_enabled(ctx);
        let mut state = previous.unwrap_or(Interaction {
            target,
            position: target,
            energy: target_energy,
            time: now,
            held: false,
            active: false,
            pulse: None,
            activated: None,
            flow_until: now,
            flow_phase: 0.0,
            velocity: Vec2::ZERO,
            tracking: self.tracking,
        });
        let dt = (now - state.time).clamp(0.0, 0.1) as f32;
        let moved = active && (target - state.target).length_sq() > 0.000004;
        if moved && dt > 0.0 {
            state.velocity = ((target - state.target) * rect.size() / dt)
                .clamp(Vec2::splat(-460.0), Vec2::splat(460.0));
        } else if dt > 0.0 {
            state.velocity *= (-14.0 * dt).exp();
        }
        let has_pulse = self.effects.scatter || self.effects.particles;
        if animated
            && has_pulse
            && ((held && !state.held) || (activated && !state.held && state.activated != Some(now)))
        {
            state.pulse = Some((now, target, state.velocity));
        }
        if activated {
            state.activated = Some(now);
        }
        if self.effects.flow
            && animated
            && (moved || (active && !state.active) || (held && !state.held))
        {
            state.flow_until = now + f64::from(0.36 / self.propagation.sqrt());
        }
        let flowing = animated && self.effects.flow && now < state.flow_until;
        if flowing {
            state.flow_phase =
                (state.flow_phase + dt * self.propagation * 2.7).rem_euclid(std::f32::consts::TAU);
        }
        if animated {
            let smoothing = 1.0 - (-dt / (0.012 + 0.095 * self.inertia)).exp();
            state.position = match self.tracking {
                LightTracking::Direct => target,
                LightTracking::Smooth => {
                    if state.position.distance(target) <= 0.0015 {
                        target
                    } else {
                        state.position.lerp(target, smoothing)
                    }
                }
                LightTracking::Spring => {
                    let mut motion = theme.motion;
                    motion.normal = 0.10 + self.inertia * 0.30;
                    let spring = |axis, initial, target| {
                        crate::motion::spring_f32_with_settings(
                            ctx,
                            id.with(axis),
                            initial,
                            target,
                            0.82,
                            motion,
                        )
                        .clamp(0.0, 1.0)
                    };
                    Pos2::new(
                        spring("x", state.position.x, target.x),
                        spring("y", state.position.y, target.y),
                    )
                }
            };
            let fade = 1.0 - (-dt * 22.0).exp();
            state.energy = if (state.energy - target_energy).abs() < 0.004 {
                target_energy
            } else {
                egui::lerp(state.energy..=target_energy, fade)
            };
        } else {
            state.position = target;
            state.energy = target_energy;
            state.pulse = None;
            state.flow_phase = 0.0;
        }
        let (pulse, pulse_point, velocity) =
            state
                .pulse
                .map_or((1.0, target, Vec2::ZERO), |(start, point, velocity)| {
                    (
                        ((now - start) as f32 * self.propagation.sqrt() / 0.48).clamp(0.0, 1.0),
                        point,
                        velocity,
                    )
                });
        if pulse >= 1.0 {
            state.pulse = None;
        }
        if state.energy <= 0.0 && pulse >= 1.0 && !active {
            if previous.is_some() {
                ctx.data_mut(|data| data.remove::<Interaction>(id));
                crate::motion::reset(ctx, id.with("x"));
                crate::motion::reset(ctx, id.with("y"));
            }
            return None;
        }
        if flowing
            || pulse < 1.0
            || (state.energy - target_energy).abs() >= 0.004
            || (animated && self.tracking == LightTracking::Smooth && state.position != target)
        {
            ctx.request_repaint();
        }
        state.target = target;
        state.held = held;
        state.active = active;
        state.time = now;
        ctx.data_mut(|data| data.insert_temp(id, state));
        let project = |point: Pos2| rect.min + point.to_vec2() * rect.size();
        Some(Sample {
            point: project(state.position),
            energy: state.energy.max(0.74 * (1.0 - pulse)),
            pulse_point: project(pulse_point),
            pulse,
            velocity,
            flow_phase: state.flow_phase,
        })
    }
}

fn bounded(value: f32, fallback: f32, min: f32, max: f32) -> f32 {
    if value.is_finite() {
        value.clamp(min, max)
    } else {
        fallback
    }
}
