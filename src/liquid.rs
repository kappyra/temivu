//! Source-connected opening/closing motion for ordinary themed surfaces.
//! Default popups expand as one bubble; Droplet merging is an explicit alternative.
//! Material, input and layout remain owned by the host and egui.
use crate::{Theme, motion::AnchorMaterial};
use egui::{
    Color32, Frame, Id, InnerResponse, LayerId, Order, Pos2, Rect, Response, Shape, Ui, Vec2,
};
use std::sync::Arc;

/// How a floating surface connects to its trigger.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum SurfaceTransition {
    /// Soft, whole-trigger expansion; the original Temivu menu animation.
    #[default]
    Bloom,
    /// Elastic separation and merging through a retracting droplet neck.
    Droplet,
}

impl SurfaceTransition {
    /// Compatibility name for the original expansion mode.
    #[allow(non_upper_case_globals)]
    pub const Expand: Self = Self::Bloom;
    /// Compatibility name for the original merging experiment.
    #[allow(non_upper_case_globals)]
    pub const Merge: Self = Self::Droplet;
}

/// Geometry shared by popovers, dropdowns and source-to-window transitions.
#[derive(Clone, Copy, Debug)]
pub struct LiquidStyle {
    /// Enable the morph; false snaps while preserving the popup's behavior.
    pub enabled: bool,
    pub transition: SurfaceTransition,
    /// Space between trigger and settled surface in logical points (0..=96).
    pub gap: f32,
    /// Smooth-union connection radius in logical points (0..=96).
    pub blend_radius: f32,
    /// Nominal spring duration at the default theme speed, in seconds.
    /// Scales with Theme::motion.normal; clamped to 0.08..=2 seconds.
    pub duration: f32,
    /// Damping ratio. Higher values reduce bounce (0.1..=0.99).
    pub damping: f32,
}
impl Default for LiquidStyle {
    fn default() -> Self {
        Self::bloom()
    }
}
impl LiquidStyle {
    /// The original, softly expanding menu preset.
    pub const fn bloom() -> Self {
        Self {
            enabled: true,
            transition: SurfaceTransition::Bloom,
            gap: 12.0,
            blend_radius: 44.0,
            duration: 0.32,
            damping: 0.82,
        }
    }
    /// A more elastic source-connected separation/merging preset.
    pub const fn droplet() -> Self {
        Self {
            transition: SurfaceTransition::Droplet,
            gap: 8.0,
            blend_radius: 34.0,
            duration: 0.34,
            damping: 0.77,
            ..Self::bloom()
        }
    }
    /// Set the default for this egui context; widget overrides take precedence.
    pub fn set_context_default(self, ctx: &egui::Context) {
        ctx.data_mut(|data| data.insert_temp(Id::new("temivu-surface-motion-default"), self));
    }
    /// Read the context default, falling back to Bloom.
    pub fn context_default(ctx: &egui::Context) -> Self {
        ctx.data(|data| data.get_temp(Id::new("temivu-surface-motion-default")))
            .unwrap_or_default()
    }
}

fn surface_progress(
    ctx: &egui::Context,
    id: Id,
    open: bool,
    motion: LiquidStyle,
    theme: &Theme,
) -> f32 {
    let mut surface_theme = theme.clone();
    surface_theme.motion.enabled &= motion.enabled;
    let duration = if motion.duration.is_finite() {
        motion.duration.clamp(0.08, 2.0)
    } else {
        0.32
    };
    let theme_scale = if theme.motion.normal.is_finite() {
        theme.motion.normal / 0.2
    } else {
        1.0
    };
    let exit_speed = if !open && motion.transition == SurfaceTransition::Droplet {
        0.82
    } else {
        1.0
    };
    surface_theme.motion.normal = (duration * theme_scale * exit_speed).clamp(0.08, 2.0);
    crate::motion::spring_f32_with_damping(
        ctx,
        id,
        0.0,
        if open { 1.0 } else { 0.0 },
        motion.damping,
        &surface_theme,
    )
    .clamp(
        0.0,
        if motion.transition == SurfaceTransition::Droplet {
            1.025
        } else {
            1.0
        },
    )
}

/// Signed distance to a rounded rectangle; negative values are inside.
pub fn rounded_rect_distance(point: Pos2, rect: Rect, radius: f32) -> f32 {
    let radius = radius.max(0.0).min(rect.width().min(rect.height()) * 0.5);
    let q = (point - rect.center()).abs() - rect.size() * 0.5 + Vec2::splat(radius);
    q.max(Vec2::ZERO).length() + q.x.max(q.y).min(0.0) - radius
}

/// Polynomial smooth minimum. Zero smoothing produces the ordinary union.
pub fn smooth_union(a: f32, b: f32, smoothing: f32) -> f32 {
    if smoothing <= 0.0 || !smoothing.is_finite() {
        return a.min(b);
    }
    let h = (1.0 - (a - b).abs() / smoothing).max(0.0);
    a.min(b) - h * h * smoothing * 0.25
}

/// Interpolate a surface from its source to its final outer rectangle.
pub fn surface_rect(source: Rect, target: Rect, amount: f32) -> Rect {
    let amount = amount.clamp(0.0, 1.06);
    Rect::from_center_size(
        source.center().lerp(target.center(), amount),
        source.size() * (1.0 - amount) + target.size() * amount,
    )
}

