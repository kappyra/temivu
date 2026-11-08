//! Optional optical material for floating surfaces and app-owned backdrops.
use crate::Theme;
use egui::{
    Color32, Context, Frame, Id, Painter, Pos2, Rect, Shape, Stroke, StrokeKind, TextureId, Vec2,
    epaint::{CornerRadiusF32, Mesh, Vertex, WHITE_UV},
};
use std::sync::Arc;

/// Appearance of floating glass. Tint ranges from clear to opaque.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Glass {
    tint: f32,
    mode: GlassMode,
}

/// Optical treatment of the live backdrop.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum GlassMode {
    /// Glassmorphism with diffuse blur, a matte tint and a soft rim.
    Frost,
    /// Optical glass with a clearer core and a refracting bevel.
    #[default]
    Prism,
}

/// The type of surface whose background is being sampled.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum GlassProfile {
    #[default]
    Floating,
    /// Wider vertical diffusion for controls along a scroll edge.
    Dock,
}

/// Paint parameters passed to an application's graphics backend.
#[derive(Clone, Copy, Debug)]
pub struct GlassRegion {
    pub rect: Rect,
    pub corners: CornerRadiusF32,
    pub mode: GlassMode,
    pub profile: GlassProfile,
    pub tint: f32,
    pub strength: f32,
}

/// A renderer adapter that samples content already painted beneath a surface.
pub trait GlassRenderer: Send + Sync {
    /// Whether live sampling is available on the current device.
    fn available(&self) -> bool {
        true
    }

    fn callback(&self, region: GlassRegion) -> egui::PaintCallback;
}

/// An app-owned, softly filtered texture in the same coordinates as the surface.
/// Paint the source artwork behind the surface at these bounds.
#[derive(Clone, Copy, Debug)]
pub struct GlassBackdrop {
    pub texture: TextureId,
    pub bounds: Rect,
}

impl Default for Glass {
    fn default() -> Self {
        Self {
            tint: 0.4,
            mode: GlassMode::Prism,
        }
    }
}

impl Glass {
    /// Use a tint between zero and one. Out-of-range values are clamped.
    pub fn new(tint: f32) -> Self {
        if tint.is_finite() {
            Self {
                tint: tint.clamp(0.0, 1.0),
                ..Self::default()
            }
        } else {
            Self::default()
        }
    }

    pub fn mode(mut self, mode: GlassMode) -> Self {
        self.mode = mode;
        self
    }

    /// Install or remove a graphics backend for live blur and refraction.
    /// Install once per context. The backend owns its graphics resources.
    pub fn set_renderer(ctx: &Context, renderer: Option<Arc<dyn GlassRenderer>>) {
        let id = Id::new("temivu-glass-renderer");
        ctx.data_mut(|data| {
            if let Some(renderer) = renderer {
                data.insert_temp(id, renderer);
            } else {
                data.remove::<Arc<dyn GlassRenderer>>(id);
            }
        });
        ctx.request_repaint();
    }

    fn renderer(ctx: &Context) -> Option<Arc<dyn GlassRenderer>> {
        ctx.data(|data| data.get_temp::<Arc<dyn GlassRenderer>>(Id::new("temivu-glass-renderer")))
            .filter(|renderer| renderer.available())
    }

    /// Enable glass on floating Temivu surfaces in this context.
    /// Pass `None` to restore opaque surfaces.
    pub fn set_context(ctx: &Context, glass: Option<Self>) {
        if Self::context(ctx) == glass {
            return;
        }
        ctx.data_mut(|data| {
            let id = Id::new("temivu-glass-material");
            if let Some(glass) = glass {
                data.insert_temp(id, glass);
            } else {
                data.remove::<Self>(id);
            }
        });
        ctx.request_repaint();
    }

    pub fn context(ctx: &Context) -> Option<Self> {
        ctx.data(|data| data.get_temp(Id::new("temivu-glass-material")))
    }

