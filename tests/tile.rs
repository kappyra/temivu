use egui_kittest::{
    Harness,
    kittest::{NodeT, Queryable},
};
use temivu::{ItemVariant, Theme, Tile};

#[test]
fn tile_exposes_description_selection_and_keyboard_activation() {
    let mut harness = Harness::builder().build_ui_state(
        |ui, selected: &mut bool| {
            let response = Tile::new("Bluetooth", &Theme::light())
                .id(egui::Id::new("bluetooth"))
                .description("Connect devices")
                .selected(*selected)
                .show(ui);
            if response.clicked() {
                *selected = !*selected;
            }
        },
        false,
    );
    harness.run();
    let node = harness.get_by_label("Bluetooth");
    assert!(node.rect().height() >= 48.0);
    assert_eq!(
        node.accesskit_node().description().as_deref(),
        Some("Connect devices")
    );
    assert_eq!(node.accesskit_node().is_selected(), Some(false));
    node.focus();
    harness.run();
    harness.key_press(egui::Key::Enter);
    harness.run();
    assert!(*harness.state());
    assert_eq!(
        harness
            .get_by_label("Bluetooth")
            .accesskit_node()
            .is_selected(),
        Some(true)
    );
}

#[test]
fn disabled_tile_ignores_stale_accessibility_activation() {
    let mut harness = Harness::builder().build_ui_state(
        |ui, count: &mut usize| {
            if Tile::new("Delete", &Theme::light())
                .variant(ItemVariant::Destructive)
                .enabled(false)
                .show(ui)
                .clicked()
            {
                *count += 1;
            }
        },
        0,
    );
    harness.run();
    let node = harness.get_by_label("Delete");
    assert!(node.accesskit_node().is_disabled());
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
    assert_eq!(*harness.state(), 0);
}

#[test]
fn stable_tile_identity_survives_insertion() {
    let mut harness = Harness::builder().build_ui_state(
        |ui, insert: &mut bool| {
            if *insert {
                Tile::new("Inserted", &Theme::light())
                    .id(egui::Id::new("inserted"))
                    .show(ui);
            }
            for name in ["First", "Second"] {
                Tile::new(name, &Theme::light())
                    .id(egui::Id::new(name))
                    .show(ui);
            }
        },
        false,
    );
    harness.run();
    harness.get_by_label("Second").focus();
    harness.run();
    assert!(harness.get_by_label("Second").is_focused());
    *harness.state_mut() = true;
    harness.run();
    assert!(harness.get_by_label("Second").is_focused());
}

#[test]
fn tile_uses_forui_card_border_and_radius() {
    for theme in [Theme::light(), Theme::dark()] {
        let fill = theme.card;
        let border = theme.border;
        let mut harness = Harness::builder().build_ui(move |ui| {
            Tile::new("Bluetooth", &theme)
                .description("Connect devices")
                .show(ui);
        });
        harness.run();
        let container = harness
            .output()
            .shapes
            .iter()
            .find_map(|shape| match &shape.shape {
                egui::Shape::Rect(rect)
                    if rect.fill == fill
                        && rect.stroke.width == 1.0
                        && rect.stroke.color == border
                        && rect.corner_radius.nw == 10 =>
                {
                    Some(rect.rect)
                }
                _ => None,
            })
            .expect("tile uses the card fill, border and md radius");
        assert!(
            container.height() >= 48.0,
            "tile height: {}",
            container.height()
        );
        let row = harness.get_by_label("Bluetooth").rect();
        assert_eq!(container.left(), row.left(), "tiles have no outer margin");
        assert_eq!(container.right(), row.right(), "tiles have no outer margin");
    }
}
