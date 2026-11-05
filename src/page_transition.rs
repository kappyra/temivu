//! Source-linked page transitions for egui content regions.
use crate::{Theme, motion};
use egui::epaint::ClippedShape;
use egui::{Context, Id, LayerId, Rect, Response, Ui, emath::TSTransform, layers::ShapeIdx};

/// Spring-driven navigation for a single page region. The caller owns routes
/// and calls one of the transition methods after painting the outgoing page.
/// Only the active route is laid out. The previous route is replayed as paint.
pub struct PageTransition {
    id: Id,
    last: Option<PaintRange>,
    original: Option<Vec<ClippedShape>>,
    composite: Option<PaintRange>,
    flight: Option<Flight>,
}

#[derive(Clone, Copy)]
struct PaintRange {
    layer: LayerId,
    start: ShapeIdx,
    end: ShapeIdx,
    pass: u64,
}

#[derive(Clone, Copy)]
enum Travel {
    Zoom {
        anchor: Id,
        source: Rect,
        opening: bool,
    },
    Slide {
        forward: bool,
    },
}

struct Flight {
    travel: Travel,
    outgoing: Vec<ClippedShape>,
}

impl PageTransition {
    pub fn new(id: Id) -> Self {
        Self {
            id,
            last: None,
            original: None,
            composite: None,
            flight: None,
        }
    }

    /// Grow the next page from the triggering widget.
    pub fn open_from(&mut self, trigger: &Response, theme: &Theme) {
        let ctx = &trigger.ctx;
        let source = motion::anchor_rect(trigger);
        if !source.is_finite() || !source.is_positive() {
            return;
        }
        let reversal = matches!(
            self.flight.as_ref().map(|flight| flight.travel),
            Some(Travel::Zoom { anchor, opening: false, .. }) if anchor == trigger.id
        );
        let outgoing = if matches!(
            self.flight.as_ref().map(|flight| flight.travel),
            Some(Travel::Slide { .. })
        ) {
            self.copy_current(ctx, self.composite)
        } else {
            self.take_page(ctx)
        };
        if !reversal {
            motion::reset(ctx, self.id);
        }
        self.flight = Some(Flight {
            travel: Travel::Zoom {
                anchor: trigger.id,
                source,
                opening: true,
            },
            outgoing,
        });
        self.progress(ctx, true, theme);
    }

    /// Return to the source of the last zoom. The caller also changes its
    /// route to the previous page. Returns false if there was no zoom.
    pub fn close(&mut self, ctx: &Context, theme: &Theme) -> bool {
        let Some(Travel::Zoom {
            anchor,
            source,
            opening: true,
        }) = self.flight.as_ref().map(|flight| flight.travel)
        else {
            return false;
        };
        let outgoing = self.take_page(ctx);
        self.flight = Some(Flight {
            travel: Travel::Zoom {
                anchor,
                source,
                opening: false,
            },
            outgoing,
        });
        self.progress(ctx, false, theme);
        ctx.memory_mut(|memory| memory.request_focus(anchor));
        true
    }

    /// Move between sibling pages. `forward` enters from the right.
    pub fn slide(&mut self, ctx: &Context, forward: bool, theme: &Theme) {
        let outgoing = self.copy_current(ctx, self.composite);
        motion::reset(ctx, self.id);
        self.flight = Some(Flight {
            travel: Travel::Slide { forward },
            outgoing,
        });
        self.progress(ctx, true, theme);
    }

    /// Draw the live page in this region. Call once per frame at a stable
    /// location, after the app's persistent header and before its navigation.
    pub fn show<R>(&mut self, ui: &mut Ui, theme: &Theme, content: impl FnOnce(&mut Ui) -> R) -> R {
        let ctx = ui.ctx().clone();
        let layer = ui.layer_id();
        let region = ui.available_rect_before_wrap().intersect(ui.clip_rect());
        let viewport = ctx.content_rect();
        let painter = ctx.layer_painter(layer);
        let composite_start = paint_cursor(&ctx, layer);
        let mut flight = self.flight.take();
        let mut incoming = None;
        let mut closing = None;
        if let Some(active) = &mut flight {
            match active.travel {
                Travel::Zoom {
                    anchor,
                    source,
                    opening,
                } => {
                    let phase = self.progress(&ctx, opening, theme);
                    if opening && phase >= 0.999 {
                        active.outgoing = Vec::new();
                    } else if !opening && phase <= 0.001 {
                        flight = None;
                    } else {
                        let source = ctx
                            .read_response(anchor)
                            .map_or(source, |response| motion::anchor_rect(&response));
                        let source = source.intersect(viewport);
                        if source.is_positive() && region.is_positive() {
                            let pose = zoom_pose(source, region, phase);
                            if opening {
                                replay(
                                    &painter,
                                    &active.outgoing,
                                    TSTransform::IDENTITY,
                                    region,
                                    1.0,
                                );
                                zoom_surface(&painter, region, pose, theme);
                                incoming = Some(pose);
                            } else {
                                closing = Some(pose);
                            }
                        }
                    }
                }
                Travel::Slide { forward } => {
                    let phase = self.progress(&ctx, true, theme);
                    if phase >= 0.999 || !region.is_positive() {
                        flight = None;
                    } else {
                        let travel = if forward { 1.0 } else { -1.0 } * region.width();
                        let old =
                            TSTransform::from_translation(egui::vec2(-travel * phase * 0.14, 0.0));
                        replay(&painter, &active.outgoing, old, region, 1.0);
                        let transform =
                            TSTransform::from_translation(egui::vec2(travel * (1.0 - phase), 0.0));
                        let bounds = transform.mul_rect(region);
                        painter
                            .with_clip_rect(region)
                            .rect_filled(bounds, 0, theme.background);
                        incoming = Some(Pose {
                            bounds: bounds.intersect(region),
                            transform,
                            radius: 0.0,
                            alpha: 1.0,
                            phase,
                        });
                    }
                }
            }
        }

        let start = paint_cursor(&ctx, layer);
        let value = content(ui);
        let end = paint_cursor(&ctx, layer);
        let range = PaintRange {
            layer,
            start,
            end,
            pass: ctx.cumulative_pass_nr(),
        };
        self.last = Some(range);
        self.original = incoming.map(|_| copy_range(&ctx, range));
        if let Some(pose) = incoming {
            ctx.graphics_mut(|graphics| {
                let list = graphics.entry(layer);
                for index in start.0..end.0 {
                    list.mutate_shape(ShapeIdx(index), |entry| {
                        transform_shape(entry, pose.transform, pose.content_clip(), pose.alpha)
                    });
                }
            });
        }
        if let (Some(pose), Some(active)) = (closing, &flight) {
            zoom_surface(&painter, region, pose, theme);
            replay(
                &painter,
                &active.outgoing,
                pose.transform,
                pose.content_clip(),
                pose.alpha,
            );
        }
        self.composite = Some(PaintRange {
            start: composite_start,
            end: paint_cursor(&ctx, layer),
            ..range
        });
        self.flight = flight;
        value
    }