    /// Paint a glass surface at its outer bounds. Without a supplied texture,
    /// regular alpha blending shows the underlying UI through the material.
    pub fn paint(
        self,
        painter: &Painter,
        theme: &Theme,
        rect: Rect,
        frame: Frame,
        backdrop: Option<GlassBackdrop>,
    ) {
        if !rect.is_finite() || !rect.is_positive() || !painter.is_visible() {
            return;
        }
        let has_backdrop =
            backdrop.is_some_and(|b| b.bounds.is_finite() && b.bounds.contains_rect(rect));
        let live = Self::renderer(painter.ctx());
        let frame = self.tinted_frame(theme, frame, has_backdrop || live.is_some());
        let corners = frame.corner_radius.into();
        if let Some(renderer) = &live {
            painter.add(self.live_shape(
                renderer.as_ref(),
                rect,
                corners,
                1.0,
                GlassProfile::Floating,
            ));
        }
        if frame.shadow.color != Color32::TRANSPARENT {
            painter.add(frame.shadow.as_shape(rect, frame.corner_radius));
        }
        if let Some(backdrop) =
            backdrop.filter(|_| live.is_none() && has_backdrop && frame.fill.a() < 255)
        {
            let mut mesh = Mesh::with_texture(backdrop.texture);
            let band = (rect.size().min_elem() * 0.2).min(12.0);
            let bend = match self.mode {
                GlassMode::Frost => 0.0,
                GlassMode::Prism => band * (1.0 - self.tint) * 0.65,
            };
            let feather = (band * 0.25).min(1.0);
            for (inset, offset, alpha) in [
                (0.0, bend, 0),
                (feather, bend, 255),
                (band * 0.46, bend * 0.8, 255),
                (band, 0.0, 255),
            ] {
                for (position, normal) in outline(rect, corners, inset) {
                    let sample = position + normal * offset;
                    let uv = (sample - backdrop.bounds.min) / backdrop.bounds.size();
                    mesh.vertices.push(Vertex {
                        pos: position,
                        uv: Pos2::new(uv.x.clamp(0.0, 1.0), uv.y.clamp(0.0, 1.0)),
                        color: Color32::from_white_alpha(alpha),
                    });
                }
            }
            let core = 3 * RING;
            for ring in 0..3 {
                connect_rings(&mut mesh, ring * RING);
            }
            for i in 1..RING - 1 {
                mesh.add_triangle(core, core + i, core + i + 1);
            }
            painter.add(Shape::mesh(mesh));
        }
        painter.rect(
            rect,
            frame.corner_radius,
            frame.fill,
            frame.stroke,
            StrokeKind::Inside,
        );
        if live.is_none() {
            painter.add(self.rim(theme, rect, corners, 0.8));
        }
    }

    fn live_shape(
        self,
        renderer: &dyn GlassRenderer,
        rect: Rect,
        corners: CornerRadiusF32,
        strength: f32,
        profile: GlassProfile,
    ) -> Shape {
        if !rect.is_finite()
            || rect.size().min_elem() < 4.0
            || strength <= 0.001
            || self.tint >= 0.999
        {
            return Shape::Noop;
        }
        Shape::Callback(renderer.callback(GlassRegion {
            rect,
            corners,
            mode: self.mode,
            profile,
            tint: self.tint,
            strength: strength.clamp(0.0, 1.0),
        }))
    }

    fn tinted_frame(self, theme: &Theme, mut frame: Frame, has_backdrop: bool) -> Frame {
        if frame.fill == Color32::TRANSPARENT {
            return frame;
        }
        let [r, g, b, alpha] = frame.fill.to_srgba_unmultiplied();
        let minimum = match (self.mode, has_backdrop) {
            (GlassMode::Frost, true) => 0.70,
            (GlassMode::Prism, true) => 0.47,
            (GlassMode::Frost, false) => 0.91,
            (GlassMode::Prism, false) => 0.85,
        };
        let opacity = egui::lerp(minimum..=1.0, self.tint);
        frame.fill =
            Color32::from_rgba_unmultiplied(r, g, b, (f32::from(alpha) * opacity).round() as u8);
        if frame.stroke.width > 0.0 {
            let edge = match (self.mode, theme.dark) {
                (GlassMode::Frost, true) => Color32::from_white_alpha(32),
                (GlassMode::Prism, true) => Color32::from_white_alpha(49),
                (GlassMode::Frost, false) => Color32::from_black_alpha(21),
                (GlassMode::Prism, false) => Color32::from_black_alpha(30),
            };
            frame.stroke = Stroke::new(frame.stroke.width, edge);
        }
        let shadow = match self.mode {
            GlassMode::Frost => 0.55,
            GlassMode::Prism => 0.65,
        };
        frame.shadow.color = frame.shadow.color.gamma_multiply(shadow);
        frame
    }

