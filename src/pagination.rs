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
/// `primary` when selected, with the md radius and no border.
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
        visual.corner_radius = 10.into();
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
    // Allocate each whole control before drawing it so a scope cannot overflow its row.
    ui.horizontal_wrapped(|ui| {
        pagination_visuals(ui, theme);
        let width = button_width(ui, theme, previous_label, max_width);
        let previous = button_slot(ui, theme, width, |ui| {
            ui.scope_builder(egui::UiBuilder::new().id(id.with("previous")), |ui| {
                ui.add_enabled(
                    state.page > 0,
                    egui::Button::new(
                        RichText::new(previous_label)
                            .size(body_size)
                            .color(theme.foreground),
                    )
                    .frame_when_inactive(false)
                    .truncate()
                    .corner_radius(10)
                    .min_size(egui::vec2(width, height)),
                )
            })
            .inner
        });
        ui.ctx()
            .accesskit_node_builder(previous.id, |node| node.set_label(previous_label));
        if previous.clicked() {
            state.previous();
        }
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
                        .corner_radius(10)
                        .min_size(egui::vec2(width, height));
                    if selected {
                        button = button.fill(theme.primary);
                    }
                    let response = button_slot(ui, theme, width, |ui| {
                        ui.scope_builder(egui::UiBuilder::new().id(id.with(("page", page))), |ui| {
                            ui.add(button)
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
                    if response.clicked() {
                        state.page = page;
                    }
                }
            }
        }
        let width = button_width(ui, theme, next_label, max_width);
        let next = button_slot(ui, theme, width, |ui| {
            ui.scope_builder(egui::UiBuilder::new().id(id.with("next")), |ui| {
                ui.add_enabled(
                    state.page + 1 < state.pages,
                    egui::Button::new(
                        RichText::new(next_label)
                            .size(body_size)
                            .color(theme.foreground),
                    )
                    .frame_when_inactive(false)
                    .truncate()
                    .corner_radius(10)
                    .min_size(egui::vec2(width, height)),
                )
            })
            .inner
        });
        ui.ctx()
            .accesskit_node_builder(next.id, |node| node.set_label(next_label));
        if next.clicked() {
            state.next();
        }
    });
}
