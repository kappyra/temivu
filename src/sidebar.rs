//! Preliminary caller-owned sidebar for navigation layouts.
//!
//! The sidebar is a content-only layout intended for [`crate::Scaffold`]'s
//! sidebar slot, not a replacement for the scaffold. It draws a sticky header
//! with a collapse toggle, a scrollable list of labelled item groups and
//! optional nested items, and clamps its own width to the space it is given.
//!
//! Icons are rendered as caller-supplied text glyphs (an interim placeholder),
//! groups have no action button, and entrance/collapse animations are not
//! ported. Screen-reader verification and upstream visual parity remain open;
//! see `COVERAGE.md` for the current limitations.
use crate::Theme;
use egui::{EventFilter, Id, Key, Response, RichText, ScrollArea, Sense, Ui, UiBuilder};

/// The preferred expanded width of a [`Sidebar`], in points. Forui's sidebar
/// style uses a fixed 256-point width.
pub const DEFAULT_WIDTH: f32 = 256.0;

/// The width used for a collapsed sidebar, in points. This is an extension
/// beyond upstream Forui, whose sidebar does not collapse.
pub const COLLAPSED_WIDTH: f32 = 56.0;

/// Caller-owned sidebar state: the collapsed flag and which items with children
/// are expanded.
///
/// Pass the same state to [`Sidebar::show`] every frame. Expansion is keyed by
/// the item's stable [`Id`], so it survives item insertion and reordering.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct SidebarState {
    /// Whether the sidebar is collapsed into a narrow, label-free rail.
    pub collapsed: bool,
    expanded: Vec<Id>,
}

impl SidebarState {
    pub fn new() -> Self {
        Self::default()
    }

    /// Whether the item with `id` currently shows its children.
    pub fn is_expanded(&self, id: Id) -> bool {
        self.expanded.contains(&id)
    }

    /// Expand or collapse an item with children. Returns whether the state
    /// changed.
    pub fn set_expanded(&mut self, id: Id, expanded: bool) -> bool {
        if expanded == self.is_expanded(id) {
            return false;
        }
        if expanded {
            self.expanded.push(id);
        } else {
            self.expanded.retain(|&expanded| expanded != id);
        }
        true
    }

    /// Toggle an item with children. Returns whether the state changed.
    pub fn toggle_expanded(&mut self, id: Id) -> bool {
        let expanded = self.is_expanded(id);
        self.set_expanded(id, !expanded)
    }
}

/// A sidebar entry. Items with children are collapsible parents; activating
/// one toggles its children instead of selecting it.
///
/// IDs must be stable and unique within the sidebar. Supply a new ID only when
/// the logical entry changes identity, so focus and expansion stay attached to
/// the same entry across insertion and reordering.
#[derive(Clone, Copy, Debug)]
pub struct SidebarItem<'a> {
    /// Stable identity of the entry within the sidebar.
    pub id: Id,
    /// Visible label and accessible name.
    pub label: &'a str,
    /// An interim text glyph rendered before the label, for example `"▤"`.
    /// Real iconography is not migrated.
    pub icon: Option<&'a str>,
    /// Whether the entry can be activated. Disabled entries stay visible.
    pub enabled: bool,
    /// Child entries, shown when the parent is expanded and the sidebar is open.
    pub children: &'a [SidebarItem<'a>],
}

impl<'a> SidebarItem<'a> {
    pub fn new(id: Id, label: &'a str) -> Self {
        Self {
            id,
            label,
            icon: None,
            enabled: true,
            children: &[],
        }
    }

    /// An interim text glyph shown before the label. Decorative: it is not part
    /// of the accessible name.
    pub fn icon(mut self, icon: &'a str) -> Self {
        self.icon = Some(icon);
        self
    }

    pub fn enabled(mut self, enabled: bool) -> Self {
        self.enabled = enabled;
        self
    }

    pub fn children(mut self, children: &'a [SidebarItem<'a>]) -> Self {
        self.children = children;
        self
    }
}

/// A labelled group of sidebar items.
///
/// The group ID must be stable and unique within the sidebar; it identifies the
/// accessible group node. A group header action button is not implemented.
#[derive(Clone, Copy, Debug)]
pub struct SidebarGroup<'a> {
    /// Stable identity of the group within the sidebar.
    pub id: Id,
    /// Optional visible and accessible heading for the group.
    pub label: Option<&'a str>,
    /// The group's items, in reading order.
    pub items: &'a [SidebarItem<'a>],
}

