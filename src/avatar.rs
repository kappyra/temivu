//! Experimental avatar; loading, image errors and transitions remain caller-managed.
use crate::Theme;
use egui::{
    Align2, Color32, FontId, Mesh, Pos2, Rect, Response, Sense, Shape, Stroke, TextureId, Ui, Vec2,
    WidgetInfo, WidgetType,
};

/// A square avatar with circular clipping and optional initials or a loaded texture.
pub struct Avatar<'a> {
    theme: &'a Theme,
    size: f32,
    label: &'a str,
    fallback: Option<&'a str>,
    image: Option<(TextureId, Vec2)>,
}
impl<'a> Avatar<'a> {
    pub fn new(theme: &'a Theme) -> Self {
        Self {
            theme,
            size: 40.0,
            label: "Avatar",
            fallback: None,
            image: None,
        }
    }
    pub fn label(mut self, label: &'a str) -> Self {
        self.label = label;
        self
    }
    pub fn fallback(mut self, initials: &'a str) -> Self {
        self.fallback = Some(initials);
        self
    }
    pub fn size(mut self, size: f32) -> Result<Self, &'static str> {
        if !size.is_finite() || size <= 0.0 {
            return Err("avatar size must be positive and finite");
        }
        self.size = size;
        Ok(self)
    }
    /// Draw a preloaded texture using a centered cover crop. The caller handles loading errors.
    pub fn image(mut self, texture: TextureId, dimensions: Vec2) -> Result<Self, &'static str> {
        if !dimensions.x.is_finite()
            || !dimensions.y.is_finite()
            || dimensions.x <= 0.0
            || dimensions.y <= 0.0
        {
            return Err("image dimensions must be positive and finite");
        }
        self.image = Some((texture, dimensions));
        Ok(self)
    }
    pub fn show(self, ui: &mut Ui) -> Response {
        let (rect, response) = ui.allocate_exact_size(Vec2::splat(self.size), Sense::hover());
        response
            .widget_info(|| WidgetInfo::labeled(WidgetType::Other, ui.is_enabled(), self.label));
        ui.ctx().accesskit_node_builder(response.id, |node| {
            node.set_role(egui::accesskit::Role::Image)
        });
        if !ui.is_rect_visible(rect) {
            return response;
        }
        let painter = ui.painter();
        let radius = self.size / 2.0;
        painter.circle_filled(rect.center(), radius, self.theme.muted);
        if let Some((texture, dimensions)) = self.image {
            let aspect = dimensions.x / dimensions.y;
            let uv_size = if aspect > 1.0 {
                Vec2::new(1.0 / aspect, 1.0)
            } else {
                Vec2::new(1.0, aspect)
            };
            let uv_rect = Rect::from_center_size(Pos2::new(0.5, 0.5), uv_size);
            let mut mesh = Mesh::with_texture(texture);
            mesh.vertices.push(egui::epaint::Vertex {
                pos: rect.center(),
                uv: uv_rect.center(),
                color: Color32::WHITE,
            });
            const STEPS: u32 = 64;
            for i in 0..=STEPS {
                let angle = (i as f32 / STEPS as f32) * std::f32::consts::TAU;
                let direction = Vec2::angled(angle);
                mesh.vertices.push(egui::epaint::Vertex {
                    pos: rect.center() + direction * radius,
                    uv: uv_rect.center() + direction * (uv_size / 2.0),
                    color: Color32::WHITE,
                });
                if i > 0 {
                    mesh.add_triangle(0, i, i + 1);
                }
            }
            painter.add(Shape::mesh(mesh));
        } else if let Some(initials) = self.fallback {
            let base = self.size * 0.35;
            let galley = painter.layout_no_wrap(
                initials.to_owned(),
                FontId::proportional(base),
                self.theme.muted_foreground,
            );
            let factor = (self.size * 0.75 / galley.size().x.max(1.0)).min(1.0);
            painter.text(
                rect.center(),
                Align2::CENTER_CENTER,
                initials,
                FontId::proportional(base * factor),
                self.theme.muted_foreground,
            );
        } else {
            // A neutral user silhouette; not the upstream Lucide asset.
            let center = rect.center();
            let color = self.theme.muted_foreground;
            painter.circle_stroke(
                center - Vec2::new(0.0, radius * 0.23),
                radius * 0.22,
                Stroke::new(1.5, color),
            );
            painter.add(Shape::line(
                vec![
                    center + Vec2::new(-radius * 0.4, radius * 0.45),
                    center + Vec2::new(-radius * 0.3, radius * 0.2),
                    center + Vec2::new(radius * 0.3, radius * 0.2),
                    center + Vec2::new(radius * 0.4, radius * 0.45),
                ],
                Stroke::new(1.5, color),
            ));
        }
        response
    }
}
