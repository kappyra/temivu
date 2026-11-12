use egui::{
    Color32, Pos2, Rect, Vec2,
    epaint::{CornerRadiusF32, Mesh},
    pos2, vec2,
};

use super::{LightQuality, Sample, Vesper};

#[derive(Clone, Copy)]
pub(super) struct Ink {
    highlight: Color32,
    pale: f32,
}

impl Ink {
    pub(super) fn new(tint: Option<Color32>, surface: Color32, fallback: Color32) -> Self {
        if let Some(highlight) = tint.filter(|&color| color != Color32::WHITE) {
            return Self {
                highlight,
                pale: 0.0,
            };
        }
        let luminance = |color: Color32| {
            let [r, g, b, _] = color.to_srgba_unmultiplied();
            (0.2126 * f32::from(r) + 0.7152 * f32::from(g) + 0.0722 * f32::from(b)) / 255.0
        };
        let luma = egui::lerp(
            luminance(fallback)..=luminance(surface),
            f32::from(surface.a()) / 255.0,
        );
        let t = ((luma - 0.57) / 0.40).clamp(0.0, 1.0);
        Self {
            highlight: Color32::WHITE,
            pale: t * t * (3.0 - 2.0 * t),
        }
    }

    fn color(self, highlight: f32, shadow: f32) -> Color32 {
        let light = self.highlight.gamma_multiply(highlight.clamp(0.0, 0.62));
        if self.pale == 0.0 || shadow <= 0.0 {
            return light;
        }
        let shade = (f32::from(255 - light.a()) * (shadow * self.pale).min(0.24)).round() as u8;
        Color32::from_rgba_premultiplied(
            light.r(),
            light.g(),
            light.b(),
            light.a().saturating_add(shade),
        )
    }
}

/// Build one untextured, bounded paint mesh. All triangles lie within a convex
/// rounded rectangle, so the rectangular egui scissor only has to clip the parent.
pub(super) fn mesh(
    rect: Rect,
    corners: CornerRadiusF32,
    style: &Vesper,
    ink: Ink,
    sample: Sample,
) -> Mesh {
    let mut mesh = Mesh::default();
    if !rect.is_finite() || !rect.is_positive() || ink.highlight.a() == 0 {
        return mesh;
    }
    let energy = sample.energy.clamp(0.0, 1.0) * style.opacity * style.intensity;
    if !energy.is_finite() || energy <= 0.001 {
        return mesh;
    }

    let shape = RoundedRect::new(rect, corners);
    let quality = match style.quality {
        LightQuality::Low => Density {
            grid: [8, 6],
            side: 6,
            arc: 3,
            particles: 4,
        },
        LightQuality::Balanced => Density {
            grid: [12, 8],
            side: 10,
            arc: 5,
            particles: 6,
        },
        LightQuality::High => Density {
            grid: [18, 12],
            side: 15,
            arc: 8,
            particles: 8,
        },
    };
    let light = Field {
        style,
        ink,
        sample,
        energy,
        shape,
    };

    let effects = style.effects;
    let scatter = effects.scatter && sample.pulse < 1.0;
    let field =
        effects.follow || scatter || (effects.flow && sample.flow != Vec2::ZERO) || effects.depth;
    let edge = effects.edge;
    let particles = effects.particles && sample.pulse < 1.0;
    let [nx, ny] = quality.grid.map(|n| n as usize);
    let samples = |cells: usize, extent: f32| {
        [
            (effects.follow, style.radius),
            (scatter, style.radius * 0.24),
        ]
        .into_iter()
        .map(|(enabled, reach)| {
            if enabled {
                1 + 2 * usize::from(extent > reach * 0.65 * cells as f32)
            } else {
                0
            }
        })
        .sum::<usize>()
    };
    let (sx, sy) = (samples(nx, rect.width()), samples(ny, rect.height()));
    let edge_rows = (4 * (quality.side + quality.arc)) as usize;
    let corner_points = 4 * quality.arc as usize;
    // Each vector allocates once, even when all lighting channels overlap.
    mesh.vertices.reserve(
        usize::from(field) * ((nx + 1 + sx) * (ny + 1 + sy) + corner_points)
            + usize::from(edge) * edge_rows * 3
            + usize::from(particles) * quality.particles * 9,
    );
    mesh.indices.reserve(
        usize::from(field) * ((nx + sx) * (ny + sy) * 6 + corner_points * 3)
            + usize::from(edge) * edge_rows * 12
            + usize::from(particles) * quality.particles * 24,
    );

    if field {
        light.paint_field(&mut mesh, quality.grid, quality.arc);
    }
    if edge {
        light.paint_edge(&mut mesh, quality.side, quality.arc);
    }
    if particles {
        light.paint_particles(&mut mesh, quality.particles);
    }
    mesh
}

