//! Paint-only fade and defocus for a previously visible surface.
use crate::{GlassRegion, Theme, glass, liquid::fade_shape};
use egui::{LayerId, Rect, Shape, Ui, ViewportId, epaint::ClippedShape, vec2};
use std::time::Duration;

/// Controls how a surface fades when it is removed from the UI.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct FadeOut {
    pub duration: Duration,
    /// Maximum defocus radius in egui points. Zero gives an opacity-only fade.
    pub blur_radius: f32,
}

impl Default for FadeOut {
    fn default() -> Self {
        Self {
            duration: Duration::from_millis(200),
            blur_radius: 4.0,
        }
    }
}

#[derive(Clone, Debug)]
struct Snapshot {
    viewport: ViewportId,
    layer: LayerId,
    bounds: Rect,
    shapes: Vec<ClippedShape>,
    glass: Option<GlassRegion>,
}

/// Keep one state per [`crate::EffectSurface`] that uses a fade-out.
#[derive(Clone, Debug, Default)]
pub struct FadeOutState {
    snapshot: Option<Snapshot>,
    started_at: Option<f64>,
}

impl FadeOutState {
    /// Whether a paint snapshot is available to render when hidden.
    pub fn has_snapshot(&self) -> bool {
        self.snapshot.is_some()
    }

    pub(crate) fn reset(&mut self) {
        self.snapshot = None;
        self.started_at = None;
    }

    pub(crate) fn paint_start(ui: &Ui, theme: &Theme, fade: FadeOut) -> Option<usize> {
        (ui.is_visible() && !ui.is_sizing_pass() && fade.enabled(ui, theme)).then(|| {
            ui.ctx().graphics(|graphics| {
                graphics
                    .get(ui.layer_id())
                    .map_or(0, |list| list.next_idx().0)
            })
        })
    }

    pub(crate) fn capture(
        &mut self,
        ui: &Ui,
        start: Option<usize>,
        bounds: Rect,
        glass: Option<GlassRegion>,
    ) {
        // An egui sizing pass is invisible and may be followed by a real pass
        // at the same timestamp. Keep the last visible paint through that pass.
        if ui.is_sizing_pass() {
            return;
        }
        let Some(start) = start.filter(|_| ui.is_rect_visible(bounds)) else {
            self.reset();
            return;
        };
        let viewport = ui.ctx().viewport_id();
        let layer = ui.layer_id();
        let snapshot = self.snapshot.get_or_insert_with(|| Snapshot {
            viewport,
            layer,
            bounds,
            shapes: Vec::new(),
            glass,
        });
        snapshot.viewport = viewport;
        snapshot.layer = layer;
        snapshot.bounds = bounds;
        snapshot.glass = glass;
        snapshot.shapes.clear();
        ui.ctx().graphics(|graphics| {
            if let Some(list) = graphics.get(layer) {
                snapshot
                    .shapes
                    .extend(list.all_entries().skip(start).map(|entry| {
                        let mut shape = snapshot_shape(&entry.shape);
                        if entry.transform_after_tessellation != egui::emath::TSTransform::IDENTITY
                        {
                            shape.transform(entry.transform_after_tessellation);
                        }
                        ClippedShape::new(entry.clip_rect, shape)
                    }));
            }
        });
        self.started_at = None;
    }

    pub(crate) fn paint(&mut self, ui: &mut Ui, theme: &Theme, fade: FadeOut) {
        let Some(snapshot) = &self.snapshot else {
            return;
        };
        if snapshot.viewport != ui.ctx().viewport_id()
            || snapshot.layer != ui.layer_id()
            || !fade.enabled(ui, theme)
        {
            self.reset();
            return;
        }
        let time = ui.input(|input| input.time);
        let since = *self.started_at.get_or_insert(time);
        let progress = ((time - since) / fade.duration.as_secs_f64()) as f32;
        if !progress.is_finite() || progress >= 1.0 {
            self.reset();
            return;
        }
        let opacity = 1.0 - smoothstep(progress);
        let radius = if fade.blur_radius.is_finite() {
            fade.blur_radius.clamp(0.0, 12.0) * smoothstep(progress / 0.80)
        } else {
            0.0
        };
        let remaining = 1.0 - smoothstep((progress - 0.58) / 0.42);
        let mut size = snapshot.bounds.size();
        if ui.layout().is_vertical() {
            size.y *= remaining;
        } else {
            size.x *= remaining;
        }
        ui.advance_cursor_after_rect(Rect::from_min_size(
            ui.available_rect_before_wrap().min,
            size,
        ));
        if let Some(mut region) = snapshot.glass {
            region.strength *= opacity;
            ui.painter().add(glass::backdrop_shape(ui.ctx(), region));
        }
        // A small weighted kernel defocuses egui paint without a readback or
        // a renderer dependency. Each shape is blurred in its own draw order.
        if ui.is_visible() {
            let clip = ui.clip_rect();
            let layer = ui.layer_id();
            let mut samples = [(egui::Vec2::ZERO, opacity); 25];
            let count = if radius <= 0.25 {
                1
            } else {
                let weights = [0.06_f32, 0.24, 0.40, 0.24, 0.06];
                for (y, wy) in weights.into_iter().enumerate() {
                    for (x, wx) in weights.into_iter().enumerate() {
                        let offset = vec2(x as f32 - 2.0, y as f32 - 2.0) * (radius * 0.5);
                        let alpha = 1.0 - (1.0 - opacity).powf(wx * wy);
                        samples[y * 5 + x] = (offset, alpha);
                    }
                }
                samples.len()
            };
            ui.ctx().graphics_mut(|graphics| {
                let list = graphics.entry(layer);
                for entry in &snapshot.shapes {
                    for &(offset, alpha) in &samples[..count] {
                        paint_sample(list, clip, entry, offset, alpha);
                    }
                }
            });
        }
        ui.ctx().request_repaint();
    }
}

impl FadeOut {
    fn enabled(self, ui: &Ui, theme: &Theme) -> bool {
        !self.duration.is_zero() && theme.motion.animations_enabled(ui.ctx())
    }
}

fn snapshot_shape(shape: &Shape) -> Shape {
    match shape {
        Shape::Callback(_) => Shape::Noop,
        Shape::Vec(shapes) => Shape::Vec(shapes.iter().map(snapshot_shape).collect()),
        shape => shape.clone(),
    }
}

fn paint_sample(
    list: &mut egui::layers::PaintList,
    clip: Rect,
    entry: &ClippedShape,
    offset: egui::Vec2,
    opacity: f32,
) {
    let clip = clip.intersect(entry.clip_rect.translate(offset));
    if opacity < 0.5 / 255.0 || !clip.is_positive() || matches!(entry.shape, Shape::Noop) {
        return;
    }
    let mut shape = entry.shape.clone();
    shape.translate(offset);
    fade_sample(&mut shape, opacity);
    list.add(clip, shape);
}

fn fade_sample(shape: &mut Shape, opacity: f32) {
    match shape {
        Shape::Text(text) => text.opacity_factor *= opacity,
        Shape::Vec(shapes) => shapes
            .iter_mut()
            .for_each(|shape| fade_sample(shape, opacity)),
        _ => fade_shape(shape, opacity),
    }
}

fn smoothstep(value: f32) -> f32 {
    let value = value.clamp(0.0, 1.0);
    value * value * (3.0 - 2.0 * value)
}
