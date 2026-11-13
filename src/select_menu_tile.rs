//! Preliminary tile-shaped selectable menu row.
//!
//! A [`SelectMenuTile`] is the select-family building block for menus and
//! popovers: a full-width, tile-height row with an optional interim icon glyph,
//! a title, an optional description and caller-owned selected/disabled state.
//! It exposes the menu-item role and its selected state.
//!
//! The upstream `FSelectMenuTile` is a tile that opens a popover containing a
//! [`crate::select_tile_group`]. That popover orchestration, the menu's scroll
//! area, dividers, intrinsic sizing and the tile group are separate components
//! here; see `COVERAGE.md` for the current limitations.
use crate::item::ItemAppearance;
use crate::{Item, Theme};
use egui::{Id, Response, Ui};

/// A tile-shaped, selectable menu row.
///
/// Activating the row is reported through [`Response::clicked`]; the caller owns
/// the selection and can close the enclosing menu with [`Ui::close`]. Disabled
/// rows cannot be activated and expose the disabled state to access tools.
///
/// ```
/// # fn example(ui: &mut egui::Ui, theme: &temivu::Theme) {
/// use temivu::SelectMenuTile;
/// let mut selected = None;
/// let response = SelectMenuTile::new("Compact", theme)
///     .id(egui::Id::unique("density-compact"))
///     .icon("=")
///     .description("Show more rows")
///     .selected(selected == Some(egui::Id::unique("density-compact")))
///     .show(ui);
/// if response.clicked() {
///     selected = Some(egui::Id::unique("density-compact"));
///     ui.close();
/// }
/// # }
/// ```
pub struct SelectMenuTile<'a> {
    item: Item<'a>,
    theme: &'a Theme,
}

impl<'a> SelectMenuTile<'a> {
    pub fn new(title: &'a str, theme: &'a Theme) -> Self {
        Self {
            item: Item::new(title, theme),
            theme,
        }
    }

    /// An interim text glyph shown before the title. Decorative: it is not part
    /// of the accessible name.
    pub fn icon(mut self, icon: &'a str) -> Self {
        self.item = self.item.icon(icon);
        self
    }

    pub fn description(mut self, description: &'a str) -> Self {
        self.item = self.item.description(description);
        self
    }

    pub fn selected(mut self, selected: bool) -> Self {
        self.item = self.item.selected(selected);
        self
    }

    pub fn enabled(mut self, enabled: bool) -> Self {
        self.item = self.item.enabled(enabled);
        self
    }

    /// Use a stable, globally unique ID so focus follows the row when the menu's
    /// content changes.
    pub fn id(mut self, id: Id) -> Self {
        self.item = self.item.id(id);
        self
    }

    /// Whether this row is enabled. Unlike [`Response::enabled`], this is
    /// accurate during egui's invisible popup sizing pass.
    pub fn is_enabled(&self) -> bool {
        self.item.is_enabled()
    }

    /// Whether this row is shown as selected.
    pub fn is_selected(&self) -> bool {
        self.item.is_selected()
    }

    /// Shows the row and returns its response.
    ///
    /// The row uses Forui's tile appearance: card background, 1-point border,
    /// 15/14.5 content padding and the 48-point tile height.
    pub fn show(self, ui: &mut Ui) -> Response {
        let response = self
            .item
            .appearance(ItemAppearance::tile(self.theme))
            .show(ui);
        ui.ctx().accesskit_node_builder(response.id, |node| {
            node.set_role(egui::accesskit::Role::MenuItem);
        });
        response
    }
}
