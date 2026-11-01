//! Preliminary grouped selectable tiles with caller-owned selection.
//!
//! [`SelectTileGroup`] draws a labelled list of [`SelectTile`] rows, exposes list
//! and list-item roles with selected state, provides Up/Down navigation between
//! tiles and keeps selection attached to stable tile IDs. Long-press, dividers,
//! scrolling and upstream visual parity remain open; see `COVERAGE.md` for the
//! current limitations.
use crate::{Item, Theme};
use egui::{EventFilter, Id, Key, Response, RichText, Stroke, Ui, UiBuilder};

/// A selectable tile inside a [`SelectTileGroup`].
///
/// The ID must be stable and unique within its group so selection and focus
/// follow the tile when the group is reordered.
#[derive(Clone, Copy, Debug)]
pub struct SelectTile<'a> {
    /// Stable identity of the tile within its group.
    pub id: Id,
    /// Visible title and accessible name.
    pub label: &'a str,
    /// Optional supporting text shown below the title.
    pub description: Option<&'a str>,
    /// Whether this tile can be toggled. Disabled tiles stay visible.
    pub enabled: bool,
}

impl<'a> SelectTile<'a> {
    pub fn new(id: Id, label: &'a str) -> Self {
        Self {
            id,
            label,
            description: None,
            enabled: true,
        }
    }

    pub fn description(mut self, description: &'a str) -> Self {
        self.description = Some(description);
        self
    }

    pub fn enabled(mut self, enabled: bool) -> Self {
        self.enabled = enabled;
        self
    }
}

/// The result of showing a [`SelectTileGroup`].
pub struct SelectTileGroupResponse {
    /// The group container's response.
    pub response: Response,
    /// Whether an activation toggled a tile this frame.
    pub changed: bool,
    /// The tile that was toggled this frame, if any.
    pub activated: Option<Id>,
}

/// A labelled group of tiles with multi-selection stored in a caller-owned set.
///
/// Selection is a caller-owned `Vec` of the selected tile IDs; passing the same
/// vector every frame preserves selection across reordering. Draw the group with
/// [`SelectTileGroup::show`]; a click, Enter or Space toggles membership.
///
/// Keyboard: Up/Down move focus between enabled tiles, Home/End jump to the
/// first/last enabled tile, and Enter/Space toggle the focused tile.
///
/// ```
/// # fn example(ui: &mut egui::Ui, theme: &temivu::Theme, selected: &mut Vec<egui::Id>) {
/// use temivu::{SelectTile, SelectTileGroup};
/// let tiles = [
///     SelectTile::new(egui::Id::new("wifi"), "Wi-Fi").description("Wireless networks"),
///     SelectTile::new(egui::Id::new("bluetooth"), "Bluetooth"),
/// ];
/// let _ = SelectTileGroup::new(egui::Id::new("radios"), "Radios", theme, &tiles, selected)
///     .description("Choose any that apply")
///     .show(ui);
/// # }
/// ```
pub struct SelectTileGroup<'a> {
    id: Id,
    label: &'a str,
    theme: &'a Theme,
    description: Option<&'a str>,
    enabled: bool,
    tiles: &'a [SelectTile<'a>],
    selected: &'a mut Vec<Id>,
}

