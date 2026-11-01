//! Experimental single-value Forui slider subset. Range sliders are in
//! `RANGE_SLIDER.md`; custom style overrides and upstream visual parity remain
//! open.
use crate::Theme;
use egui::{Response, Ui};

/// A tick and/or label at a normalized position on a slider's track.
///
/// This approximates Forui's `FSliderMark`: labels are painted below a
/// horizontal slider, while a vertical slider only paints the ticks.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct SliderMark<'a> {
    value: f64,
    tick: bool,
    label: &'a str,
}

impl<'a> SliderMark<'a> {
    pub fn new(value: f64) -> Result<Self, &'static str> {
        if !value.is_finite() || !(0.0..=1.0).contains(&value) {
            return Err("slider mark value must be between zero and one");
        }
        Ok(Self {
            value,
            tick: true,
            label: "",
        })
    }
    pub fn tick(mut self, tick: bool) -> Self {
        self.tick = tick;
        self
    }
    pub fn label(mut self, label: &'a str) -> Self {
        self.label = label;
        self
    }
    pub fn value(&self) -> f64 {
        self.value
    }
}

/// A normalized single-value slider. Values are owned by the caller.
pub struct Slider<'a> {
    value: &'a mut f64,
    theme: &'a Theme,
    label: &'a str,
    enabled: bool,
    vertical: bool,
    step: Option<f64>,
    marks: &'a [SliderMark<'a>],
}
impl<'a> Slider<'a> {
    pub fn new(value: &'a mut f64, theme: &'a Theme) -> Result<Self, &'static str> {
        if !value.is_finite() || !(0.0..=1.0).contains(value) {
            return Err("slider value must be between zero and one");
        }
        Ok(Self {
            value,
            theme,
            label: "Slider",
            enabled: true,
            vertical: false,
            step: None,
            marks: &[],
        })
    }
    pub fn label(mut self, label: &'a str) -> Self {
        self.label = label;
        self
    }
    pub fn enabled(mut self, enabled: bool) -> Self {
        self.enabled = enabled;
        self
    }
    pub fn vertical(mut self, vertical: bool) -> Self {
        self.vertical = vertical;
        self
    }
    /// Draw ticks and labels at normalized positions along the track.
    ///
    /// Horizontal sliders paint tick dots and (when present) a caption label
    /// below the track; vertical sliders only paint ticks. Mark labels are
    /// painted, not exposed as separate accessibility nodes.
    pub fn marks(mut self, marks: &'a [SliderMark<'a>]) -> Self {
        self.marks = marks;
        self
    }
    /// Constrain values to uniform discrete intervals. Does not support arbitrary marks.
    pub fn step(mut self, step: f64) -> Result<Self, &'static str> {
        if !step.is_finite() || !(0.0 < step && step <= 1.0) {
            return Err("step must be positive and at most one");
        }
        self.step = Some(step);
        Ok(self)
    }
    pub fn show(self, ui: &mut Ui) -> Response {
        ui.scope(|ui| {
            ui.visuals_mut().selection.bg_fill = self.theme.primary;
            ui.visuals_mut().widgets.inactive.bg_fill = self.theme.secondary;
            let effective_enabled = self.enabled && ui.is_enabled();
            // Paint the label above the slider so the slider's own response rect
            // contains only the track, which keeps mark positions rail-precise.
            if !self.label.is_empty() {
                let label_size = self.theme.typography.body.size;
                let label_height = label_size + 4.0;
                let top_left = ui.cursor().min;
                ui.add_space(label_height);
                let color = if effective_enabled {
                    self.theme.foreground
                } else {
                    self.theme
                        .disable_blend(self.theme.foreground, self.theme.background)
                };
                ui.painter().text(
                    top_left,
                    egui::Align2::LEFT_TOP,
                    self.label,
                    egui::FontId::proportional(label_size),
                    color,
                );
            }
            let mut scratch = *self.value;
            let target = if effective_enabled {
                self.value
            } else {
                &mut scratch
            };
            let mut slider = egui::Slider::new(target, 0.0..=1.0).show_value(false);
            if self.vertical {
                slider = slider.vertical();
            }
            if let Some(step) = self.step {
                slider = slider.step_by(step);
            }
            let mut response = ui.add_enabled(self.enabled, slider);
            if !self.label.is_empty() {
                ui.ctx().accesskit_node_builder(response.id, |node| {
                    node.set_label(self.label);
                });
            }
            if !self.marks.is_empty() {
                Self::paint_marks(
                    ui,
                    &response,
                    self.marks,
                    self.theme,
                    self.vertical,
                    effective_enabled,
                );
            }
            // egui adds adjustment actions even for disabled sliders; hide them from AT.
            if !effective_enabled {
                response.flags.remove(egui::response::Flags::CHANGED);
                ui.ctx()
                    .accesskit_node_builder(response.id, |node| node.clear_actions());
            }
            response
        })
        .inner
    }

    fn paint_marks(
        ui: &mut Ui,
        response: &Response,
        marks: &[SliderMark<'_>],
        theme: &Theme,
        vertical: bool,
        enabled: bool,
    ) {
        let rect = response.rect;
        // Mirror egui's own rail geometry: the position range is the response
        // range shrunk by the handle radius scaled by the style's handle shape
        // (the default rectangular handle shrinks by `radius * aspect_ratio`),
        // and vertical positions are flipped so the maximum is at the top.
        let handle_shape = ui.style().visuals.handle_shape;
        let radius = |extent: f32| extent / 2.5;
        let inset = |extent: f32| match handle_shape {
            egui::style::HandleShape::Circle => radius(extent),
            egui::style::HandleShape::Rect { aspect_ratio } => radius(extent) * aspect_ratio,
        };
        let position = |value: f64| -> egui::Pos2 {
            if vertical {
                let range = rect.y_range().shrink(inset(rect.width())).flip();
                egui::pos2(
                    rect.center().x,
                    egui::lerp(range.min..=range.max, value as f32),
                )
            } else {
                let range = rect.x_range().shrink(inset(rect.height()));
                egui::pos2(
                    egui::lerp(range.min..=range.max, value as f32),
                    rect.center().y,
                )
            }
        };
        let tick_color = if enabled && ui.is_enabled() {
            theme.muted_foreground
        } else {
            theme.disable_blend(theme.muted_foreground, theme.background)
        };
        let label_color = tick_color;
        let mut needs_label_space = false;
        for mark in marks {
            let center = position(mark.value);
            if mark.tick {
                ui.painter().circle_filled(center, 1.5, tick_color);
            }
            if !mark.label.is_empty() && !vertical {
                needs_label_space = true;
                ui.painter().text(
                    egui::pos2(center.x, rect.bottom() + 4.0),
                    egui::Align2::CENTER_TOP,
                    mark.label,
                    egui::FontId::proportional(theme.typography.caption.size),
                    label_color,
                );
            }
        }
        if needs_label_space {
            ui.add_space(theme.typography.caption.size + 4.0);
        }
    }
}