impl<'a> SidebarGroup<'a> {
    pub fn new(id: Id, items: &'a [SidebarItem<'a>]) -> Self {
        Self {
            id,
            label: None,
            items,
        }
    }

    pub fn label(mut self, label: &'a str) -> Self {
        self.label = Some(label);
        self
    }
}

/// The result of showing a [`Sidebar`].
pub struct SidebarResponse {
    /// The sidebar container's response. Its rectangle is clamped to the space
    /// the sidebar was given.
    pub response: Response,
    /// The item selected by an activation this frame, if any.
    pub selected: Option<Id>,
    /// The item whose expansion changed through activation or keyboard this
    /// frame, if any.
    pub toggled: Option<Id>,
    /// Whether the collapse toggle was activated this frame.
    pub collapsed_changed: bool,
}

/// A caller-owned, collapsible navigation sidebar.
///
/// Show it inside [`crate::Scaffold`]'s sidebar slot. The caller owns the panel
/// width: read [`SidebarState::collapsed`] and pass
/// [`Sidebar::recommended_width`] to [`crate::Scaffold::sidebar`]. The sidebar
/// clamps its content to the width it is actually given, so long labels
/// truncate instead of overflowing a narrow window.
///
/// Keyboard: Up/Down move focus between visible entries, Right/Left expand and
/// collapse an entry with children, and Enter/Space activate the focused entry.
/// The header toggle exposes "Collapse sidebar"/"Expand sidebar" as its
/// accessible name.
///
/// ```
/// # fn example(ui: &mut egui::Ui, theme: &temivu::Theme, state: &mut temivu::sidebar::SidebarState) {
/// use temivu::sidebar::{Sidebar, SidebarGroup, SidebarItem};
/// let mut selected = None;
/// let inbox = [SidebarItem::new(egui::Id::new("inbox"), "Inbox")];
/// let groups = [SidebarGroup::new(egui::Id::new("mail"), &inbox).label("Mail")];
/// let width = if state.collapsed {
///     Sidebar::COLLAPSED_WIDTH
/// } else {
///     Sidebar::DEFAULT_WIDTH
/// };
/// // `Scaffold::new(...).sidebar(width)` then show inside that slot:
/// let response = Sidebar::new(egui::Id::new("sidebar"), "Main navigation", theme, state)
///     .show(ui, &groups, &mut selected);
/// assert!(response.response.rect.width() <= width);
/// # }
/// ```
pub struct Sidebar<'a> {
    id: Id,
    label: &'a str,
    theme: &'a Theme,
    state: &'a mut SidebarState,
    title: Option<&'a str>,
    width: f32,
    collapsed_width: f32,
    framed: bool,
}

impl<'a> Sidebar<'a> {
    /// The preferred expanded width, in points.
    pub const DEFAULT_WIDTH: f32 = DEFAULT_WIDTH;
    /// The width of a collapsed sidebar, in points.
    pub const COLLAPSED_WIDTH: f32 = COLLAPSED_WIDTH;

    /// Creates a sidebar. `label` is its accessible name.
    pub fn new(id: Id, label: &'a str, theme: &'a Theme, state: &'a mut SidebarState) -> Self {
        Self {
            id,
            label,
            theme,
            state,
            title: None,
            width: DEFAULT_WIDTH,
            collapsed_width: COLLAPSED_WIDTH,
            framed: true,
        }
    }

    /// The optional visible header title. Hidden while collapsed.
    pub fn title(mut self, title: &'a str) -> Self {
        self.title = Some(title);
        self
    }

    /// The preferred expanded width in points. Defaults to [`DEFAULT_WIDTH`].
    ///
    /// Non-finite or non-positive values are ignored so they cannot poison the
    /// layout.
    pub fn width(mut self, width: f32) -> Self {
        if width.is_finite() && width > 0.0 {
            self.width = width;
        }
        self
    }

    /// The collapsed width in points. Defaults to [`COLLAPSED_WIDTH`].
    ///
    /// Non-finite or non-positive values are ignored so they cannot poison the
    /// layout.
    pub fn collapsed_width(mut self, width: f32) -> Self {
        if width.is_finite() && width > 0.0 {
            self.collapsed_width = width;
        }
        self
    }