impl<'a> SelectTileGroup<'a> {
    /// Creates a group. `selected` is the caller-owned multi-selection and is
    /// updated when a tile is toggled.
    pub fn new(
        id: Id,
        label: &'a str,
        theme: &'a Theme,
        tiles: &'a [SelectTile<'a>],
        selected: &'a mut Vec<Id>,
    ) -> Self {
        Self {
            id,
            label,
            theme,
            description: None,
            enabled: true,
            tiles,
            selected,
        }
    }

    /// Supporting text shown below the label and exposed as the group's
    /// description.
    pub fn description(mut self, description: &'a str) -> Self {
        self.description = Some(description);
        self
    }

    /// Disables every tile in the group. The inherited egui enabled state also
    /// applies, so a disabled parent UI disables the group.
    pub fn enabled(mut self, enabled: bool) -> Self {
        self.enabled = enabled;
        self
    }

    /// Draws the group. Call it every frame.
    pub fn show(self, ui: &mut Ui) -> SelectTileGroupResponse {
        let Self {
            id,
            label,
            theme,
            description,
            enabled,
            tiles,
            selected,
        } = self;
        let enabled = enabled && ui.is_enabled();
        let mut rows = Vec::with_capacity(tiles.len());
        let mut changed = false;
        let mut activated = None;
        let scope = ui.scope_builder(UiBuilder::new().id(id), |ui| {
            ui.vertical(|ui| {
                ui.spacing_mut().item_spacing.y = 2.0;
                ui.add(
                    egui::Label::new(
                        RichText::new(label)
                            .color(theme.foreground)
                            .strong()
                            .size(theme.typography.label.size),
                    )
                    .truncate(),
                );
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
                ui.add_space(2.0);
                for (index, tile) in tiles.iter().enumerate() {
                    let is_selected = selected.contains(&tile.id);
                    let tile_enabled = enabled && tile.enabled;
                    let mut item = Item::new(tile.label, theme)
                        .selected(is_selected)
                        .enabled(tile_enabled)
                        .id(id.with(("tile", tile.id)));
                    if let Some(description) = tile.description {
                        item = item.description(description);
                    }
                    let response = item.show(ui);
                    ui.ctx().accesskit_node_builder(response.id, |node| {
                        node.set_role(egui::accesskit::Role::ListItem);
                        node.set_label(tile.label);
                        node.set_selected(is_selected);
                        node.set_position_in_set(index);
                        node.set_size_of_set(tiles.len());
                    });
                    if response.has_focus() {
                        ui.memory_mut(|memory| {
                            memory.set_focus_lock_filter(
                                response.id,
                                EventFilter {
                                    horizontal_arrows: true,
                                    vertical_arrows: true,
                                    ..Default::default()
                                },
                            );
                        });
                        ui.painter().rect_stroke(
                            response.rect.expand(1.0),
                            6,
                            Stroke::new(2.0, theme.foreground),
                            egui::StrokeKind::Outside,
                        );
                    }
                    if tile_enabled && response.clicked() {
                        if is_selected {
                            selected.retain(|&selected| selected != tile.id);
                        } else {
                            selected.push(tile.id);
                        }
                        changed = true;
                        activated = Some(tile.id);
                        ui.ctx().request_repaint();
                    }
                    rows.push((response.id, tile_enabled));
                }
                keyboard_navigation(ui, &rows);
            });
        });
        let response = scope.response;
        ui.ctx().accesskit_node_builder(id, |node| {
            node.set_role(egui::accesskit::Role::List);
            node.set_label(label);
            if let Some(description) = description {
                node.set_description(description);
            }
            node.set_size_of_set(tiles.len());
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
        SelectTileGroupResponse {
            response,
            changed,
            activated,
        }
    }
}

/// Up/Down move focus between enabled tiles; Home/End jump to the ends.
fn keyboard_navigation(ui: &mut Ui, rows: &[(Id, bool)]) {
    let Some(focused) = ui.memory(|memory| memory.focused()) else {
        return;
    };
    let Some(index) = rows.iter().position(|(id, _)| *id == focused) else {
        return;
    };
    let (down, up, home, end) = ui.input(|input| {
        (
            input.key_pressed(Key::ArrowDown),
            input.key_pressed(Key::ArrowUp),
            input.key_pressed(Key::Home),
            input.key_pressed(Key::End),
        )
    });
    if !(down || up || home || end) {
        return;
    }
    let count = rows.len();
    let enabled_rows: Vec<usize> = (0..count).filter(|&index| rows[index].1).collect();
    if enabled_rows.is_empty() {
        return;
    }
    let target_index = if home {
        enabled_rows[0]
    } else if end {
        enabled_rows[enabled_rows.len() - 1]
    } else {
        let position = enabled_rows
            .iter()
            .position(|&enabled| enabled == index)
            .unwrap_or(0);
        let next = if down {
            (position + 1) % enabled_rows.len()
        } else {
            (position + enabled_rows.len() - 1) % enabled_rows.len()
        };
        enabled_rows[next]
    };
    let target = rows[target_index].0;
    if target != focused {
        ui.ctx().memory_mut(|memory| memory.request_focus(target));
        ui.ctx().request_repaint();
    }
}
