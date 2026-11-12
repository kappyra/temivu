//! Preliminary Forui pagination controller and egui view.
//! Custom child builders, upstream styling, localization and visual parity remain open.
use crate::{Messages, Theme};
use egui::{Color32, FontId, RichText, Stroke, Ui};

/// A zero-based, caller-owned pagination controller.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct PaginationState {
    pages: usize,
    page: usize,
    siblings: usize,
    show_edges: bool,
}

/// An item in the page strip; the ellipsis is decorative.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum PageItem {
    Page(usize),
    Ellipsis,
}

impl PaginationState {
    /// Limit sibling count to 1000 to bound the number of allocated page items.
    pub fn new(
        pages: usize,
        page: usize,
        siblings: usize,
        show_edges: bool,
    ) -> Result<Self, &'static str> {
        if siblings > 1000 {
            return Err("at most 1000 siblings are supported");
        }
        if pages == 0 || page >= pages {
            return Err("pages must be nonzero and page must be in range");
        }
        Ok(Self {
            pages,
            page,
            siblings,
            show_edges,
        })
    }
    pub fn page(&self) -> usize {
        self.page
    }
    pub fn pages(&self) -> usize {
        self.pages
    }
    pub fn set_page(&mut self, page: usize) -> Result<(), &'static str> {
        if page >= self.pages {
            return Err("page is out of range");
        }
        self.page = page;
        Ok(())
    }
    /// Changing the page count clamps the selected page to the new last page.
    pub fn set_pages(&mut self, pages: usize) -> Result<(), &'static str> {
        if pages == 0 {
            return Err("pages must be nonzero");
        }
        self.pages = pages;
        self.page = self.page.min(pages - 1);
        Ok(())
    }
    pub fn previous(&mut self) {
        self.page = self.page.saturating_sub(1);
    }
    pub fn next(&mut self) {
        self.page = self.page.saturating_add(1).min(self.pages - 1);
    }

    /// Page indices around the current page, plus optional edge items and gaps.
    /// Saturating arithmetic prevents overflow for large user-provided counts.
    pub fn items(&self) -> Vec<PageItem> {
        let last = self.pages - 1;
        let minimum = self
            .siblings
            .saturating_add(1)
            .saturating_add(usize::from(self.show_edges));
        let threshold = if self.pages
            <= minimum.saturating_add(self.siblings.saturating_mul(2).saturating_add(2))
        {
            self.pages
        } else {
            minimum
        };
        let (start, end) = if self.pages <= threshold {
            (0, last)
        } else {
            let start = if self.page < self.siblings {
                0
            } else if self.page > last.saturating_sub(threshold) {
                last.saturating_sub(threshold).saturating_sub(self.siblings)
            } else if self.page <= threshold {
                0
            } else {
                self.page - self.siblings
            };
            let end = if self.page.saturating_add(self.siblings) > last {
                last
            } else if self.page < threshold.saturating_add(1) {
                threshold.saturating_add(self.siblings).min(last)
            } else if self.page >= last.saturating_sub(threshold) {
                last
            } else {
                self.page + self.siblings
            };
            (start, end)
        };
        let mut result = Vec::new();
        if self.page > threshold {
            if self.show_edges {
                result.push(PageItem::Page(0));
            }
            result.push(PageItem::Ellipsis);
        }
        result.extend((start..=end).map(PageItem::Page));
        if self.page < last.saturating_sub(threshold) {
            result.push(PageItem::Ellipsis);
            if self.show_edges {
                result.push(PageItem::Page(last));
            }
        }
        result
    }
}

/// Draw the current strip. Button labels are supplied by the caller for localization.
pub fn pagination(
    ui: &mut Ui,
    state: &mut PaginationState,
    theme: &Theme,
    previous_label: &str,
    next_label: &str,
) {
    let id = ui.next_auto_id().with("temivu-pagination");
    draw_pagination(ui, state, theme, previous_label, next_label, None, id);
}

