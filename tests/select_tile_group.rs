use egui_kittest::{
    Harness,
    kittest::{NodeT, Queryable},
};
use temivu::{SelectTile, SelectTileGroup, Theme};

struct State {
    selected: Vec<egui::Id>,
    changed: bool,
    activated: Option<egui::Id>,
    reorder: bool,
    group_enabled: bool,
    parent_disabled: bool,
}

impl Default for State {
    fn default() -> Self {
        Self {
            selected: Vec::new(),
            changed: false,
            activated: None,
            reorder: false,
            group_enabled: true,
            parent_disabled: false,
        }
    }
}

fn tiles(reorder: bool) -> Vec<SelectTile<'static>> {
    let wifi = SelectTile::new(egui::Id::new("wifi"), "Wi-Fi").description("Wireless networks");
    let bluetooth = SelectTile::new(egui::Id::new("bluetooth"), "Bluetooth");
    let nfc = SelectTile::new(egui::Id::new("nfc"), "NFC").enabled(false);
    if reorder {
        vec![bluetooth, wifi, nfc]
    } else {
        vec![wifi, bluetooth, nfc]
    }
}

fn view(ui: &mut egui::Ui, state: &mut State) {
    let theme = Theme::light();
    let State {
        selected,
        changed,
        activated,
        reorder,
        group_enabled,
        parent_disabled,
    } = state;
    if *parent_disabled {
        ui.disable();
    }
    let tiles = tiles(*reorder);
    let group = SelectTileGroup::new(egui::Id::new("radios"), "Radios", &theme, &tiles, selected)
        .description("Choose any that apply")
        .enabled(*group_enabled)
        .show(ui);
    *changed |= group.changed;
    if let Some(id) = group.activated {
        *activated = Some(id);
    }
}

fn build_harness(state: State) -> Harness<'static, State> {
    Harness::builder()
        .with_size(egui::vec2(360.0, 360.0))
        .build_ui_state(view, state)
}

#[test]
fn group_and_tiles_expose_list_roles_positions_and_selection() {
    let mut harness = build_harness(State::default());
    harness.run();

    let group = harness.get_by_role_and_label(egui::accesskit::Role::List, "Radios");
    assert_eq!(
        group.accesskit_node().description().as_deref(),
        Some("Choose any that apply")
    );
    assert_eq!(group.accesskit_node().size_of_set(), Some(3));

    let wifi = harness.get_by_role_and_label(egui::accesskit::Role::ListItem, "Wi-Fi");
    assert_eq!(wifi.accesskit_node().position_in_set(), Some(0));
    assert_eq!(wifi.accesskit_node().size_of_set(), Some(3));
    assert_eq!(wifi.accesskit_node().is_selected(), Some(false));
    assert_eq!(
        wifi.accesskit_node().description().as_deref(),
        Some("Wireless networks")
    );

    let nfc = harness.get_by_role_and_label(egui::accesskit::Role::ListItem, "NFC");
    assert!(nfc.accesskit_node().is_disabled());
}

#[test]
fn pointer_toggles_multi_selection_in_caller_state() {
    let mut harness = build_harness(State::default());
    harness.run();

    harness.get_by_label("Wi-Fi").click();
    harness.run();
    assert_eq!(harness.state().selected, vec![egui::Id::new("wifi")]);
    assert_eq!(harness.state().activated, Some(egui::Id::new("wifi")));
    assert!(harness.state().changed);

    harness.get_by_label("Bluetooth").click();
    harness.run();
    assert_eq!(
        harness.state().selected,
        vec![egui::Id::new("wifi"), egui::Id::new("bluetooth")]
    );

    harness.get_by_label("Wi-Fi").click();
    harness.run();
    assert_eq!(harness.state().selected, vec![egui::Id::new("bluetooth")]);
    assert_eq!(
        harness
            .get_by_role_and_label(egui::accesskit::Role::ListItem, "Wi-Fi")
            .accesskit_node()
            .is_selected(),
        Some(false)
    );
}

#[test]
fn arrow_keys_move_focus_skip_disabled_tiles_and_toggle_selection() {
    let mut harness = build_harness(State::default());
    harness.run();

    harness.get_by_label("Wi-Fi").focus();
    harness.run();
    harness.key_press(egui::Key::ArrowDown);
    harness.run();
    assert!(
        harness.get_by_label("Bluetooth").is_focused(),
        "ArrowDown should move focus to the next enabled tile"
    );

    // NFC is disabled, so the next enabled tile wraps back to Wi-Fi.
    harness.key_press(egui::Key::ArrowDown);
    harness.run();
    assert!(harness.get_by_label("Wi-Fi").is_focused());

    harness.key_press(egui::Key::Enter);
    harness.run();
    assert_eq!(harness.state().selected, vec![egui::Id::new("wifi")]);

    harness.key_press(egui::Key::End);
    harness.run();
    assert!(
        harness.get_by_label("Bluetooth").is_focused(),
        "End should focus the last enabled tile"
    );
    harness.key_press(egui::Key::Home);
    harness.run();
    assert!(harness.get_by_label("Wi-Fi").is_focused());
}

#[test]
fn disabled_tiles_and_disabled_groups_ignore_activation() {
    let mut harness = build_harness(State::default());
    harness.run();
    let (target_node, target_tree) = {
        let nfc = harness.get_by_role_and_label(egui::accesskit::Role::ListItem, "NFC");
        nfc.click();
        nfc.accesskit_node().locate()
    };
    harness.run();
    assert!(harness.state().selected.is_empty());
    harness.event(egui::Event::AccessKitActionRequest(
        egui::accesskit::ActionRequest {
            action: egui::accesskit::Action::Click,
            target_node,
            target_tree,
            data: None,
        },
    ));
    harness.run();
    assert!(harness.state().selected.is_empty());

    let mut harness = build_harness(State {
        group_enabled: false,
        ..State::default()
    });
    harness.run();
    assert!(
        harness
            .get_by_role_and_label(egui::accesskit::Role::List, "Radios")
            .accesskit_node()
            .is_disabled()
    );
    harness.get_by_label("Wi-Fi").click();
    harness.run();
    assert!(harness.state().selected.is_empty());
}

#[test]
fn inherited_disabled_parent_ui_disables_the_group() {
    let mut harness = build_harness(State {
        parent_disabled: true,
        ..State::default()
    });
    harness.run();
    let wifi = harness.get_by_role_and_label(egui::accesskit::Role::ListItem, "Wi-Fi");
    assert!(wifi.accesskit_node().is_disabled());
    harness.get_by_label("Wi-Fi").click();
    harness.run();
    assert!(harness.state().selected.is_empty());
}

#[test]
fn selection_follows_stable_ids_when_tiles_are_reordered() {
    let mut harness = build_harness(State::default());
    harness.run();
    harness.get_by_label("Wi-Fi").click();
    harness.run();
    assert_eq!(harness.state().selected, vec![egui::Id::new("wifi")]);

    harness.state_mut().reorder = true;
    harness.run();
    let wifi = harness.get_by_role_and_label(egui::accesskit::Role::ListItem, "Wi-Fi");
    assert_eq!(
        wifi.accesskit_node().is_selected(),
        Some(true),
        "selection must stay attached to the stable tile ID after reordering"
    );
    assert_eq!(wifi.accesskit_node().position_in_set(), Some(1));
    assert_eq!(harness.state().selected, vec![egui::Id::new("wifi")]);
}
