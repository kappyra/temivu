//! Preliminary popover menu. Submenus, scrolling, hover-to-focus and full
//! parity remain open.
use crate::{Item, Popover, SelectMenuTile};
use egui::{Id, Response, Ui};

/// The items drawn in a [`PopoverMenu`], in reading order.
///
/// Draw every item through [`MenuItems::item`] so the menu can expose the
/// menu-item role and focus the first enabled item when it opens.
#[derive(Default)]
pub struct MenuItems {
    first_enabled: Option<Response>,
}

impl MenuItems {
    pub fn new() -> Self {
        Self::default()
    }

    /// Draw an [`Item`] as a menu item.
    ///
    /// Check [`Response::clicked`] and call [`Ui::close`] after acting on the
    /// item to dismiss the menu.
    pub fn item(&mut self, ui: &mut Ui, item: Item<'_>) -> Response {
        let enabled = item.is_enabled();
        let response = item.show(ui);
        ui.ctx().accesskit_node_builder(response.id, |node| {
            node.set_role(egui::accesskit::Role::MenuItem);
        });
        if enabled && self.first_enabled.is_none() {
            self.first_enabled = Some(response.clone());
        }
        response
    }

    /// Draw a [`SelectMenuTile`] as a selectable menu item.
    ///
    /// The tile exposes the menu-item role and its selected state. Check
    /// [`Response::clicked`] and call [`Ui::close`] after acting on it.
    pub fn select_menu_tile(&mut self, ui: &mut Ui, tile: SelectMenuTile<'_>) -> Response {
        let enabled = tile.is_enabled();
        let response = tile.show(ui);
        if enabled && self.first_enabled.is_none() {
            self.first_enabled = Some(response.clone());
        }
        response
    }
}

/// A caller-controlled menu anchored to a trigger.
///
/// The menu reuses [`Popover`] for anchoring, pointer dismissal and focus
/// restoration. It reports the menu role and focuses the first enabled item
/// when the trigger is keyboard-focused and the menu opens. Items must be
/// drawn through [`MenuItems::item`]. IDs must be stable and globally unique;
/// pass a stable [`Item::id`] to preserve item focus when the menu's content
/// changes.
///
/// ```
/// # fn example(ui: &mut egui::Ui, theme: &temivu::Theme, open: &mut bool) {
/// use temivu::{Item, PopoverMenu};
/// let anchor = ui.button("Actions");
/// if anchor.clicked() {
///     *open = !*open;
/// }
/// PopoverMenu::new(egui::Id::new("actions-menu"), "Actions", open).show(
///     &anchor,
///     |ui, menu| {
///         if menu.item(ui, Item::new("Rename", theme)).clicked() {
///             ui.close();
///         }
///         let _ = menu.item(ui, Item::new("Delete", theme).enabled(false));
///     },
/// );
/// # }
/// ```
pub struct PopoverMenu<'a> {
    id: Id,
    label: &'a str,
    open: &'a mut bool,
    width: f32,
}

impl<'a> PopoverMenu<'a> {
    pub fn new(id: Id, label: &'a str, open: &'a mut bool) -> Self {
        Self {
            id,
            label,
            open,
            width: 200.0,
        }
    }

    /// The menu's content width. Defaults to 200 points; Forui's reference
    /// range is 150 to 250. There is no intrinsic sizing yet.
    pub fn width(mut self, width: f32) -> Self {
        self.width = width;
        self
    }

    /// Shows the menu and returns the popup response while it is open.
    ///
    /// `open` is updated by pointer dismissal, Escape and [`Ui::close`].
    pub fn show(
        self,
        anchor: &Response,
        content: impl FnOnce(&mut Ui, &mut MenuItems),
    ) -> Option<Response> {
        let Self {
            id,
            label,
            open,
            width,
        } = self;
        let mut items = MenuItems::new();
        let shown = Popover::new(id, label, open)
            .focus_on_open(true)
            .show(anchor, |ui| {
                ui.set_width(width.max(1.0));
                content(ui, &mut items);
                items.first_enabled.take().unwrap_or_else(|| ui.response())
            })?;
        let popup = shown.response;
        anchor.ctx.accesskit_node_builder(popup.id, |node| {
            node.set_role(egui::accesskit::Role::Menu);
            node.set_label(label);
        });
        Some(popup)
    }
}