/// Draw compact previous and next controls with a caller-formatted page label.
///
/// The formatter receives the one-based page number and the total page count.
/// The arrows have the caller's localized accessible names.
pub fn pagination_compact(
    ui: &mut Ui,
    state: &mut PaginationState,
    theme: &Theme,
    previous_label: &str,
    next_label: &str,
    current_label: impl FnOnce(usize, usize) -> String,
) {
    let input = if theme.touch {
        crate::InputMode::Touch
    } else {
        crate::InputMode::Desktop
    };
    let width = ui.available_width().clamp(1.0, 340.0);
    let size = crate::ButtonSize::Medium.height(input).min(width / 3.0);
    let lane = size + 8.0;
    let text = current_label(state.page + 1, state.pages);
    let rtl = ui.layout().prefer_right_to_left();
    ui.vertical_centered(|ui| {
        // Equal side lanes keep the arrows stable when the page number changes.
        let inline = width - 2.0 * lane >= 80.0;
        if !inline {
            ui.add(egui::Label::new(RichText::new(&text).strong()).wrap());
        }
        let (row, _) = ui.allocate_exact_size(egui::vec2(width, size), egui::Sense::hover());
        if inline {
            let caption = row.shrink2(egui::vec2(lane, 0.0));
            ui.put(
                caption,
                egui::Label::new(RichText::new(&text).strong())
                    .halign(egui::Align::Center)
                    .truncate()
                    .selectable(false),
            )
            .on_hover_text(&text);
        }
        let inset = if inline {
            size * 0.5
        } else {
            width * 0.5 - size * 0.5 - 4.0
        };
        let offset = width * 0.5 - inset;
        let forward = if rtl { -1.0 } else { 1.0 };
        let button = |ui: &mut Ui, direction, label, enabled| {
            let center = row.center() + egui::vec2(offset * direction, 0.0);
            let slot = egui::Rect::from_center_size(center, egui::Vec2::splat(size));
            compact_arrow(ui, theme, slot, direction, label, enabled)
        };
        let previous = button(ui, -forward, previous_label, state.page > 0);
        let next = button(ui, forward, next_label, state.page + 1 < state.pages);
        if previous.clicked() {
            state.previous();
            ui.ctx().request_repaint();
        } else if next.clicked() {
            state.next();
            ui.ctx().request_repaint();
        }
    });
}

fn compact_arrow(
    ui: &mut Ui,
    theme: &Theme,
    slot: egui::Rect,
    direction: f32,
    label: &str,
    enabled: bool,
) -> egui::Response {
    let mut ui = ui.new_child(
        egui::UiBuilder::new()
            .id_salt(("page-arrow", direction.is_sign_positive()))
            .max_rect(slot)
            .layout(egui::Layout::centered_and_justified(
                egui::Direction::LeftToRight,
            )),
    );
    let response = crate::button::press_native(&mut ui, theme, |ui| {
        let response = ui.add_enabled(
            enabled,
            egui::Button::new("")
                .fill(theme.card)
                .stroke(Stroke::new(1.0, theme.border))
                .corner_radius(crate::SurfaceShape::Control.radius(slot.size()))
                .min_size(slot.size()),
        );
        let center = response.rect.center();
        let radius = (slot.width() * 0.11).min(4.5);
        let wing = egui::vec2(-direction * radius * 0.45, radius);
        let tip = center + egui::vec2(direction * radius * 0.45, 0.0);
        let ink = if response.enabled() {
            theme.foreground
        } else {
            theme.muted_foreground
        };
        ui.painter().add(egui::Shape::line(
            vec![center + wing, tip, center + wing * egui::vec2(1.0, -1.0)],
            Stroke::new(1.6, ink),
        ));
        response
    });
    ui.ctx()
        .accesskit_node_builder(response.id, |node| node.set_label(label));
    response.on_hover_text(label)
}

/// Draw pagination with a caller-provided ID that stays stable across layout changes.
/// The ID must be unique among pagination instances in the context.
pub fn pagination_with_id(
    ui: &mut Ui,
    state: &mut PaginationState,
    theme: &Theme,
    previous_label: &str,
    next_label: &str,
    id: egui::Id,
) {
    draw_pagination(ui, state, theme, previous_label, next_label, None, id);
}

/// Draw pagination with translated button and one-based page labels.
pub fn pagination_localized(
    ui: &mut Ui,
    state: &mut PaginationState,
    theme: &Theme,
    messages: &Messages,
) {
    let id = ui.next_auto_id().with("temivu-pagination");
    pagination_localized_with_id(ui, state, theme, messages, id);
}

