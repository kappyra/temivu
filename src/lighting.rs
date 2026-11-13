//! Optional interaction lighting with opt-in sharing between nearby surfaces.
use crate::Theme;
use egui::{Color32, Context, CornerRadius, Id, Painter, Pos2, Rect, Response, Shape, Vec2};

mod render;
pub(crate) mod spill;
pub use spill::LightSpill;

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
    /// An optional tint. The default is white with a neutral shadow on pale surfaces.
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
    /// Share light with nearby participating surfaces. Independent of local lighting.
    /// `None` disables emission and reception for this material.
    pub spill: Option<LightSpill>,
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
    flow: Vec2,
}

#[derive(Clone, Copy)]
struct Interaction {
    target: Pos2,
    position: Pos2,
    energy: f32,
    time: f64,
    held: bool,
    press_start: Option<f64>,
    keyboard_focus: bool,
    pulse: Option<(f64, Pos2, Vec2)>,
    activated: Option<f64>,
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
            intensity: 0.78,
            opacity: 0.41,
            radius: 84.0,
            softness: 0.56,
            propagation: 1.0,
            attenuation: 1.6,
            inertia: 0.32,
            elevation: 0.35,
            spill: None,
        }
    }

    pub const fn expressive() -> Self {
        let mut style = Self::subtle();
        style.effects.flow = true;
        style.tracking = LightTracking::Spring;
        style.intensity = 0.86;
        style.opacity = 0.56;
        style.radius = 108.0;
        style.elevation = 0.53;
        style
    }

    /// Set an inherited material for Temivu buttons and switches in this context.
    /// The default is disabled.
    pub fn set_context(ctx: &Context, style: Option<Self>) {
        if style.is_some_and(|style| style.spill.is_some()) {
            spill::install(ctx);
        }
        let id = Id::unique("temivu-vesper-default");
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
        ctx.data(|data| data.get_temp(Id::unique("temivu-vesper-default")))
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
        self.paint_on(painter, theme, response, rect, corners, theme.card);
    }

    /// Paint lighting over a known surface color. This lets a default white
    /// highlight adapt to both dark controls and pale controls in one theme.
    pub fn paint_on(
        &self,
        painter: &Painter,
        theme: &Theme,
        response: &Response,
        rect: Rect,
        corners: CornerRadius,
        surface: Color32,
    ) {
        if let Some(shape) =
            self.interactive_shape_on(painter, theme, response, rect, corners.into(), surface)
        {
            painter.add(shape);
        }
        if response.enabled() {
            self.receive(painter, theme, response.id, rect, corners, surface);
        }
    }

    /// Receive nearby interaction light on an arbitrary painted object.
    /// Call after its fill and before its text/icons. This does not capture input.
    /// Both this material and the emitter must opt in through [`Self::spill`].
    /// Illumination stays within the object's silhouette and the parent's clip.
    pub fn receive(
        &self,
        painter: &Painter,
        theme: &Theme,
        id: Id,
        rect: Rect,
        corners: CornerRadius,
        surface: Color32,
    ) {
        if self.spill.is_some() && self.can_paint(painter, rect) {
            spill::receive(
                painter,
                id,
                rect,
                corners.into(),
                &self.bounded(),
                surface,
                theme.card,
            );
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
            flow: Vec2::ZERO,
        };
        if let Some(shape) = style.shape(theme, rect, corners.into(), theme.card, sample) {
            painter.add(shape);
        }
    }

    pub(crate) fn interactive_shape(
        &self,
        painter: &Painter,
        theme: &Theme,
        response: &Response,
        rect: Rect,
        corners: egui::epaint::CornerRadiusF32,
    ) -> Option<Shape> {
        self.interactive_shape_on(painter, theme, response, rect, corners, theme.card)
    }

    pub(crate) fn interactive_shape_on(
        &self,
        painter: &Painter,
        theme: &Theme,
        response: &Response,
        rect: Rect,
        corners: egui::epaint::CornerRadiusF32,
        surface: Color32,
    ) -> Option<Shape> {
        if !response.enabled() || !self.can_paint(painter, rect) {
            return None;
        }
        let style = self.bounded();
        let sample = style.sample(response, theme, rect)?;
        // Keyboard focus keeps its local cue; only pointer-driven light travels.
        if style.spill.is_some()
            && response.ctx.data(|data| {
                data.get_temp::<Interaction>(response.id.with("vesper"))
                    .is_some_and(|state| !state.keyboard_focus)
            })
        {
            spill::emit(painter, response.id, rect, &style, sample);
        }
        style.shape(theme, rect, corners, surface, sample)
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
        surface: Color32,
        sample: Sample,
    ) -> Option<Shape> {
        let ink = render::Ink::new(self.color, surface, theme.card);
        let mesh = render::mesh(rect, corners, self, ink, sample);
        (!mesh.is_empty()).then(|| Shape::mesh(mesh))
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
        let activated = response.clicked();
        let (now, primary_press) = ctx.input(|input| {
            let pointer = &input.pointer;
            let press = if pointer.primary_down() && !input.key_pressed(egui::Key::Escape) {
                pointer
                    .press_start_time()
                    .zip(pointer.press_origin())
                    .map(|(start, origin)| (start, origin, pointer.interact_pos()))
            } else {
                None
            };
            (input.time, press)
        });
        if !now.is_finite() {
            return None;
        }
        let press_start = primary_press.map(|(start, _, _)| start);
        let same_press =
            |state: &Interaction| press_start.is_some() && state.press_start == press_start;
        let previous = ctx
            .data(|data| data.get_temp::<Interaction>(id))
            .filter(|state| {
                let age = now - state.time;
                age >= 0.0 && (age <= 1.0 || same_press(state)) && state.tracking == self.tracking
            });
        let to_local = |point| {
            ctx.layer_transform_from_global(response.layer_id)
                .map_or(point, |transform| transform * point)
        };
        // A hover-only surface can observe a contact without capturing its
        // child's click. egui can also hand an owned touch to its long-press
        // handling. Preserve the original contact until that press ends.
        let passive_hold = !response.sense.interactive()
            && response.contains_pointer()
            && primary_press.is_some_and(|(_, origin, _)| {
                response
                    .interact_rect
                    .intersect(rect)
                    .contains(to_local(origin))
            });
        let held = response.is_pointer_button_down_on()
            || response.dragged()
            || previous.as_ref().is_some_and(same_press)
            || passive_hold;
        let pointer_pressed = held || (activated && response.interact_pointer_pos().is_some());
        let keyboard_focus = response.has_focus()
            && !pointer_pressed
            && (previous.is_some_and(|state| state.keyboard_focus)
                || (response.gained_focus() && !response.hovered()));
        let active = held || response.hovered() || keyboard_focus;
        if !active && !activated && previous.is_none() {
            return None;
        }
        let mut motion = theme.motion;
        motion.normal = 0.10 + self.inertia * 0.30;
        let spring = crate::motion::MotionTrack::new(ctx, id.with("spring-position"), motion);
        if previous.is_none() && self.tracking == LightTracking::Spring {
            spring.reset::<2>();
        }

        // Track normalized widget coordinates so a resize does not move the light
        // into an unrelated part of the next frame.
        let pointer = if held {
            response
                .interact_pointer_pos()
                .or_else(|| primary_press.and_then(|(_, _, position)| position.map(to_local)))
        } else if activated {
            response
                .interact_pointer_pos()
                .or_else(|| response.hover_pos())
        } else {
            response.hover_pos()
        };
        let target = pointer.filter(|p| p.is_finite()).map_or_else(
            || {
                previous
                    .filter(|_| !active || held)
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
        } else if response.hovered() {
            0.48
        } else if keyboard_focus {
            0.36
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
            press_start: None,
            keyboard_focus: false,
            pulse: None,
            activated: None,
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
            && ((held && (!state.held || state.press_start != press_start))
                || (activated && !state.held && state.activated != Some(now)))
        {
            state.pulse = Some((now, target, state.velocity));
        }
        if activated {
            state.activated = Some(now);
        }
        let flowing = animated && self.effects.flow && state.velocity.length_sq() > 1.0;
        if animated {
            let smoothing = crate::motion::tracking_alpha(dt, 0.012 + 0.095 * self.inertia);
            state.position = match self.tracking {
                LightTracking::Direct => target,
                LightTracking::Smooth => {
                    if state.position.distance(target) <= 0.0015 {
                        target
                    } else {
                        state.position.lerp(target, smoothing)
                    }
                }
                LightTracking::Spring => spring
                    .sample(
                        state.position,
                        target,
                        crate::motion::MotionSpec::Spring(crate::motion::Spring::response(
                            motion.normal,
                            0.82,
                        )),
                    )
                    .value
                    .clamp(Pos2::ZERO, Pos2::new(1.0, 1.0)),
            };
            let fade = crate::motion::tracking_alpha(
                dt,
                if target_energy > state.energy {
                    1.0 / 35.0
                } else {
                    1.0 / 19.0
                },
            );
            state.energy = if (state.energy - target_energy).abs() < 0.004 {
                target_energy
            } else {
                egui::lerp(state.energy..=target_energy, fade)
            };
        } else {
            state.position = target;
            state.energy = target_energy;
            state.pulse = None;
        }
        let (pulse, pulse_point, velocity) =
            state
                .pulse
                .map_or((1.0, target, Vec2::ZERO), |(start, point, velocity)| {
                    (
                        ((now - start) as f32 * self.propagation.sqrt() / 0.36).clamp(0.0, 1.0),
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
                if self.tracking == LightTracking::Spring {
                    spring.reset::<2>();
                }
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
        state.press_start = press_start.filter(|_| held);
        state.keyboard_focus = keyboard_focus;
        state.time = now;
        ctx.data_mut(|data| data.insert_temp(id, state));
        let project = |point: Pos2| rect.min + point.to_vec2() * rect.size();
        Some(Sample {
            point: project(if held {
                state.position.lerp(target, 0.90)
            } else {
                state.position
            }),
            energy: state
                .energy
                .max(if held { 0.94 } else { 0.77 * (1.0 - pulse) }),
            pulse_point: project(pulse_point),
            pulse,
            velocity,
            flow: if flowing {
                state.velocity * (0.028 / self.propagation.sqrt())
            } else {
                Vec2::ZERO
            },
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