struct Density {
    grid: [u32; 2],
    side: u32,
    arc: u32,
    particles: usize,
}

#[derive(Clone, Copy)]
struct RoundedRect {
    rect: Rect,
    corners: CornerRadiusF32,
}

impl RoundedRect {
    fn new(rect: Rect, corners: CornerRadiusF32) -> Self {
        let limit = rect.size().min_elem() * 0.5;
        let radius = |r: f32| {
            if r.is_finite() {
                r.clamp(0.0, limit)
            } else {
                0.0
            }
        };
        Self {
            rect,
            corners: CornerRadiusF32 {
                nw: radius(corners.nw),
                ne: radius(corners.ne),
                sw: radius(corners.sw),
                se: radius(corners.se),
            },
        }
    }

    fn span(self, y: f32) -> [f32; 2] {
        let cut = |r_top: f32, r_bottom: f32| {
            let top = (self.rect.top() + r_top - y).max(0.0);
            if top > 0.0 {
                return circle_inset(r_top, top);
            }
            let bottom = (y - (self.rect.bottom() - r_bottom)).max(0.0);
            circle_inset(r_bottom, bottom)
        };
        [
            self.rect.left() + cut(self.corners.nw, self.corners.sw),
            self.rect.right() - cut(self.corners.ne, self.corners.se),
        ]
    }

    fn project(self, point: Pos2) -> Pos2 {
        let y = point.y.clamp(self.rect.top(), self.rect.bottom());
        let [left, right] = self.span(y);
        pos2(point.x.clamp(left, right.max(left)), y)
    }

    fn contains(self, point: Pos2) -> bool {
        self.rect.contains(point) && self.project(point).distance_sq(point) < 0.000_001
    }

    fn arc_centers(self) -> [(Pos2, f32); 4] {
        let r = self.corners;
        [
            (self.rect.right_top() + vec2(-r.ne, r.ne), r.ne),
            (self.rect.right_bottom() - Vec2::splat(r.se), r.se),
            (self.rect.left_bottom() + vec2(r.sw, -r.sw), r.sw),
            (self.rect.left_top() + Vec2::splat(r.nw), r.nw),
        ]
    }
}

fn circle_inset(radius: f32, vertical: f32) -> f32 {
    radius - (radius * radius - vertical * vertical).max(0.0).sqrt()
}

struct Field<'a> {
    style: &'a Vesper,
    ink: Ink,
    sample: Sample,
    energy: f32,
    shape: RoundedRect,
}