fn smoothstep(value: f32) -> f32 {
    let value = value.clamp(0.0, 1.0);
    value * value * (3.0 - 2.0 * value)
}

fn droplet_seed(source: Rect, target: Rect) -> f32 {
    (source
        .height()
        .min(source.width())
        .min(target.width().min(target.height()))
        * 0.72)
        .max(1.0)
}

fn contact_x(source: Rect, target: Rect, seed: f32) -> f32 {
    let reach = ((source.width() - seed) * 0.36).max(0.0);
    source.center().x + ((target.center().x - source.center().x) * 0.36).clamp(-reach, reach)
}

fn elongation(target: Rect) -> f32 {
    smoothstep((target.height() / target.width().max(1.0) - 0.75) / 1.35)
}

/// Moving body of Droplet. It starts inside the trigger and stretches along
/// its path before reaching the final width.
pub fn droplet_rect(source: Rect, target: Rect, amount: f32) -> Rect {
    let phase = amount.clamp(0.0, 1.0);
    let seed = droplet_seed(source, target);
    let stretch = elongation(target);
    let width_start = egui::lerp(0.12..=0.24, stretch);
    let height_finish = egui::lerp(0.95..=0.82, stretch);
    let size = egui::vec2(
        egui::lerp(
            seed..=target.width(),
            smoothstep((phase - width_start) / (1.0 - width_start)),
        ),
        egui::lerp(seed..=target.height(), smoothstep(phase / height_finish)),
    );
    let travel = smoothstep(phase);
    let below = target.center().y >= source.center().y;
    let near_edge = if below {
        egui::lerp((source.bottom() - seed)..=target.top(), travel)
    } else {
        egui::lerp((source.top() + seed)..=target.bottom(), travel)
    };
    let center = egui::pos2(
        egui::lerp(contact_x(source, target, seed)..=target.center().x, travel),
        near_edge + if below { size.y * 0.5 } else { -size.y * 0.5 },
    );
    let overshoot = (amount - 1.0).clamp(0.0, 0.025);
    Rect::from_center_size(center, size * (1.0 + overshoot * 0.4))
}

fn motion_rect(source: Rect, target: Rect, amount: f32, transition: SurfaceTransition) -> Rect {
    match transition {
        SurfaceTransition::Bloom => surface_rect(source, target, amount),
        SurfaceTransition::Droplet => droplet_rect(source, target, amount),
    }
}

#[derive(Clone)]
struct Snapshot {
    rect: Rect,
    shapes: Arc<Vec<egui::epaint::ClippedShape>>,
    backdrop: Option<egui::epaint::ClippedShape>,
    frame: Frame,
}

fn opacity(amount: f32) -> f32 {
    let value = ((amount - 0.1) / 0.75).clamp(0.0, 1.0);
    value * value * (3.0 - 2.0 * value)
}

fn content_opacity(amount: f32, transition: SurfaceTransition) -> f32 {
    match transition {
        SurfaceTransition::Bloom => opacity(amount),
        SurfaceTransition::Droplet => smoothstep((amount - 0.48) / 0.42),
    }
}

// Menu labels retain their normal proportions rather than growing from tiny
// glyphs. The expanding surface reveals them with a restrained 4% scale/fade.
fn content_transform(
    source: Rect,
    target: Rect,
    amount: f32,
    transition: SurfaceTransition,
) -> egui::emath::TSTransform {
    let pivot = egui::pos2(
        source.center().x.clamp(target.left(), target.right()),
        if target.center().y >= source.center().y {
            target.top()
        } else {
            target.bottom()
        },
    );
    let lift = if transition == SurfaceTransition::Droplet {
        {
            let travel = source.center() - target.center();
            travel / travel.length().max(1.0) * (3.0 * (1.0 - amount.clamp(0.0, 1.0)))
        }
    } else {
        Vec2::ZERO
    };
    egui::emath::TSTransform::from_translation(pivot.to_vec2() + lift)
        * egui::emath::TSTransform::from_scaling(0.96 + 0.04 * amount.clamp(0.0, 1.0))
        * egui::emath::TSTransform::from_translation(-pivot.to_vec2())
}

pub(crate) struct PopupSurface {
    pub match_compact_trigger: bool,
    pub width: Option<f32>,
    pub frame: Frame,
    pub motion: LiquidStyle,
    pub close_behavior: egui::PopupCloseBehavior,
}

