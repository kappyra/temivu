//! Preliminary breadcrumb navigation. Collapsed menus, custom content and
//! upstream visual parity are not yet implemented.
use crate::Theme;
use egui::{Response, RichText, Ui};

/// A text crumb. `interactive` only reports activation; callers own routing.
#[derive(Clone, Copy, Debug)]
pub struct BreadcrumbItem<'a> {
    pub label: &'a str,
    pub current: bool,
    pub interactive: bool,
}

impl<'a> BreadcrumbItem<'a> {
    pub fn link(label: &'a str) -> Self {
        Self {
            label,
            current: false,
            interactive: true,
        }
    }

    pub fn current(label: &'a str) -> Self {
        Self {
            label,
            current: true,
            interactive: false,
        }
    }
}

/// Horizontal breadcrumb with a text separator and explicit theme.
pub struct Breadcrumb<'a> {
    items: &'a [BreadcrumbItem<'a>],
    theme: &'a Theme,
    separator: &'a str,
}

impl<'a> Breadcrumb<'a> {
    pub fn new(items: &'a [BreadcrumbItem<'a>], theme: &'a Theme) -> Self {
        Self {
            items,
            theme,
            separator: "/",
        }
    }

    pub fn separator(mut self, separator: &'a str) -> Self {
        self.separator = separator;
        self
    }

    /// Returns the activated item index. No navigation is performed by the widget.
    pub fn show(self, ui: &mut Ui) -> Option<usize> {
        let mut activated = None;
        ui.horizontal(|ui| {
            // Forui breadcrumb items carry 6 points of horizontal padding each,
            // so adjacent labels are 12 points apart. Touch items also get 10
            // points of vertical padding.
            ui.spacing_mut().item_spacing.x = 12.0;
            ui.spacing_mut().button_padding =
                egui::vec2(0.0, if self.theme.touch { 10.0 } else { 0.0 });
            for (index, item) in self.items.iter().enumerate() {
                if index != 0 {
                    ui.label(
                        RichText::new(self.separator)
                            .size(self.theme.typography.body.size)
                            .color(self.theme.muted_foreground),
                    );
                }
                let color = if item.current {
                    self.theme.foreground
                } else {
                    self.theme.muted_foreground
                };
                let text = RichText::new(item.label)
                    .size(self.theme.typography.body.size)
                    .color(color);
                let response: Response = if item.interactive {
                    let response = ui.add(egui::Button::new(text).frame(false));
                    ui.ctx().accesskit_node_builder(response.id, |node| {
                        node.set_role(egui::accesskit::Role::Link);
                        if item.current {
                            node.set_selected(true);
                        }
                    });
                    response
                } else {
                    let response = ui.label(text);
                    if item.current {
                        ui.ctx()
                            .accesskit_node_builder(response.id, |node| node.set_selected(true));
                    }
                    response
                };
                if response.clicked() && item.interactive {
                    activated = Some(index);
                }
            }
        });
        activated
    }
}