impl Field<'_> {
    fn paint_field(&self, mesh: &mut Mesh, [nx, ny]: [u32; 2], arcs: u32) {
        let rect = self.shape.rect;
        let style = self.style;
        let sample = &self.sample;
        let radius = style.radius.max(1.0);
        let inverse_radius_sq = radius.recip().powi(2);
        let progress = sample.pulse.clamp(0.0, 1.0);
        let scatter = style.effects.scatter && progress < 1.0;
        let fade = 1.0 - progress;
        let spread = radius * (0.24 + 0.62 * (1.0 - fade * fade));
        let inverse_spread_sq = spread.recip().powi(2);
        let pulse_energy = 0.62 * fade * fade;
        let flow = style.effects.flow && sample.flow != Vec2::ZERO;
        let flow_origin = sample.point - sample.flow;
        let flow_energy = (sample.flow.length_sq() * 0.007).min(0.14);
        let fill = |point: Pos2| {
            let mut light = 0.0;
            let mut shadow = 0.0;
            let distance_sq = point.distance_sq(sample.point) * inverse_radius_sq;
            if style.effects.follow {
                let wash = falloff(distance_sq, style.softness, style.attenuation);
                let core = (1.0 - distance_sq * 4.2).max(0.0).powi(2);
                light += 0.47 * wash + 0.34 * core;
                // White cannot become brighter over a white fill. Make the
                // center read as light by shading the softer falloff around it.
                shadow += 0.70 * wash * (1.0 - 0.86 * core) + 0.10 * core;
            }
            if scatter {
                let d = point.distance_sq(sample.pulse_point) * inverse_spread_sq;
                let wave = falloff(d, style.softness, style.attenuation);
                light += pulse_energy * wave;
                shadow += pulse_energy * wave * (d * 2.0).min(0.68);
            }
            if flow {
                let d = point.distance_sq(flow_origin) * inverse_radius_sq * 1.9;
                light += flow_energy * falloff(d, 0.78, 1.6);
            }
            if style.effects.depth {
                let height = (point.y - rect.top()) / rect.height();
                let relief = (1.0 - height) * (1.0 - distance_sq).max(0.0);
                light += 0.14 * style.elevation * relief * relief;
            }
            let highlight = light * self.energy;
            let excess = (highlight - 0.26).max(0.0);
            // Keep labels legible and preserve the gradient through bright presses.
            let face = highlight.min(0.26) + 0.09 * excess / (0.09 + excess);
            self.ink.color(face, shadow * self.energy)
        };

        // Reserve a sample at each active source. Otherwise a narrow contact
        // pulse can pass between the fixed vertices of a wide surface.
        let follow = style.effects.follow.then_some(sample.point);
        let pulse = scatter.then_some(sample.pulse_point);
        let x = GridAxis::new(
            nx,
            follow.map(|p| ((p.x - rect.left()) / rect.width(), radius / rect.width())),
            pulse.map(|p| ((p.x - rect.left()) / rect.width(), spread / rect.width())),
        );
        let y = GridAxis::new(
            ny,
            follow.map(|p| ((p.y - rect.top()) / rect.height(), radius / rect.height())),
            pulse.map(|p| ((p.y - rect.top()) / rect.height(), spread / rect.height())),
        );
        let columns = x.points().len() as u32;
        let rows = y.points().len() as u32;
        for &t in y.points() {
            let y = egui::lerp(rect.y_range(), t);
            let [left, right] = self.shape.span(y);
            for &t in x.points() {
                let point = pos2(egui::lerp(left..=right, t), y);
                mesh.colored_vertex(point, fill(point));
            }
        }
        for row in 0..rows - 1 {
            for column in 0..columns - 1 {
                let top = row * columns + column;
                let bottom = top + columns;
                mesh.add_triangle(top, top + 1, bottom);
                mesh.add_triangle(top + 1, bottom + 1, bottom);
            }
        }
        self.paint_field_corners(mesh, y.points(), columns, arcs, fill);
    }

    fn paint_field_corners(
        &self,
        mesh: &mut Mesh,
        rows: &[f32],
        columns: u32,
        arcs: u32,
        fill: impl Fn(Pos2) -> Color32,
    ) {
        let rect = self.shape.rect;
        let corners = self.shape.corners;
        for (side, radii) in [[corners.nw, corners.sw], [corners.ne, corners.se]]
            .into_iter()
            .enumerate()
        {
            let column = if side == 0 { 0 } else { columns - 1 };
            for (half, radius) in radii.into_iter().enumerate() {
                if radius == 0.0 {
                    continue;
                }
                let extent = radius / rect.height();
                let mut previous_row = usize::MAX;
                let mut previous_vertex = 0;
                // Fill the narrow gaps between the grid's side chords and the arc.
                // Only the perimeter gains vertices, even on a wide or tall control.
                for step in 1..=arcs {
                    let arc_step = if half == 0 { step } else { arcs + 1 - step };
                    let angle = arc_step as f32 / arcs as f32 * std::f32::consts::FRAC_PI_2;
                    let offset = extent * (1.0 - angle.cos());
                    let t = if half == 0 { offset } else { 1.0 - offset };
                    let lower = rows.partition_point(|&row| row < t);
                    if lower == 0
                        || lower == rows.len()
                        || t - rows[lower - 1] < 0.000_01
                        || rows[lower] - t < 0.000_01
                    {
                        continue;
                    }
                    let bottom = lower as u32 * columns + column;
                    if lower != previous_row {
                        previous_row = lower;
                        previous_vertex = bottom - columns;
                    }
                    let y = egui::lerp(rect.y_range(), t);
                    let point = pos2(self.shape.span(y)[side], y);
                    let vertex = mesh.vertices.len() as u32;
                    mesh.colored_vertex(point, fill(point));
                    if side == 0 {
                        mesh.add_triangle(previous_vertex, bottom, vertex);
                    } else {
                        mesh.add_triangle(previous_vertex, vertex, bottom);
                    }
                    previous_vertex = vertex;
                }
            }
        }
    }

    fn paint_edge(&self, mesh: &mut Mesh, sides: u32, arcs: u32) {
        let rect = self.shape.rect;
        let first = mesh.vertices.len() as u32;
        let arcs_and_centers = self.shape.arc_centers();
        let width = (rect.size().min_elem() * 0.106).clamp(1.0, 4.0);
        let inverse_radius_sq = (self.style.radius * 1.03).max(1.0).recip().powi(2);
        let shift = (self.sample.point - rect.center()) / (rect.size() * 0.5);
        let side_density = sides as f32 / rect.size().max_elem();
        let ink = self.ink;
        let mut push = |point: Pos2, normal: Vec2| {
            let distance = point.distance_sq(self.sample.point) * inverse_radius_sq;
            let reflection = falloff(distance, 0.69, 2.1);
            let facing = (0.56 - normal.y * 0.19 + normal.dot(shift) * 0.26).clamp(0.25, 1.0);
            let sheen = if self.style.effects.depth {
                self.style.elevation * reflection * 0.24
            } else {
                0.0
            };
            let opacity = self.energy * (reflection * facing * 1.12 + sheen);
            mesh.colored_vertex(point, ink.color(opacity, self.energy * reflection * 0.22));
            mesh.colored_vertex(
                self.shape.project(point - normal * width * 0.46),
                ink.color(opacity * 0.28, self.energy * reflection * 0.68),
            );
            mesh.colored_vertex(
                self.shape.project(point - normal * width),
                Color32::TRANSPARENT,
            );
        };
        for (index, (center, radius)) in arcs_and_centers.iter().copied().enumerate() {
            let start = (index as f32 - 1.0) * std::f32::consts::FRAC_PI_2;
            for step in 0..arcs {
                let angle = start + step as f32 / arcs as f32 * std::f32::consts::FRAC_PI_2;
                let (sin, cos) = angle.sin_cos();
                let normal = vec2(cos, sin);
                push(center + normal * radius, normal);
            }
            let normal = [Vec2::X, Vec2::Y, -Vec2::X, -Vec2::Y][index];
            let (next, next_radius) = arcs_and_centers[(index + 1) % 4];
            let from = center + normal * radius;
            let to = next + normal * next_radius;
            let steps = ((from.distance(to) * side_density).ceil() as u32).min(sides);
            for step in 0..steps {
                push(from.lerp(to, step as f32 / steps as f32), normal);
            }
        }
        let rows = (mesh.vertices.len() as u32 - first) / 3;
        for index in 0..rows {
            let a = first + 3 * index;
            let b = first + 3 * ((index + 1) % rows);
            for lane in 0..2 {
                mesh.add_triangle(a + lane, b + lane, a + lane + 1);
                mesh.add_triangle(b + lane, b + lane + 1, a + lane + 1);
            }
        }
    }

    fn paint_particles(&self, mesh: &mut Mesh, count: usize) {
        let progress = self.sample.pulse.clamp(0.0, 1.0);
        let distance = self.style.radius.min(70.0) * progress.sqrt() * 0.72;
        let inertia = self.sample.velocity / self.sample.velocity.length().max(320.0);
        let alpha = self.energy * (1.0 - progress).powi(2) * 0.74;
        if alpha <= 0.001 || distance < 1.0 {
            return;
        }
        for (i, direction) in DIRECTIONS.into_iter().take(count).enumerate() {
            let travel = 0.45 + i as f32 * 0.068;
            let position =
                self.sample.pulse_point + (direction * travel + inertia * 0.26) * distance;
            if !self.shape.contains(position) {
                continue;
            }
            let size = (1.25 + (i % 3) as f32 * 0.25) * (1.0 - progress * 0.57);
            let first = mesh.vertices.len() as u32;
            mesh.colored_vertex(position, self.ink.color(alpha, 0.0));
            for d in DIRECTIONS {
                mesh.colored_vertex(
                    self.shape.project(position + d * size),
                    Color32::TRANSPARENT,
                );
            }
            for i in 0..8 {
                mesh.add_triangle(first, first + 1 + i, first + 1 + ((i + 1) % 8));
            }
        }
    }
}