    /// Give the sidebar its own background. Turn off inside a styled panel.
    pub fn framed(mut self, framed: bool) -> Self {
        self.framed = framed;
        self
    }

    /// The width this sidebar recommends for its current collapsed state.
    pub fn recommended_width(state: &SidebarState) -> f32 {
        if state.collapsed {
            COLLAPSED_WIDTH
        } else {
            DEFAULT_WIDTH
        }
    }

    /// Shows the sidebar and updates `selected` when the user activates a leaf
    /// entry. Call it every frame.
    pub fn show(
        self,
        ui: &mut Ui,
        groups: &[SidebarGroup<'_>],
        selected: &mut Option<Id>,
    ) -> SidebarResponse {
        let Self {
            id,
            label,
            theme,
            state,
            title,
            width,
            collapsed_width,
            framed,
        } = self;
        let collapsed = state.collapsed;
        let enabled = ui.is_enabled();
        let requested = if collapsed { collapsed_width } else { width };
        // Clamp to the space the caller gave us: narrow windows truncate
        // labels instead of overflowing the panel.
        let width = requested.min(ui.available_width().max(1.0));
        let mut entries = Vec::new();
        let mut activated = None;
        let mut toggled = None;
        let mut collapsed_changed = false;
        let scope = ui.scope_builder(UiBuilder::new().id(id).sense(Sense::hover()), |ui| {
            ui.set_width(width);
            let margin = if framed { 7 } else { 0 };
            egui::Frame::new()
                .fill(if framed {
                    theme.muted
                } else {
                    egui::Color32::TRANSPARENT
                })
                .corner_radius(14)
                .inner_margin(margin)
                .show(ui, |ui| {
                    ui.set_width((width - 2.0 * f32::from(margin)).max(1.0));
                    ui.spacing_mut().item_spacing.y = 4.0;
                    collapsed_changed = header(ui, title, theme, state, collapsed, enabled);
                    ui.add_space(4.0);
                    ScrollArea::vertical()
                        .auto_shrink([false, !framed])
                        .show(ui, |ui| {
                            for group in groups {
                                draw_group(
                                    ui,
                                    id,
                                    theme,
                                    state,
                                    collapsed,
                                    enabled,
                                    group,
                                    selected,
                                    &mut activated,
                                    &mut toggled,
                                    &mut entries,
                                );
                            }
                            keyboard_navigation(ui, state, &entries, &mut toggled);
                        });
                });
        });
        let response = scope.response;
        ui.ctx().accesskit_node_builder(id, |node| {
            node.set_role(egui::accesskit::Role::Tree);
            node.set_label(label);
            node.set_bounds(accesskit_rect(response.rect));
            if !enabled {
                node.set_disabled();
            }
        });
        SidebarResponse {
            response,
            selected: activated,
            toggled,
            collapsed_changed,
        }
    }
}

/// Draws the sticky header row and returns whether the collapse toggle was
/// activated.
fn header(
    ui: &mut Ui,
    title: Option<&str>,
    theme: &Theme,
    state: &mut SidebarState,
    collapsed: bool,
    enabled: bool,
) -> bool {
    let toggle = if collapsed { "+" } else { "−" };
    let toggle_label = if collapsed {
        "Expand sidebar"
    } else {
        "Collapse sidebar"
    };
    let (_, toggle_response) = egui::Sides::new().shrink_left().height(32.0).show(
        ui,
        |ui| {
            if !collapsed && let Some(title) = title {
                ui.add(
                    egui::Label::new(
                        RichText::new(title)
                            .color(theme.foreground)
                            .strong()
                            .size(theme.typography.label.size),
                    )
                    .truncate(),
                );
            }
        },
        |ui| {
            crate::Button::new(toggle, theme)
                .variant(crate::ButtonVariant::Outline)
                .size(crate::ButtonSize::ExtraSmall)
                .enabled(enabled)
                .show(ui)
        },
    );
    ui.ctx().accesskit_node_builder(toggle_response.id, |node| {
        node.set_label(toggle_label);
    });
    let mut changed = false;
    if toggle_response.clicked() {
        state.collapsed = !state.collapsed;
        changed = true;
        ui.ctx().request_repaint();
    }
    changed
}

/// Draws a labelled group of items and registers its accessible group node.
#[allow(clippy::too_many_arguments)]
fn draw_group(
    ui: &mut Ui,
    sidebar_id: Id,
    theme: &Theme,
    state: &mut SidebarState,
    collapsed: bool,
    enabled: bool,
    group: &SidebarGroup<'_>,
    selected: &mut Option<Id>,
    activated: &mut Option<Id>,
    toggled: &mut Option<Id>,
    entries: &mut Vec<ItemEntry>,
) {
    let group_id = sidebar_id.with(("sidebar-group", group.id));
    let scope = ui.scope_builder(UiBuilder::new().id(group_id), |ui| {
        if !collapsed {
            if let Some(label) = group.label {
                ui.add(
                    egui::Label::new(
                        RichText::new(label)
                            .color(theme.muted_foreground)
                            .size(theme.typography.caption.size),
                    )
                    .truncate(),
                );
                ui.add_space(2.0);
            }
        } else if group.label.is_some() {
            ui.add_space(4.0);
        }
        for item in group.items {
            draw_item(
                ui, sidebar_id, theme, state, collapsed, enabled, item, selected, activated,
                toggled, entries,
            );
        }
    });
    ui.ctx().accesskit_node_builder(group_id, |node| {
        node.set_role(egui::accesskit::Role::Group);
        node.set_bounds(accesskit_rect(scope.response.rect));
        if let Some(label) = group.label {
            node.set_label(label);
        }
    });
}

/// A visible entry, in reading order, used for keyboard navigation.
struct ItemEntry {
    row_id: Id,
    item_id: Id,
    enabled: bool,
    has_children: bool,
    expanded: bool,
}

#[allow(clippy::too_many_arguments)]
fn draw_item(
    ui: &mut Ui,
    sidebar_id: Id,
    theme: &Theme,
    state: &mut SidebarState,
    collapsed: bool,
    enabled: bool,
    item: &SidebarItem<'_>,
    selected: &mut Option<Id>,
    activated: &mut Option<Id>,
    toggled: &mut Option<Id>,
    entries: &mut Vec<ItemEntry>,
) {
    let has_children = !item.children.is_empty();
    let expanded = has_children && state.is_expanded(item.id);
    let item_is_selected = *selected == Some(item.id);
    let item_enabled = enabled && item.enabled;

    let text = item_text(item, collapsed, has_children, expanded);
    let response = crate::Item::new(&text, theme)
        .id(sidebar_id.with(("sidebar-item", item.id)))
        .flat(true)
        .radius(12)
        .enabled(item_enabled)
        .selected(item_is_selected)
        .show(ui);
    let reveal = crate::motion::spring_bool(
        ui.ctx(),
        response.id.with("sidebar-indicator"),
        item_is_selected,
        theme,
    )
    .clamp(0.0, 1.0);
    if reveal > 0.001 {
        let edge = if ui.layout().prefer_right_to_left() {
            response.rect.right() - 5.0
        } else {
            response.rect.left() + 5.0
        };
        let mark = egui::Rect::from_center_size(
            egui::pos2(edge, response.rect.center().y),
            egui::vec2(3.0, 17.0 * reveal),
        );
        ui.painter()
            .rect_filled(mark, 2, theme.primary.gamma_multiply(reveal));
    }
    ui.ctx().accesskit_node_builder(response.id, |node| {
        node.set_role(egui::accesskit::Role::TreeItem);
        node.set_label(item.label);
        node.set_selected(item_is_selected);
        if has_children {
            node.set_expanded(expanded);
        } else {
            node.clear_expanded();
        }
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
    }
    entries.push(ItemEntry {
        row_id: response.id,
        item_id: item.id,
        enabled: item_enabled,
        has_children,
        expanded,
    });

    if item_enabled && response.clicked() {
        if has_children {
            if state.toggle_expanded(item.id) {
                *toggled = Some(item.id);
            }
        } else {
            *selected = Some(item.id);
            *activated = Some(item.id);
        }
        ui.ctx().request_repaint();
    }

    if expanded && !collapsed && !item.children.is_empty() {
        ui.indent(("sidebar-children", item.id), |ui| {
            for child in item.children {
                draw_item(
                    ui, sidebar_id, theme, state, collapsed, enabled, child, selected, activated,
                    toggled, entries,
                );
            }
        });
    }
}

/// The visible row text. Collapsed rows show only the icon glyph or the label's
/// first character, so the rail stays narrow; the accessible name is always the
/// full label.
fn item_text(
    item: &SidebarItem<'_>,
    collapsed: bool,
    has_children: bool,
    expanded: bool,
) -> String {
    let base = if collapsed {
        match item.icon {
            Some(icon) => icon.to_owned(),
            None => item
                .label
                .chars()
                .next()
                .map(|first| first.to_uppercase().collect())
                .unwrap_or_default(),
        }
    } else {
        match item.icon {
            Some(icon) => format!("{icon}  {}", item.label),
            None => item.label.to_owned(),
        }
    };
    if collapsed || !has_children {
        base
    } else if expanded {
        format!("{base}   ▾")
    } else {
        format!("{base}   ▸")
    }
}

/// Up/Down move focus between visible entries, Right/Left expand and collapse
/// entries with children.
fn keyboard_navigation(
    ui: &mut Ui,
    state: &mut SidebarState,
    entries: &[ItemEntry],
    toggled: &mut Option<Id>,
) {
    let Some(focused) = ui.memory(|memory| memory.focused()) else {
        return;
    };
    let Some(index) = entries.iter().position(|entry| entry.row_id == focused) else {
        return;
    };
    let (down, up, right, left) = ui.input(|input| {
        (
            input.key_pressed(Key::ArrowDown),
            input.key_pressed(Key::ArrowUp),
            input.key_pressed(Key::ArrowRight),
            input.key_pressed(Key::ArrowLeft),
        )
    });
    if up || down {
        let count = entries.len();
        for step in 1..=count {
            let candidate = if down {
                (index + step) % count
            } else {
                (index + count - step) % count
            };
            if entries[candidate].enabled {
                ui.ctx()
                    .memory_mut(|memory| memory.request_focus(entries[candidate].row_id));
                ui.ctx().request_repaint();
                return;
            }
        }
    }
    let entry = &entries[index];
    if !entry.has_children {
        return;
    }
    let expand = right && !entry.expanded;
    let collapse = left && entry.expanded;
    if (expand || collapse) && state.set_expanded(entry.item_id, expand) {
        *toggled = Some(entry.item_id);
        ui.ctx().request_repaint();
    }
}

fn accesskit_rect(rect: egui::Rect) -> egui::accesskit::Rect {
    egui::accesskit::Rect {
        x0: f64::from(rect.min.x),
        y0: f64::from(rect.min.y),
        x1: f64::from(rect.max.x),
        y1: f64::from(rect.max.y),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn width_builders_ignore_invalid_values() {
        let theme = Theme::light();
        let mut state = SidebarState::new();
        {
            let sidebar = Sidebar::new(Id::new("sidebar"), "Navigation", &theme, &mut state)
                .width(f32::NAN)
                .width(0.0)
                .width(-12.0)
                .collapsed_width(f32::INFINITY)
                .collapsed_width(0.0);
            assert_eq!(sidebar.width, DEFAULT_WIDTH);
            assert_eq!(sidebar.collapsed_width, COLLAPSED_WIDTH);
        }
        let sidebar = Sidebar::new(Id::new("sidebar"), "Navigation", &theme, &mut state)
            .width(320.0)
            .collapsed_width(48.0);
        assert_eq!(sidebar.width, 320.0);
        assert_eq!(sidebar.collapsed_width, 48.0);
    }

    #[test]
    fn recommended_width_tracks_the_collapsed_state() {
        let mut state = SidebarState::new();
        assert_eq!(Sidebar::recommended_width(&state), DEFAULT_WIDTH);
        state.collapsed = true;
        assert_eq!(Sidebar::recommended_width(&state), COLLAPSED_WIDTH);
    }

    #[test]
    fn expansion_state_follows_stable_item_ids() {
        let mut state = SidebarState::new();
        let projects = Id::new("projects");
        assert!(!state.is_expanded(projects));
        assert!(state.toggle_expanded(projects));
        assert!(state.is_expanded(projects));
        assert!(!state.set_expanded(projects, true));
        assert!(state.set_expanded(projects, false));
        assert!(!state.is_expanded(projects));
    }
}
