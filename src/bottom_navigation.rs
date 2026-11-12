//! Preliminary bottom navigation; icons, localized position semantics and parity remain open.
use crate::{Theme, Vesper};
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
    lighting: Option<Option<Vesper>>,
}

impl<'a> FloatingNavigation<'a> {
    pub const BOTTOM_CLEARANCE: f32 = 92.0;

    pub fn new(id: Id, theme: &'a Theme) -> Self {
        Self {
            id,
            theme,
            frame: None,
            select_on_drag: false,
            lighting: None,
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

    /// Override the inherited Vesper material for dock destinations.
    pub fn lighting(mut self, lighting: Option<Vesper>) -> Self {
        self.lighting = Some(lighting);
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
        let lighting = self.lighting.unwrap_or_else(|| Vesper::context(ctx));
        let mut frame = crate::glass::floating_frame(ctx, theme, frame);
        if glass.is_some() && self.frame.is_none() {
            frame.shadow.offset = [0, 4];
            frame.shadow.blur = 15;
        }
        let viewport = ctx.content_rect();
        let max_slot = if glass.is_some() { 90.0 } else { 104.0 };
        let margin = (viewport.width() * 0.038).clamp(8.0, 20.0);
        let width = (viewport.width() - 2.0 * margin - frame.total_margin().sum().x)
            .clamp(1.0, 5.0 * max_slot);
        let slot = (width / items.len() as f32)
            .clamp(48.0, max_slot)
            .min(width);
        let icon_only = slot < 61.0;
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
            let pressed = items.iter().position(|item| {
                item.enabled
                    && ctx
                        .read_response(self.id.with(item.id))
                        .is_some_and(|response| response.is_pointer_button_down_on())
            });
            let preview = drag_target.or(pressed).or(active);
            let indicator_slot = ui.painter().add(egui::Shape::Noop);
            let light_slot = lighting.map(|_| ui.painter().add(egui::Shape::Noop));
            let indicator = preview.map(|preview| {
                let target = visual_index(preview) as f32;
                let position = crate::motion::spring_f32(
                    ctx,
                    self.id.with("selection"),
                    target,
                    target,
                    theme,
                )
                .clamp(0.0, (items.len() - 1) as f32);
                Rect::from_min_size(
                    row.min + egui::vec2(slot * position, 0.0),
                    egui::vec2(slot, row.height()),
                )
                .shrink2(egui::vec2(if glass.is_some() { 4.0 } else { 2.0 }, 3.0))
            });
            let mut indicator_press = if drag_target.is_some() { 0.35 } else { 0.0 };
            let mut light = None;
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
                if preview == Some(index) {
                    indicator_press = f32::max(indicator_press, press);
                }
                if let Some(style) = lighting {
                    let surface = if preview == Some(index) {
                        indicator.map(|rect| dock_indicator(rect, indicator_press))
                    } else {
                        None
                    };
                    let (surface, radius) = surface.unwrap_or((rect.shrink(5.0), 20));
                    if let Some(shape) = style.interactive_shape(
                        ui.painter(),
                        theme,
                        &response,
                        surface,
                        f32::from(radius).into(),
                    ) {
                        // At most the hovered destination and the focused one
                        // can produce lighting. The hovered light takes priority.
                        if light.is_none() || response.hovered() {
                            light = Some(shape);
                        }
                    }
                    if enabled {
                        crate::lighting::spill::receive(
                            ui.painter(),
                            response.id,
                            surface,
                            f32::from(radius).into(),
                            &style,
                            theme.secondary,
                            theme.card,
                        );
                    }
                }
                if scrolls && response.gained_focus() {
                    response.scroll_to_me(None);
                }
                if response.has_focus() {
                    ui.painter().rect_stroke(
                        rect.shrink(2.0),
                        255,
                        egui::Stroke::new(2.0, theme.foreground),
                        egui::StrokeKind::Inside,
                    );
                }
                let compact = icon_only && item.icon.is_some();
                let icon_y = rect.center().y - if compact { 0.0 } else { 7.0 };
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
                if !compact {
                    let mut job = egui::text::LayoutJob::simple(
                        item.label.to_owned(),
                        egui::FontId::proportional(11.0),
                        color,
                        (slot - 6.0).max(1.0),
                    );
                    job.wrap.max_rows = 1;
                    job.wrap.break_anywhere = true;
                    let label = ui.painter().layout_job(job);
                    ui.painter().galley(
                        egui::pos2(rect.center().x, label_y) - label.size() * 0.5,
                        label,
                        color,
                    );
                }
                response.clone().on_hover_text(item.label);
                if enabled && response.clicked() {
                    clicked = Some(item.id);
                }
            }
            if let Some(indicator) = indicator {
                let (surface, radius) = dock_indicator(indicator, indicator_press);
                ui.painter().set(
                    indicator_slot,
                    egui::epaint::RectShape::filled(surface, radius, active_fill),
                );
            }
            if let (Some(slot), Some(light)) = (light_slot, light) {
                ui.painter().set(slot, light);
            }
            if released && let Some(index) = drag_target {
                Some(items[index].id)
            } else {
                clicked
            }
        };
        let bottom_gap = if viewport.width() < 520.0 { 16.0 } else { 12.0 };
        let next = egui::Area::new(self.id)
            .anchor(egui::Align2::CENTER_BOTTOM, egui::vec2(0.0, -bottom_gap))
            .order(egui::Order::Middle)
            .fade_in(false)
            .show(ctx, |ui| {
                if glass.is_some() && ui.is_visible() {
                    paint_scroll_edge(
                        ui.painter(),
                        ctx.viewport_rect(),
                        viewport,
                        theme.background,
                    );
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
                                .auto_shrink([false, true])
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

fn dock_indicator(rect: Rect, press: f32) -> (Rect, u8) {
    let press = press.clamp(0.0, 1.0);
    let surface = Rect::from_center_size(
        rect.center(),
        rect.size() * egui::vec2(1.0 - 0.085 * press, 1.0 - 0.04 * press),
    );
    let radius = (surface.size().min_elem() * (0.5 - 0.14 * press))
        .max(0.0)
        .round() as u8;
    (surface, radius)
}

// Paint only. The fade sits beneath the dock's backdrop pass and never captures input.
fn paint_scroll_edge(painter: &egui::Painter, viewport: Rect, safe: Rect, background: Color32) {
    if !viewport.is_positive() {
        return;
    }
    // Finish the fade above the safe edge and carry the same color to the
    // actual window edge, including any area beneath the OS gesture control.
    let end = (safe.bottom() - 8.0).clamp(viewport.top(), viewport.bottom());
    let depth = (end - viewport.top()).min(104.0);
    if depth <= 0.0 {
        return;
    }
    let mut mesh = egui::Mesh::default();
    let mut row = |y, opacity| {
        let color = background.gamma_multiply(opacity);
        mesh.colored_vertex(egui::pos2(viewport.left(), y), color);
        mesh.colored_vertex(egui::pos2(viewport.right(), y), color);
    };
    for (distance, opacity) in [
        (1.0, 0.0),
        (0.75, 0.04),
        (0.50, 0.18),
        (0.25, 0.46),
        (0.0, 1.0),
    ] {
        row(end - depth * distance, opacity);
    }
    row(viewport.bottom(), 1.0);
    for top in [0, 2, 4, 6, 8] {
        mesh.add_triangle(top, top + 1, top + 2);
        mesh.add_triangle(top + 1, top + 3, top + 2);
    }
    let mut painter = painter.clone();
    painter.set_clip_rect(viewport);
    painter.add(egui::Shape::mesh(mesh));
}