struct GridAxis {
    points: [f32; 25],
    length: usize,
}

impl GridAxis {
    fn new(cells: u32, follow: Option<(f32, f32)>, pulse: Option<(f32, f32)>) -> Self {
        let mut axis = Self {
            points: [0.0; 25],
            length: cells as usize + 1,
        };
        for (index, point) in axis.points[..axis.length].iter_mut().enumerate() {
            *point = index as f32 / cells as f32;
        }
        for (source, reach) in [follow, pulse].into_iter().flatten() {
            axis.insert(source);
            // On a wide surface, place a sample on each shoulder of a narrow
            // glow so a bright press cannot smear into the next distant cell.
            if reach * 0.65 * (cells as f32) < 1.0 {
                axis.insert(source - reach * 0.45);
                axis.insert(source + reach * 0.45);
            }
        }
        axis.points[..axis.length].sort_unstable_by(f32::total_cmp);
        axis
    }

    fn insert(&mut self, point: f32) {
        if (0.0..1.0).contains(&point) && !self.points().iter().any(|p| (point - p).abs() < 0.0001)
        {
            self.points[self.length] = point;
            self.length += 1;
        }
    }

    fn points(&self) -> &[f32] {
        &self.points[..self.length]
    }
}

fn falloff(distance_sq: f32, softness: f32, attenuation: f32) -> f32 {
    let u = (1.0 - distance_sq).clamp(0.0, 1.0);
    let squared = u * u;
    let t = if attenuation < 2.0 {
        egui::lerp(u..=squared, (attenuation - 0.5) / 1.5)
    } else {
        egui::lerp(squared..=squared * squared, (attenuation - 2.0) * 0.5)
    };
    let softened = t * t * (3.0 - 2.0 * t);
    egui::lerp(t..=softened, softness)
}

