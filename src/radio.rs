//! Forui radio control port. Group coordination and animation are not yet
//! implemented; each caller owns its boolean value across frames.
use crate::Theme;
use egui::{FontId, Response, RichText, Stroke, TextStyle, Ui};

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
        let mut response = ui
            .scope(|ui| {
                ui.spacing_mut().icon_width = size;
                ui.spacing_mut().icon_width_inner = indicator_size;
                ui.spacing_mut().icon_spacing = 8.0;
                ui.style_mut().text_styles.insert(
                    TextStyle::Body,
                    FontId::proportional(self.theme.typography.body.size),
                );
                let widgets = &mut ui.visuals_mut().widgets;
                for visual in [
                    &mut widgets.inactive,
                    &mut widgets.hovered,
                    &mut widgets.active,
                    &mut widgets.noninteractive,
                ] {
                    visual.bg_fill = self.theme.card;
                    visual.bg_stroke = Stroke::new(1.0, border);
                    visual.fg_stroke.color = indicator;
                }
                let text = RichText::new(self.label).color(self.theme.foreground);
                ui.add_enabled(self.enabled, egui::RadioButton::new(*self.value, text))
            })
            .inner;
        if response.clicked() {
            *self.value = !*self.value;
            response.mark_changed();
        }
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
