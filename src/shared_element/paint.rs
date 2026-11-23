use super::{SharedElementFit, SharedElementPose, SharedElementStyle};
use egui::emath::TSTransform;
use egui::epaint::{ClippedShape, CornerRadiusF32, Mesh, PathShape, Primitive, Vertex};
use egui::{Color32, Context, InnerResponse, Painter, Pos2, Rect, Shape, Stroke, Ui, Vec2};

/// Reusable egui paint for a shared element. Keep any application textures
/// alive until the transition has finished.
pub struct SharedElementSnapshot {
    bounds: Rect,
    pieces: Vec<PaintPiece>,
}

struct PaintPiece {
    clip: Rect,
    bounds: Rect,
    mesh: Mesh,
}

impl SharedElementSnapshot {
    /// Prepare paint-only content in arbitrary source coordinates. Call during
    /// an egui frame, after any glyphs in the shapes have been laid out.
    pub fn from_shapes(
        ctx: &Context,
        bounds: Rect,
        shapes: impl IntoIterator<Item = Shape>,
    ) -> Self {
        Self::prepare(
            ctx,
            bounds,
            shapes
                .into_iter()
                .map(|shape| ClippedShape::new(bounds, shape))
                .collect(),
        )
    }

    /// Capture the measured output of a live widget such as an egui Frame or
    /// a Temivu Card. Call on the frame that leaves that widget or when its
    /// content changes. The widget remains interactive in this pass.
    /// Capture covers regular shapes on the widget's own layer.
    pub fn capture<R>(
        ui: &mut Ui,
        draw: impl FnOnce(&mut Ui) -> InnerResponse<R>,
    ) -> (InnerResponse<R>, Self) {
        let ctx = ui.ctx().clone();
        let layer = ui.layer_id();
        let start =
            ctx.graphics(|graphics| graphics.get(layer).map_or(0, |list| list.next_idx().0));
        let result = draw(ui);
        let bounds = result.response.rect;
        let shapes = ctx.graphics(|graphics| {
            graphics.get(layer).map_or_else(Vec::new, |list| {
                list.all_entries().skip(start).cloned().collect()
            })
        });
        (result, Self::prepare(&ctx, bounds, shapes))
    }

    fn prepare(ctx: &Context, bounds: Rect, mut shapes: Vec<ClippedShape>) -> Self {
        if !bounds.is_finite() || !bounds.is_positive() || !bounds.size().is_finite() {
            return Self {
                bounds: Rect::NOTHING,
                pieces: Vec::new(),
            };
        }
        for shape in &mut shapes {
            shape.clip_rect = shape.clip_rect.intersect(bounds);
        }
        let pieces = ctx
            .tessellate(shapes, ctx.pixels_per_point())
            .into_iter()
            .filter_map(|entry| {
                let Primitive::Mesh(mesh) = entry.primitive else {
                    return None;
                };
                let bounds = mesh.calc_bounds();
                (bounds.is_positive() && entry.clip_rect.intersects(bounds)).then_some(PaintPiece {
                    clip: entry.clip_rect,
                    bounds,
                    mesh,
                })
            })
            .collect();
        Self { bounds, pieces }
    }

