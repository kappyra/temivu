//! Responsive navigation surfaces using egui's modal/popover focus and dismissal.
use crate::{Button, ButtonVariant, Dialog, NavigationItem, Popover, Theme};
use egui::{Id, Response, Ui};

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum MenuPresentation {
    /// A left drawer on narrow viewports, a floating island on wider viewports.
    #[default]
    Adaptive,
    FullScreen,
    /// A familiar left-edge navigation drawer with a sliding spring transition.
    Drawer,
    Floating,
}

/// A caller-controlled navigation menu. Call every frame for focus restoration.
/// Reuses the same destinations as bottom navigation; no platform-specific code.
pub struct NavigationMenu<'a> {
    id: Id,
    title: &'a str,
    open: &'a mut bool,
    theme: &'a Theme,
    presentation: MenuPresentation,
    width: f32,
    frame: Option<egui::Frame>,
    liquid: Option<crate::liquid::LiquidStyle>,
}

impl<'a> NavigationMenu<'a> {
    pub fn new(id: Id, title: &'a str, open: &'a mut bool, theme: &'a Theme) -> Self {
        Self {
            id,
            title,
            open,
            theme,
            presentation: MenuPresentation::Adaptive,
            width: 320.0,
            frame: None,
            liquid: None,
        }
    }
    /// Override the context's popup/window animation preset.
    pub fn liquid(mut self, motion: crate::liquid::LiquidStyle) -> Self {
        self.liquid = Some(motion);
        self
    }
    pub fn presentation(mut self, presentation: MenuPresentation) -> Self {
        self.presentation = presentation;
        self
    }
    pub fn width(mut self, width: f32) -> Self {
        if width.is_finite() {
            self.width = width.max(1.0);
        }
        self
    }
    /// Customize colors, border, corners, spacing and shadow with egui's frame.
    pub fn frame(mut self, frame: egui::Frame) -> Self {
        self.frame = Some(frame);
        self
    }
    /// Clicking the trigger toggles the menu. Selecting an enabled destination
    /// updates `selected`, closes the menu and returns the chosen destination.
    pub fn show(
        self,
        anchor: &Response,
        items: &[NavigationItem<'_>],
        selected: &mut Option<Id>,
    ) -> Option<Id> {
        if anchor.clicked() {
            anchor.request_focus();
            *self.open = !*self.open;
        }
        let was_open = *self.open;
        let viewport = anchor.ctx.content_rect();
        let motion = self
            .liquid
            .unwrap_or_else(|| crate::liquid::LiquidStyle::context_default(&anchor.ctx));
        let drawer = self.presentation == MenuPresentation::Drawer
            || (self.presentation == MenuPresentation::Adaptive && viewport.width() < 640.0);
        let full = self.presentation == MenuPresentation::FullScreen || drawer;
        let mut chosen = None;
        if full {
            let frame = self
                .frame
                .unwrap_or_else(|| crate::surface::popup_frame(self.theme, 24).corner_radius(32));
            let mut dialog = Dialog::new(self.id, self.title)
                .theme(self.theme)
                .liquid(motion)
                .source_response(anchor)
                .frame(frame);
            if drawer {
                dialog = dialog.slide_from_left().area(
                    egui::Modal::default_area(self.id)
                        .anchor(egui::Align2::LEFT_TOP, egui::vec2(8.0, 8.0)),
                );
            }
            dialog.show(&anchor.ctx, self.open, |ui| {
                let size = (viewport.size() - egui::Vec2::splat(96.0)).max(egui::Vec2::ZERO);
                ui.set_width(if drawer {
                    (viewport.width() * 0.86 - 48.0).clamp(120.0, 292.0)
                } else {
                    size.x
                });
                ui.set_min_height(size.y);
                menu_contents(
                    ui,
                    self.id,
                    MenuContent {
                        title: self.title,
                        theme: self.theme,
                        items,
                        selected: *selected,
                    },
                    &mut chosen,
                    (size.y - 96.0).max(48.0),
                )
            });
        } else {
            Popover::new(self.id, self.title, self.open)
                .match_compact_trigger()
                .theme(self.theme)
                .liquid(motion)
                .width(self.width)
                .frame(self.frame.unwrap_or_else(|| {
                    crate::surface::popup_frame(self.theme, 16).corner_radius(28)
                }))
                .focus_on_open(true)
                .show(anchor, |ui| {
                    menu_contents(
                        ui,
                        self.id,
                        MenuContent {
                            title: self.title,
                            theme: self.theme,
                            items,
                            selected: *selected,
                        },
                        &mut chosen,
                        (viewport.height() - 160.0).clamp(48.0, 480.0),
                    )
                });
        }
        if was_open && !*self.open {
            anchor.request_focus();
            anchor.ctx.request_repaint();
        }
        if let Some(id) = chosen {
            *selected = Some(id);
            *self.open = false;
            anchor.request_focus();
            anchor.ctx.request_repaint();
        }
        chosen
    }
}

struct MenuContent<'a> {
    title: &'a str,
    theme: &'a Theme,
    items: &'a [NavigationItem<'a>],
    selected: Option<Id>,
}

