//! Preliminary bottom navigation; icons, localized position semantics and parity remain open.
use crate::Theme;
use egui::{Color32, Id, Painter, Rect, RichText, Ui};

/// Draw an icon inside the supplied rectangle using the current foreground.
pub type NavigationIcon = fn(&Painter, Rect, Color32);

/// Appearance overrides for a navigation row.
#[derive(Clone, Copy, Debug)]
pub struct NavigationStyle {
    pub selected_fill: Color32,
    pub selected_text: Color32,
    pub text: Color32,
    pub radius: u8,
}

impl NavigationStyle {
    pub fn from_theme(theme: &Theme) -> Self {
        Self {
            selected_fill: theme.primary,
            selected_text: theme.primary_foreground,
            text: theme.foreground,
            radius: 255,
        }
    }
}

/// A navigation destination with a stable ID unique within its navigation row.
#[derive(Clone, Copy, Debug)]
pub struct NavigationItem<'a> {
    pub id: Id,
    pub label: &'a str,
    pub enabled: bool,
    pub icon: Option<NavigationIcon>,
}
impl<'a> NavigationItem<'a> {
    pub fn new(id: Id, label: &'a str) -> Self {
        Self {
            id,
            label,
            enabled: true,
            icon: None,
        }
    }
    pub fn icon(mut self, icon: NavigationIcon) -> Self {
        self.icon = Some(icon);
        self
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
    bottom_navigation_styled(ui, id, items, selected, &NavigationStyle::from_theme(theme))
}

/// Navigation with caller-provided colors and corner radius.
pub fn bottom_navigation_styled(
    ui: &mut Ui,
    id: Id,
    items: &[NavigationItem<'_>],
    selected: &mut Option<Id>,
    style: &NavigationStyle,
) -> Option<Id> {
    if items.is_empty() {
        return None;
    }
    let theme = crate::surface::context_theme(ui.ctx());
    let spacing = ui.spacing().item_spacing.x;
    let available = ui.available_width().max(0.0);
    let preferred =
        ((available - spacing * (items.len().saturating_sub(1) as f32)) / items.len() as f32 - 2.0)
            .floor()
            .max(48.0);
    let max_width = available.max(48.0);
    let compact = preferred < 110.0 && items.iter().all(|item| item.icon.is_some());
    let mut changed = None;
    ui.horizontal_wrapped(|ui| {
        for item in items {
            let is_selected = *selected == Some(item.id);
            let font = egui::TextStyle::Button.resolve(ui.style());
            let label_width = ui.fonts_mut(|fonts| {
                fonts
                    .layout_no_wrap(item.label.to_owned(), font, style.text)
                    .size()
                    .x
            });
            let width = if compact {
                preferred
            } else {
                (label_width
                    + 2.0 * ui.spacing().button_padding.x
                    + if item.icon.is_some() { 24.0 } else { 0.0 })
                .max(preferred)
                .min(max_width)
            };
            let color = if is_selected {
                style.selected_text
            } else {
                style.text
            };
            let text = RichText::new(if compact {
                String::new()
            } else if item.icon.is_some() {
                format!("     {}", item.label)
            } else {
                item.label.to_owned()
            })
            .color(color);
            let mut button = egui::Button::new(text)
                .truncate()
                .selected(is_selected)
                .frame_when_inactive(is_selected)
                .corner_radius(style.radius)
                .min_size(egui::vec2(width, 48.0));
            if is_selected {
                button = button.fill(style.selected_fill);
            }
            let response = ui
                .allocate_ui_with_layout(
                    egui::vec2(width, 48.0),
                    egui::Layout::left_to_right(egui::Align::Center),
                    |ui| {
                        ui.scope_builder(egui::UiBuilder::new().id(id.with(item.id)), |ui| {
                            crate::button::press_native(ui, &theme, |ui| {
                                ui.add_enabled(item.enabled, button)
                            })
                        })
                        .inner
                    },
                )
                .inner;
            if let Some(icon) = item.icon {
                let rect = Rect::from_center_size(
                    egui::pos2(
                        if compact {
                            response.rect.center().x
                        } else {
                            response.rect.left() + 17.0
                        },
                        response.rect.center().y,
                    ),
                    egui::vec2(16.0, 16.0),
                );
                icon(
                    ui.painter(),
                    rect,
                    if item.enabled {
                        color
                    } else {
                        color.gamma_multiply(0.5)
                    },
                );
            }
            let response = response.on_hover_text(item.label);
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

/// A floating tab bar anchored to the bottom of egui's safe content area.
/// Place it after the main content and leave at least [`Self::BOTTOM_CLEARANCE`]
/// points at the end of scrollable content so the final item can scroll clear.
pub struct FloatingNavigation<'a> {
    id: Id,
    theme: &'a Theme,
    frame: Option<egui::Frame>,
    select_on_drag: bool,
}

impl<'a> FloatingNavigation<'a> {
    pub const BOTTOM_CLEARANCE: f32 = 84.0;

    pub fn new(id: Id, theme: &'a Theme) -> Self {
        Self {
            id,
            theme,
            frame: None,
            select_on_drag: false,
        }
    }

    /// Override the capsule surface using egui's frame options.
    pub fn frame(mut self, frame: egui::Frame) -> Self {
        self.frame = Some(frame);
        self
    }

    /// Preview tabs while dragging and select the tab under the pointer on release.
    /// When the destinations overflow, dragging is reserved for scrolling the bar.
    pub fn select_on_drag(mut self, enabled: bool) -> Self {
        self.select_on_drag = enabled;
        self
    }

    /// Returns the newly selected destination, if it changed.
    pub fn show(
        self,
        ui: &Ui,
        items: &[NavigationItem<'_>],
        selected: &mut Option<Id>,
    ) -> Option<Id> {
        if items.is_empty() {
            return None;
        }
        let theme = self.theme;
        let frame = self.frame.unwrap_or_else(|| {
            egui::Frame::new()
                .fill(theme.card)
                .stroke(egui::Stroke::new(1.0, theme.border))
                .inner_margin(egui::Margin::same(5))
                .corner_radius(255)
                .shadow(egui::epaint::Shadow {
                    offset: [0, 5],
                    blur: 20,
                    spread: 0,
                    color: Color32::from_black_alpha(if theme.dark { 100 } else { 39 }),
                })
        });
        let ctx = ui.ctx();
        let glass = crate::Glass::context(ctx);
        let mut frame = crate::glass::floating_frame(ctx, theme, frame);
        if glass.is_some() && self.frame.is_none() {
            frame.shadow.offset = [0, 4];
            frame.shadow.blur = 15;
        }
        let max_slot = if glass.is_some() { 88.0 } else { 110.0 };
        let width = (ctx.content_rect().width() - 24.0 - frame.total_margin().sum().x)
            .clamp(1.0, 5.0 * max_slot);
        let slot = (width / items.len() as f32).clamp(48.0, max_slot);
        let active_fill = if glass.is_some() {
            theme
                .foreground
                .gamma_multiply(if theme.dark { 0.19 } else { 0.12 })
        } else {
            theme.secondary
        };
        let inactive_ink = if glass.is_some() {
            theme.foreground.lerp_to_gamma(theme.muted_foreground, 0.48)
        } else {
            theme.muted_foreground
        };
        let strip_width = slot * items.len() as f32;
        let scrolls = strip_width > width + 0.5;
        let can_drag = self.select_on_drag && !scrolls && ui.is_enabled();
        let rtl = ui.layout().prefer_right_to_left();
        let active = items.iter().position(|item| Some(item.id) == *selected);
        let visual_index = |index| if rtl { items.len() - 1 - index } else { index };
        let draw = |ui: &mut Ui| {
            let (row, _) =
                ui.allocate_exact_size(egui::vec2(strip_width, 54.0), egui::Sense::hover());
            let mut released = false;
            let drag_target = if can_drag
                && let Some(origin) = ctx.dragged_id().or_else(|| ctx.drag_stopped_id())
                && items
                    .iter()
                    .any(|item| item.enabled && self.id.with(item.id) == origin)
                && !ctx.any_popup_open()
                && ctx.memory(|memory| memory.top_modal_layer().is_none())
            {
                let pointer = ctx.input(|input| {
                    released = input.pointer.primary_released();
                    if released {
                        input.pointer.interact_pos()
                    } else if input.pointer.primary_down() {
                        input.pointer.latest_pos()
                    } else {
                        None
                    }
                });
                pointer
                    .filter(|pos| row.intersect(ui.clip_rect()).contains(*pos))
                    .and_then(|pos| {
                        let visual = (((pos.x - row.left()) / slot) as usize).min(items.len() - 1);
                        let index = visual_index(visual);
                        items[index].enabled.then_some(index)
                    })
            } else {
                None
            };
            let preview = drag_target.or(active);
            if let Some(preview) = preview {
                let target = visual_index(preview) as f32;
                let position = crate::motion::spring_f32(
                    ctx,
                    self.id.with("selection"),
                    target,
                    target,
                    theme,
                )
                .clamp(0.0, (items.len() - 1) as f32);
                let indicator = Rect::from_min_size(
                    row.min + egui::vec2(slot * position, 0.0),
                    egui::vec2(slot, row.height()),
                )
                .shrink2(if glass.is_some() {
                    egui::vec2(5.0, 4.0)
                } else {
                    egui::Vec2::splat(2.0)
                });
                ui.painter().rect_filled(indicator, 255, active_fill);
            }
            let mut clicked = None;
            for (index, item) in items.iter().enumerate() {
                let rect = Rect::from_min_size(
                    row.min + egui::vec2(slot * visual_index(index) as f32, 0.0),
                    egui::vec2(slot, row.height()),
                );
                let enabled = ui.is_enabled() && item.enabled;
                let sense = if !enabled {
                    egui::Sense::hover()
                } else if can_drag {
                    egui::Sense::click_and_drag()
                } else {
                    egui::Sense::click()
                };
                let response = ui.interact(rect, self.id.with(item.id), sense);
                response.widget_info(|| {
                    egui::WidgetInfo::labeled(egui::WidgetType::Button, enabled, item.label)
                });
                ctx.accesskit_node_builder(response.id, |node| {
                    node.set_selected(active == Some(index))
                });
                let color = if !enabled {
                    theme.disable(theme.foreground)
                } else if preview == Some(index) || response.hovered() {
                    theme.foreground
                } else {
                    inactive_ink
                };
                let press = crate::motion_controls::press_amount(&response, theme);
                if press > 0.001 && preview == Some(index) {
                    let pressed_fill = if glass.is_some() {
                        theme.foreground.gamma_multiply(0.12)
                    } else {
                        theme.hover(active_fill)
                    };
                    let surface = rect
                        .shrink(3.0)
                        .scale_from_center(crate::motion::press_scale(press));
                    ui.painter().add(crate::motion::press_shape(
                        egui::epaint::RectShape::filled(
                            surface,
                            24,
                            pressed_fill.gamma_multiply(press.clamp(0.0, 1.0)),
                        ),
                        press,
                    ));
                }
                if response.has_focus() {
                    ui.painter().rect_stroke(
                        rect.shrink(2.0),
                        255,
                        egui::Stroke::new(2.0, theme.foreground),
                        egui::StrokeKind::Inside,
                    );
                }
                let icon_y = rect.center().y - 7.0;
                let label_y = if let Some(icon) = item.icon {
                    let size = 20.0 - 2.5 * press;
                    icon(
                        ui.painter(),
                        Rect::from_center_size(
                            egui::pos2(rect.center().x, icon_y),
                            egui::Vec2::splat(size),
                        ),
                        color,
                    );
                    rect.center().y + 12.0
                } else {
                    rect.center().y
                };
                let mut job = egui::text::LayoutJob::simple(
                    item.label.to_owned(),
                    egui::FontId::proportional(10.5),
                    color,
                    (slot - 8.0).max(1.0),
                );
                job.wrap.max_rows = 1;
                job.wrap.break_anywhere = true;
                let label = ui.painter().layout_job(job);
                ui.painter().galley(
                    egui::pos2(rect.center().x, label_y) - label.size() * 0.5,
                    label,
                    color,
                );
                if enabled && response.clicked() {
                    clicked = Some(item.id);
                }
            }
            if released && let Some(index) = drag_target {
                Some(items[index].id)
            } else {
                clicked
            }
        };
        let next = egui::Area::new(self.id)
            .anchor(egui::Align2::CENTER_BOTTOM, egui::vec2(0.0, -12.0))
            .order(egui::Order::Middle)
            .fade_in(false)
            .show(ctx, |ui| {
                if glass.is_some() && ui.is_visible() {
                    paint_scroll_edge(ui.painter(), ctx.content_rect(), theme.background);
                }
                crate::glass::show_frame(
                    ui,
                    frame,
                    glass.map(|_| crate::GlassProfile::Dock),
                    |ui| {
                        let rim = glass
                            .filter(|_| !crate::glass::live_enabled(ui.ctx()))
                            .map(|_| ui.painter().add(egui::Shape::Noop));
                        let result = if scrolls {
                            ui.set_width(width);
                            egui::ScrollArea::horizontal()
                                .max_width(width)
                                .scroll_bar_visibility(
                                    egui::scroll_area::ScrollBarVisibility::AlwaysHidden,
                                )
                                .show(ui, draw)
                                .inner
                        } else {
                            draw(ui)
                        };
                        if let (Some(glass), Some(slot)) = (glass, rim) {
                            let rect = frame.widget_rect(ui.min_rect());
                            ui.painter().set(
                                slot,
                                glass.rim(theme, rect, frame.corner_radius.into(), 0.55),
                            );
                        }
                        result
                    },
                )
                .inner
            })
            .inner;

        if let Some(next) = next.filter(|id| Some(*id) != *selected) {
            *selected = Some(next);
            ctx.request_repaint();
            Some(next)
        } else {
            None
        }
    }
}

// Paint only. The fade sits beneath the dock's backdrop pass and never captures input.
fn paint_scroll_edge(painter: &egui::Painter, viewport: Rect, background: Color32) {
    let depth = viewport.height().min(112.0);
    if depth <= 0.0 {
        return;
    }
    let mut mesh = egui::Mesh::default();
    for (distance, opacity) in [
        (1.0, 0.0),
        (0.75, 0.06),
        (0.50, 0.28),
        (0.25, 0.63),
        (0.0, 0.91),
    ] {
        let y = viewport.bottom() - depth * distance;
        let color = background.gamma_multiply(opacity);
        mesh.colored_vertex(egui::pos2(viewport.left(), y), color);
        mesh.colored_vertex(egui::pos2(viewport.right(), y), color);
    }
    for top in [0, 2, 4, 6] {
        mesh.add_triangle(top, top + 1, top + 2);
        mesh.add_triangle(top + 1, top + 3, top + 2);
    }
    painter
        .with_clip_rect(viewport)
        .add(egui::Shape::mesh(mesh));
}
