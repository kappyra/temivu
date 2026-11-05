use egui_kittest::{
    Harness,
    kittest::{NodeT, Queryable},
};
use temivu::{NavigationItem, Theme, bottom_navigation};

#[test]
fn five_short_items_share_one_row_when_they_fit() {
    let mut harness = Harness::builder()
        .with_size(egui::vec2(1100.0, 200.0))
        .build_ui(|ui| {
            let items = ["Overview", "Controls", "Overlays", "Data", "Localization"]
                .map(|name| NavigationItem::new(egui::Id::new(name), name));
            bottom_navigation(
                ui,
                egui::Id::new("row"),
                &items,
                &mut None,
                &Theme::mint_light(),
            );
        });
    harness.run();
    let first = harness.get_by_label("Overview").rect();
    let last = harness.get_by_label("Localization").rect();
    assert!(
        (first.top() - last.top()).abs() < 1.0,
        "first={first:?} last={last:?}"
    );
}

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
    let settings = harness.get_by_label("Settings").rect();
    assert!(settings.right() <= 200.0);
    assert!(harness.output().shapes.iter().any(|shape| matches!(
        &shape.shape, egui::Shape::Text(text) if text.galley.job.text == "Settings" && !text.galley.elided
    )));
}

#[test]
fn floating_tabs_stay_inside_viewport_and_can_be_selected_by_keyboard() {
    let mut harness = Harness::builder()
        .with_size(egui::vec2(360.0, 500.0))
        .build_ui_state(
            |ui, selected: &mut Option<egui::Id>| {
                let items = ["Home", "Controls", "Overlays", "Data", "Localization"]
                    .map(|label| NavigationItem::new(egui::Id::new(label), label));
                temivu::FloatingNavigation::new(egui::Id::new("float-tabs"), &Theme::light())
                    .show(ui, &items, selected);
            },
            Some(egui::Id::new("Home")),
        );
    harness.run_steps(4);
    let first = harness.get_by_label("Home").rect();
    let last = harness.get_by_label("Localization");
    assert!(first.left() >= 12.0 && last.rect().right() <= 348.0);
    assert_eq!(first.center().y, last.rect().center().y);
    assert!(first.bottom() < 500.0 && first.top() > 400.0);
    last.focus();
    harness.run();
    harness.key_press(egui::Key::Enter);
    harness.run_steps(4);
    assert_eq!(*harness.state(), Some(egui::Id::new("Localization")));
    assert_eq!(
        harness
            .get_by_label("Localization")
            .accesskit_node()
            .is_selected(),
        Some(true)
    );
}

#[test]
fn floating_tabs_commit_drag_on_release_and_ignore_hover_or_disabled_targets() {
    let home = egui::Id::new("Home");
    let controls = egui::Id::new("Controls");
    let mut harness = Harness::builder()
        .with_size(egui::vec2(360.0, 500.0))
        .build_ui_state(
            move |ui, selected| {
                let items = [
                    NavigationItem::new(home, "Home"),
                    NavigationItem::new(controls, "Controls"),
                    NavigationItem::new(egui::Id::new("Disabled"), "Disabled").enabled(false),
                ];
                temivu::FloatingNavigation::new(egui::Id::new("drag-tabs"), &Theme::light())
                    .select_on_drag(true)
                    .show(ui, &items, selected);
            },
            Some(home),
        );
    harness.run_steps(4);
    let left = harness.get_by_label("Home").rect().center();
    let middle = harness.get_by_label("Controls").rect().center();
    let disabled = harness.get_by_label("Disabled").rect().center();
    harness.get_by_label("Controls").hover();
    harness.run_steps(4);
    assert_eq!(*harness.state(), Some(home));

    harness.event(egui::Event::PointerMoved(left));
    harness.drag_at(left);
    harness.event(egui::Event::PointerMoved(middle));
    harness.run_steps(3);
    assert_eq!(*harness.state(), Some(home));
    assert!(harness.output().shapes.iter().any(|shape| matches!(
        &shape.shape, egui::Shape::Rect(rect)
            if rect.fill == Theme::light().secondary && rect.rect.contains(middle)
    )));
    harness.drop_at(middle);
    harness.run_steps(3);
    assert_eq!(*harness.state(), Some(controls));

    for target in [disabled, egui::pos2(middle.x, middle.y - 90.0)] {
        harness.event(egui::Event::PointerMoved(middle));
        harness.drag_at(middle);
        harness.event(egui::Event::PointerMoved(target));
        harness.step();
        harness.drop_at(target);
        harness.run_steps(3);
        assert_eq!(*harness.state(), Some(controls));
    }

    harness.event(egui::Event::PointerMoved(middle));
    harness.drag_at(middle);
    harness.event(egui::Event::PointerMoved(left));
    harness.step();
    harness.drop_at(left);
    harness.run_steps(3);
    assert_eq!(*harness.state(), Some(home));
}