/// Localized pagination with a caller-provided globally unique ID.
pub fn pagination_localized_with_id(
    ui: &mut Ui,
    state: &mut PaginationState,
    theme: &Theme,
    messages: &Messages,
    id: egui::Id,
) {
    draw_pagination(
        ui,
        state,
        theme,
        messages.previous,
        messages.next,
        Some(messages),
        id,
    );
}

/// Forui pagination item constraints: 32x32 on desktop, 44x44 on touch.
fn item_size(theme: &Theme) -> f32 {
    if theme.touch { 44.0 } else { 32.0 }
}

/// Forui item decoration: transparent idle, `secondary` on hover/press and
/// `primary` when selected, with a spring-driven selection marker.
fn pagination_visuals(ui: &mut Ui, theme: &Theme) {
    ui.spacing_mut().button_padding = egui::vec2(8.0, 0.0);
    ui.spacing_mut().item_spacing.x = 4.0;
    let widgets = &mut ui.visuals_mut().widgets;
    for visual in [
        &mut widgets.inactive,
        &mut widgets.hovered,
        &mut widgets.active,
        &mut widgets.noninteractive,
    ] {
        visual.corner_radius = 255.into();
        visual.bg_stroke = Stroke::NONE;
    }
    widgets.inactive.weak_bg_fill = Color32::TRANSPARENT;
    widgets.hovered.weak_bg_fill = theme.secondary;
    widgets.active.weak_bg_fill = theme.secondary;
    widgets.noninteractive.weak_bg_fill = Color32::TRANSPARENT;
}

fn button_width(ui: &mut Ui, theme: &Theme, label: &str, max_width: f32) -> f32 {
    let font = FontId::proportional(theme.typography.body.size);
    let text_width = ui.fonts_mut(|fonts| {
        fonts
            .layout_no_wrap(label.to_owned(), font, Color32::WHITE)
            .size()
            .x
    });
    (text_width + 16.0).clamp(item_size(theme), max_width)
}

fn button_slot(
    ui: &mut Ui,
    theme: &Theme,
    width: f32,
    draw: impl FnOnce(&mut Ui) -> egui::Response,
) -> egui::Response {
    ui.allocate_ui_with_layout(
        egui::vec2(width, item_size(theme)),
        egui::Layout::left_to_right(egui::Align::Center),
        draw,
    )
    .inner
}

