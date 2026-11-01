use egui_kittest::{
    Harness,
    kittest::{NodeT, Queryable},
};
use temivu::{NavigationItem, Theme, bottom_navigation};

#[test]
fn keyboard_selection_and_disabled_actions() {
    let home = egui::Id::new("home");
    let search = egui::Id::new("search");
    let locked = egui::Id::new("locked");
    let mut harness = Harness::builder().build_ui_state(
        move |ui, selected: &mut Option<egui::Id>| {
            let items = [
                NavigationItem::new(home, "Home"),
                NavigationItem::new(search, "Search"),
                NavigationItem::new(locked, "Locked").enabled(false),
            ];
            bottom_navigation(ui, egui::Id::new("nav"), &items, selected, &Theme::light());
        },
        Some(home),
    );
    harness.run();
    assert_eq!(
        harness.get_by_label("Home").accesskit_node().is_selected(),
        Some(true)
    );
    assert_eq!(
        harness.get_by_label("Home").accesskit_node().toggled(),
        None
    );
    assert!(
        harness
            .get_by_label("Locked")
            .accesskit_node()
            .is_disabled()
    );
    harness.get_by_label("Search").focus();
    harness.run();
    harness.key_press(egui::Key::Enter);
    harness.run();
    assert_eq!(*harness.state(), Some(search));
    let node = harness.get_by_label("Locked");
    let (target_node, target_tree) = node.accesskit_node().locate();
    harness.event(egui::Event::AccessKitActionRequest(
        egui::accesskit::ActionRequest {
            action: egui::accesskit::Action::Click,
            target_node,
            target_tree,
            data: None,
        },
    ));
    harness.run();
    assert_eq!(*harness.state(), Some(search));
}

#[test]
fn long_labels_and_small_viewport_remain_accessible() {
    let mut harness = Harness::builder()
        .with_size(egui::vec2(200.0, 350.0))
        .build_ui_state(
            |ui, selected: &mut Option<egui::Id>| {
                let items = [
                    NavigationItem::new(egui::Id::new(1), "Long translated navigation label"),
                    NavigationItem::new(egui::Id::new(2), "Another translated navigation label"),
                    NavigationItem::new(egui::Id::new(3), "Settings"),
                ];
                bottom_navigation(
                    ui,
                    egui::Id::new("small-nav"),
                    &items,
                    selected,
                    &Theme::light(),
                );
            },
            None,
        );
    harness.run();
    for label in [
        "Long translated navigation label",
        "Another translated navigation label",
        "Settings",
    ] {
        let node = harness.get_by_label(label);
        assert!(node.rect().right() <= 200.0, "{label}: {:?}", node.rect());
        assert!(node.rect().width() >= 48.0 && node.rect().height() >= 48.0);
    }
    harness.get_by_label("Settings").click();
    harness.run();
    assert_eq!(*harness.state(), Some(egui::Id::new(3)));
}

#[test]
fn focus_tracks_item_identity_across_reordering_and_empty_list_is_safe() {
    let first = egui::Id::new("first");
    let second = egui::Id::new("second");
    let mut harness = Harness::builder().build_ui_state(
        move |ui, state: &mut (Option<egui::Id>, bool)| {
            let mut items = [
                NavigationItem::new(first, "First"),
                NavigationItem::new(second, "Second"),
            ];
            if state.1 {
                items.swap(0, 1);
            }
            assert!(
                bottom_navigation(
                    ui,
                    egui::Id::new("empty"),
                    &[],
                    &mut state.0,
                    &Theme::light()
                )
                .is_none()
            );
            bottom_navigation(
                ui,
                egui::Id::new("stable-nav"),
                &items,
                &mut state.0,
                &Theme::light(),
            );
        },
        (None, false),
    );
    harness.get_by_label("Second").focus();
    harness.run();
    harness.state_mut().1 = true;
    harness.run();
    assert!(harness.get_by_label("Second").accesskit_node().is_focused());
    harness.key_press(egui::Key::Enter);
    harness.run();
    assert_eq!(harness.state().0, Some(second));
}

#[test]
fn short_destination_names_wrap_without_visual_truncation_at_zoom() {
    let mut harness = Harness::builder()
        .with_size(egui::vec2(300.0, 180.0))
        .build_ui(|ui| {
            egui::CentralPanel::default().show(ui, |ui| {
                let items = [
                    NavigationItem::new(egui::Id::new(1), "Home"),
                    NavigationItem::new(egui::Id::new(2), "Search"),
                    NavigationItem::new(egui::Id::new(3), "Settings"),
                ];
                let mut selected = Some(egui::Id::new(1));
                bottom_navigation(
                    ui,
                    egui::Id::new("zoom-nav"),
                    &items,
                    &mut selected,
                    &Theme::light(),
                );
            });
        });
    harness.ctx.set_zoom_factor(1.5);
    harness.run();
    let home = harness.get_by_label("Home").rect();
    let settings = harness.get_by_label("Settings").rect();
    assert!(settings.top() >= home.bottom());
    assert!(settings.right() <= 200.0);
    assert!(harness.output().shapes.iter().any(|shape| matches!(
        &shape.shape, egui::Shape::Text(text) if text.galley.job.text == "Settings" && !text.galley.elided
    )));
}
