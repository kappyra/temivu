//! Preliminary port of Forui's picker wheel: a scrollable list with a fixed
//! centre selection band over caller-owned items.
//!
//! The wheel is an egui scroll area whose scroll offset snaps to whole rows.
//! Scrolling changes the selection when the wheel settles, pointer presses and
//! the keyboard select rows directly, and no frame requests a repaint unless
//! the selection, the scroll offset or the focus actually changed.
//!
//! Looping (wheel wraparound), animation, haptics, per-item custom content,
//! 12-hour periods and upstream visual parity remain open. See `docs/PICKER.md`.
//!
//! ```
//! # fn example(ui: &mut egui::Ui) {
//! use temivu::{Picker, PickerItem, Theme};
//!
//! let theme = Theme::light();
//! let items = [
//!     PickerItem::new(egui::Id::unique("small"), "Small"),
//!     PickerItem::new(egui::Id::unique("medium"), "Medium"),
//!     PickerItem::new(egui::Id::unique("large"), "Large"),
//! ];
//! let mut selected = Some(egui::Id::unique("medium"));
//! let output = Picker::new(&mut selected, &items, &theme)
//!     .label("Size")
//!     .show(ui);
//! if output.changed {
//!     let _chosen = output.selected;
//! }
//! # }
//! ```

use crate::Theme;
use crate::tokens::Sizes;
use egui::scroll_area::ScrollBarVisibility;
use egui::{Align2, FontId, Id, Rect, Response, Sense, Stroke, StrokeKind, Ui, Vec2};

/// One selectable row in a [`Picker`].
///
/// Give every item an ID that is stable across frames and unique within the
/// picker, so focus, the selection band and the accessibility row nodes stay
/// attached to the same value when rows are inserted or reordered.
#[derive(Clone, Copy, Debug)]
pub struct PickerItem<'a> {
    pub id: Id,
    pub label: &'a str,
    pub enabled: bool,
}

impl<'a> PickerItem<'a> {
    /// Create an enabled item from a stable ID and its visible label.
    pub fn new(id: Id, label: &'a str) -> Self {
        Self {
            id,
            label,
            enabled: true,
        }
    }

    /// Disable the item: it stays visible and announced, but cannot be selected.
    pub fn enabled(mut self, enabled: bool) -> Self {
        self.enabled = enabled;
        self
    }
}

/// What [`Picker::show`] reports for the current frame.
pub struct PickerResponse {
    /// The wheel's focusable response; it owns keyboard focus for navigation.
    pub response: Response,
    /// The caller-owned selection after this frame.
    pub selected: Option<Id>,
    /// Whether the user changed the selection by pointer, keyboard or scroll.
    pub changed: bool,
}

/// What the picker remembers between frames in egui's temporary data.
#[derive(Clone, Copy, Debug, Default)]
struct PickerMemory {
    last_selected: Option<Id>,
    snap_offset: Option<f32>,
}

/// A wheel-style picker over caller-owned [`PickerItem`]s.
///
/// The selection is caller-owned. Pass a stable [`Id`] through [`Picker::id`]
/// when several pickers in the same window could otherwise receive the same
/// automatic ID, for example when pickers are inserted conditionally.
pub struct Picker<'a> {
    selected: &'a mut Option<Id>,
    items: &'a [PickerItem<'a>],
    theme: &'a Theme,
    label: &'a str,
    id: Option<Id>,
    visible_rows: usize,
    item_height: Option<f32>,
    sizes: Sizes,
    width: Option<f32>,
    looping: bool,
    enabled: bool,
}