/// An anchored surface with a logical open state and a noninteractive exit tail.
/// Closed exits replay paint only: they cannot accept stale clicks or steal focus.
pub(crate) fn popup<R>(
    anchor: &Response,
    id: Id,
    open: &mut bool,
    surface: PopupSurface,
    theme: &Theme,
    content: impl FnOnce(&mut Ui) -> R,
) -> Option<InnerResponse<R>> {
    let PopupSurface {
        match_compact_trigger,
        width,
        frame,
        motion,
        close_behavior,
    } = surface;
    let ctx = &anchor.ctx;
    let key = id.with("liquid-snapshot");
    // Layout bounds stay fixed during the trigger's own press spring.
    let source = ctx
        .layer_transform_to_global(anchor.layer_id)
        .map_or(anchor.rect, |transform| transform * anchor.rect);
    let amount = surface_progress(ctx, id.with("liquid-progress"), *open, motion, theme);
    let painted_source = if motion.transition == SurfaceTransition::Droplet && amount < 0.96 {
        crate::motion::anchor_rect(anchor)
    } else {
        source
    };
    let matched = match_compact_trigger
        && motion.transition == SurfaceTransition::Droplet
        && (0.0..=64.0).contains(&source.height())
        && source.width() > 0.0
        && source.width() <= 1.55 * source.height();
    let source_material =
        (motion.transition == SurfaceTransition::Droplet && amount > 0.001 && amount < 0.96)
            .then(|| crate::motion::anchor_material(anchor))
            .flatten();
    let surface_shape = |target| {
        if matched {
            matched_menu_shape(source, target, amount, frame, source_material)
        } else {
            surface_shape(
                source,
                painted_source,
                target,
                amount,
                frame,
                motion,
                source_material,
            )
        }
    };
    let content_pose = |target| {
        if motion.transition == SurfaceTransition::Droplet {
            let phase = amount.min(1.0);
            let (body, radii, min_scale, reveal) = if matched {
                let body = matched_menu_rect(source, target, amount);
                let radii = matched_menu_radii(body, amount, frame.corner_radius);
                (body, radii, 0.80, smoothstep((phase - 0.40) / 0.46))
            } else {
                let body = droplet_rect(source, target, amount);
                let radii = droplet_radii(source, target, body, phase, frame.corner_radius);
                (
                    body,
                    radii,
                    0.96,
                    content_opacity(amount, motion.transition),
                )
            };
            let max_radius = radii.nw.max(radii.ne).max(radii.sw).max(radii.se);
            // The diagonal inset lies inside even a fully rounded corner.
            // Fit the rows as a group so their backgrounds cannot make square
            // corners or clip the leading letters during the narrow phase.
            let inset = max_radius.min(body.size().min_elem() * 0.5) * 0.293;
            let inside = body.shrink(inset);
            let inner_target = target.shrink(frame.stroke.width) - frame.inner_margin;
            let fit = inside.size() / inner_target.size().max(Vec2::splat(1.0));
            let scale = egui::lerp(min_scale..=1.0, smoothstep(phase)).min(fit.min_elem());
            let transform = egui::emath::TSTransform {
                scaling: scale,
                translation: inside.center().to_vec2() - inner_target.center().to_vec2() * scale,
            };
            let alpha = reveal * smoothstep((scale - 0.54) / 0.27);
            (transform, inside, alpha)
        } else {
            (
                content_transform(source, target, amount, motion.transition),
                motion_rect(source, target, amount, motion.transition),
                content_opacity(amount, motion.transition),
            )
        }
    };
    let layer = LayerId::new(Order::Foreground, id);
    let clip = ctx.content_rect();
    if !*open {
        if amount > 0.001
            && let Some(snapshot) = ctx.data(|data| data.get_temp::<Snapshot>(key))
        {
            let painter = ctx.layer_painter(layer);
            painter.add(surface_shape(snapshot.rect));
            let (transform, body, alpha) = content_pose(snapshot.rect);
            let content_clip = body.intersect(clip);
            for entry in snapshot.shapes.iter() {
                let mut entry = entry.clone();
                entry.shape.transform(transform);
                fade_shape(&mut entry.shape, alpha);
                painter
                    .with_clip_rect(transform.mul_rect(entry.clip_rect).intersect(content_clip))
                    .add(entry.shape);
            }
        } else {
            ctx.data_mut(|data| data.remove::<Snapshot>(key));
        }
        return None;
    }
    let painter = ctx.layer_painter(layer);
    let background = painter.add(Shape::Noop);
    let start = ctx.graphics(|graphics| graphics.get(layer).unwrap().next_idx());
    let mut popup = egui::Popup::from_response(anchor)
        .id(id)
        .gap(if motion.gap.is_finite() {
            motion.gap.clamp(0.0, 96.0)
        } else {
            12.0
        })
        .open_bool(open)
        .close_behavior(close_behavior)
        .frame(Frame::NONE.inner_margin(frame.inner_margin));
    if matched {
        let align = if source.center().x > clip.center().x {
            egui::RectAlign::BOTTOM_END
        } else {
            egui::RectAlign::BOTTOM_START
        };
        // A small overlap at the outer edges lets the rounded menu cover
        // the icon button even if the two surfaces have different radii.
        let overlap = 6.0;
        popup = popup
            .anchor(source.expand(overlap))
            .gap(-source.height() - 2.0 * overlap)
            .align(align);
    }
    if let Some(width) = width {
        popup = popup.width(width.min((clip.width() - 24.0).max(1.0)));
    }
    let mut sizing = false;
    let result = popup.show(|ui| {
        sizing = ui.is_sizing_pass();
        if sizing {
            // Area's first/reopened pass contains invisible placeholder shapes.
            // Resolve layout in this frame instead of displaying an empty morph.
            ui.ctx().request_discard("liquid surface sizing");
        }
        content(ui)
    })?;
    if sizing {
        return Some(result);
    }
    let target = result.response.rect;
    let end = ctx.graphics(|graphics| graphics.get(layer).unwrap().next_idx());
    let snapshot = ctx.graphics(|graphics| {
        graphics
            .get(layer)
            .unwrap()
            .all_entries()
            .skip(start.0)
            .take(end.0 - start.0)
            .cloned()
            .collect::<Vec<_>>()
    });
    ctx.data_mut(|data| {
        data.insert_temp(
            key,
            Snapshot {
                rect: target,
                shapes: Arc::new(snapshot),
                backdrop: None,
                frame,
            },
        )
    });
    let (transform, body, alpha) = content_pose(target);
    let content_clip = body.intersect(clip);
    let shape = surface_shape(target);
    ctx.graphics_mut(|graphics| {
        let list = graphics.entry(layer);
        list.set(background, clip, shape);
        if (amount - 1.0).abs() > 0.001 {
            list.transform_range(start, end, transform);
            for index in start.0..end.0 {
                list.mutate_shape(egui::layers::ShapeIdx(index), |entry| {
                    entry.clip_rect = entry.clip_rect.intersect(content_clip);
                    fade_shape(&mut entry.shape, alpha);
                });
            }
        }
    });
    Some(result)
}

