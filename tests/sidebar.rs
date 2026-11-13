use egui_kittest::{
    Harness,
    kittest::{NodeT, Queryable},
};
use temivu::{Scaffold, Sidebar, SidebarGroup, SidebarItem, SidebarState, Theme};

#[derive(Default)]
struct State {
    sidebar: SidebarState,
    selected: Option<egui::Id>,
    activated: Option<egui::Id>,
    other: usize,
}

fn view(ui: &mut egui::Ui, state: &mut State) {
    let theme = Theme::light();
    let children = [
        SidebarItem::new(egui::Id::unique("settings"), "Settings"),
        SidebarItem::new(egui::Id::unique("members"), "Members").enabled(false),
    ];
    let items = [
        SidebarItem::new(egui::Id::unique("inbox"), "Inbox").icon("="),
        SidebarItem::new(egui::Id::unique("projects"), "Projects").children(&children),
        SidebarItem::new(egui::Id::unique("trash"), "Trash").enabled(false),
    ];
    let groups = [SidebarGroup::new(egui::Id::unique("main"), &items).label("Main")];
    let State {
        sidebar,
        selected,
        activated,
        other,
    } = state;
    let _ = other;
    let response = Sidebar::new(
        egui::Id::unique("sidebar"),
        "Main navigation",
        &theme,
        sidebar,
    )
    .title("Workspace")
    .show(ui, &groups, selected);
    if let Some(id) = response.selected {
        *activated = Some(id);
    }
}

fn harness() -> Harness<'static, State> {
    Harness::builder()
        .with_size(egui::vec2(320.0, 480.0))
        .build_ui_state(view, State::default())
}

#[test]
fn sidebar_exposes_tree_group_and_item_roles() {
    let mut harness = harness();
    harness.run();

    let tree = harness.get_by_role_and_label(egui::accesskit::Role::Tree, "Main navigation");
    assert_eq!(
        tree.accesskit_node().label().as_deref(),
        Some("Main navigation"),
        "the sidebar container should expose its accessible name"
    );
    assert!(tree.rect().width() > 0.0);

    harness.get_by_role_and_label(egui::accesskit::Role::Group, "Main");
    harness.get_by_label("Workspace");
    harness.get_by_label("Collapse sidebar");

    let inbox = harness.get_by_role_and_label(egui::accesskit::Role::TreeItem, "Inbox");
    assert_eq!(inbox.accesskit_node().data().is_expanded(), None);
    assert_eq!(inbox.accesskit_node().is_selected(), Some(false));

    let projects = harness.get_by_role_and_label(egui::accesskit::Role::TreeItem, "Projects");
    assert_eq!(
        projects.accesskit_node().data().is_expanded(),
        Some(false),
        "an entry with children exposes a collapsed state"
    );

    let trash = harness.get_by_role_and_label(egui::accesskit::Role::TreeItem, "Trash");
    assert!(
        trash.accesskit_node().is_disabled(),
        "a disabled entry exposes the disabled state"
    );
}

#[test]
fn pointer_activation_updates_caller_owned_selection() {
    let mut harness = harness();
    harness.run();

    harness.get_by_label("Inbox").click();
    harness.run();
    assert_eq!(harness.state().selected, Some(egui::Id::unique("inbox")));
    assert_eq!(harness.state().activated, Some(egui::Id::unique("inbox")));
    assert_eq!(
        harness
            .get_by_role_and_label(egui::accesskit::Role::TreeItem, "Inbox")
            .accesskit_node()
            .is_selected(),
        Some(true)
    );
}

#[test]
fn collapse_toggle_flips_caller_state_and_clamps_the_rail() {
    let mut harness = harness();
    harness.run();
    let expanded_width = harness.get_by_label("Inbox").rect().width();
    assert!(expanded_width > 100.0, "{expanded_width}");

    harness.get_by_label("Collapse sidebar").click();
    harness.run();
    assert!(harness.state().sidebar.collapsed);
    harness.get_by_label("Expand sidebar");
    assert!(harness.query_by_label("Collapse sidebar").is_none());
    // Collapsed entries keep their full accessible name but render in a narrow
    // rail.
    let collapsed_width = harness.get_by_label("Inbox").rect().width();
    assert!(collapsed_width < 64.0, "{collapsed_width}");

    harness.get_by_label("Expand sidebar").click();
    harness.run();
    assert!(!harness.state().sidebar.collapsed);
    harness.get_by_label("Collapse sidebar");
}