const D: f32 = std::f32::consts::FRAC_1_SQRT_2;
const DIRECTIONS: [Vec2; 8] = [
    vec2(1.0, 0.0),
    vec2(D, D),
    vec2(0.0, 1.0),
    vec2(-D, D),
    vec2(-1.0, 0.0),
    vec2(-D, -D),
    vec2(0.0, -1.0),
    vec2(D, -D),
];

#[cfg(test)]
mod tests {
    use super::*;

    fn fixture() -> (Rect, CornerRadiusF32, Vesper, Sample) {
        let rect = Rect::from_min_size(pos2(10.0, 12.0), vec2(150.0, 44.0));
        let corners = CornerRadiusF32 {
            nw: 20.0,
            ne: 13.0,
            sw: 8.0,
            se: 5.0,
        };
        let mut style = Vesper::default();
        style.effects.follow = true;
        style.effects.edge = true;
        style.effects.scatter = true;
        style.effects.flow = true;
        style.effects.particles = true;
        style.effects.depth = true;
        let sample = Sample {
            point: pos2(81.0, 27.0),
            energy: 1.0,
            pulse_point: pos2(70.0, 28.0),
            pulse: 0.42,
            velocity: vec2(150.0, -30.0),
            flow: vec2(6.0, -2.0),
        };
        (rect, corners, style, sample)
    }

    #[test]
    fn mesh_is_bounded_and_rounded_clip_applies_to_all_effects() {
        let (rect, corners, mut style, sample) = fixture();
        let shape = RoundedRect::new(rect, corners);
        let mut count = 0;
        for quality in [
            LightQuality::Low,
            LightQuality::Balanced,
            LightQuality::High,
        ] {
            style.quality = quality;
            let mesh = mesh(rect, corners, &style, test_ink(), sample);
            assert!(mesh.is_valid());
            assert!(mesh.vertices.len() > count && mesh.vertices.len() < 680);
            for vertex in &mesh.vertices {
                assert!(
                    shape.project(vertex.pos).distance_sq(vertex.pos) < 0.000_1,
                    "{:?}",
                    vertex.pos
                );
            }
            count = mesh.vertices.len();
        }
    }

    #[test]
    fn field_reaches_rounded_corners_on_wide_and_tall_surfaces() {
        let (_, _, mut style, sample) = fixture();
        style.effects = crate::LightEffects {
            follow: true,
            ..Default::default()
        };
        for size in [vec2(352.0, 224.0), vec2(76.0, 372.0)] {
            let rect = Rect::from_min_size(pos2(10.0, 12.0), size);
            let corners = CornerRadiusF32 {
                nw: 24.0,
                ne: 18.0,
                sw: 16.0,
                se: 26.0,
            };
            let shape = RoundedRect::new(rect, corners);
            let mesh = mesh(rect, corners, &style, test_ink(), sample);
            for ((center, radius), direction) in shape.arc_centers().into_iter().zip([7, 1, 3, 5]) {
                let probe = center + DIRECTIONS[direction] * (radius - 0.75);
                let covered = mesh.indices.as_chunks::<3>().0.iter().any(|indices| {
                    let [a, b, c] = std::array::from_fn(|i| mesh.vertices[indices[i] as usize].pos);
                    let side = |a: Pos2, b: Pos2, p: Pos2| (p - a).rot90().dot(b - a);
                    side(a, b, c) > 0.0
                        && side(a, b, probe) >= 0.0
                        && side(b, c, probe) >= 0.0
                        && side(c, a, probe) >= 0.0
                });
                assert!(covered, "missing coverage at {probe:?} in {rect:?}");
            }
        }
    }