fn matched_menu_rect(source: Rect, target: Rect, amount: f32) -> Rect {
    let phase = amount.clamp(0.0, 1.0);
    let stretch = elongation(target);
    let overshoot = amount - phase;
    let x = egui::lerp(phase..=phase * phase, stretch) + overshoot;
    let y = egui::lerp(phase..=phase * (2.0 - phase), stretch) + overshoot;
    let corner = |start: Pos2, end: Pos2| {
        egui::pos2(
            egui::lerp(start.x..=end.x, x),
            egui::lerp(start.y..=end.y, y),
        )
    };
    Rect::from_min_max(
        corner(source.min, target.min),
        corner(source.max, target.max),
    )
}

fn matched_menu_radii(
    body: Rect,
    amount: f32,
    corners: egui::CornerRadius,
) -> egui::epaint::CornerRadiusF32 {
    let round = body.size().min_elem() * 0.5;
    let progress = smoothstep((amount - 0.30) / 0.65);
    let radius = |end| egui::lerp(round..=f32::from(end), progress);
    egui::epaint::CornerRadiusF32 {
        nw: radius(corners.nw),
        ne: radius(corners.ne),
        sw: radius(corners.sw),
        se: radius(corners.se),
    }
}

// A compact command menu takes the place of the icon button. The same
// silhouette grows out of it and collapses back without a second outline.
fn matched_menu_shape(
    source: Rect,
    target: Rect,
    amount: f32,
    frame: Frame,
    source_material: Option<AnchorMaterial>,
) -> Shape {
    if amount <= 0.001 {
        return Shape::Noop;
    }
    if (amount - 1.0).abs() < 0.001 {
        return frame.paint(target.shrink(frame.stroke.width) - frame.inner_margin);
    }
    let body = matched_menu_rect(source, target, amount);
    let radii = matched_menu_radii(body, amount, frame.corner_radius);
    let mut shadow = frame.shadow;
    shadow.color = shadow
        .color
        .gamma_multiply(smoothstep((amount - 0.07) / 0.67));
    let material = surface_material(frame);
    let material = source_material.map_or(material, |source| {
        mix_material(source, material, smoothstep((amount - 0.12) / 0.38))
    });
    let panel = crate::motion::rounded_shape(
        body,
        radii,
        material.fill,
        egui::Stroke::new(frame.stroke.width, material.border),
        egui::StrokeKind::Inside,
    );
    let mut shape = Shape::Vec(vec![shadow.as_shape(body, radii).into(), panel]);
    fade_shape(&mut shape, smoothstep(amount / 0.17));
    shape
}

fn droplet_radii(
    source: Rect,
    target: Rect,
    body: Rect,
    phase: f32,
    corners: egui::CornerRadius,
) -> egui::epaint::CornerRadiusF32 {
    let seed = droplet_seed(source, target);
    let stretch = elongation(target);
    let start_radius = egui::lerp(seed * 0.5..=body.size().min_elem() * 0.5, stretch);
    let radius = |end| egui::lerp(start_radius..=f32::from(end), smoothstep(phase));
    egui::epaint::CornerRadiusF32 {
        nw: radius(corners.nw),
        ne: radius(corners.ne),
        sw: radius(corners.sw),
        se: radius(corners.se),
    }
}