#[test]
fn arrow_keys_move_focus_and_skip_disabled_entries() {
    let mut harness = harness();
    harness.run();

    harness.get_by_label("Inbox").focus();
    harness.run();
    assert!(harness.get_by_label("Inbox").is_focused());

    harness.key_press(egui::Key::ArrowDown);
    harness.run();
    assert!(
        harness.get_by_label("Projects").is_focused(),
        "ArrowDown should move focus to the next visible entry"
    );

    // Trash is disabled, so the next enabled entry wraps back to Inbox.
    harness.key_press(egui::Key::ArrowDown);
    harness.run();
    assert!(harness.get_by_label("Inbox").is_focused());

    harness.key_press(egui::Key::Enter);
    harness.run();
    assert_eq!(harness.state().selected, Some(egui::Id::unique("inbox")));
}

#[test]
fn right_and_left_expand_and_collapse_an_entry_with_children() {
    let mut harness = harness();
    harness.run();

    harness.get_by_label("Projects").focus();
    harness.run();
    harness.key_press(egui::Key::ArrowRight);
    harness.run();
    assert!(
        harness
            .state()
            .sidebar
            .is_expanded(egui::Id::unique("projects"))
    );
    harness.get_by_label("Settings");
    assert_eq!(
        harness
            .get_by_role_and_label(egui::accesskit::Role::TreeItem, "Projects")
            .accesskit_node()
            .data()
            .is_expanded(),
        Some(true)
    );

    harness.key_press(egui::Key::ArrowLeft);
    harness.run();
    assert!(
        !harness
            .state()
            .sidebar
            .is_expanded(egui::Id::unique("projects"))
    );
    assert!(harness.query_by_label("Settings").is_none());
}

#[test]
fn disabled_entry_ignores_stale_accessibility_activation() {
    let mut harness = harness();
    harness.run();
    let trash = harness.get_by_role_and_label(egui::accesskit::Role::TreeItem, "Trash");
    let (target_node, target_tree) = trash.accesskit_node().locate();
    harness.event(egui::Event::AccessKitActionRequest(
        egui::accesskit::ActionRequest {
            action: egui::accesskit::Action::Click,
            target_node,
            target_tree,
            data: None,
        },
    ));
    harness.run();
    assert_eq!(harness.state().selected, None);
    assert_eq!(harness.state().activated, None);
}

#[test]
fn narrow_windows_clamp_the_sidebar_inside_a_scaffold_slot() {
    let mut harness = Harness::builder()
        .with_size(egui::vec2(320.0, 480.0))
        .build_ui_state(
            |ui, state: &mut State| {
                let theme = Theme::light();
                let children = [
                    SidebarItem::new(egui::Id::unique("settings"), "Settings"),
                    SidebarItem::new(egui::Id::unique("members"), "Members").enabled(false),
                ];
                let items = [
                    SidebarItem::new(egui::Id::unique("inbox"), "Inbox").icon("="),
                    SidebarItem::new(egui::Id::unique("projects"), "Projects").children(&children),
                    SidebarItem::new(egui::Id::unique("trash"), "Trash").enabled(false),
                ];
                let groups = [SidebarGroup::new(egui::Id::unique("main"), &items).label("Main")];
                let State {
                    sidebar, selected, ..
                } = state;
                Scaffold::new(egui::Id::unique("app-shell"), &theme)
                    .sidebar(256.0)
                    .unwrap()
                    .show(
                        ui,
                        |ui| {
                            Sidebar::new(
                                egui::Id::unique("sidebar"),
                                "Main navigation",
                                &theme,
                                sidebar,
                            )
                            .show(ui, &groups, selected);
                        },
                        |_| {},
                        |_| {},
                        |ui| {
                            ui.label("Content");
                        },
                    );
            },
            State::default(),
        );
    harness.run();

    // The scaffold caps the sidebar at 40% of a 320-point window.
    let inbox = harness.get_by_label("Inbox");
    assert!(inbox.rect().width() <= 128.0, "{:?}", inbox.rect());
    assert!(inbox.rect().right() <= 320.0, "{:?}", inbox.rect());
    harness.get_by_label("Content");
}