fn draw_pagination(
    ui: &mut Ui,
    state: &mut PaginationState,
    theme: &Theme,
    previous_label: &str,
    next_label: &str,
    messages: Option<&Messages>,
    id: egui::Id,
) {
    let max_width = ui.available_width().max(32.0);
    let height = item_size(theme);
    let body_size = theme.typography.body.size;
    let page_at_start = state.page;
    // Allocate each whole control before drawing it so a scope cannot overflow its row.
    ui.horizontal_wrapped(|ui| {
        pagination_visuals(ui, theme);
        ui.spacing_mut().item_spacing.y = 6.0;
        let width = button_width(ui, theme, previous_label, max_width);
        let previous = button_slot(ui, theme, width, |ui| {
            ui.scope_builder(egui::UiBuilder::new().id(id.with("previous")), |ui| {
                crate::button::press_native(ui, theme, |ui| {
                    ui.add_enabled(
                        state.page > 0,
                        egui::Button::new(
                            RichText::new(previous_label)
                                .size(body_size)
                                .color(theme.foreground),
                        )
                        .frame_when_inactive(false)
                        .truncate()
                        .corner_radius(255)
                        .min_size(egui::vec2(width, height)),
                    )
                })
            })
            .inner
        });
        ui.ctx()
            .accesskit_node_builder(previous.id, |node| node.set_label(previous_label));
        if previous.clicked() {
            state.previous();
        }
        let mut active = None;
        let mut row_bounds = egui::Rect::NOTHING;
        for item in state.items() {
            match item {
                PageItem::Ellipsis => {
                    let (rect, _) =
                        ui.allocate_exact_size(egui::Vec2::splat(height), egui::Sense::hover());
                    ui.painter().text(
                        rect.center(),
                        egui::Align2::CENTER_CENTER,
                        "…",
                        FontId::proportional(body_size),
                        theme.foreground,
                    );
                }
                PageItem::Page(page) => {
                    let selected = page == state.page;
                    let page_label = (page + 1).to_string();
                    let width = button_width(ui, theme, &page_label, max_width);
                    let label = RichText::new(&page_label)
                        .size(body_size)
                        .color(if selected {
                            theme.primary_foreground
                        } else {
                            theme.foreground
                        });
                    let mut button = egui::Button::new(label)
                        .selected(selected)
                        .frame_when_inactive(selected)
                        .truncate()
                        .corner_radius(255)
                        .min_size(egui::vec2(width, height));
                    if selected {
                        button = button.fill(theme.primary);
                    }
                    let response = button_slot(ui, theme, width, |ui| {
                        ui.scope_builder(egui::UiBuilder::new().id(id.with(("page", page))), |ui| {
                            crate::button::press_native(ui, theme, |ui| ui.add(button))
                        })
                        .inner
                    });
                    ui.ctx().accesskit_node_builder(response.id, |node| {
                        node.clear_toggled();
                        node.set_selected(selected);
                        node.set_label(messages.map_or_else(
                            || page_label.clone(),
                            |messages| messages.page_label(page + 1),
                        ));
                    });
                    let rect = response.rect;
                    if (row_bounds.center().y - rect.center().y).abs() < height * 0.5 {
                        row_bounds = row_bounds.union(rect);
                    } else {
                        row_bounds = rect;
                    }
                    if selected {
                        active = Some((rect, row_bounds));
                    } else if let Some((selected_rect, bounds)) = &mut active
                        && (selected_rect.center().y - rect.center().y).abs() < height * 0.5
                    {
                        *bounds = row_bounds;
                    }
                    if response.has_focus() {
                        ui.painter().rect_stroke(
                            response.rect.shrink(1.0),
                            10,
                            Stroke::new(2.0, theme.foreground),
                            egui::StrokeKind::Inside,
                        );
                    }
                    if response.clicked() {
                        state.page = page;
                    }
                }
            }
        }
        if let Some((rect, bounds)) = active {
            let origin = previous.rect.min;
            let row = rect.bottom() - origin.y;
            let target = rect.center().x - origin.x;
            let row_key = id.with("indicator-row");
            let x_key = id.with("indicator-x");
            let same_row = ui
                .ctx()
                .data(|data| data.get_temp::<f32>(row_key))
                .is_some_and(|last| (row - last).abs() < 0.5);
            if !same_row {
                crate::motion::reset(ui.ctx(), x_key);
            }
            ui.ctx().data_mut(|data| data.insert_temp(row_key, row));
            let x = origin.x + crate::motion::spring_f32(ui.ctx(), x_key, target, target, theme);
            let lift = 1.0 - ((x - rect.center().x).abs() / height).min(1.0);
            let radius = (height * 0.35).min(bounds.width() * 0.5);
            let baseline = rect.bottom() + 2.0;
            ui.painter().rect_filled(
                egui::Rect::from_min_max(
                    egui::pos2(bounds.left(), baseline - 1.5),
                    egui::pos2(bounds.right(), baseline + 1.5),
                ),
                2,
                theme.muted_foreground.gamma_multiply(0.4),
            );
            let center = x.clamp(
                bounds.left() + radius,
                (bounds.right() - radius).max(bounds.left() + radius),
            );
            let mound = egui::Rect::from_min_max(
                egui::pos2(center - radius, baseline - 5.0 - lift * 4.0),
                egui::pos2(center + radius, baseline + 1.5),
            );
            ui.painter().rect_filled(
                mound,
                egui::CornerRadius {
                    nw: 9,
                    ne: 9,
                    sw: 2,
                    se: 2,
                },
                theme.primary,
            );
        }
        let width = button_width(ui, theme, next_label, max_width);
        let next = button_slot(ui, theme, width, |ui| {
            ui.scope_builder(egui::UiBuilder::new().id(id.with("next")), |ui| {
                crate::button::press_native(ui, theme, |ui| {
                    ui.add_enabled(
                        state.page + 1 < state.pages,
                        egui::Button::new(
                            RichText::new(next_label)
                                .size(body_size)
                                .color(theme.foreground),
                        )
                        .frame_when_inactive(false)
                        .truncate()
                        .corner_radius(255)
                        .min_size(egui::vec2(width, height)),
                    )
                })
            })
            .inner
        });
        ui.ctx()
            .accesskit_node_builder(next.id, |node| node.set_label(next_label));
        if next.clicked() {
            state.next();
        }
    });
    ui.add_space(4.0);
    if page_at_start != state.page {
        ui.ctx().request_repaint();
    }
}