fn surface_shape(
    source: Rect,
    painted_source: Rect,
    target: Rect,
    amount: f32,
    frame: Frame,
    motion: LiquidStyle,
    source_material: Option<AnchorMaterial>,
) -> Shape {
    if (amount - 1.0).abs() < 0.001 {
        return frame.paint(target.shrink(frame.stroke.width) - frame.inner_margin);
    }
    if amount <= 0.001 {
        return Shape::Noop;
    }
    let pose = surface_rect(source, target, amount);
    if motion.transition == SurfaceTransition::Bloom {
        let mut bubble = frame;
        let start_radius = source.height().min(source.width()) * 0.5;
        let radius = |end: u8| {
            egui::lerp(start_radius..=f32::from(end), amount.clamp(0.0, 1.0))
                .round()
                .clamp(0.0, 255.0) as u8
        };
        bubble.corner_radius = egui::CornerRadius {
            nw: radius(frame.corner_radius.nw),
            ne: radius(frame.corner_radius.ne),
            sw: radius(frame.corner_radius.sw),
            se: radius(frame.corner_radius.se),
        };
        bubble.shadow.color = bubble.shadow.color.gamma_multiply(smoothstep(amount));
        return bubble.paint(pose.shrink(frame.stroke.width) - frame.inner_margin);
    }
    let phase = amount.clamp(0.0, 1.0);
    let body = droplet_rect(source, target, amount);
    let radii = droplet_radii(source, target, body, phase, frame.corner_radius);
    let stretch = elongation(target);
    let mut body_frame = frame;
    body_frame.corner_radius = radii.into();
    body_frame.shadow.color = frame
        .shadow
        .color
        .gamma_multiply(smoothstep((phase - 0.78) / 0.21));

    let blend_radius = if motion.blend_radius.is_finite() {
        motion.blend_radius.clamp(0.0, 96.0)
    } else {
        34.0
    };
    let gap = if target.center().y >= source.center().y {
        body.top() - painted_source.bottom()
    } else {
        painted_source.top() - body.bottom()
    };
    let release_start = egui::lerp(0.67..=0.52, stretch);
    let release = 1.0 - smoothstep((phase - release_start) / (0.96 - release_start));
    let smoothing = blend_radius * smoothstep(phase / 0.24) * release;
    let shadow = body_frame.shadow.as_shape(body, body_frame.corner_radius);
    let stroke = frame.stroke;
    // Outside each field's smoothing range the panel is already a rounded
    // rectangle. Keep its fractional geometry through the paint handoff.
    if (gap >= smoothing && gap >= 0.0) || release == 0.0 {
        let panel =
            crate::motion::rounded_shape(body, radii, frame.fill, stroke, egui::StrokeKind::Inside);
        return Shape::Vec(vec![shadow.into(), panel]);
    }
    // A stable origin and leaf size prevent the outline sampling grid from
    // shifting as the shape moves. Large interiors are emitted as single quads.
    let envelope = source.union(target).expand(blend_radius * 0.25 + 6.0);
    let side = envelope.size().max_elem();
    let bounds = Rect::from_min_size(envelope.min, Vec2::splat(side));
    let material = surface_material(frame);
    let mut outline = Contour {
        mesh: egui::Mesh::default(),
        edge_mesh: egui::Mesh::default(),
        anchor: RoundedField::new(
            painted_source,
            (painted_source.size().min_elem() * 0.5).into(),
        ),
        panel: RoundedField::new(body, radii),
        smoothing,
        material,
        // Keep the real trigger color at the join until the neck detaches.
        // The colored band recedes with the smoothing range.
        source_material: source_material.filter(|source| *source != material),
        color_reach: (painted_source.height() * 0.55).min(smoothing).max(0.001),
        stroke_width: stroke.width,
    };
    outline.tile(bounds, (side / 128.0).max(3.2));
    // A neighboring fill triangle can overlap the inside edge of a stroke.
    // Draw every fill before the outline to keep that edge continuous.
    outline.mesh.append(outline.edge_mesh);
    Shape::Vec(vec![shadow.into(), Shape::mesh(outline.mesh)])
}

struct RoundedField {
    center: Pos2,
    half: Vec2,
    radii: egui::epaint::CornerRadiusF32,
}

impl RoundedField {
    fn new(rect: Rect, radii: egui::epaint::CornerRadiusF32) -> Self {
        Self {
            center: rect.center(),
            half: rect.size() * 0.5,
            radii,
        }
    }

    fn corner(&self, point: Pos2) -> (Vec2, Vec2, f32) {
        let local = point - self.center;
        let radius = match (local.x < 0.0, local.y < 0.0) {
            (true, true) => self.radii.nw,
            (false, true) => self.radii.ne,
            (true, false) => self.radii.sw,
            (false, false) => self.radii.se,
        }
        .clamp(0.0, self.half.min_elem().max(0.0));
        (local, local.abs() - self.half + Vec2::splat(radius), radius)
    }

    fn distance(&self, point: Pos2) -> f32 {
        let (_, q, radius) = self.corner(point);
        if q.x > 0.0 && q.y > 0.0 {
            q.length() - radius
        } else {
            q.x.max(q.y) - radius
        }
    }

    fn sample(&self, point: Pos2) -> (f32, Vec2) {
        let (local, q, radius) = self.corner(point);
        if q.x > 0.0 && q.y > 0.0 {
            let length = q.length();
            let normal = egui::vec2(q.x.copysign(local.x), q.y.copysign(local.y)) / length;
            (length - radius, normal)
        } else if q.x > q.y {
            (q.x - radius, egui::vec2(1.0_f32.copysign(local.x), 0.0))
        } else {
            (q.y - radius, egui::vec2(0.0, 1.0_f32.copysign(local.y)))
        }
    }
}

