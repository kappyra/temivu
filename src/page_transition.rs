//! Source-linked page transitions for egui content regions.
use crate::{
    Theme,
    liquid::{self, LiquidStyle},
    motion,
};
use egui::epaint::ClippedShape;
use egui::{Context, Id, LayerId, Rect, Response, Ui, emath::TSTransform, layers::ShapeIdx};

/// Spring-driven navigation for a single page region. The caller owns routes
/// and calls one of the transition methods after painting the outgoing page.
/// Only the active route is laid out. The previous route is replayed as paint.
pub struct PageTransition {
    id: Id,
    last: Option<PaintRange>,
    original: Option<Vec<CachedPaint>>,
    composite: Option<PaintRange>,
    flight: Option<Flight>,
}

struct CachedPaint {
    entry: ClippedShape,
    bounds: Rect,
}

#[derive(Clone, Copy)]
struct PaintRange {
    layer: LayerId,
    start: ShapeIdx,
    end: ShapeIdx,
    pass: u64,
    region: Rect,
}

#[derive(Clone, Copy)]
enum Travel {
    Anchored {
        anchor: Id,
        source: Rect,
        material: Option<motion::AnchorMaterial>,
        opening: bool,
    },
    Slide {
        forward: bool,
    },
}

struct Flight {
    travel: Travel,
    outgoing: Vec<CachedPaint>,
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
            Some(Travel::Anchored { anchor, opening: false, .. }) if anchor == trigger.id
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
            travel: Travel::Anchored {
                anchor: trigger.id,
                source,
                material: motion::anchor_material(trigger),
                opening: true,
            },
            outgoing,
        });
        self.progress(ctx, true, theme);
    }

    /// Return to the source of the last anchored transition. The caller also changes its
    /// route to the previous page. Returns false if there was no anchored transition.
    pub fn close(&mut self, ctx: &Context, theme: &Theme) -> bool {
        let Some(Travel::Anchored {
            anchor,
            source,
            material,
            opening: true,
        }) = self.flight.as_ref().map(|flight| flight.travel)
        else {
            return false;
        };
        let outgoing = self.take_page(ctx);
        let source = ctx
            .read_response(anchor)
            .map_or(source, |response| motion::anchor_rect(&response));
        self.flight = Some(Flight {
            travel: Travel::Anchored {
                anchor,
                source,
                material,
                opening: false,
            },
            outgoing,
        });
        self.progress(ctx, false, theme);
        ctx.memory_mut(|memory| memory.request_focus(anchor));
        true
    }

    /// Expand a sibling page from the indicated edge. `forward` starts at the right.
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
                Travel::Anchored {
                    source,
                    material,
                    opening,
                    ..
                } => {
                    let phase = self.progress(&ctx, opening, theme);
                    if opening && phase >= 0.999 {
                        active.outgoing = Vec::new();
                    } else if !opening && phase <= 0.001 {
                        flight = None;
                    } else {
                        let source = source.intersect(viewport);
                        if source.is_positive() && region.is_positive() {
                            let pose = droplet_pose(source, region, phase);
                            if opening {
                                replay(
                                    &painter,
                                    &active.outgoing,
                                    TSTransform::IDENTITY,
                                    region,
                                    1.0,
                                );
                                page_surface(&painter, source, region, pose, theme, material);
                                incoming = Some(pose);
                            } else {
                                closing = Some((source, pose, material));
                            }
                        }
                    }
                }
                Travel::Slide { forward } => {
                    let phase = self.progress(&ctx, true, theme);
                    if phase >= 0.999 || !region.is_positive() {
                        flight = None;
                    } else {
                        let edge = if forward {
                            region.right()
                        } else {
                            region.left()
                        };
                        let seed = egui::vec2(52.0, 64.0).min(region.size());
                        let x = if forward {
                            edge - seed.x * 0.5
                        } else {
                            edge + seed.x * 0.5
                        };
                        let source = Rect::from_center_size(egui::pos2(x, region.center().y), seed);
                        replay(
                            &painter,
                            &active.outgoing,
                            TSTransform::IDENTITY,
                            region,
                            1.0,
                        );
                        let pose = droplet_pose(source, region, phase);
                        page_surface(&painter, source, region, pose, theme, None);
                        incoming = Some(pose);
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
            region,
        };
        self.last = Some(range);
        if incoming.is_some() {
            let snapshot = self.original.get_or_insert_with(Vec::new);
            snapshot.clear();
            copy_range(&ctx, range, snapshot);
        } else {
            self.original = None;
        }
        if let Some(pose) = incoming {
            ctx.graphics_mut(|graphics| {
                let list = graphics.entry(layer);
                for index in start.0..end.0 {
                    list.mutate_shape(ShapeIdx(index), |entry| {
                        transform_shape(entry, pose.transform, pose.clip, pose.alpha)
                    });
                }
            });
        }
        if let (Some((source, pose, material)), Some(active)) = (closing, &flight) {
            page_surface(&painter, source, region, pose, theme, material);
            replay(
                &painter,
                &active.outgoing,
                pose.transform,
                pose.clip,
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

    fn take_page(&mut self, ctx: &Context) -> Vec<CachedPaint> {
        self.original
            .take()
            .unwrap_or_else(|| self.copy_current(ctx, self.last))
    }

    fn copy_current(&self, ctx: &Context, range: Option<PaintRange>) -> Vec<CachedPaint> {
        let mut snapshot = Vec::new();
        if let Some(range) = range.filter(|range| range.pass == ctx.cumulative_pass_nr()) {
            copy_range(ctx, range, &mut snapshot);
        }
        snapshot
    }

    fn progress(&self, ctx: &Context, opening: bool, theme: &Theme) -> f32 {
        liquid::surface_progress(ctx, self.id, opening, LiquidStyle::droplet(), theme).min(1.0)
    }
}

#[derive(Clone, Copy)]
struct Pose {
    bounds: Rect,
    clip: Rect,
    transform: TSTransform,
    radius: f32,
    alpha: f32,
    phase: f32,
}

fn droplet_pose(source: Rect, target: Rect, phase: f32) -> Pose {
    let bounds = liquid::matched_rect(source, target, phase);
    let radius = (source.size().min_elem() * 0.5).min(30.0) * (1.0 - phase);
    let toward_source = (source.center() - target.center()).normalized();
    let transform = TSTransform::from_translation(toward_source * (12.0 * (1.0 - phase)));
    Pose {
        bounds,
        clip: bounds.shrink(radius * 0.293).intersect(target),
        transform,
        radius,
        alpha: crate::Easing::EaseInOut.apply((phase - 0.08) / 0.68),
        phase,
    }
}

fn page_surface(
    painter: &egui::Painter,
    source: Rect,
    region: Rect,
    pose: Pose,
    theme: &Theme,
    material: Option<motion::AnchorMaterial>,
) {
    let fill = material.map_or(theme.background, |material| {
        material.fill.lerp_to_gamma(
            theme.background,
            crate::Easing::EaseInOut.apply(pose.phase / 0.38),
        )
    });
    let opacity = crate::Easing::EaseInOut.apply(pose.phase / 0.52);
    painter
        .with_clip_rect(region.union(source))
        .add(motion::rounded_shape(
            pose.bounds,
            pose.radius.into(),
            fill.gamma_multiply(opacity),
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

fn copy_range(ctx: &Context, range: PaintRange, snapshot: &mut Vec<CachedPaint>) {
    ctx.graphics(|graphics| {
        let Some(list) = graphics.get(range.layer) else {
            return;
        };
        snapshot.extend(
            list.all_entries()
                .skip(range.start.0)
                .take(range.end.0 - range.start.0)
                .filter_map(|entry| {
                    let bounds = entry
                        .shape
                        .visual_bounding_rect()
                        .intersect(entry.clip_rect);
                    bounds.intersects(range.region).then(|| CachedPaint {
                        entry: entry.clone(),
                        bounds,
                    })
                }),
        );
    });
}

fn transform_shape(entry: &mut ClippedShape, transform: TSTransform, clip: Rect, alpha: f32) {
    entry.clip_rect = transform.mul_rect(entry.clip_rect).intersect(clip);
    if alpha <= 0.0
        || !entry.clip_rect.is_positive()
        || !transform
            .mul_rect(entry.shape.visual_bounding_rect())
            .intersects(entry.clip_rect)
    {
        entry.shape = egui::Shape::Noop;
        return;
    }
    pose_paint(&mut entry.shape, transform, alpha);
}

fn pose_paint(shape: &mut egui::Shape, transform: TSTransform, alpha: f32) {
    match shape {
        egui::Shape::Text(text) => {
            // Page content only translates. Keep the original laid-out glyphs.
            text.pos = transform * text.pos;
            text.opacity_factor *= alpha;
        }
        egui::Shape::Vec(shapes) => {
            for shape in shapes {
                pose_paint(shape, transform, alpha);
            }
        }
        _ => {
            if transform != TSTransform::IDENTITY {
                shape.transform(transform);
            }
            if alpha < 1.0 {
                liquid::fade_shape(shape, alpha);
            }
        }
    }
}

fn replay(
    painter: &egui::Painter,
    entries: &[CachedPaint],
    transform: TSTransform,
    clip: Rect,
    alpha: f32,
) {
    if alpha <= 0.0 || !clip.is_positive() {
        return;
    }
    for cached in entries {
        if !transform.mul_rect(cached.bounds).intersects(clip) {
            continue;
        }
        let mut entry = cached.entry.clone();
        entry.clip_rect = transform.mul_rect(entry.clip_rect).intersect(clip);
        pose_paint(&mut entry.shape, transform, alpha);
        painter.with_clip_rect(entry.clip_rect).add(entry.shape);
    }
}
