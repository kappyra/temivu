//! Experimental textual tile, built on the preliminary Item implementation.
//! Rich slots, groups, long press, and upstream style parity remain open.
use crate::item::ItemAppearance;
use crate::{Item, ItemVariant, Theme, Vesper};
use egui::{Id, Response, Ui};

/// A textual, caller-controlled tile with a minimum 48-point interaction height.
///
/// Supply a unique stable ID when tiles can be inserted or reordered so focus
/// remains with the same tile. A click is returned in the egui response.
///
/// ```
/// # fn example(ui: &mut egui::Ui, selected: &mut bool) {
/// use temivu::{Theme, Tile};
/// let theme = Theme::light();
/// if Tile::new("Bluetooth", &theme)
///     .id(egui::Id::new("bluetooth"))
///     .description("Connect devices")
///     .selected(*selected)
///     .show(ui)
///     .clicked()
/// {
///     *selected = !*selected;
/// }
/// # }
/// ```
pub struct Tile<'a> {
    item: Item<'a>,
}

impl<'a> Tile<'a> {
    pub fn new(title: &'a str, theme: &'a Theme) -> Self {
        Self {
            item: Item::new(title, theme).appearance(ItemAppearance::tile(theme)),
        }
    }
    pub fn description(mut self, description: &'a str) -> Self {
        self.item = self.item.description(description);
        self
    }
    pub fn icon(mut self, icon: &'a str) -> Self {
        self.item = self.item.icon(icon);
        self
    }
    pub fn radius(mut self, radius: u8) -> Self {
        self.item = self.item.radius(radius);
        self
    }
    pub fn lighting(mut self, lighting: Option<Vesper>) -> Self {
        self.item = self.item.lighting(lighting);
        self
    }
    pub fn variant(mut self, variant: ItemVariant) -> Self {
        self.item = self.item.variant(variant);
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
    pub fn id(mut self, id: Id) -> Self {
        self.item = self.item.id(id);
        self
    }
    pub fn show(self, ui: &mut Ui) -> Response {
        self.item.show(ui)
    }
}