// Refine only around the new contour and the narrow band that hides the old
// trigger border. Both rounded-rectangle fields are 1-Lipschitz.
struct Contour {
    mesh: egui::Mesh,
    edge_mesh: egui::Mesh,
    anchor: RoundedField,
    panel: RoundedField,
    smoothing: f32,
    material: AnchorMaterial,
    source_material: Option<AnchorMaterial>,
    color_reach: f32,
    stroke_width: f32,
}

fn surface_material(frame: Frame) -> AnchorMaterial {
    AnchorMaterial {
        fill: frame.fill,
        border: frame.stroke.color,
    }
}

fn mix_material(a: AnchorMaterial, b: AnchorMaterial, t: f32) -> AnchorMaterial {
    AnchorMaterial {
        fill: a.fill.lerp_to_gamma(b.fill, t),
        border: a.border.lerp_to_gamma(b.border, t),
    }
}

impl Contour {
    fn material_at(&self, anchor_distance: f32) -> AnchorMaterial {
        self.source_material
            .filter(|_| anchor_distance < self.color_reach)
            .map_or(self.material, |source| {
                mix_material(
                    source,
                    self.material,
                    smoothstep(anchor_distance / self.color_reach),
                )
            })
    }

    fn distances(&self, point: Pos2) -> (f32, f32) {
        let anchor = self.anchor.distance(point);
        let panel = self.panel.distance(point);
        (smooth_union(anchor, panel, self.smoothing), anchor)
    }

    fn normal(&self, point: Pos2) -> Vec2 {
        let (a, anchor) = self.anchor.sample(point);
        let (b, panel) = self.panel.sample(point);
        let weight = if self.smoothing > 0.0 {
            (0.5 + (a - b) / (2.0 * self.smoothing)).clamp(0.0, 1.0)
        } else if a < b {
            0.0
        } else {
            1.0
        };
        (anchor * (1.0 - weight) + panel * weight).normalized()
    }

    fn tile(&mut self, rect: Rect, step: f32) {
        let mid = rect.center();
        let reach = rect.size().max_elem() * std::f32::consts::FRAC_1_SQRT_2;
        let anchor = self.anchor.distance(mid);
        if anchor + reach < -5.0 {
            return;
        }
        let panel = self.panel.distance(mid);
        if panel - anchor >= self.smoothing + 2.0 * reach {
            return;
        }
        let value = smooth_union(anchor, panel, self.smoothing);
        if value > reach {
            return;
        }
        let flat_distance = if self.source_material.is_some() {
            self.color_reach.max(1.5)
        } else {
            1.5
        };
        if value < -reach && anchor - reach >= flat_distance {
            self.mesh.add_colored_rect(rect, self.material.fill);
            return;
        }
        let step_here = if anchor.abs() < reach + 3.0 && panel - anchor < self.smoothing {
            step * 0.5
        } else {
            step
        };
        if rect.width() > step_here
            && mid.x > rect.left()
            && mid.x < rect.right()
            && mid.y > rect.top()
            && mid.y < rect.bottom()
        {
            for (min, max) in [
                (rect.min, mid),
                (
                    egui::pos2(mid.x, rect.top()),
                    egui::pos2(rect.right(), mid.y),
                ),
                (
                    egui::pos2(rect.left(), mid.y),
                    egui::pos2(mid.x, rect.bottom()),
                ),
                (mid, rect.max),
            ] {
                self.tile(Rect::from_min_max(min, max), step);
            }
            return;
        }
        let p = [
            rect.left_top(),
            rect.right_top(),
            rect.right_bottom(),
            rect.left_bottom(),
        ];
        let d = p.map(|point| self.distances(point));
        self.triangle([p[0], p[1], p[2]], [d[0], d[1], d[2]]);
        self.triangle([p[0], p[2], p[3]], [d[0], d[2], d[3]]);
    }

    fn triangle(&mut self, p: [Pos2; 3], d: [(f32, f32); 3]) {
        let mut crossings = [(Pos2::ZERO, 0.0, 0.0); 2];
        let mut count = 0;
        let base = self.mesh.vertices.len() as u32;
        for a in 0..3 {
            let b = (a + 1) % 3;
            let (value, anchor) = d[a];
            if value <= 0.0 {
                let color = self
                    .material_at(anchor)
                    .fill
                    .gamma_multiply(Self::coverage(value, anchor));
                self.mesh.colored_vertex(p[a], color);
            }
            if (value <= 0.0) != (d[b].0 <= 0.0) {
                let t = value / (value - d[b].0);
                let q = p[a].lerp(p[b], t);
                let anchor = egui::lerp(anchor..=d[b].1, t);
                let alpha = Self::coverage(0.0, anchor);
                self.mesh
                    .colored_vertex(q, self.material_at(anchor).fill.gamma_multiply(alpha));
                crossings[count] = (q, alpha, anchor);
                count += 1;
            }
        }
        let end = self.mesh.vertices.len() as u32;
        for i in base + 1..end.saturating_sub(1) {
            self.mesh.add_triangle(base, i, i + 1);
        }
        if count == 2 {
            let [(a, alpha_a, anchor_a), (b, alpha_b, anchor_b)] = crossings;
            if alpha_a == 0.0 && alpha_b == 0.0 {
                return;
            }
            // Adjacent triangles share the same field normals at crossings,
            // so their antialiasing fringes meet without gaps on curves.
            let normal_a = self.normal(a);
            let normal_b = self.normal(b);
            let edge = |anchor, alpha| {
                let material = self.material_at(anchor);
                let color = if self.stroke_width > 0.0 {
                    material.border
                } else {
                    material.fill
                };
                color.gamma_multiply(alpha)
            };
            let color_a = edge(anchor_a, alpha_a);
            let color_b = edge(anchor_b, alpha_b);
            if self.stroke_width > 0.0 {
                self.quad([
                    (a, color_a),
                    (b, color_b),
                    (b - normal_b * self.stroke_width, color_b),
                    (a - normal_a * self.stroke_width, color_a),
                ]);
            }
            self.quad([
                (a, color_a),
                (b, color_b),
                (b + normal_b, Color32::TRANSPARENT),
                (a + normal_a, Color32::TRANSPARENT),
            ]);
        }
    }