impl<'a> Picker<'a> {
    /// Create a picker for the caller-owned `selected` value and `items`.
    ///
    /// The label is the wheel's accessible name; replace the default with
    /// [`Picker::label`].
    pub fn new(
        selected: &'a mut Option<Id>,
        items: &'a [PickerItem<'a>],
        theme: &'a Theme,
    ) -> Self {
        Self {
            selected,
            items,
            theme,
            label: "Picker",
            id: None,
            visible_rows: 5,
            item_height: None,
            sizes: Sizes::desktop(),
            width: None,
            looping: false,
            enabled: true,
        }
    }

    /// Set the accessible name announced for the wheel.
    pub fn label(mut self, label: &'a str) -> Self {
        self.label = label;
        self
    }

    /// Give the wheel a stable, globally unique ID across layout changes.
    ///
    /// The default derives an ID from the call order, which changes when a
    /// preceding conditional widget appears or disappears.
    pub fn id(mut self, id: Id) -> Self {
        self.id = Some(id);
        self
    }

    /// How many rows are visible at once. Defaults to 5; the centre row is the
    /// selection band.
    pub fn visible_rows(mut self, rows: usize) -> Self {
        self.visible_rows = rows.max(1);
        self
    }

    /// The height of one row in points. Defaults to the size token (`Sizes::item`).
    pub fn item_height(mut self, height: f32) -> Self {
        self.item_height = Some(height.max(8.0));
        self
    }

    /// The responsive size tokens used for the default row height.
    pub fn sizes(mut self, sizes: Sizes) -> Self {
        self.sizes = sizes;
        self
    }

    /// A fixed wheel width in points, clamped to the available width.
    ///
    /// Use this when several pickers share a horizontal layout.
    pub fn width(mut self, width: f32) -> Self {
        self.width = Some(width.max(1.0));
        self
    }

    /// Wrap keyboard navigation past the first and last enabled item.
    /// Defaults to `false` (clamp).
    pub fn looping(mut self, looping: bool) -> Self {
        self.looping = looping;
        self
    }

    /// Disable the whole wheel: it stays visible and announced but cannot change.
    pub fn enabled(mut self, enabled: bool) -> Self {
        self.enabled = enabled;
        self
    }

    /// Draw the wheel and return its response.
    pub fn show(self, ui: &mut Ui) -> PickerResponse {
        let id = self
            .id
            .unwrap_or_else(|| ui.next_auto_id().with("temivu-picker"));
        ui.scope_builder(egui::UiBuilder::new().scope_id(id), |ui| {
            self.show_inner(ui, id)
        })
        .inner
    }

    fn show_inner(self, ui: &mut Ui, id: Id) -> PickerResponse {
        let Self {
            selected,
            items,
            theme,
            label,
            visible_rows,
            item_height,
            sizes,
            width,
            looping,
            enabled,
            ..
        } = self;
        let row_height = item_height.unwrap_or(sizes.item).max(8.0);
        let viewport_height = row_height * visible_rows as f32;
        let list_width = width
            .map_or_else(
                || ui.available_width(),
                |width| width.min(ui.available_width()),
            )
            .max(1.0);
        let memory_key = id.with("temivu-picker-memory");
        let memory = ui
            .ctx()
            .data(|data| data.get_temp::<PickerMemory>(memory_key))
            .unwrap_or_default();
        let focus_id = id.with("wheel");
        let wheel_focused = ui.memory(|memory| memory.has_focus(focus_id));
        let mut changed = false;

        // Keyboard navigation. The focus lock filter keeps egui's spatial focus
        // navigation from stealing the arrow keys that drive the wheel.
        if wheel_focused && enabled {
            lock_arrows(ui, focus_id);
            let (up, down, home, end) = ui.input(|input| {
                (
                    input.key_pressed(egui::Key::ArrowUp),
                    input.key_pressed(egui::Key::ArrowDown),
                    input.key_pressed(egui::Key::Home),
                    input.key_pressed(egui::Key::End),
                )
            });
            let target = if up {
                step_enabled(items, *selected, -1, looping)
            } else if down {
                step_enabled(items, *selected, 1, looping)
            } else if home {
                step_enabled(items, None, 1, false)
            } else if end {
                step_enabled(items, None, -1, false)
            } else {
                None
            };
            if let Some(target) = target
                && *selected != Some(target)
            {
                *selected = Some(target);
                changed = true;
                ui.ctx().request_repaint();
            }
        }

        // Follow caller-driven selection changes (and this frame's keyboard
        // change) by placing the selected row in the centre band. Wheel-driven
        // changes are already at the centre, so they are not scrolled again.
        let selection_index = selected.and_then(|selected| {
            items
                .iter()
                .position(|item| item.id == selected)
                .map(|index| index as f32)
        });
        let explicit_offset = if memory.last_selected != *selected {
            selection_index.map(|index| index * row_height)
        } else {
            None
        };

        // Wheel input this frame: do not snap while the user is still scrolling.
        let scrolling_input =
            ui.input(|input| input.smooth_scroll_delta.y != 0.0 || input.pointer.any_down());
        let pending_snap = memory.snap_offset.filter(|_| !scrolling_input);

        let (viewport_rect, _) =
            ui.allocate_exact_size(Vec2::new(list_width, viewport_height), Sense::hover());
        let band_rect =
            Rect::from_center_size(viewport_rect.center(), Vec2::new(list_width, row_height));
        let painter = ui.painter();
        painter.rect_filled(band_rect.shrink2(Vec2::new(2.0, 1.0)), 14, theme.secondary);
        painter.rect_stroke(
            band_rect.shrink2(Vec2::new(2.0, 1.0)),
            14,
            Stroke::new(1.0, theme.border),
            StrokeKind::Inside,
        );
        if !enabled {
            painter.rect_stroke(
                band_rect,
                4.0,
                Stroke::new(1.0, theme.border),
                StrokeKind::Inside,
            );
        }

        let mut response = ui.interact(
            viewport_rect,
            focus_id,
            if enabled {
                Sense::click()
            } else {
                Sense::hover()
            },
        );
        if response.gained_focus() {
            // The arrow-key focus lock is stored for the next pass; run one more
            // frame so it takes effect before the next key press.
            ui.ctx().request_repaint();
        }

        let scroll_id = id.with("scroll");
        let output = ui
            .scope_builder(
                egui::UiBuilder::new()
                    .max_rect(viewport_rect)
                    .layout(egui::Layout::top_down(egui::Align::Min))
                    .accessibility_parent(focus_id),
                |ui| {
                    ui.set_width(list_width);
                    let mut area = egui::ScrollArea::vertical()
                        .id_salt(scroll_id)
                        .scroll_bar_visibility(ScrollBarVisibility::AlwaysHidden)
                        .auto_shrink([false, false])
                        .max_height(viewport_height)
                        .scroll_source(
                            egui::scroll_area::ScrollSource::MOUSE_WHEEL
                                | egui::scroll_area::ScrollSource::DRAG,
                        );
                    if let Some(offset) = explicit_offset.or(pending_snap) {
                        area = area.vertical_scroll_offset(offset);
                    }
                    area.show(ui, |ui| {
                        // Rows must be exactly `row_height` apart: neither the
                        // layout spacing nor the scroll offset arithmetic may
                        // add a gap.
                        ui.spacing_mut().item_spacing = Vec2::ZERO;
                        ui.set_width(list_width);
                        let pad = (viewport_height - row_height).max(0.0) / 2.0;
                        ui.add_space(pad);
                        for item in items {
                            let (row_rect, _) = ui.allocate_exact_size(
                                Vec2::new(list_width, row_height),
                                Sense::hover(),
                            );
                            let selectable = enabled && item.enabled;
                            let row_response = ui.interact(
                                row_rect,
                                id.with(("row", item.id)),
                                if selectable {
                                    Sense::click()
                                } else {
                                    Sense::hover()
                                },
                            );
                            if selectable && row_response.hovered() {
                                ui.painter().rect_filled(
                                    row_rect.shrink2(Vec2::new(2.0, 1.0)),
                                    3.0,
                                    theme.card,
                                );
                                ui.output_mut(|output| {
                                    output.cursor_icon = egui::CursorIcon::PointingHand;
                                });
                            }
                            let is_selected = *selected == Some(item.id);
                            let distance = ((row_rect.center().y - band_rect.center().y).abs()
                                / row_height)
                                .clamp(0.0, 2.0);
                            let color = if selectable && is_selected {
                                theme.foreground
                            } else {
                                theme.muted_foreground
                            };
                            let press = crate::motion_controls::press_amount(&row_response, theme);
                            ui.painter().text(
                                row_rect.center(),
                                Align2::CENTER_CENTER,
                                item.label,
                                FontId::proportional(
                                    theme.typography.body.size.max(8.0)
                                        * (1.08 - distance * 0.06)
                                        * crate::motion::press_scale(press),
                                ),
                                color,
                            );
                            ui.ctx().accesskit_node_builder(row_response.id, |node| {
                                node.set_role(egui::accesskit::Role::ListBoxOption);
                                node.set_label(item.label);
                                node.set_selected(is_selected);
                                node.set_bounds(accesskit_rect(row_rect));
                                if !selectable {
                                    node.set_disabled();
                                }
                            });
                            if selectable && row_response.clicked() {
                                *selected = Some(item.id);
                                changed = true;
                                ui.memory_mut(|memory| memory.request_focus(focus_id));
                                ui.ctx().request_repaint();
                            }
                        }
                        ui.add_space(pad);
                    })
                },
            )
            .inner;

        // Reconcile the wheel position with the selection. Scrolling selects
        // the nearest enabled row; once the wheel stops, the offset snaps to a
        // whole row so the band stays centred. Both steps only run on change.
        let mut snap_offset = None;
        if enabled && !items.is_empty() {
            let offset = output.state.clamped_offset().y;
            let nearest = (offset / row_height)
                .round()
                .clamp(0.0, (items.len() - 1) as f32) as usize;
            let target = nearest_enabled_index(items, nearest);
            if scrolling_input && *selected != Some(items[target].id) {
                *selected = Some(items[target].id);
                changed = true;
                ui.ctx().request_repaint();
            }
            if !scrolling_input
                && output.state.overscroll().y == 0.0
                && output.state.velocity().y.abs() < 1.0
            {
                let snapped = target as f32 * row_height;
                if (offset - snapped).abs() > 0.5 {
                    snap_offset = Some(snapped);
                    ui.ctx().request_repaint();
                }
            }
        }

        ui.ctx().data_mut(|data| {
            data.insert_temp(
                memory_key,
                PickerMemory {
                    last_selected: *selected,
                    snap_offset,
                },
            );
        });

        let selected_label = selected
            .and_then(|selected| items.iter().find(|item| item.id == selected))
            .map_or("", |item| item.label);
        ui.ctx().accesskit_node_builder(focus_id, |node| {
            node.set_role(egui::accesskit::Role::ListBox);
            node.set_label(label);
            node.set_value(selected_label);
            node.set_bounds(accesskit_rect(viewport_rect));
            if !enabled {
                node.set_disabled();
            } else {
                node.clear_disabled();
            }
        });

        if response.has_focus() {
            ui.painter().rect_stroke(
                band_rect,
                4.0,
                Stroke::new(1.5, theme.foreground),
                StrokeKind::Inside,
            );
        }
        if changed {
            response.mark_changed();
        }
        PickerResponse {
            response,
            selected: *selected,
            changed,
        }
    }
}

