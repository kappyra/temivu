//! Experimental text field. Form integration and upstream styling are incomplete.
use crate::{SurfaceShape, Theme, Vesper};
use egui::{Response, RichText, Ui};

/// Result of editing and evaluating the optional synchronous validator.
pub struct TextFieldOutput {
    pub response: Response,
    pub error: Option<String>,
}

/// Synchronous validator returning an optional error message.
pub type Validator<'a> = dyn Fn(&str) -> Option<String> + 'a;

/// A caller-owned string with a visible label and optional validation.
pub struct TextField<'a> {
    value: &'a mut String,
    theme: &'a Theme,
    label: &'a str,
    hint: &'a str,
    description: Option<&'a str>,
    validator: Option<&'a Validator<'a>>,
    enabled: bool,
    read_only: bool,
    password: bool,
    multiline: bool,
    max_chars: Option<usize>,
    id: Option<egui::Id>,
    padding: egui::Margin,
    alignment: Option<egui::Align2>,
    radius: Option<u8>,
    min_height: Option<f32>,
    fill: Option<egui::Color32>,
    border: Option<egui::Stroke>,
    focus_stroke: Option<egui::Stroke>,
    error_stroke: Option<egui::Stroke>,
    lighting: Option<Option<Vesper>>,
}
impl<'a> TextField<'a> {
    pub fn new(value: &'a mut String, theme: &'a Theme, label: &'a str) -> Self {
        Self {
            value,
            theme,
            label,
            hint: "",
            description: None,
            validator: None,
            enabled: true,
            read_only: false,
            password: false,
            multiline: false,
            max_chars: None,
            id: None,
            padding: egui::Margin::symmetric(14, 8),
            alignment: None,
            radius: None,
            min_height: None,
            fill: None,
            border: None,
            focus_stroke: None,
            error_stroke: None,
            lighting: None,
        }
    }
    pub fn hint(mut self, hint: &'a str) -> Self {
        self.hint = hint;
        self
    }
    pub fn description(mut self, description: &'a str) -> Self {
        self.description = Some(description);
        self
    }
    pub fn validator(mut self, validator: &'a Validator<'a>) -> Self {
        self.validator = Some(validator);
        self
    }
    pub fn enabled(mut self, enabled: bool) -> Self {
        self.enabled = enabled;
        self
    }
    pub fn read_only(mut self, read_only: bool) -> Self {
        self.read_only = read_only;
        self
    }
    pub fn password(mut self, password: bool) -> Self {
        self.password = password;
        self
    }
    pub fn multiline(mut self, multiline: bool) -> Self {
        self.multiline = multiline;
        self
    }
    /// Limit input by Unicode scalar values; this is not a grapheme-cluster limit.
    pub fn max_chars(mut self, count: usize) -> Self {
        self.max_chars = Some(count);
        self
    }
    /// Give the editor a stable, globally unique ID across layout changes.
    pub fn id(mut self, id: egui::Id) -> Self {
        self.id = Some(id);
        self
    }
    /// Override the editable area's inner spacing.
    pub fn padding(mut self, padding: egui::Margin) -> Self {
        self.padding = padding;
        self
    }

    /// Align text and placeholder within the editable area.
    /// Single-line fields default to left-center and multiline fields to left-top.
    pub fn alignment(mut self, alignment: egui::Align2) -> Self {
        self.alignment = Some(alignment);
        self
    }

    /// Override the radius of the editable surface.
    pub fn radius(mut self, radius: u8) -> Self {
        self.radius = Some(radius);
        self
    }

    /// Override the field's minimum height in logical points.
    pub fn min_height(mut self, height: f32) -> Self {
        self.min_height = Some(height);
        self
    }

    /// Override the background color of the editable surface.
    pub fn fill(mut self, fill: egui::Color32) -> Self {
        self.fill = Some(fill);
        self
    }

    /// Override the resting outline. Use `Stroke::NONE` to remove it.
    pub fn border(mut self, border: egui::Stroke) -> Self {
        self.border = Some(border);
        self
    }

