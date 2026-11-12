//! Resolve nearby light after all opted-in objects have painted for this pass.
use std::collections::HashMap;

use egui::{
    Color32, Context, Id, LayerId, Painter, Pos2, Rect, Shape, Ui, Vec2, ViewportId,
    epaint::{CornerRadiusF32, RectShape},
    layers::{GraphicLayers, ShapeIdx},
    plugin::Plugin,
};

use super::{LightEffects, LightQuality, Sample, Vesper, bounded, render};

const MAX_EMITTERS: usize = 16;
const MAX_RECEIVERS: usize = 256;

/// Optional light transfer between nearby surfaces in the same UI layer.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct LightSpill {
    /// Maximum reach in logical points.
    pub radius: f32,
    /// Brightness of the emitted light, between zero and one.
    pub intensity: f32,
}

impl Default for LightSpill {
    fn default() -> Self {
        Self {
            radius: 144.0,
            intensity: 0.55,
        }
    }
}

impl LightSpill {
    fn bounded(self) -> Self {
        let defaults = Self::default();
        Self {
            radius: bounded(self.radius, defaults.radius, 12.0, 420.0),
            intensity: bounded(self.intensity, defaults.intensity, 0.0, 1.0),
        }
    }
}

#[derive(Clone, Copy)]
struct Marker {
    id: Id,
    layer: LayerId,
    slot: ShapeIdx,
    keep_background: bool,
}

impl Marker {
    fn update(self, graphics: &mut GraphicLayers, apply: impl FnOnce(&mut Shape, RectShape, Rect)) {
        if let Some(list) = graphics.get_mut(self.layer) {
            list.mutate_shape(self.slot, |entry| {
                let shape = if self.keep_background {
                    let Shape::Vec(shapes) = &mut entry.shape else {
                        return;
                    };
                    let Some(shape) = shapes.last_mut() else {
                        return;
                    };
                    shape
                } else {
                    &mut entry.shape
                };
                if matches!(shape, Shape::Rect(rect) if rect.fill == Color32::TRANSPARENT)
                    && let Shape::Rect(rect) = std::mem::replace(shape, Shape::Noop)
                {
                    apply(shape, rect, entry.clip_rect);
                }
            });
        }
    }
}

struct Emitter {
    marker: Marker,
    position: Vec2,
    original_width: f32,
    radius: f32,
    energy: f32,
    color: Option<Color32>,
}

#[derive(Clone, Copy)]
struct ActiveSource {
    marker: Marker,
    point: Pos2,
    radius: f32,
    energy: f32,
    color: Option<Color32>,
}

struct Receiver {
    marker: Marker,
    corners: CornerRadiusF32,
    original_width: f32,
    style: Vesper,
    surface: Color32,
    fallback: Color32,
    opacity: f32,
}

#[derive(Default)]
struct Scene {
    pass: u64,
    emitters: Vec<Emitter>,
    receivers: Vec<Receiver>,
}

#[derive(Default)]
struct SpillPlugin {
    pending: HashMap<ViewportId, Scene>,
}

impl SpillPlugin {
    fn scene(&mut self, ctx: &Context) -> &mut Scene {
        let pass = ctx.cumulative_pass_nr();
        let scene = self.pending.entry(ctx.viewport_id()).or_default();
        if scene.pass != pass {
            scene.pass = pass;
            scene.emitters.clear();
            scene.receivers.clear();
        }
        scene
    }
}

