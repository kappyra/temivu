//! Preliminary grouping container for select fields.
//!
//! A [`SelectGroup`] draws a shared label, description and validation message
//! around caller-supplied fields, exposes the group role and propagates its
//! disabled and validation state to the fields drawn inside it. Form
//! integration, rich labels and upstream visual parity remain open; see
//! `COVERAGE.md` for the current limitations.
use crate::Theme;
use egui::{Id, Response, RichText, Ui, UiBuilder};

/// The egui temporary-data key under which a [`SelectGroup`] publishes its
/// validation message while it draws its content.
fn inherited_error_key() -> Id {
    Id::unique("temivu-select-group-inherited-error")
}

/// The validation message of the enclosing [`SelectGroup`], if any.
///
/// Field widgets call this while they are drawn inside a group and mark
/// themselves invalid. Callers can use it for their own fields too; the value
/// is only valid during that group's content callback.
pub fn inherited_select_error(ui: &Ui) -> Option<String> {
    ui.data(|data| {
        data.get_temp::<Option<String>>(inherited_error_key())
            .flatten()
    })
}

/// The result of showing a [`SelectGroup`].
pub struct SelectGroupResponse<R> {
    /// The group container's response.
    pub response: Response,
    /// The content callback's return value.
    pub inner: R,
    /// Whether the group is enabled (its own flag and the inherited UI state).
    pub enabled: bool,
    /// Whether the group shows a validation error.
    pub invalid: bool,
}

/// A labelled, optionally disabled and invalid group of select fields.
///
/// The group is a layout and accessibility container: draw one or more
/// [`crate::Select`] fields (or any other fields) inside the content callback.
/// While the callback runs:
/// - child widgets are disabled when the group is disabled, through egui's
///   inherited enabled state;
/// - [`inherited_select_error`] returns the group's validation message, and
///   [`crate::Select`] marks itself invalid and exposes that message as its
///   description.
///
/// The group node itself is exposed with the group role, its label and its
/// description; screen-reader verification remains open.
///
/// ```
/// # fn example(ui: &mut egui::Ui, theme: &temivu::Theme, color: &mut Option<egui::Id>) {
/// use temivu::{Select, SelectGroup, SelectOption};
/// let options = [SelectOption::new(egui::Id::unique("red"), "Red")];
/// let group = SelectGroup::new(egui::Id::unique("shipping"), "Shipping color", theme)
///     .description("Used on the label")
///     .show(ui, |ui| {
///         Select::new(
///             egui::Id::unique("shipping-color"),
///             "Color",
///             "Choose a color",
///             &options,
///             color,
///         )
///         .show(ui)
///     });
/// assert!(group.enabled);
/// # }
/// ```
pub struct SelectGroup<'a> {
    id: Id,
    label: &'a str,
    theme: &'a Theme,
    description: Option<&'a str>,
    error: Option<&'a str>,
    enabled: bool,
}

impl<'a> SelectGroup<'a> {
    /// Creates a group with a stable ID and a visible, accessible label.
    pub fn new(id: Id, label: &'a str, theme: &'a Theme) -> Self {
        Self {
            id,
            label,
            theme,
            description: None,
            error: None,
            enabled: true,
        }
    }

    /// Supporting text shown below the label and exposed as the group's
    /// description.
    pub fn description(mut self, description: &'a str) -> Self {
        self.description = Some(description);
        self
    }

    /// Show a validation error. The group is marked invalid and the error text
    /// is shown below the content, added to the group's accessible description
    /// and published to child fields through [`inherited_select_error`].
    pub fn error(mut self, error: &'a str) -> Self {
        self.error = Some(error);
        self
    }

    pub fn enabled(mut self, enabled: bool) -> Self {
        self.enabled = enabled;
        self
    }

    /// Draws the group around `content`. Call it every frame.
    pub fn show<R>(
        self,
        ui: &mut Ui,
        content: impl FnOnce(&mut Ui) -> R,
    ) -> SelectGroupResponse<R> {
        let Self {
            id,
            label,
            theme,
            description,
            error,
            enabled,
        } = self;
        let enabled = enabled && ui.is_enabled();
        let mut label_response = None;
        // Publish the validation message for child fields, restoring the
        // previous value afterwards so sibling groups do not leak state.
        let previous_error = ui.data_mut(|data| {
            let previous = data
                .get_temp::<Option<String>>(inherited_error_key())
                .flatten();
            data.insert_temp(inherited_error_key(), error.map(str::to_owned));
            previous
        });
        let scope = ui.scope_builder(UiBuilder::new().scope_id(id), |ui| {
            ui.vertical(|ui| {
                ui.spacing_mut().item_spacing.y = 6.0;
                let label = ui.add(
                    egui::Label::new(
                        RichText::new(label)
                            .color(theme.foreground)
                            .strong()
                            .size(theme.typography.label.size),
                    )
                    .truncate(),
                );
                label_response = Some(label);
                if let Some(description) = description {
                    ui.add(
                        egui::Label::new(
                            RichText::new(description)
                                .color(theme.muted_foreground)
                                .size(theme.typography.caption.size),
                        )
                        .truncate(),
                    );
                }
                let content_response = ui.add_enabled_ui(enabled, content);
                if let Some(error) = error {
                    ui.add(
                        egui::Label::new(
                            RichText::new(error)
                                .color(theme.error)
                                .size(theme.typography.caption.size),
                        )
                        .wrap(),
                    );
                }
                content_response.inner
            })
            .inner
        });
        ui.data_mut(|data| match previous_error {
            Some(previous) => {
                data.insert_temp(inherited_error_key(), Some(previous));
            }
            None => data.remove::<Option<String>>(inherited_error_key()),
        });
        let inner = scope.inner;
        let response = scope.response;
        if let Some(label_response) = label_response {
            ui.ctx().accesskit_node_builder(response.id, |node| {
                node.push_labelled_by(label_response.id.accesskit_id());
            });
        }
        ui.ctx().accesskit_node_builder(response.id, |node| {
            node.set_role(egui::accesskit::Role::Group);
            node.set_label(label);
            let mut parts = Vec::new();
            if let Some(description) = description {
                parts.push(description.to_owned());
            }
            if let Some(error) = error {
                parts.push(error.to_owned());
            }
            if !parts.is_empty() {
                node.set_description(parts.join(". "));
            }
            if error.is_some() {
                node.set_invalid(egui::accesskit::Invalid::True);
            }
            if !enabled {
                node.set_disabled();
            }
            node.set_bounds(egui::accesskit::Rect {
                x0: f64::from(response.rect.min.x),
                y0: f64::from(response.rect.min.y),
                x1: f64::from(response.rect.max.x),
                y1: f64::from(response.rect.max.y),
            });
        });
        SelectGroupResponse {
            response,
            inner,
            enabled,
            invalid: error.is_some(),
        }
    }
}