    #[test]
    fn idle_mesh_does_not_allocate_geometry() {
        let (rect, corners, style, mut sample) = fixture();
        sample.energy = 0.0;
        let mesh = mesh(rect, corners, &style, test_ink(), sample);
        assert_eq!(mesh.vertices.capacity(), 0);
        assert_eq!(mesh.indices.capacity(), 0);
    }

    #[test]
    fn neutral_highlight_adapts_to_the_control_fill_and_preserves_tints() {
        let white = Color32::WHITE;
        let dark = Color32::from_gray(25);
        let on_pale = Ink::new(None, white, dark).color(0.2, 0.1);
        let on_dark = Ink::new(None, dark, white).color(0.2, 0.1);
        assert_eq!(on_dark.r(), on_dark.a());
        assert!(on_pale.a() > on_dark.a());
        assert_eq!(on_pale.r(), on_dark.r());
        assert_eq!(on_pale.r(), on_pale.g());
        assert_eq!(on_pale.g(), on_pale.b());

        let amber = Color32::from_rgb(252, 219, 180);
        assert_eq!(
            Ink::new(Some(amber), white, dark).color(0.2, 0.1),
            amber.gamma_multiply(0.2)
        );
    }

    #[test]
    fn steady_light_has_a_visible_gradient_on_white_and_dark_fills() {
        let (rect, corners, _, mut sample) = fixture();
        sample.point = rect.center() - vec2(27.0, 0.0);
        sample.pulse = 1.0;
        sample.flow = Vec2::ZERO;
        let style = Vesper::subtle();
        let colors = |fill: Color32| {
            mesh(rect, corners, &style, Ink::new(None, fill, fill), sample)
                .vertices
                .into_iter()
                .map(move |vertex| {
                    let color = vertex.color;
                    let composite = u16::from(color.r())
                        + u16::from(255 - color.a()) * u16::from(fill.r()) / 255;
                    (vertex.pos, composite)
                })
        };
        let on_white: Vec<_> = colors(Color32::WHITE).collect();
        let center = on_white
            .iter()
            .min_by(|(a, _), (b, _)| {
                a.distance_sq(sample.point)
                    .total_cmp(&b.distance_sq(sample.point))
            })
            .unwrap()
            .1;
        assert!(on_white.iter().any(|(_, value)| *value <= 220));
        assert!(on_white.iter().any(|(_, value)| *value + 12 <= center));
        assert!(
            on_white
                .iter()
                .any(|(point, value)| point.x >= rect.right() - 3.0 && *value == 255)
        );
        let brightest = colors(Color32::from_gray(24))
            .map(|(_, color)| color)
            .max()
            .unwrap();
        assert!(
            (78..=145).contains(&brightest),
            "brightest dark-surface sample: {brightest}"
        );
    }

    #[test]
    #[ignore = "CPU mesh-generation benchmark; run in release mode"]
    fn measure_mesh_generation() {
        use std::{hint::black_box, time::Instant};

        let (rect, corners, mut style, sample) = fixture();
        style.quality = LightQuality::Balanced;
        let repetitions = 500;
        for count in [1, 32, 128] {
            let start = Instant::now();
            let mut vertices = 0;
            for _ in 0..repetitions {
                for _ in 0..count {
                    vertices += black_box(mesh(rect, corners, &style, test_ink(), sample))
                        .vertices
                        .len();
                }
            }
            println!(
                "{count} widgets: {:.1} µs/batch, {} vertices/batch",
                start.elapsed().as_secs_f64() * 1e6 / repetitions as f64,
                vertices / repetitions,
            );
        }
    }

    fn test_ink() -> Ink {
        Ink::new(None, Color32::WHITE, Color32::WHITE)
    }
}