    pub(crate) fn rim(
        self,
        theme: &Theme,
        rect: Rect,
        corners: CornerRadiusF32,
        polish: f32,
    ) -> Shape {
        if !rect.is_finite() || !rect.is_positive() {
            return Shape::Noop;
        }
        let outer = 1.0_f32.min(rect.size().min_elem() * 0.15);
        let inner = (outer + 2.5).min(rect.size().min_elem() * 0.4);
        let mut mesh = Mesh::default();
        let strength = match (self.mode, theme.dark) {
            (GlassMode::Frost, _) => 44.0,
            (GlassMode::Prism, true) => 105.0,
            (GlassMode::Prism, false) => 176.0,
        };
        for (inset, weight) in [(outer, 1.0), (inner, 0.0)] {
            for (position, normal) in outline(rect, corners, inset) {
                let sheen = (-normal.y * 0.82 - normal.x * 0.45).max(0.0);
                let alpha =
                    (sheen * weight * strength * polish * (1.0 - self.tint * 0.55)).round() as u8;
                mesh.vertices.push(Vertex {
                    pos: position,
                    uv: WHITE_UV,
                    color: Color32::from_white_alpha(alpha),
                });
            }
        }
        connect_rings(&mut mesh, 0);
        Shape::mesh(mesh)
    }
}

// A fixed contour grid keeps the lens cost independent of the surface area.
const QUADRANT: [Vec2; 9] = [
    Vec2::new(0.0, -1.0),
    Vec2::new(0.19509, -0.980785),
    Vec2::new(0.382683, -0.92388),
    Vec2::new(0.55557, -0.83147),
    Vec2::new(
        std::f32::consts::FRAC_1_SQRT_2,
        -std::f32::consts::FRAC_1_SQRT_2,
    ),
    Vec2::new(0.83147, -0.55557),
    Vec2::new(0.92388, -0.382683),
    Vec2::new(0.980785, -0.19509),
    Vec2::new(1.0, 0.0),
];
const RING: u32 = 4 * QUADRANT.len() as u32;

fn outline(rect: Rect, radius: CornerRadiusF32, inset: f32) -> impl Iterator<Item = (Pos2, Vec2)> {
    let inner = rect.shrink(inset);
    let max_radius = inner.size().min_elem() * 0.5;
    let rounded = |r: f32| (r - inset).clamp(0.0, max_radius);
    [
        (inner.right_top(), Vec2::new(-1.0, 1.0), rounded(radius.ne)),
        (
            inner.right_bottom(),
            Vec2::new(-1.0, -1.0),
            rounded(radius.se),
        ),
        (
            inner.left_bottom(),
            Vec2::new(1.0, -1.0),
            rounded(radius.sw),
        ),
        (inner.left_top(), Vec2::new(1.0, 1.0), rounded(radius.nw)),
    ]
    .into_iter()
    .enumerate()
    .flat_map(move |(corner, (point, inward, radius))| {
        let center = point + inward * radius;
        QUADRANT.into_iter().map(move |point| {
            let normal = match corner {
                0 => point,
                1 => Vec2::new(-point.y, point.x),
                2 => -point,
                _ => Vec2::new(point.y, -point.x),
            };
            (center + normal * radius, normal)
        })
    })
}

