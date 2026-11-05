//! Experimental text field. Form integration and upstream styling are incomplete.
use crate::Theme;
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
            padding: egui::Margin::symmetric(10, 8),
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

    pub fn show(self, ui: &mut Ui) -> TextFieldOutput {
        ui.vertical(|ui| {
            let label = ui.label(self.label);
            let frame = egui::Frame::new()
                .inner_margin(self.padding)
                .fill(if self.read_only {
                    egui::Color32::TRANSPARENT
                } else {
                    self.theme.card
                })
                .stroke(egui::Stroke::new(1.0, self.theme.border))
                .corner_radius(14);
            let response = if self.read_only {
                let mut view = self.value.as_str();
                let edit = make_edit(
                    &mut view,
                    self.multiline,
                    self.hint,
                    self.password,
                    self.max_chars,
                    self.id,
                );
                ui.add_enabled(
                    self.enabled,
                    edit.frame(frame).desired_width(ui.available_width()),
                )
            } else {
                let edit = make_edit(
                    self.value,
                    self.multiline,
                    self.hint,
                    self.password,
                    self.max_chars,
                    self.id,
                );
                ui.add_enabled(
                    self.enabled,
                    edit.frame(frame).desired_width(ui.available_width()),
                )
            }
            .labelled_by(label.id);
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
            let emphasis = focused.max(invalid);
            if emphasis > 0.001 {
                let color = egui::Color32::from(
                    egui::Rgba::from(self.theme.foreground) * (1.0 - invalid)
                        + egui::Rgba::from(self.theme.error) * invalid,
                );
                ui.painter().rect_stroke(
                    response.rect.expand(0.8 + 1.8 * focused),
                    13,
                    egui::Stroke::new(1.0 + 0.6 * focused, color.gamma_multiply(emphasis)),
                    egui::StrokeKind::Outside,
                );
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