impl Plugin for SpillPlugin {
    fn debug_name(&self) -> &'static str {
        "Temivu Vesper spill"
    }

    fn setup(&mut self, ctx: &Context) {
        // egui snapshots the plugin list before the pass. If installed during
        // drawing, the next repaint will be the first pass that resolves spill.
        ctx.request_repaint();
    }

    fn on_end_pass(&mut self, ui: &mut Ui) {
        let ctx = ui.ctx();
        let Some(mut scene) = self.pending.remove(&ctx.viewport_id()) else {
            return;
        };
        if scene.pass != ctx.cumulative_pass_nr() {
            return;
        }

        ctx.graphics_mut(|graphics| {
            let mut sources = [None; MAX_EMITTERS];
            for (source, emitter) in sources.iter_mut().zip(scene.emitters.drain(..)) {
                emitter.marker.update(graphics, |_, marker, clip| {
                    let point = marker.rect.min + emitter.position * marker.rect.size();
                    if marker.rect.is_positive() && point.is_finite() && clip.contains(point) {
                        let scale = marker.rect.width() / emitter.original_width;
                        *source = Some(ActiveSource {
                            marker: emitter.marker,
                            point,
                            radius: (emitter.radius * scale).clamp(1.0, 420.0),
                            energy: emitter.energy,
                            color: emitter.color,
                        });
                    }
                });
            }
            for receiver in scene.receivers.drain(..) {
                receiver.marker.update(graphics, |slot, marker, clip| {
                    let bounds = marker.rect.intersect(clip);
                    if !bounds.is_positive() {
                        return;
                    }
                    let source = sources
                        .iter()
                        .flatten()
                        .filter(|source| {
                            source.marker.layer == receiver.marker.layer
                                && source.marker.id != receiver.marker.id
                        })
                        .filter_map(|source| {
                            let proximity = 1.0
                                - bounds.distance_sq_to_pos(source.point) / source.radius.powi(2);
                            (proximity > 0.0).then_some((source, source.energy * proximity.powi(2)))
                        })
                        .max_by(|(_, a), (_, b)| a.total_cmp(b));
                    let Some((source, _)) = source else {
                        return;
                    };
                    let mut style = receiver.style;
                    style.quality = LightQuality::Low;
                    style.radius = source.radius;
                    style.effects = LightEffects {
                        follow: true,
                        edge: style.effects.edge,
                        ..LightEffects::default()
                    };
                    let ink = render::Ink::new(
                        source.color.or(style.color),
                        receiver.surface,
                        receiver.fallback,
                    );
                    let sample = Sample {
                        point: source.point,
                        energy: source.energy * receiver.opacity,
                        pulse_point: source.point,
                        pulse: 1.0,
                        velocity: Vec2::ZERO,
                        flow: Vec2::ZERO,
                    };
                    let scale = marker.rect.width() / receiver.original_width;
                    let mesh =
                        render::mesh(marker.rect, receiver.corners * scale, &style, ink, sample);
                    if !mesh.is_empty() {
                        *slot = Shape::mesh(mesh);
                    }
                });
            }
        });
    }
}

pub(crate) fn install(ctx: &Context) {
    ctx.plugin_or_default::<SpillPlugin>();
}

pub(super) fn emit(painter: &Painter, id: Id, rect: Rect, style: &Vesper, sample: Sample) {
    let Some(settings) = style.spill.map(LightSpill::bounded) else {
        return;
    };
    let energy = sample.energy * settings.intensity * style.intensity * painter.opacity();
    if !energy.is_finite()
        || energy <= 0.004
        || !painter.clip_rect().contains(sample.point)
        || style.color.is_some_and(|color| color.a() == 0)
    {
        return;
    }
    let ctx = painter.ctx();
    let plugin = ctx.plugin_or_default::<SpillPlugin>();
    let mut plugin = plugin.lock();
    let scene = plugin.scene(ctx);
    if scene.emitters.len() == MAX_EMITTERS {
        return;
    }
    let slot = painter.add(marker_shape(rect));
    scene.emitters.push(Emitter {
        marker: Marker {
            id,
            layer: painter.layer_id(),
            slot,
            keep_background: false,
        },
        position: (sample.point - rect.min) / rect.size(),
        original_width: rect.width(),
        radius: settings.radius,
        energy: energy.clamp(0.0, 1.0),
        color: style.color,
    });
}

fn marker_shape(rect: Rect) -> Shape {
    RectShape::filled(rect, 0, Color32::TRANSPARENT).into()
}

pub(crate) fn receive(
    painter: &Painter,
    id: Id,
    rect: Rect,
    corners: CornerRadiusF32,
    style: &Vesper,
    surface: Color32,
    fallback: Color32,
) {
    register_receiver(painter, id, rect, corners, style, surface, fallback, None);
}

#[allow(clippy::too_many_arguments)]
pub(crate) fn receive_at(
    painter: &Painter,
    id: Id,
    rect: Rect,
    corners: CornerRadiusF32,
    style: &Vesper,
    surface: Color32,
    fallback: Color32,
    slot: ShapeIdx,
) {
    register_receiver(
        painter,
        id,
        rect,
        corners,
        style,
        surface,
        fallback,
        Some((slot, false)),
    );
}