    fn paint(&self, painter: &Painter, mask: &RoundedMask, fit: SharedElementFit, opacity: f32) {
        if self.pieces.is_empty() || opacity <= 0.0 {
            return;
        }
        let rect = mask.rect;
        let ratio = rect.size() / self.bounds.size();
        let scale = match fit {
            SharedElementFit::Contain => ratio.min_elem(),
            SharedElementFit::Cover => ratio.max_elem(),
        };
        if !scale.is_finite() || scale <= 0.0 {
            return;
        }
        let transform = TSTransform::new(rect.center() - self.bounds.center() * scale, scale);
        let viewport = painter.clip_rect().intersect(rect);
        for piece in &self.pieces {
            let clip = viewport.intersect(transform.mul_rect(piece.clip));
            let bounds = transform.mul_rect(piece.bounds);
            if !clip.is_positive() || !clip.intersects(bounds) {
                continue;
            }
            let transform_vertex = |mut vertex: Vertex| {
                vertex.pos = transform * vertex.pos;
                vertex.color = vertex.color.gamma_multiply(opacity);
                vertex
            };
            let mut mesh = Mesh::with_texture(piece.mesh.texture_id);
            if mask.contains_rect(bounds) {
                mesh.indices.clone_from(&piece.mesh.indices);
                mesh.vertices
                    .extend(piece.mesh.vertices.iter().copied().map(transform_vertex));
            } else {
                mesh.vertices.reserve(piece.mesh.vertices.len());
                mesh.indices.reserve(piece.mesh.indices.len());
                for &triangle in piece.mesh.indices.as_chunks::<3>().0 {
                    let [a, b, c] =
                        triangle.map(|index| transform_vertex(piece.mesh.vertices[index as usize]));
                    let outside = [a.pos, b.pos, c.pos];
                    if outside.iter().all(|p| p.x < clip.left())
                        || outside.iter().all(|p| p.x > clip.right())
                        || outside.iter().all(|p| p.y < clip.top())
                        || outside.iter().all(|p| p.y > clip.bottom())
                    {
                        continue;
                    }
                    mask.append_triangle([a, b, c], &mut mesh);
                }
            }
            if !mesh.is_empty() {
                painter.with_clip_rect(clip).add(Shape::mesh(mesh));
            }
        }
    }
}

pub(super) fn paint(
    painter: &Painter,
    style: SharedElementStyle,
    pose: SharedElementPose,
    content: [Option<&SharedElementSnapshot>; 2],
) {
    if !painter.is_visible()
        || !pose.rect.is_finite()
        || !pose.rect.is_positive()
        || !pose.rect.size().is_finite()
        || !painter.clip_rect().intersects(pose.rect)
    {
        return;
    }
    if style.scrim != Color32::TRANSPARENT && pose.progress > 0.0 {
        painter.rect_filled(
            painter.clip_rect().intersect(painter.ctx().viewport_rect()),
            0,
            style.scrim.gamma_multiply(pose.progress),
        );
    }
    let mask = RoundedMask::new(pose.rect, pose.corners);
    if pose.fill != Color32::TRANSPARENT {
        painter.add(PathShape::convex_polygon(
            mask.points().to_vec(),
            pose.fill,
            Stroke::NONE,
        ));
    }
    // A photo or icon can keep a single continuous identity while the two
    // layouts change around it.
    if let [Some(source), Some(destination)] = content
        && std::ptr::eq(source, destination)
    {
        source.paint(painter, &mask, style.fit, 1.0);
        return;
    }
    for (snapshot, opacity) in content.into_iter().zip(pose.opacity) {
        if let Some(snapshot) = snapshot {
            snapshot.paint(painter, &mask, style.fit, opacity);
        }
    }
}

const ARC_STEPS: usize = 10;
const MASK_CAPACITY: usize = 4 * (ARC_STEPS + 1);
const CLIP_CAPACITY: usize = MASK_CAPACITY + 3;

struct RoundedMask {
    rect: Rect,
    interior: Rect,
    points: [Pos2; MASK_CAPACITY],
    len: usize,
}

impl RoundedMask {
    fn new(rect: Rect, corners: CornerRadiusF32) -> Self {
        let corners = super::bounded_corners(corners, rect);
        let radius = corners.nw.max(corners.ne).max(corners.se).max(corners.sw);
        let mut mask = Self {
            rect,
            interior: rect.shrink(radius * 0.30),
            points: [Pos2::ZERO; MASK_CAPACITY],
            len: 0,
        };
        let semicorner = std::f32::consts::FRAC_PI_2;
        for (step, (corner, radius, direction)) in [
            (rect.right_top(), corners.ne, Vec2::new(-1.0, 1.0)),
            (rect.right_bottom(), corners.se, Vec2::new(-1.0, -1.0)),
            (rect.left_bottom(), corners.sw, Vec2::new(1.0, -1.0)),
            (rect.left_top(), corners.nw, Vec2::new(1.0, 1.0)),
        ]
        .into_iter()
        .enumerate()
        {
            if radius == 0.0 {
                mask.push(corner);
                continue;
            }
            let start = (step as f32 - 1.0) * semicorner;
            let center = corner + direction * radius;
            for point in 0..=ARC_STEPS {
                let angle = start + semicorner * (point as f32 / ARC_STEPS as f32);
                mask.push(rect.clamp(center + Vec2::angled(angle) * radius));
            }
        }
        mask
    }

