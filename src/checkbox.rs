//! Rounded, animated checkbox with caller-owned state and keyboard semantics.
use crate::Theme;
use egui::{FontId, Response, Sense, Stroke, Ui, WidgetInfo, WidgetType};

/// A controlled checkbox. The caller owns its value across frames.
pub struct Checkbox<'a> {
    value: &'a mut bool,
    theme: &'a Theme,
    label: &'a str,
    enabled: bool,
    touch: bool,
    error: bool,
    radius: u8,
}

impl<'a> Checkbox<'a> {
    pub fn new(value: &'a mut bool, theme: &'a Theme, label: &'a str) -> Self {
        Self {
            value,
            theme,
            label,
            enabled: true,
            touch: theme.touch,
            error: false,
            radius: 6,
        }
    }
    pub fn enabled(mut self, enabled: bool) -> Self {
        self.enabled = enabled;
        self
    }
    pub fn touch(mut self, touch: bool) -> Self {
        self.touch = touch;
        self
    }
    pub fn error(mut self, error: bool) -> Self {
        self.error = error;
        self
    }
    /// Set the checkbox box and focus-ring radius.
    pub fn radius(mut self, radius: u8) -> Self {
        self.radius = radius;
        self
    }

    /// Renders the control and returns egui's response, including `changed()`.
    pub fn show(self, ui: &mut Ui) -> Response {
        let ui_enabled = ui.is_enabled();
        let enabled = self.enabled && ui_enabled;
        let size = if self.touch { 22.0 } else { 20.0 };
        let label = ui.painter().layout(
            self.label.to_owned(),
            FontId::proportional(self.theme.typography.body.size),
            self.theme.foreground,
            (ui.available_width() - size - 12.0).max(1.0),
        );
        let (rect, mut response) = ui.allocate_exact_size(
            egui::vec2(size + 12.0 + label.size().x, 44.0_f32.max(label.size().y)),
            if enabled {
                Sense::click()
            } else {
                Sense::hover()
            },
        );
        if enabled && response.clicked() {
            *self.value = !*self.value;
            response.mark_changed();
        }
        response.widget_info(|| {
            WidgetInfo::selected(WidgetType::Checkbox, enabled, *self.value, self.label)
        });
        if self.error {
            ui.ctx().accesskit_node_builder(response.id, |node| {
                node.set_invalid(egui::accesskit::Invalid::True)
            });
        }
        if ui.is_rect_visible(rect) {
            let amount = crate::motion::animate_bool(
                ui.ctx(),
                response.id.with("check"),
                *self.value,
                self.theme,
            );
            let color = if self.error {
                self.theme.error
            } else {
                self.theme.primary
            };
            let mix = |a: egui::Color32, b: egui::Color32| {
                let result = egui::Color32::from(
                    egui::Rgba::from(a) * (1.0 - amount) + egui::Rgba::from(b) * amount,
                );
                if !self.enabled && ui_enabled {
                    self.theme.disable(result)
                } else {
                    result
                }
            };
            let box_rect = egui::Rect::from_center_size(
                egui::pos2(rect.left() + size / 2.0, rect.center().y),
                egui::Vec2::splat(size),
            );
            ui.painter().rect(
                box_rect,
                self.radius,
                mix(self.theme.card, color),
                Stroke::new(1.5, mix(self.theme.muted_foreground, color)),
                egui::StrokeKind::Inside,
            );
            if amount > 0.0 {
                let ink = if self.error {
                    self.theme.error_foreground
                } else {
                    self.theme.primary_foreground
                };
                let ink = if !self.enabled && ui_enabled {
                    self.theme.disable(ink)
                } else {
                    ink
                };
                let points = [
                    box_rect.min + egui::vec2(size * 0.25, size * 0.50),
                    box_rect.min + egui::vec2(size * 0.43, size * 0.68),
                    box_rect.min + egui::vec2(size * 0.76, size * 0.32),
                ];
                let clip = egui::Rect::from_min_max(
                    box_rect.min,
                    egui::pos2(box_rect.left() + size * amount, box_rect.bottom()),
                );
                ui.painter()
                    .with_clip_rect(clip.intersect(ui.clip_rect()))
                    .add(egui::Shape::line(points.to_vec(), Stroke::new(2.0, ink)));
            }
            if response.has_focus() {
                ui.painter().rect_stroke(
                    box_rect.expand(3.0),
                    self.radius.saturating_add(2),
                    Stroke::new(2.0, self.theme.foreground),
                    egui::StrokeKind::Outside,
                );
            }
            ui.painter().galley(
                egui::pos2(
                    box_rect.right() + 12.0,
                    rect.center().y - label.size().y / 2.0,
                ),
                label,
                if enabled {
                    self.theme.foreground
                } else {
                    self.theme.muted_foreground
                },
            );
        }
        response
    }
}