    /// Override the outline's color and width while focused.
    pub fn focus_stroke(mut self, stroke: egui::Stroke) -> Self {
        self.focus_stroke = Some(stroke);
        self
    }

    /// Override the outline's color and width while validation fails.
    pub fn error_stroke(mut self, stroke: egui::Stroke) -> Self {
        self.error_stroke = Some(stroke);
        self
    }

    /// Override inherited lighting. Pass `None` to disable it for this field.
    pub fn lighting(mut self, lighting: Option<Vesper>) -> Self {
        self.lighting = Some(lighting);
        self
    }

    pub fn show(self, ui: &mut Ui) -> TextFieldOutput {
        ui.vertical(|ui| {
            let label = ui.label(self.label);
            let shape = ui.painter().add(egui::Shape::Noop);
            let frame = egui::Frame::new().inner_margin(self.padding);
            let mut view = self.value.as_str();
            let buffer: &mut dyn egui::TextBuffer = if self.read_only {
                &mut view
            } else {
                self.value
            };
            let alignment = self.alignment.unwrap_or(if self.multiline {
                egui::Align2::LEFT_TOP
            } else {
                egui::Align2::LEFT_CENTER
            });
            let edit = make_edit(
                buffer,
                self.multiline,
                self.hint,
                self.password,
                self.max_chars,
                self.id,
            )
            .horizontal_align(alignment.x())
            .vertical_align(alignment.y())
            .frame(frame)
            .min_size(egui::vec2(
                0.0,
                self.min_height
                    .filter(|height| height.is_finite() && *height >= 0.0)
                    .unwrap_or(self.theme.sizes.field.lg),
            ))
            .desired_width(ui.available_width());
            let response = ui.add_enabled(self.enabled, edit).labelled_by(label.id);
            let error = self.validator.and_then(|validator| validator(self.value));
            let focused = crate::motion::spring_bool(
                ui.ctx(),
                response.id.with("field-focus"),
                response.has_focus(),
                self.theme,
            )
            .clamp(0.0, 1.0);
            let invalid = crate::motion::animate_bool(
                ui.ctx(),
                response.id.with("field-error"),
                error.is_some(),
                self.theme,
            );
            if ui.is_rect_visible(response.rect) {
                let radius = self.radius.unwrap_or_else(|| {
                    if self.multiline {
                        SurfaceShape::Card
                    } else {
                        SurfaceShape::Field
                    }
                    .radius(response.rect.size())
                });
                let fill = self.fill.unwrap_or(if self.read_only {
                    egui::Color32::TRANSPARENT
                } else {
                    self.theme.card
                });
                let resting = self
                    .border
                    .unwrap_or(egui::Stroke::new(1.0, self.theme.border));
                let focus = self
                    .focus_stroke
                    .unwrap_or(egui::Stroke::new(2.0, self.theme.foreground));
                let error_stroke = self
                    .error_stroke
                    .unwrap_or(egui::Stroke::new(2.0, self.theme.error));
                let mut stroke =
                    mix_stroke(mix_stroke(resting, focus, focused), error_stroke, invalid);
                let opacity = if !self.enabled && ui.is_enabled() {
                    self.theme.disabled_opacity
                } else {
                    1.0
                };
                stroke.color = stroke.color.gamma_multiply(opacity);
                let fill = fill.gamma_multiply(opacity);
                let light = self
                    .lighting
                    .unwrap_or_else(|| Vesper::context(ui.ctx()))
                    .and_then(|style| {
                        style.interactive_shape_on(
                            ui.painter(),
                            self.theme,
                            &response,
                            response.rect,
                            egui::CornerRadius::same(radius).into(),
                            fill,
                        )
                    });
                // Reserve the paint slot before laying out the editor so both the
                // material and its single outline remain behind text and selection.
                let shape_group = match light {
                    Some(light) => egui::Shape::Vec(vec![
                        egui::Shape::rect_filled(response.rect, radius, fill),
                        light,
                        egui::Shape::rect_stroke(
                            response.rect,
                            radius,
                            stroke,
                            egui::StrokeKind::Inside,
                        ),
                    ]),
                    None => egui::epaint::RectShape::new(
                        response.rect,
                        radius,
                        fill,
                        stroke,
                        egui::StrokeKind::Inside,
                    )
                    .into(),
                };
                ui.painter().set(shape, shape_group);
                if response.enabled()
                    && let Some(style) = self.lighting.unwrap_or_else(|| Vesper::context(ui.ctx()))
                {
                    crate::lighting::spill::receive_background_at(
                        ui.painter(),
                        response.id,
                        response.rect,
                        egui::CornerRadius::same(radius).into(),
                        &style,
                        fill,
                        self.theme.card,
                        shape,
                    );
                }
            }
            let detail = match (self.description, error.as_deref()) {
                (Some(description), Some(error)) => Some(format!("{description}\n{error}")),
                (Some(description), None) => Some(description.to_owned()),
                (None, Some(error)) => Some(error.to_owned()),
                (None, None) => None,
            };
            ui.ctx().accesskit_node_builder(response.id, |node| {
                if let Some(detail) = detail {
                    node.set_description(detail);
                }
                if error.is_some() {
                    node.set_invalid(egui::accesskit::Invalid::True);
                }
            });
            if let Some(description) = self.description {
                ui.label(RichText::new(description).color(self.theme.muted_foreground));
            }
            if let Some(error) = error.as_deref() {
                ui.label(RichText::new(error).color(self.theme.error));
            }
            TextFieldOutput { response, error }
        })
        .inner
    }
}