#[allow(clippy::too_many_arguments)]
pub(crate) fn receive_background_at(
    painter: &Painter,
    id: Id,
    rect: Rect,
    corners: CornerRadiusF32,
    style: &Vesper,
    surface: Color32,
    fallback: Color32,
    slot: ShapeIdx,
) {
    register_receiver(
        painter,
        id,
        rect,
        corners,
        style,
        surface,
        fallback,
        Some((slot, true)),
    );
}

#[allow(clippy::too_many_arguments)]
fn register_receiver(
    painter: &Painter,
    id: Id,
    rect: Rect,
    corners: CornerRadiusF32,
    style: &Vesper,
    surface: Color32,
    fallback: Color32,
    placement: Option<(ShapeIdx, bool)>,
) {
    if !style
        .spill
        .is_some_and(|settings| settings.bounded().intensity > 0.0)
        || !style.can_paint(painter, rect)
    {
        return;
    }
    let ctx = painter.ctx();
    let plugin = ctx.plugin_or_default::<SpillPlugin>();
    let mut plugin = plugin.lock();
    let scene = plugin.scene(ctx);
    if scene.receivers.len() == MAX_RECEIVERS {
        return;
    }
    let layer = painter.layer_id();
    let (slot, keep_background) = match placement {
        None => (painter.add(marker_shape(rect)), false),
        Some((slot, keep_background)) => {
            if keep_background {
                ctx.graphics_mut(|graphics| {
                    graphics.entry(layer).mutate_shape(slot, |entry| {
                        let background = std::mem::replace(&mut entry.shape, Shape::Noop);
                        entry.shape = Shape::Vec(vec![background, marker_shape(rect)]);
                    });
                });
            } else {
                painter.set(slot, marker_shape(rect));
            }
            (slot, keep_background)
        }
    };
    scene.receivers.push(Receiver {
        marker: Marker {
            id,
            layer,
            slot,
            keep_background,
        },
        corners,
        original_width: rect.width(),
        style: style.bounded(),
        surface,
        fallback,
        opacity: painter.opacity(),
    });
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn spill_resolves_current_sources_behind_earlier_receivers() {
        let ctx = Context::default();
        install(&ctx);
        let style = Vesper {
            spill: Some(LightSpill::default()),
            ..Vesper::default()
        };
        let receiver = Rect::from_min_size(egui::pos2(110.0, 20.0), egui::vec2(70.0, 40.0));
        let emitter = Rect::from_min_size(egui::pos2(20.0, 20.0), egui::vec2(70.0, 40.0));
        let point = emitter.center();
        let render = |source: Option<(Rect, bool)>, opted_in: bool| {
            let mut output = ctx.run_ui(Default::default(), |ui| {
                let painter = ui.painter();
                let background = painter.rect_filled(receiver, 12, Color32::DARK_GRAY);
                let mut recipient = style;
                if !opted_in {
                    recipient.spill = None;
                }
                receive_background_at(
                    painter,
                    Id::new("receiver"),
                    receiver,
                    12.0.into(),
                    &recipient,
                    Color32::DARK_GRAY,
                    Color32::WHITE,
                    background,
                );
                if let Some((clip, separate_layer)) = source {
                    let painter = painter.with_clip_rect(clip);
                    let painter = if separate_layer {
                        painter
                            .with_layer_id(LayerId::new(egui::Order::Foreground, Id::new("other")))
                    } else {
                        painter
                    };
                    emit(
                        &painter,
                        Id::new("emitter"),
                        emitter,
                        &style,
                        Sample {
                            point,
                            energy: 1.0,
                            pulse_point: point,
                            pulse: 1.0,
                            velocity: Vec2::ZERO,
                            flow: Vec2::ZERO,
                        },
                    );
                }
            });
            output.textures_delta.clear();
            output.shapes.iter().any(|entry| {
                let Shape::Vec(shapes) = &entry.shape else {
                    return false;
                };
                matches!(shapes.as_slice(), [Shape::Rect(frame), Shape::Mesh(mesh)]
                    if frame.fill == Color32::DARK_GRAY
                        && mesh.is_valid()
                        && mesh.vertices.iter().all(|vertex| receiver.contains(vertex.pos)))
            })
        };
        assert!(render(Some((Rect::EVERYTHING, false)), true));
        assert!(!render(None, true));
        assert!(!render(Some((receiver, false)), true));
        assert!(!render(Some((Rect::EVERYTHING, true)), true));
        assert!(!render(Some((Rect::EVERYTHING, false)), false));
    }
}
