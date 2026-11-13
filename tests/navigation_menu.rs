use egui_kittest::{
    Harness,
    kittest::{NodeT, Queryable},
};
use temivu::{MenuPresentation, NavigationItem, NavigationMenu, Theme};

fn build(presentation: MenuPresentation) -> Harness<'static, (bool, Option<egui::Id>)> {
    Harness::builder()
        .with_size(egui::vec2(390.0, 720.0))
        .build_ui_state(
            move |ui, state: &mut (bool, Option<egui::Id>)| {
                let theme = Theme::light();
                theme.apply(ui.ctx());
                let trigger = ui.button("Open navigation");
                let items = [
                    NavigationItem::new(egui::Id::unique("home"), "Home"),
                    NavigationItem::new(egui::Id::unique("locked"), "Locked").enabled(false),
                    NavigationItem::new(egui::Id::unique("settings"), "Settings"),
                ];
                NavigationMenu::new(
                    egui::Id::unique("navigation"),
                    "Navigation",
                    &mut state.0,
                    &theme,
                )
                .presentation(presentation)
                .show(&trigger, &items, &mut state.1);
            },
            (false, None),
        )
}

#[test]
fn both_presentations_select_and_dismiss_with_escape() {
    for presentation in [
        MenuPresentation::FullScreen,
        MenuPresentation::Floating,
        MenuPresentation::Drawer,
    ] {
        let mut harness = build(presentation);
        harness.run();
        harness.get_by_label("Open navigation").click();
        harness.run();
        if presentation == MenuPresentation::Drawer {
            let drawer = harness
                .get_by_role_and_label(egui::accesskit::Role::Dialog, "Navigation")
                .rect();
            assert!(drawer.left() >= 0.0 && drawer.right() < 390.0, "{drawer:?}");
            assert!(
                drawer.top() >= 0.0 && drawer.bottom() <= 720.0,
                "{drawer:?}"
            );
            assert!(drawer.height() >= 680.0, "{drawer:?}");
            harness.get_by_label("Find your next destination.");
        }
        let locked = harness.get_by_role_and_label(egui::accesskit::Role::MenuItem, "Locked");
        assert!(locked.accesskit_node().is_disabled());
        let settings = harness.get_by_role_and_label(egui::accesskit::Role::MenuItem, "Settings");
        assert!(settings.rect().right() <= 390.0);
        settings.click();
        harness.run();
        assert_eq!(harness.state().1, Some(egui::Id::unique("settings")));
        assert!(!harness.state().0);
        harness.get_by_label("Open navigation").click();
        harness.run();
        harness
            .get_by_role_and_label(egui::accesskit::Role::MenuItem, "Home")
            .focus();
        harness.run();
        harness.key_press(egui::Key::Escape);
        harness.run();
        assert!(!harness.state().0);
        assert!(
            harness.get_by_label("Open navigation").is_focused(),
            "focus after Escape: {presentation:?}"
        );
    }
}
