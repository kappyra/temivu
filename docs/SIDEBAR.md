# Sidebar (experimental)

A caller-owned collapsible navigation sidebar. It is a content-only layout meant to be shown inside `Scaffold`'s sidebar slot; the caller owns the collapsed flag and the panel width.

```rust
use temivu::sidebar::{Sidebar, SidebarGroup, SidebarItem, SidebarState};
use temivu::{Scaffold, Theme};

fn example(ui: &mut egui::Ui, theme: &Theme, state: &mut SidebarState, selected: &mut Option<egui::Id>) {
let children = [SidebarItem::new(egui::Id::unique("settings"), "Settings")];
let items = [
    SidebarItem::new(egui::Id::unique("inbox"), "Inbox").icon("▤"),
    SidebarItem::new(egui::Id::unique("projects"), "Projects").children(&children),
];
let groups = [SidebarGroup::new(egui::Id::unique("main"), &items).label("Main")];
let width = if state.collapsed {
    Sidebar::COLLAPSED_WIDTH
} else {
    Sidebar::DEFAULT_WIDTH
};
Scaffold::new(egui::Id::unique("app-shell"), theme)
    .sidebar(width)
    .unwrap()
    .show(
        ui,
        |ui| {
            Sidebar::new(egui::Id::unique("sidebar"), "Main navigation", theme, state)
                .title("Workspace")
                .show(ui, &groups, selected);
        },
        |ui| ui.label("Header"),
        |ui| ui.label("Footer"),
        |ui| ui.label("Content"),
    );
}
```

The sidebar draws a sticky header with an optional title and a collapse toggle, then a scrollable list of labelled groups. Entries with children expand in place and are indented; child expansion is stored in `SidebarState` keyed by stable entry IDs, so it survives insertion and reordering. The selected leaf entry is reported back through `SidebarResponse::selected`, but the selected value itself remains caller-owned (`&mut Option<egui::Id>`).

Width: when collapsed the rail uses `Sidebar::COLLAPSED_WIDTH` (56 points) and hides labels, showing an interim text glyph or the label's first character; the accessible name stays the full label. Otherwise the preferred width defaults to Forui's 256 points. `Sidebar::show` clamps itself to the width the caller gives it, so long labels truncate instead of overflowing a narrow panel; `Scaffold::sidebar` additionally caps the panel at 40% of the window.

Accessibility: the container exposes the tree role with the sidebar's accessible name, each group exposes the group role and its label, entries expose the tree-item role with selected/expanded/disabled state, and the collapse toggle is named "Collapse sidebar" or "Expand sidebar". Keyboard: Up/Down move focus between visible entries, Right/Left expand and collapse an entry with children, and Enter/Space activate the focused entry.

Unsupported or unverified: real iconography (the `icon` slot takes a decorative text glyph), group header action buttons, sticky footer content, swipe/touch gestures, expand/collapse animation, RTL, complete Forui style parity, screen-reader testing and a rendered visual comparison. A collapsible sidebar is an extension beyond upstream Forui, whose sidebar has a fixed width. See `COVERAGE.md` for the current limitations.
