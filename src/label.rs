//! Preliminary form-field label layout. Error animation and full parity remain open.
use crate::Theme;
use egui::{Color32, Response, RichText, Ui};

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum LabelLayout {
    HorizontalLeading,
    HorizontalTrailing,
    #[default]
    Vertical,
}

/// Groups text with an interactive field. The field callback must return its
/// response so that the visible label can be associated with it for accesskit.
pub struct FieldLabel<'a> {
    theme: &'a Theme,
    layout: LabelLayout,
    label: Option<&'a str>,
    description: Option<&'a str>,
    error: Option<&'a str>,
}

impl<'a> FieldLabel<'a> {
    pub fn new(theme: &'a Theme, layout: LabelLayout) -> Self {
        Self {
            theme,
            layout,
            label: None,
            description: None,
            error: None,
        }
    }
    pub fn label(mut self, label: &'a str) -> Self {
        self.label = Some(label);
        self
    }
    pub fn description(mut self, description: &'a str) -> Self {
        self.description = Some(description);
        self
    }
    /// Show a validation error. Supply `None` by omitting this call when valid.
    pub fn error(mut self, error: &'a str) -> Self {
        self.error = Some(error);
        self
    }

    pub fn show(self, ui: &mut Ui, field: impl FnOnce(&mut Ui) -> Response) -> Response {
        if self.label.is_none() && self.description.is_none() && self.error.is_none() {
            return field(ui);
        }
        let response = match self.layout {
            LabelLayout::Vertical => {
                ui.vertical(|ui| {
                    let label = self
                        .label
                        .map(|text| ui.label(RichText::new(text).color(self.theme.foreground)));
                    if label.is_some() {
                        ui.add_space(6.0);
                    }
                    let response = field(ui);
                    if self.description.is_some() || self.error.is_some() {
                        ui.add_space(6.0);
                    }
                    supporting_text(
                        ui,
                        self.description,
                        self.error,
                        self.theme.muted_foreground,
                        self.theme.error,
                    );
                    attach_label(response, label)
                })
                .inner
            }
            LabelLayout::HorizontalLeading => {
                ui.horizontal(|ui| {
                    let label = ui
                        .vertical(|ui| {
                            let label = self.label.map(|text| {
                                ui.label(RichText::new(text).color(self.theme.foreground))
                            });
                            supporting_text(
                                ui,
                                self.description,
                                self.error,
                                self.theme.muted_foreground,
                                self.theme.error,
                            );
                            label
                        })
                        .inner;
                    ui.add_space(12.0);
                    attach_label(field(ui), label)
                })
                .inner
            }
            LabelLayout::HorizontalTrailing => {
                ui.horizontal(|ui| {
                    let response = field(ui);
                    ui.add_space(8.0);
                    let label = ui
                        .vertical(|ui| {
                            let label = self.label.map(|text| {
                                ui.label(RichText::new(text).color(self.theme.foreground))
                            });
                            supporting_text(
                                ui,
                                self.description,
                                self.error,
                                self.theme.muted_foreground,
                                self.theme.error,
                            );
                            label
                        })
                        .inner;
                    attach_label(response, label)
                })
                .inner
            }
        };
        response.ctx.accesskit_node_builder(response.id, |node| {
            let mut descriptions = Vec::new();
            if let Some(existing) = node.description() {
                descriptions.push(existing.to_owned());
            }
            if let Some(description) = self.description {
                descriptions.push(description.to_owned());
            }
            if let Some(error) = self.error {
                descriptions.push(error.to_owned());
                node.set_invalid(egui::accesskit::Invalid::True);
            }
            if !descriptions.is_empty() {
                node.set_description(descriptions.join(". "));
            }
        });
        response
    }
}

fn attach_label(response: Response, label: Option<Response>) -> Response {
    if let Some(label) = label {
        response.labelled_by(label.id)
    } else {
        response
    }
}

fn supporting_text(
    ui: &mut Ui,
    description: Option<&str>,
    error: Option<&str>,
    muted: Color32,
    danger: Color32,
) {
    if let Some(text) = description {
        ui.label(RichText::new(text).color(muted));
    }
    if let Some(text) = error {
        ui.label(RichText::new(text).color(danger));
    }
}
