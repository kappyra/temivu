use std::f32::consts::{PI, TAU};

use egui::{Color32, Id, Mesh, Painter, Pos2, Rect, Shape, Ui, Vec2};

use super::SpinnerStyle;
use crate::theme::MotionSettings;

const DOTS: usize = 6;
const SEGMENTS: usize = 16;
const PASSES_PER_SIDE: f32 = 5.0;
const PASSES: f32 = 2.0 * PASSES_PER_SIDE;
const SWAP_START: f32 = 4.0;
const SWAP_STAGGER: f32 = 0.128;
const SWAP_DURATION: f32 = 0.36;

#[derive(Clone, Copy)]
struct Clock {
    started: f64,
    last_frame: u64,
}

#[derive(Clone, Copy)]
pub(super) struct Frame {
    passed: f32,
    spread: f32,
}

impl Frame {
    pub fn fixed(phase: f32) -> Self {
        Self {
            passed: phase.rem_euclid(1.0) * PASSES,
            spread: 1.0,
        }
    }

    pub fn animate(ui: &Ui, id: Id, style: SpinnerStyle, settings: MotionSettings) -> Self {
        let ctx = ui.ctx();
        let now = ui.input(|input| input.time);
        if !now.is_finite() {
            return Self::fixed(0.018);
        }
        let frame = ctx.cumulative_frame_nr();
        let key = id.with("luminous-spinner");
        let started = ctx.data_mut(|data| {
            let clock = data.get_temp::<Clock>(key).filter(|clock| {
                frame <= clock.last_frame.saturating_add(1) && now >= clock.started
            });
            let started = clock.map_or(now, |clock| clock.started);
            data.insert_temp(
                key,
                Clock {
                    started,
                    last_frame: frame,
                },
            );
            started
        });
        let period = if style.period.is_finite() {
            style.period.clamp(0.28, 5.0)
        } else {
            SpinnerStyle::default().period
        };
        let elapsed =
            (now - started) / f64::from(period * settings.normal_scale().clamp(0.25, 4.0));
        ctx.request_repaint();
        Self {
            passed: elapsed.rem_euclid(f64::from(PASSES)) as f32,
            spread: smoothstep((elapsed / 0.36).min(1.0) as f32),
        }
    }

    fn dots(self) -> [Dot; DOTS] {
        let side = self.passed >= PASSES_PER_SIDE;
        let swap = self.passed.rem_euclid(PASSES_PER_SIDE) - SWAP_START;
        let drift = self.passed / PASSES - 5.0 / 12.0;
        std::array::from_fn(|index| {
            let angle = drift + index as f32 / DOTS as f32 + if side { 0.5 } else { 0.0 };
            let axis = Vec2::angled(TAU * angle);
            // Start each dot's turn just after the preceding one. Projection
            // through the center lets a moving dot pass over a settled dot.
            let step = ((swap - index as f32 * SWAP_STAGGER) / SWAP_DURATION).clamp(0.0, 1.0);
            let turn = smoothstep(step);
            let (forward, across) = (PI * turn).sin_cos();
            let forward = forward.max(0.0);
            let wave =
                |angle: f32| (0.5 + 0.5 * (TAU * (self.passed - angle - 0.07)).cos()).powf(1.55);
            let light = egui::lerp(wave(angle)..=wave(angle + 0.5), turn);
            let energy = egui::lerp(light..=1.0, 0.65 * forward);
            let orbit = (0.305 + 0.010 * light) * self.spread;
            Dot {
                offset: axis * (orbit * across) + axis.rot90() * (0.054 * forward * self.spread),
                axis: (axis.rot90() + axis * (0.85 * forward)).normalized(),
                depth: forward * (1.0 + 0.3 * axis.y),
                energy,
            }
        })
    }

