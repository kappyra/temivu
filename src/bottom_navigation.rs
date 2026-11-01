//! Preliminary bottom navigation; icons, localized position semantics and parity remain open.
use crate::Theme;
use egui::{Id, RichText, Ui};

/// A navigation destination with a stable ID unique within its navigation row.
#[derive(Clone, Copy, Debug)]
pub struct NavigationItem<'a> {
    pub id: Id,
    pub label: &'a str,
    pub enabled: bool,
}
impl<'a> NavigationItem<'a> {
    pub fn new(id: Id, label: &'a str) -> Self {
        Self {
            id,
            label,
            enabled: true,
        }
    }
    pub fn enabled(mut self, enabled: bool) -> Self {
        self.enabled = enabled;
        self
    }
}

/// Draws a navigation row and returns the newly selected ID, if it changed.
/// The group ID must be globally unique and stable; item IDs must be unique in the group.
pub fn bottom_navigation(
    ui: &mut Ui,
    id: Id,
    items: &[NavigationItem<'_>],
    selected: &mut Option<Id>,
    theme: &Theme,
) -> Option<Id> {
    if items.is_empty() {
        return None;
    }
    let spacing = ui.spacing().item_spacing.x;
    let available = ui.available_width().max(0.0);
    let preferred = ((available - spacing * (items.len().saturating_sub(1) as f32))
        / items.len() as f32)
        .max(48.0);
    let max_width = available.max(48.0);
    let mut changed = None;
    ui.horizontal_wrapped(|ui| {
        for item in items {
            let is_selected = *selected == Some(item.id);
            let font = egui::TextStyle::Button.resolve(ui.style());
            let label_width = ui.fonts_mut(|fonts| {
                fonts
                    .layout_no_wrap(item.label.to_owned(), font, theme.foreground)
                    .size()
                    .x
            });
            let width = (label_width + 2.0 * ui.spacing().button_padding.x)
                .max(preferred)
                .min(max_width);
            let text = RichText::new(item.label).color(if is_selected {
                theme.primary_foreground
            } else {
                theme.foreground
            });
            let mut button = egui::Button::new(text)
                .truncate()
                .selected(is_selected)
                .frame_when_inactive(is_selected)
                .min_size(egui::vec2(width, 48.0));
            if is_selected {
                button = button.fill(theme.primary);
            }
            let response = ui
                .allocate_ui_with_layout(
                    egui::vec2(width, 48.0),
                    egui::Layout::left_to_right(egui::Align::Center),
                    |ui| {
                        ui.scope_builder(egui::UiBuilder::new().id(id.with(item.id)), |ui| {
                            ui.add_enabled(item.enabled, button)
                        })
                        .inner
                    },
                )
                .inner;
            ui.ctx().accesskit_node_builder(response.id, |node| {
                node.set_label(item.label);
                node.clear_toggled();
                node.set_selected(is_selected);
            });
            if response.clicked() && !is_selected {
                *selected = Some(item.id);
                changed = Some(item.id);
            }
        }
    });
    changed
}
