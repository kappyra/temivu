//! A standalone radio control with spring selection feedback.
use crate::Theme;
use egui::{FontId, Response, Role, Stroke, Ui, WidgetInfo, vec2};

pub struct Radio<'a> {
    value: &'a mut bool,
    theme: &'a Theme,
    label: &'a str,
    enabled: bool,
    touch: bool,
    error: bool,
}

impl<'a> Radio<'a> {
    pub fn new(value: &'a mut bool, theme: &'a Theme, label: &'a str) -> Self {
        Self {
            value,
            theme,
            label,
            enabled: true,
            touch: theme.touch,
            error: false,
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

    /// Each activation flips the boolean, matching a standalone FRadio. A
    /// group must enforce any mutual exclusion in its owning application.
    pub fn show(self, ui: &mut Ui) -> Response {
        // Forui: 16 point outer circle with an 8 point indicator on desktop,
        // 20/12 on touch (3 point padding and a 1 point border).
        let size = if self.touch { 20.0 } else { 16.0 };
        let indicator_size = if self.touch { 12.0 } else { 8.0 };
        let indicator = if self.error {
            self.theme.error
        } else {
            self.theme.primary
        };
        let border = if self.error {
            self.theme.error
        } else {
            self.theme.muted_foreground
        };
        let enabled = self.enabled && ui.is_enabled();
        let text = ui.painter().layout(
            self.label.to_owned(),
            FontId::proportional(self.theme.typography.body.size),
            self.theme.foreground,
            (ui.available_width() - size - 32.0).max(1.0),
        );
        let desired = vec2(
            size + text.size().x + 32.0,
            (text.size().y + 14.0).max(44.0),
        );
        let mut response = ui
            .add_enabled_ui(self.enabled, |ui| {
                crate::motion_controls::button_response(ui, desired)
            })
            .inner;
        if response.clicked() {
            *self.value = !*self.value;
            response.mark_changed();
        }
        response.widget_info(|| {
            WidgetInfo::selected(Role::RadioButton, enabled, *self.value, self.label)
        });
        let active = crate::motion::spring_bool(
            ui.ctx(),
            response.id.with("radio-selected"),
            *self.value,
            self.theme,
        );
        let press = crate::motion_controls::press_amount(&response, self.theme).clamp(0.0, 1.0);
        let mix = |from, to, amount: f32| {
            let t = amount.clamp(0.0, 1.0);
            egui::Color32::from(egui::Rgba::from(from) * (1.0 - t) + egui::Rgba::from(to) * t)
        };
        let dim = |color| {
            if enabled {
                color
            } else {
                self.theme.disable(color)
            }
        };
        let rect = response.rect;
        let painter = ui.painter().with_clip_rect(rect.intersect(ui.clip_rect()));
        let fill = mix(self.theme.card, self.theme.secondary, active);
        painter.rect(
            rect,
            16,
            dim(fill),
            Stroke::new(1.0, dim(self.theme.border)),
            egui::StrokeKind::Inside,
        );
        let center = egui::pos2(rect.left() + 12.0 + size / 2.0, rect.center().y);
        painter.circle(
            center,
            size / 2.0,
            dim(self.theme.card),
            Stroke::new(1.0, dim(mix(border, indicator, active))),
        );
        if active > 0.0 {
            painter.circle_filled(
                center,
                indicator_size / 2.0 * active.clamp(0.0, 1.12) * (1.0 - press * 0.12),
                dim(indicator),
            );
        }
        let text_height = text.size().y;
        painter.galley(
            egui::pos2(
                center.x + size / 2.0 + 8.0,
                rect.center().y - text_height / 2.0,
            ),
            text,
            dim(self.theme.foreground),
        );
        if response.has_focus() {
            ui.painter().rect_stroke(
                rect,
                16,
                Stroke::new(2.0, self.theme.primary),
                egui::StrokeKind::Inside,
            );
        }
        response
    }
}