    pub fn paint(self, painter: &Painter, rect: Rect, style: SpinnerStyle, foreground: Color32) {
        let glow = if style.glow.is_finite() {
            style.glow.clamp(0.0, 1.0)
        } else {
            0.0
        };
        let ink = style.color.unwrap_or(foreground);
        let mut dots = self.dots();
        dots.sort_by(|a, b| a.depth.total_cmp(&b.depth));
        let unit = std::array::from_fn(|i| Vec2::angled(TAU * i as f32 / SEGMENTS as f32));
        let mut mesh = Mesh::default();
        let halo = usize::from(glow > 0.0);
        mesh.vertices
            .reserve(DOTS * (2 * SEGMENTS + 1 + halo * (3 * SEGMENTS + 1)));
        mesh.indices.reserve(DOTS * SEGMENTS * (9 + 15 * halo));
        let size = rect.width();
        for dot in dots {
            let center = rect.center() + dot.offset * size;
            let stretch = 1.0 + 0.18 * dot.energy + 0.20 * dot.depth;
            let radius = size * (0.043 + 0.058 * dot.energy + 0.013 * dot.depth);
            let brush = Brush {
                center,
                axis: dot.axis,
                stretch,
                unit: &unit,
            };
            if glow > 0.0 {
                let emission = ink
                    .additive()
                    .gamma_multiply(glow * (0.025 + 0.54 * dot.energy));
                brush.paint(
                    &mut mesh,
                    (radius * 2.65).min(size * 0.182 / stretch),
                    emission,
                    &[
                        (0.27, emission),
                        (0.57, emission.gamma_multiply(0.28)),
                        (1.0, Color32::TRANSPARENT),
                    ],
                );
            }
            let core = ink.gamma_multiply(0.68 + 0.32 * dot.energy);
            brush.paint(
                &mut mesh,
                radius,
                core,
                &[(0.74, core), (1.0, Color32::TRANSPARENT)],
            );
        }
        painter.add(Shape::mesh(mesh));
    }
}

fn smoothstep(t: f32) -> f32 {
    t * t * (3.0 - 2.0 * t)
}

#[derive(Clone, Copy)]
struct Dot {
    offset: Vec2,
    axis: Vec2,
    depth: f32,
    energy: f32,
}

struct Brush<'a> {
    center: Pos2,
    axis: Vec2,
    stretch: f32,
    unit: &'a [Vec2; SEGMENTS],
}

impl Brush<'_> {
    fn paint(&self, mesh: &mut Mesh, radius: f32, core: Color32, rings: &[(f32, Color32)]) {
        let first = mesh.vertices.len() as u32;
        mesh.colored_vertex(self.center, core);
        let minor = self.axis.rot90();
        for &(scale, color) in rings {
            for v in self.unit {
                let offset = self.axis * (v.x * self.stretch) + minor * v.y;
                mesh.colored_vertex(self.center + offset * (radius * scale), color);
            }
        }
        let count = SEGMENTS as u32;
        for index in 0..count {
            let a = first + 1 + index;
            let b = first + 1 + (index + 1) % count;
            mesh.add_triangle(first, a, b);
            for row in 0..(rings.len() as u32 - 1) {
                let a = a + row * count;
                let b = b + row * count;
                mesh.add_triangle(a, a + count, b);
                mesh.add_triangle(b, a + count, b + count);
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn each_dot_crosses_the_center_then_takes_its_opposite_place() {
        let pose = |passed| Frame::fixed(passed / PASSES).dots();
        for start in [0.0, PASSES_PER_SIDE] {
            let before = pose(start + SWAP_START);
            let after = pose(start + PASSES_PER_SIDE - 0.005);
            for index in 0..DOTS {
                let elapsed = SWAP_START + index as f32 * SWAP_STAGGER + SWAP_DURATION / 2.0;
                let crossing = pose(start + elapsed);
                assert!(crossing[index].offset.length() < 0.065);
                assert!(crossing[(index + 3) % DOTS].offset.length() > 0.28);
                assert!(before[index].offset.dot(after[index].offset) < -0.07);
            }
        }
    }

    #[test]
    fn the_orbit_and_light_repeat_without_jumping() {
        let at = Frame::fixed(0.0).dots();
        let end = Frame::fixed(1.0).dots();
        let approaching = Frame::fixed(1.0 - 0.00001).dots();
        for ((a, b), c) in at.into_iter().zip(end).zip(approaching) {
            assert_eq!(a.offset, b.offset);
            assert!((a.offset - c.offset).length() < 0.001);
            assert!((a.energy - c.energy).abs() < 0.002);
        }
    }
}
