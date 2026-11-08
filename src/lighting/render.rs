use egui::{
    Color32, Pos2, Rect, Vec2,
    epaint::{CornerRadiusF32, Mesh},
    pos2, vec2,
};

use super::{LightQuality, Sample, Vesper};

/// Build one untextured, bounded paint mesh. All triangles lie within a convex
/// rounded rectangle, so the rectangular egui scissor only has to clip the parent.
pub(super) fn mesh(
    rect: Rect,
    corners: CornerRadiusF32,
    style: &Vesper,
    color: Color32,
    sample: Sample,
) -> Mesh {
    let mut mesh = Mesh::default();
    if !rect.is_finite() || !rect.is_positive() || color.a() == 0 {
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
            side: 8,
            arc: 3,
            particles: 4,
        },
        LightQuality::Balanced => Density {
            grid: [12, 8],
            side: 14,
            arc: 5,
            particles: 6,
        },
        LightQuality::High => Density {
            grid: [18, 12],
            side: 22,
            arc: 8,
            particles: 8,
        },
    };
    let light = Field {
        style,
        color,
        sample,
        energy,
        shape,
    };

    let effects = style.effects;
    let scatter = effects.scatter && sample.pulse < 1.0;
    let field = effects.follow || scatter || effects.flow || effects.depth;
    let edge = effects.edge;
    let particles = effects.particles && sample.pulse < 1.0;
    let [nx, ny] = quality.grid.map(|n| n as usize);
    let sources = usize::from(effects.follow) + usize::from(scatter);
    let edge_pairs = (4 * (quality.side + quality.arc)) as usize;
    let corner_points = 4 * quality.arc as usize;
    // Each vector allocates once, even when all lighting channels overlap.
    mesh.vertices.reserve(
        usize::from(field) * ((nx + 1 + sources) * (ny + 1 + sources) + corner_points)
            + usize::from(edge) * edge_pairs * 2
            + usize::from(particles) * quality.particles * 9,
    );
    mesh.indices.reserve(
        usize::from(field) * ((nx + sources) * (ny + sources) * 6 + corner_points * 3)
            + usize::from(edge) * edge_pairs * 6
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
    color: Color32,
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
        let progress = sample.pulse.clamp(0.0, 1.0);
        let scatter = style.effects.scatter && progress < 1.0;
        let spread = radius * (0.13 + 0.87 * progress.sqrt());
        let pulse_energy = 0.58 * (1.0 - progress).powi(2);
        let light_position = (sample.point - rect.center()) / rect.size();
        let (sin, cos) = if style.effects.flow {
            sample.flow_phase.sin_cos()
        } else {
            (0.0, 1.0)
        };
        let fill = |point: Pos2| {
            let mut light = 0.0;
            if style.effects.follow {
                let distance = point.distance(sample.point) / radius;
                light += 0.64 * falloff(distance, style.softness, style.attenuation);
            }
            if scatter {
                let distance = point.distance(sample.pulse_point) / spread;
                // The envelope expands and softens while losing energy.
                light += pulse_energy * falloff(distance, style.softness, style.attenuation);
            }
            let uv = (point - rect.center()) / rect.size();
            if style.effects.flow {
                let across = uv.y + 0.46 * uv.x * cos + 0.60 * uv.x * uv.x * sin;
                let envelope = (1.0 - (uv.x - light_position.x * 0.55).abs()).clamp(0.0, 1.0);
                light += 0.22 * envelope * falloff((across - 0.17 * sin).abs() * 2.0, 0.8, 1.4);
            }
            if style.effects.depth {
                let relief = (0.50 - uv.y + uv.dot(light_position) * 0.45).clamp(0.0, 1.0);
                light += 0.17 * style.elevation * relief * relief;
            }
            self.color
                .gamma_multiply((light * self.energy).clamp(0.0, 0.58))
        };

        // Reserve a sample at each active source. Otherwise a narrow contact
        // pulse can pass between the fixed vertices of a wide surface.
        let follow = style.effects.follow.then_some(sample.point);
        let pulse = scatter.then_some(sample.pulse_point);
        let x = GridAxis::new(
            nx,
            follow.map(|p| (p.x - rect.left()) / rect.width()),
            pulse.map(|p| (p.x - rect.left()) / rect.width()),
        );
        let y = GridAxis::new(
            ny,
            follow.map(|p| (p.y - rect.top()) / rect.height()),
            pulse.map(|p| (p.y - rect.top()) / rect.height()),
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
        let width = (rect.size().min_elem() * 0.11).clamp(1.0, 4.0);
        let radius = (self.style.radius * 1.12).max(1.0);
        let shift = (self.sample.point - rect.center()).normalized();
        let ink = self.color;
        let mut push = |point: Pos2, normal: Vec2| {
            let toward_light = self.sample.point - point;
            let distance = toward_light.length();
            let angle = (normal.dot(toward_light) / distance.max(0.001)).max(0.0);
            let falloff = falloff(distance / radius, 0.65, 1.25);
            let sheen = if self.style.effects.depth {
                (0.60 - normal.y * 0.37 + normal.dot(shift) * 0.12) * self.style.elevation * 0.26
            } else {
                0.0
            };
            let opacity = self.energy * (falloff * (0.60 + angle * 0.16) + sheen);
            mesh.colored_vertex(point, ink.gamma_multiply(opacity.min(0.76)));
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
            for step in 0..sides {
                push(from.lerp(to, step as f32 / sides as f32), normal);
            }
        }
        let pairs = (sides + arcs) * 4;
        for index in 0..pairs {
            let a = first + 2 * index;
            let b = first + 2 * ((index + 1) % pairs);
            mesh.add_triangle(a, b, a + 1);
            mesh.add_triangle(b, b + 1, a + 1);
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
            mesh.colored_vertex(position, self.color.gamma_multiply(alpha));
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
    points: [f32; 21],
    length: usize,
}

impl GridAxis {
    fn new(cells: u32, follow: Option<f32>, pulse: Option<f32>) -> Self {
        let mut axis = Self {
            points: [0.0; 21],
            length: cells as usize + 1,
        };
        for (index, point) in axis.points[..axis.length].iter_mut().enumerate() {
            *point = index as f32 / cells as f32;
        }
        for source in [follow, pulse].into_iter().flatten() {
            if source.is_finite()
                && (0.0..1.0).contains(&source)
                && !axis.points().iter().any(|p| (source - p).abs() < 0.0001)
            {
                axis.points[axis.length] = source;
                axis.length += 1;
            }
        }
        axis.points[..axis.length].sort_unstable_by(f32::total_cmp);
        axis
    }

    fn points(&self) -> &[f32] {
        &self.points[..self.length]
    }
}

fn falloff(distance: f32, softness: f32, attenuation: f32) -> f32 {
    let t = (1.0 - distance.clamp(0.0, 1.0)).powf(attenuation);
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
            flow_phase: 1.3,
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
            let mesh = mesh(rect, corners, &style, Color32::WHITE, sample);
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
            let mesh = mesh(rect, corners, &style, Color32::WHITE, sample);
            for ((center, radius), direction) in shape.arc_centers().into_iter().zip([7, 1, 3, 5]) {
                let probe = center + DIRECTIONS[direction] * (radius - 0.75);
                let covered = mesh.indices.chunks_exact(3).any(|indices| {
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
        let mesh = mesh(rect, corners, &style, Color32::WHITE, sample);
        assert_eq!(mesh.vertices.capacity(), 0);
        assert_eq!(mesh.indices.capacity(), 0);
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
                    vertices += black_box(mesh(rect, corners, &style, Color32::WHITE, sample))
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
}
