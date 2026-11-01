//! Forui checkbox port. Error semantics, custom icons and animations are not
//! implemented; sizes, radius, colors and label typography follow the upstream style.
use crate::Theme;
use egui::{Color32, FontId, Response, Stroke, TextStyle, Ui};

/// A controlled checkbox. The caller owns its value across frames.
pub struct Checkbox<'a> {
    value: &'a mut bool,
    theme: &'a Theme,
    label: &'a str,
    enabled: bool,
    touch: bool,
    error: bool,
}

impl<'a> Checkbox<'a> {
    pub fn new(value: &'a mut bool, theme: &'a Theme, label: &'a str) -> Self {
        Self {
            value,
            theme,
            label,
            enabled: true,
            touch: false,
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

    /// Renders the control and returns egui's response, including `changed()`.
    pub fn show(self, ui: &mut Ui) -> Response {
        let size = if self.touch { 20.0 } else { 16.0 };
        let selected = *self.value;
        let fill = if selected {
            if self.error {
                self.theme.error
            } else {
                self.theme.primary
            }
        } else {
            self.theme.card
        };
        let border = if selected {
            Color32::TRANSPARENT
        } else if self.error {
            self.theme.error
        } else {
            self.theme.muted_foreground
        };
        let mark = if self.error {
            self.theme.error_foreground
        } else {
            self.theme.primary_foreground
        };
        let response = ui
            .scope(|ui| {
                ui.spacing_mut().icon_width = size;
                ui.spacing_mut().icon_width_inner = size - 2.0;
                // Forui FLabel trailing padding is 8; the label uses body.sm.
                ui.spacing_mut().icon_spacing = 8.0;
                ui.style_mut().text_styles.insert(
                    TextStyle::Body,
                    FontId::proportional(self.theme.typography.body.size),
                );
                ui.visuals_mut().override_text_color = Some(self.theme.foreground);
                let widgets = &mut ui.visuals_mut().widgets;
                for visual in [
                    &mut widgets.inactive,
                    &mut widgets.hovered,
                    &mut widgets.active,
                    &mut widgets.noninteractive,
                ] {
                    visual.bg_fill = fill;
                    visual.bg_stroke = Stroke::new(if selected { 0.0 } else { 0.6 }, border);
                    visual.fg_stroke.color = mark;
                    visual.corner_radius = 4.into();
                }
                ui.add_enabled(self.enabled, egui::Checkbox::new(self.value, self.label))
            })
            .inner;
        if response.has_focus() {
            ui.painter().rect_stroke(
                response.rect.expand(2.0),
                4,
                Stroke::new(2.0, self.theme.foreground),
                egui::StrokeKind::Outside,
            );
        }
        response
    }
}