fn menu_contents(
    ui: &mut Ui,
    id: Id,
    view: MenuContent<'_>,
    chosen: &mut Option<Id>,
    max_height: f32,
) -> Response {
    let MenuContent {
        title,
        theme,
        items,
        selected,
    } = view;
    let close = ui
        .horizontal(|ui| {
            ui.label(
                egui::RichText::new(title)
                    .size(20.0)
                    .strong()
                    .color(theme.foreground),
            );
            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                let close = Button::new("Close", theme)
                    .variant(ButtonVariant::Ghost)
                    .show(ui);
                if close.clicked() {
                    ui.close();
                }
                close
            })
            .inner
        })
        .inner;
    ui.add_space(8.0);
    ui.label(egui::RichText::new("Find your next destination.").color(theme.muted_foreground));
    ui.add_space(20.0);
    let mut responses = Vec::with_capacity(items.len());
    let mut direction = None;
    // Give a floating Area its natural bounded list height; otherwise a
    // ScrollArea can keep the Area at its initial height and clip the last row.
    let list_height =
        (items.len() as f32 * 52.0 + items.len().saturating_sub(1) as f32 * 8.0).min(max_height);
    ui.allocate_ui_with_layout(
        egui::vec2(ui.available_width(), list_height),
        egui::Layout::top_down(egui::Align::Min),
        |ui| {
            egui::ScrollArea::vertical()
                .id_salt(id.with("destinations"))
                .max_height(max_height)
                .show(ui, |ui| {
                    ui.spacing_mut().item_spacing.y = 8.0;
                    for item in items {
                        let active = selected == Some(item.id);
                        let text_color = if active {
                            theme.primary_foreground
                        } else {
                            theme.foreground
                        };
                        let response = ui
                            .push_id(item.id, |ui| {
                                crate::button::press_native(ui, theme, |ui| {
                                    ui.add_enabled(
                                        item.enabled,
                                        egui::Button::new({
                                            let mut label = egui::text::LayoutJob::default();
                                            label.append(
                                                item.label,
                                                if item.icon.is_some() { 40.0 } else { 8.0 },
                                                egui::TextFormat {
                                                    font_id: egui::FontId::proportional(16.0),
                                                    color: text_color,
                                                    ..Default::default()
                                                },
                                            );
                                            label
                                        })
                                        .min_size(egui::vec2(ui.available_width(), 52.0))
                                        .truncate()
                                        .fill(if active {
                                            theme.primary
                                        } else {
                                            theme.secondary
                                        })
                                        .stroke(egui::Stroke::NONE)
                                        .corner_radius(16),
                                    )
                                })
                            })
                            .inner;
                        if let Some(icon) = item.icon {
                            icon(
                                ui.painter(),
                                egui::Rect::from_center_size(
                                    egui::pos2(
                                        response.rect.left() + 24.0,
                                        response.rect.center().y,
                                    ),
                                    egui::Vec2::splat(18.0),
                                ),
                                if item.enabled {
                                    text_color
                                } else {
                                    theme.muted_foreground
                                },
                            );
                        }
                        ui.ctx().accesskit_node_builder(response.id, |node| {
                            node.set_role(egui::accesskit::Role::MenuItem);
                            node.set_label(item.label);
                            node.set_selected(active);
                        });
                        if response.clicked() {
                            *chosen = Some(item.id);
                            ui.close();
                        }
                        if response.has_focus() {
                            ui.memory_mut(|memory| {
                                memory.set_focus_lock_filter(
                                    response.id,
                                    egui::EventFilter {
                                        vertical_arrows: true,
                                        ..Default::default()
                                    },
                                )
                            });
                            direction = ui.input(|input| {
                                if input.key_pressed(egui::Key::ArrowDown) {
                                    Some((responses.len(), 1_isize))
                                } else if input.key_pressed(egui::Key::ArrowUp) {
                                    Some((responses.len(), -1_isize))
                                } else {
                                    None
                                }
                            });
                        }
                        responses.push((response, item.enabled));
                    }
                });
        },
    );
    if let Some((from, delta)) = direction {
        for step in 1..=responses.len() {
            let index = (from as isize + step as isize * delta).rem_euclid(responses.len() as isize)
                as usize;
            if responses[index].1 {
                responses[index].0.request_focus();
                break;
            }
        }
    }
    responses
        .into_iter()
        .find(|(_, enabled)| *enabled)
        .map_or(close, |(response, _)| response)
}