fn mix_stroke(from: egui::Stroke, to: egui::Stroke, amount: f32) -> egui::Stroke {
    if amount <= 0.0 {
        return from;
    }
    if amount >= 1.0 {
        return to;
    }
    egui::Stroke::new(
        egui::lerp(from.width..=to.width, amount),
        egui::Color32::from(
            egui::Rgba::from(from.color) * (1.0 - amount) + egui::Rgba::from(to.color) * amount,
        ),
    )
}

/// Style the internal single-line editors that need egui's own edit options.
pub(crate) fn show_native(
    ui: &mut Ui,
    theme: &Theme,
    enabled: bool,
    edit: egui::TextEdit<'_>,
) -> Response {
    ui.scope(|ui| {
        let height = theme.sizes.field.lg;
        let radius = SurfaceShape::Field.radius(egui::Vec2::splat(height));
        let visuals = ui.visuals_mut();
        visuals.selection.stroke = egui::Stroke::new(1.0, theme.foreground);
        visuals.text_edit_bg_color = Some(theme.card);
        for state in [
            &mut visuals.widgets.inactive,
            &mut visuals.widgets.hovered,
            &mut visuals.widgets.active,
            &mut visuals.widgets.noninteractive,
        ] {
            state.corner_radius = radius.into();
            state.bg_stroke = egui::Stroke::new(1.0, theme.border);
            state.expansion = 0.0;
        }
        ui.add_enabled(
            enabled,
            edit.margin(egui::Margin::symmetric(14, 8))
                .vertical_align(egui::Align::Center)
                .min_size(egui::vec2(0.0, height)),
        )
    })
    .inner
}

fn make_edit<'a>(
    buffer: &'a mut dyn egui::TextBuffer,
    multiline: bool,
    hint: &'a str,
    password: bool,
    max_chars: Option<usize>,
    id: Option<egui::Id>,
) -> egui::TextEdit<'a> {
    let mut edit = if multiline {
        egui::TextEdit::multiline(buffer)
    } else {
        egui::TextEdit::singleline(buffer)
    };
    edit = edit.hint_text(hint).password(password);
    if let Some(id) = id {
        edit = edit.id(id);
    }
    if let Some(max_chars) = max_chars {
        edit = edit.char_limit(max_chars);
    }
    edit
}