    fn coverage(value: f32, anchor: f32) -> f32 {
        // Follow the source contour while preserving its original label.
        smoothstep((anchor - value) / 1.5) * smoothstep((anchor + 5.0) / 2.5)
    }

    fn quad(&mut self, vertices: [(Pos2, Color32); 4]) {
        let base = self.edge_mesh.vertices.len() as u32;
        for (point, color) in vertices {
            self.edge_mesh.colored_vertex(point, color);
        }
        self.edge_mesh.add_triangle(base, base + 1, base + 2);
        self.edge_mesh.add_triangle(base, base + 2, base + 3);
    }
}

#[derive(Clone, Copy, Default)]
pub(crate) enum OverlayTravel {
    #[default]
    Source,
    Center,
    Left,
    Bottom,
}

/// Paint-only transition for modal surfaces. The live egui modal owns
/// focus and input. Its closing snapshot is deliberately noninteractive.
pub(crate) struct OverlayPass {
    id: Id,
    layer: LayerId,
    amount: f32,
    source: Rect,
    source_material: Option<AnchorMaterial>,
    travel: OverlayTravel,
    motion: LiquidStyle,
    drag: f32,
}

impl OverlayPass {
    pub fn begin(
        ctx: &egui::Context,
        layer: LayerId,
        open: bool,
        source: (Rect, Option<AnchorMaterial>),
        travel: OverlayTravel,
        motion: LiquidStyle,
        theme: &Theme,
    ) -> Self {
        let id = layer.id;
        let (source, source_material) = source;
        let progress = id.with("overlay-progress");
        let amount = if matches!(travel, OverlayTravel::Bottom) {
            crate::motion::surface_reveal(ctx, progress, open, theme)
        } else {
            surface_progress(ctx, progress, open, motion, theme)
        };
        Self {
            id,
            layer,
            amount,
            source,
            source_material,
            travel,
            motion,
            drag: if matches!(travel, OverlayTravel::Bottom) {
                crate::sheet::drag_offset(ctx, id)
            } else {
                0.0
            },
        }
    }

    pub fn interactive(&self) -> bool {
        self.amount >= 0.99
    }

    fn phase(&self) -> f32 {
        self.amount.min(1.0)
    }

    fn slide(&self, rect: Rect, clip: Rect) -> egui::emath::TSTransform {
        let offset = match self.travel {
            OverlayTravel::Source | OverlayTravel::Center => Vec2::ZERO,
            OverlayTravel::Left => egui::vec2(clip.left() - rect.right() - 8.0, 0.0),
            OverlayTravel::Bottom => egui::vec2(0.0, clip.bottom() - rect.top() + 8.0),
        };
        let displacement =
            offset * (1.0 - self.phase()) + egui::vec2(0.0, self.drag * self.phase());
        let slide = egui::emath::TSTransform::from_translation(displacement);
        if matches!(self.travel, OverlayTravel::Bottom) {
            slide
                * crate::motion::scale_around(
                    rect.center_bottom(),
                    egui::lerp(0.95..=1.0, self.phase()),
                )
        } else {
            slide
        }
    }

    fn surface_style(&self, rect: Rect) -> LiquidStyle {
        if self.source.intersects(rect) {
            LiquidStyle {
                transition: SurfaceTransition::Bloom,
                ..self.motion
            }
        } else {
            self.motion
        }
    }

    fn surface(&self, rect: Rect, frame: Frame, clip: Rect) -> Shape {
        match self.travel {
            OverlayTravel::Source | OverlayTravel::Center => {
                let style = self.surface_style(rect);
                let amount = if style.transition == SurfaceTransition::Droplet {
                    self.amount
                } else {
                    self.phase()
                };
                let mut shape = surface_shape(
                    self.source,
                    self.source,
                    rect,
                    amount,
                    frame,
                    style,
                    self.source_material,
                );
                if matches!(self.travel, OverlayTravel::Center) {
                    fade_shape(&mut shape, smoothstep(self.phase() / 0.28));
                }
                shape
            }
            OverlayTravel::Left | OverlayTravel::Bottom => {
                let mut shape = frame.paint(rect.shrink(frame.stroke.width) - frame.inner_margin);
                shape.transform(self.slide(rect, clip));
                shape
            }
        }
    }

    pub fn start(&self, ctx: &egui::Context) -> egui::layers::ShapeIdx {
        ctx.graphics_mut(|graphics| graphics.entry(self.layer).next_idx())
    }