fn accesskit_rect(rect: Rect) -> egui::accesskit::Rect {
    egui::accesskit::Rect {
        x0: f64::from(rect.min.x),
        y0: f64::from(rect.min.y),
        x1: f64::from(rect.max.x),
        y1: f64::from(rect.max.y),
    }
}

fn lock_arrows(ui: &Ui, id: Id) {
    ui.memory_mut(|memory| {
        memory.set_focus_lock_filter(
            id,
            egui::EventFilter {
                horizontal_arrows: true,
                vertical_arrows: true,
                ..Default::default()
            },
        );
    });
}

/// Move from `current` to the next enabled item in `delta` direction.
///
/// Returns `current` when no enabled item exists in that direction.
fn step_enabled(
    items: &[PickerItem<'_>],
    current: Option<Id>,
    delta: isize,
    looping: bool,
) -> Option<Id> {
    if delta == 0 || items.is_empty() {
        return current;
    }
    let len = items.len() as isize;
    let step = delta.signum();
    let mut index =
        match current.and_then(|current| items.iter().position(|item| item.id == current)) {
            Some(position) => position as isize,
            None if step > 0 => -1,
            None => len,
        };
    for _ in 0..items.len() {
        index += step;
        if looping {
            index = index.rem_euclid(len);
        } else if index < 0 || index >= len {
            return current;
        }
        let item = &items[index as usize];
        if item.enabled {
            return Some(item.id);
        }
    }
    current
}

/// The enabled row closest to `index`, scanning outwards. Falls back to `index`
/// when every row is disabled.
fn nearest_enabled_index(items: &[PickerItem<'_>], index: usize) -> usize {
    if items.get(index).is_some_and(|item| item.enabled) {
        return index;
    }
    for delta in 1..items.len() {
        if index + delta < items.len() && items[index + delta].enabled {
            return index + delta;
        }
        if index >= delta && items[index - delta].enabled {
            return index - delta;
        }
    }
    index.min(items.len().saturating_sub(1))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn items() -> [PickerItem<'static>; 3] {
        [
            PickerItem::new(Id::unique("a"), "A"),
            PickerItem::new(Id::unique("b"), "B").enabled(false),
            PickerItem::new(Id::unique("c"), "C"),
        ]
    }

    #[test]
    fn keyboard_step_skips_disabled_rows_and_clamps() {
        let items = items();
        assert_eq!(step_enabled(&items, None, 1, false), Some(Id::unique("a")));
        assert_eq!(
            step_enabled(&items, Some(Id::unique("a")), 1, false),
            Some(Id::unique("c"))
        );
        assert_eq!(
            step_enabled(&items, Some(Id::unique("c")), 1, false),
            Some(Id::unique("c"))
        );
        assert_eq!(
            step_enabled(&items, Some(Id::unique("a")), -1, false),
            Some(Id::unique("a"))
        );
        assert_eq!(step_enabled(&items, None, -1, false), Some(Id::unique("c")));
    }

    #[test]
    fn looping_wraps_past_the_ends() {
        let items = items();
        assert_eq!(
            step_enabled(&items, Some(Id::unique("c")), 1, true),
            Some(Id::unique("a"))
        );
        assert_eq!(
            step_enabled(&items, Some(Id::unique("a")), -1, true),
            Some(Id::unique("c"))
        );
    }

    #[test]
    fn nearest_enabled_never_returns_a_disabled_row() {
        let items = items();
        assert_eq!(nearest_enabled_index(&items, 1), 2);
        let all_disabled = [
            PickerItem::new(Id::unique("a"), "A").enabled(false),
            PickerItem::new(Id::unique("b"), "B").enabled(false),
        ];
        assert_eq!(nearest_enabled_index(&all_disabled, 1), 1);
    }
}