    fn take_page(&mut self, ctx: &Context) -> Vec<ClippedShape> {
        self.original
            .take()
            .unwrap_or_else(|| self.copy_current(ctx, self.last))
    }

    fn copy_current(&self, ctx: &Context, range: Option<PaintRange>) -> Vec<ClippedShape> {
        range
            .filter(|range| range.pass == ctx.cumulative_pass_nr())
            .map_or_else(Vec::new, |range| copy_range(ctx, range))
    }

    fn progress(&self, ctx: &Context, opening: bool, theme: &Theme) -> f32 {
        let mut theme = theme.clone();
        theme.motion.normal *= if opening { 1.9 } else { 1.5 };
        motion::spring_f32_with_damping(
            ctx,
            self.id,
            0.0,
            if opening { 1.0 } else { 0.0 },
            0.86,
            &theme,
        )
        .clamp(0.0, 1.0)
    }
}

#[derive(Clone, Copy)]
struct Pose {
    bounds: Rect,
    transform: TSTransform,
    radius: f32,
    alpha: f32,
    phase: f32,
}

impl Pose {
    fn content_clip(self) -> Rect {
        self.bounds.shrink(self.radius * 0.293)
    }
}

fn zoom_pose(source: Rect, target: Rect, phase: f32) -> Pose {
    let bounds = crate::liquid::surface_rect(source, target, phase);
    let radius = (source.height().min(source.width()) * 0.5).min(30.0) * (1.0 - phase);
    let inside = bounds.shrink(radius * 0.293);
    let scale = (inside.size() / target.size()).min_elem().clamp(0.01, 1.0);
    let alpha = ((phase - 0.26) / 0.52).clamp(0.0, 1.0);
    Pose {
        bounds,
        transform: TSTransform {
            scaling: scale,
            translation: inside.center().to_vec2() - target.center().to_vec2() * scale,
        },
        radius,
        alpha: alpha * alpha * (3.0 - 2.0 * alpha),
        phase,
    }
}

fn zoom_surface(painter: &egui::Painter, region: Rect, pose: Pose, theme: &Theme) {
    let shade = (40.0 * pose.phase * (1.0 - pose.phase) * 4.0) as u8;
    painter.rect_filled(region, 0, egui::Color32::from_black_alpha(shade));
    let shadow = egui::epaint::Shadow {
        offset: [0, 4],
        blur: 24,
        spread: 0,
        color: egui::Color32::from_black_alpha(shade),
    };
    painter.add(shadow.as_shape(pose.bounds, pose.radius.round() as u8));
    painter.add(motion::rounded_shape(
        pose.bounds,
        pose.radius.into(),
        theme
            .background
            .gamma_multiply((pose.phase / 0.08).clamp(0.0, 1.0)),
        egui::Stroke::NONE,
        egui::StrokeKind::Inside,
    ));
}

fn paint_cursor(ctx: &Context, layer: LayerId) -> ShapeIdx {
    ctx.graphics(|graphics| {
        graphics
            .get(layer)
            .map_or(ShapeIdx(0), |list| list.next_idx())
    })
}

fn copy_range(ctx: &Context, range: PaintRange) -> Vec<ClippedShape> {
    ctx.graphics(|graphics| {
        graphics
            .get(range.layer)
            .into_iter()
            .flat_map(|list| list.all_entries())
            .skip(range.start.0)
            .take(range.end.0 - range.start.0)
            .cloned()
            .collect()
    })
}

fn transform_shape(entry: &mut ClippedShape, transform: TSTransform, clip: Rect, alpha: f32) {
    entry.shape.transform(transform);
    entry.clip_rect = transform.mul_rect(entry.clip_rect).intersect(clip);
    if alpha < 1.0 {
        egui::epaint::shape_transform::adjust_colors(&mut entry.shape, move |color| {
            if *color != egui::Color32::PLACEHOLDER {
                *color = color.gamma_multiply(alpha);
            }
        });
    }
}

fn replay(
    painter: &egui::Painter,
    entries: &[ClippedShape],
    transform: TSTransform,
    clip: Rect,
    alpha: f32,
) {
    for entry in entries {
        let mut entry = entry.clone();
        transform_shape(&mut entry, transform, clip, alpha);
        painter.with_clip_rect(entry.clip_rect).add(entry.shape);
    }
}