fn connect_rings(mesh: &mut Mesh, start: u32) {
    for i in 0..RING {
        let next = (i + 1) % RING;
        mesh.add_triangle(start + i, start + next, start + RING + i);
        mesh.add_triangle(start + next, start + RING + next, start + RING + i);
    }
}

pub(crate) fn floating_frame(ctx: &Context, theme: &Theme, frame: Frame) -> Frame {
    Glass::context(ctx).map_or(frame, |glass| {
        glass.tinted_frame(theme, frame, Glass::renderer(ctx).is_some())
    })
}

pub(crate) fn live_enabled(ctx: &Context) -> bool {
    Glass::context(ctx).is_some_and(|glass| glass.tint < 0.999) && Glass::renderer(ctx).is_some()
}

/// Keeps the base material intact until a live backdrop can take over.
pub(crate) struct GlassTransition {
    enabled: bool,
    live: Option<(Glass, Arc<dyn GlassRenderer>)>,
    mix: f32,
}

impl GlassTransition {
    pub fn new(ctx: &Context, phase: f32, eligible: bool) -> Self {
        let glass = if eligible { Glass::context(ctx) } else { None };
        let t = ((phase - 0.82) / 0.18).clamp(0.0, 1.0);
        let mix = t * t * (3.0 - 2.0 * t);
        let live = glass
            .filter(|glass| glass.tint < 0.999 && mix > 0.001)
            .and_then(|glass| Glass::renderer(ctx).map(|renderer| (glass, renderer)));
        Self {
            enabled: glass.is_some(),
            live,
            mix,
        }
    }

    pub fn enabled(&self) -> bool {
        self.enabled
    }

    pub fn has_backdrop(&self) -> bool {
        self.live.is_some()
    }

    pub fn frame(&self, theme: &Theme, mut frame: Frame) -> Frame {
        if let Some((glass, _)) = &self.live {
            let material = glass.tinted_frame(theme, frame, true);
            frame.fill = frame.fill.lerp_to_gamma(material.fill, self.mix);
            frame.stroke.color = frame
                .stroke
                .color
                .lerp_to_gamma(material.stroke.color, self.mix);
            frame.shadow.color = frame
                .shadow
                .color
                .lerp_to_gamma(material.shadow.color, self.mix);
        }
        frame
    }

    pub fn seed_opacity(&self, phase: f32) -> f32 {
        let t = (phase / if self.enabled { 0.055 } else { 0.17 }).clamp(0.0, 1.0);
        t * t * (3.0 - 2.0 * t)
    }

    pub fn with_backdrop(&self, shape: Shape, rect: Rect, corners: CornerRadiusF32) -> Shape {
        if let Some((glass, renderer)) = &self.live {
            // Blend the surface tint over fully filtered pixels. Fading the
            // callback itself would mix raw text back into the glass.
            let backdrop = glass.live_shape(
                renderer.as_ref(),
                rect,
                corners,
                1.0,
                GlassProfile::Floating,
            );
            if !matches!(backdrop, Shape::Noop) {
                return Shape::Vec(vec![backdrop, shape]);
            }
        }
        shape
    }
}

pub(crate) fn show_frame<R>(
    ui: &mut egui::Ui,
    frame: Frame,
    profile: Option<GlassProfile>,
    add: impl FnOnce(&mut egui::Ui) -> R,
) -> egui::InnerResponse<R> {
    let Some(profile) = profile.filter(|_| ui.is_visible()) else {
        return frame.show(ui, add);
    };
    let Some(glass) = Glass::context(ui.ctx()).filter(|glass| glass.tint < 0.999) else {
        return frame.show(ui, add);
    };
    let Some(renderer) = Glass::renderer(ui.ctx()) else {
        return frame.show(ui, add);
    };
    let slot = ui.painter().add(Shape::Noop);
    let response = frame.show(ui, add);
    let rect = response.response.rect - frame.outer_margin;
    ui.painter().set(
        slot,
        glass.live_shape(
            renderer.as_ref(),
            rect,
            frame.corner_radius.into(),
            1.0,
            profile,
        ),
    );
    response
}