    fn push(&mut self, point: Pos2) {
        if self.len == 0 || self.points[self.len - 1].distance_sq(point) > 0.000_000_1 {
            self.points[self.len] = point;
            self.len += 1;
        }
    }

    fn points(&self) -> &[Pos2] {
        &self.points[..self.len]
    }

    fn edges(&self) -> impl Iterator<Item = (Pos2, Vec2)> + '_ {
        self.points()
            .iter()
            .copied()
            .zip(self.points().iter().copied().cycle().skip(1))
            .map(|(start, end)| (start, end - start))
    }

    fn contains(&self, point: Pos2) -> bool {
        self.interior.contains(point)
            || (self.rect.contains(point)
                && self
                    .edges()
                    .all(|(start, edge)| signed_distance(start, edge, point) >= -0.0001))
    }

    fn contains_rect(&self, rect: Rect) -> bool {
        [
            rect.left_top(),
            rect.right_top(),
            rect.left_bottom(),
            rect.right_bottom(),
        ]
        .into_iter()
        .all(|point| self.contains(point))
    }

    fn append_triangle(&self, triangle: [Vertex; 3], mesh: &mut Mesh) {
        if triangle.iter().all(|vertex| self.contains(vertex.pos)) {
            append_fan(&triangle, mesh);
            return;
        }
        const EMPTY: Vertex = Vertex {
            pos: Pos2::ZERO,
            uv: Pos2::ZERO,
            color: Color32::TRANSPARENT,
        };
        let mut polygon = [EMPTY; CLIP_CAPACITY];
        let mut buffer = [EMPTY; CLIP_CAPACITY];
        polygon[..3].copy_from_slice(&triangle);
        let mut len = 3;
        for (start, edge) in self.edges() {
            let mut size = 0;
            let mut previous = polygon[len - 1];
            let mut d_previous = signed_distance(start, edge, previous.pos);
            for &current in &polygon[..len] {
                let d_current = signed_distance(start, edge, current.pos);
                if (d_previous < 0.0) != (d_current < 0.0) {
                    let t = (d_previous / (d_previous - d_current)).clamp(0.0, 1.0);
                    if t > 0.0 && t < 1.0 {
                        buffer[size] = Vertex {
                            pos: previous.pos.lerp(current.pos, t),
                            uv: previous.uv.lerp(current.uv, t),
                            color: previous.color.lerp_to_gamma(current.color, t),
                        };
                        size += 1;
                    }
                }
                if d_current >= 0.0 {
                    buffer[size] = current;
                    size += 1;
                }
                previous = current;
                d_previous = d_current;
            }
            if size < 3 {
                return;
            }
            std::mem::swap(&mut polygon, &mut buffer);
            len = size;
        }
        append_fan(&polygon[..len], mesh);
    }
}

fn signed_distance(origin: Pos2, edge: Vec2, point: Pos2) -> f32 {
    let delta = point - origin;
    edge.x * delta.y - edge.y * delta.x
}

fn append_fan(vertices: &[Vertex], mesh: &mut Mesh) {
    let base = mesh.vertices.len() as u32;
    mesh.vertices.extend_from_slice(vertices);
    for index in 1..(vertices.len() as u32 - 1) {
        mesh.add_triangle(base, base + index, base + index + 1);
    }
}