    pub fn finish(
        &mut self,
        ctx: &egui::Context,
        start: egui::layers::ShapeIdx,
        surface: egui::layers::ShapeIdx,
        rect: Rect,
        frame: Frame,
    ) {
        if matches!(self.travel, OverlayTravel::Bottom) {
            self.drag = crate::sheet::drag_offset(ctx, self.id);
        }
        let clip = ctx.content_rect();
        let is_backdrop = |shape: &Shape| matches!(shape, Shape::Rect(value) if value.rect == clip);
        let (end, backdrop, shapes) = ctx.graphics(|graphics| {
            let list = graphics.get(self.layer).unwrap();
            let mut backdrop = None;
            let mut shapes = Vec::new();
            for (index, entry) in list.all_entries().enumerate().skip(start.0) {
                if index == surface.0 {
                    continue;
                }
                if backdrop.is_none() && index < surface.0 && is_backdrop(&entry.shape) {
                    backdrop = Some(entry.clone());
                } else if index > surface.0 {
                    shapes.push(entry.clone());
                }
            }
            (list.next_idx(), backdrop, shapes)
        });
        ctx.data_mut(|data| {
            data.insert_temp(
                self.id.with("overlay-snapshot"),
                Snapshot {
                    rect,
                    shapes: Arc::new(shapes),
                    backdrop,
                    frame,
                },
            );
        });
        if (self.amount - 1.0).abs() < 0.001 && self.drag == 0.0 {
            return;
        }
        let animated_surface = self.surface(rect, frame, clip);
        ctx.graphics_mut(|graphics| {
            let list = graphics.entry(self.layer);
            list.set(surface, clip, animated_surface);
            for index in start.0..end.0 {
                if index != surface.0 {
                    list.mutate_shape(egui::layers::ShapeIdx(index), |entry| {
                        if index < surface.0 && is_backdrop(&entry.shape) {
                            fade_shape(&mut entry.shape, opacity(self.phase()));
                        } else if index > surface.0 {
                            self.decorate_content(entry, rect, clip);
                        }
                    });
                }
            }
        });
    }

    fn decorate_content(&self, entry: &mut egui::epaint::ClippedShape, rect: Rect, clip: Rect) {
        let phase = self.phase();
        let (transform, visible, alpha) =
            if matches!(self.travel, OverlayTravel::Source | OverlayTravel::Center) {
                let style = self.surface_style(rect).transition;
                (
                    content_transform(self.source, rect, phase, style),
                    motion_rect(self.source, rect, phase, style),
                    content_opacity(phase, style),
                )
            } else {
                let transform = self.slide(rect, clip);
                (transform, transform.mul_rect(rect), 1.0)
            };
        entry.shape.transform(transform);
        entry.clip_rect = transform
            .mul_rect(entry.clip_rect)
            .intersect(visible)
            .intersect(clip);
        fade_shape(&mut entry.shape, alpha);
    }

    pub fn paint_exit(&self, ctx: &egui::Context) {
        let key = self.id.with("overlay-snapshot");
        if self.amount <= 0.001 {
            ctx.data_mut(|data| data.remove::<Snapshot>(key));
            if matches!(self.travel, OverlayTravel::Bottom) && self.drag != 0.0 {
                crate::sheet::set_drag_offset(ctx, self.id, 0.0);
            }
            return;
        }
        if let Some(snapshot) = ctx.data(|data| data.get_temp::<Snapshot>(key)) {
            let painter = ctx.layer_painter(self.layer);
            let clip = ctx.content_rect();
            if let Some(mut backdrop) = snapshot.backdrop {
                fade_shape(&mut backdrop.shape, opacity(self.phase()));
                painter
                    .with_clip_rect(backdrop.clip_rect)
                    .add(backdrop.shape);
            }
            painter.add(self.surface(snapshot.rect, snapshot.frame, clip));
            for entry in snapshot.shapes.iter() {
                let mut entry = entry.clone();
                self.decorate_content(&mut entry, snapshot.rect, clip);
                painter.with_clip_rect(entry.clip_rect).add(entry.shape);
            }
        }
    }
}

pub(crate) fn fade_shape(shape: &mut Shape, alpha: f32) {
    egui::epaint::shape_transform::adjust_colors(shape, move |color| {
        if *color != Color32::PLACEHOLDER {
            *color = color.gamma_multiply(alpha);
        }
    });
}

/// Draw the same SDF morph in a custom widget without choosing a GPU backend.
/// The caller owns progress and input; this function only emits bounded paint.
/// `target` is the final outer rectangle, including the frame's inner padding.
pub fn paint_surface(
    painter: &egui::Painter,
    source: Rect,
    target: Rect,
    amount: f32,
    frame: Frame,
    motion: LiquidStyle,
) {
    if amount.is_finite()
        && source.is_finite()
        && target.is_finite()
        && source.width() >= 0.0
        && source.height() >= 0.0
        && target.is_positive()
        && source.size().is_finite()
        && target.size().is_finite()
        && source.union(target).size().is_finite()
    {
        painter.add(surface_shape(
            source, source, target, amount, frame, motion, None,
        ));
    }
}
