use egui_kittest::{
    Harness,
    kittest::{NodeT, Queryable},
};
use temivu::{Select, SelectOption};

#[test]
fn selecting_an_option_updates_caller_state_and_response() {
    let red = egui::Id::new("red");
    let blue = egui::Id::new("blue");
    let mut harness = Harness::builder().build_ui_state(
        move |ui, state: &mut (Option<egui::Id>, bool)| {
            let options = [
                SelectOption::new(red, "Red"),
                SelectOption::new(blue, "Blue"),
            ];
            let response = Select::new(
                egui::Id::new("color"),
                "Color",
                "Choose a color",
                &options,
                &mut state.0,
            )
            .show(ui);
            state.1 |= response.changed();
        },
        (None, false),
    );
    harness.run();
    let combo = harness.get_by_role_and_label(egui::accesskit::Role::ComboBox, "Color");
    assert!(combo.rect().height() >= 32.0);
    combo.click();
    harness.run();
    assert!(harness.get_by_label("Blue").rect().height() >= 32.0);
    harness.get_by_label("Blue").click();
    harness.run();
    assert_eq!(harness.state().0, Some(blue));
    assert!(harness.state().1);
    assert_eq!(
        harness
            .get_by_role_and_label(egui::accesskit::Role::ComboBox, "Color")
            .accesskit_node()
            .value()
            .as_deref(),
        Some("Blue")
    );
}

#[test]
fn disabled_option_ignores_stale_accessibility_click() {
    let mut harness = Harness::builder().build_ui_state(
        |ui, selected: &mut Option<egui::Id>| {
            let options =
                [SelectOption::new(egui::Id::new("blocked"), "Unavailable").enabled(false)];
            Select::new(
                egui::Id::new("choices"),
                "Choice",
                "Choose",
                &options,
                selected,
            )
            .show(ui);
        },
        None,
    );
    harness
        .get_by_role_and_label(egui::accesskit::Role::ComboBox, "Choice")
        .click();
    harness.run();
    let node = harness.get_by_label("Unavailable");
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
    assert_eq!(*harness.state(), None);
}

#[test]
fn keyboard_can_open_and_choose_an_option() {
    let target = egui::Id::new("keyboard-target");
    let mut harness = Harness::builder().build_ui_state(
        move |ui, selected: &mut Option<egui::Id>| {
            let options = [SelectOption::new(target, "Keyboard option")];
            Select::new(
                egui::Id::new("keyboard-select"),
                "Choices",
                "Choose",
                &options,
                selected,
            )
            .show(ui);
        },
        None,
    );
    harness
        .get_by_role_and_label(egui::accesskit::Role::ComboBox, "Choices")
        .focus();
    harness.run();
    harness.key_press(egui::Key::Enter);
    harness.run();
    harness.get_by_label("Keyboard option").focus();
    harness.run();
    harness.key_press(egui::Key::Enter);
    harness.run();
    assert_eq!(*harness.state(), Some(target));
}

#[test]
fn disabling_an_open_select_blocks_stale_option_actions() {
    for inherited in [false, true] {
        let mut harness = Harness::builder().build_ui_state(
            move |ui, state: &mut (Option<egui::Id>, bool)| {
                if inherited && !state.1 {
                    ui.disable();
                }
                let options = [SelectOption::new(egui::Id::new("target"), "Target")];
                Select::new(
                    egui::Id::new("disable-open"),
                    "Choices",
                    "Choose",
                    &options,
                    &mut state.0,
                )
                .enabled(inherited || state.1)
                .show(ui);
            },
            (None, true),
        );
        harness
            .get_by_role_and_label(egui::accesskit::Role::ComboBox, "Choices")
            .click();
        harness.run();
        let node = harness.get_by_label("Target");
        let (target_node, target_tree) = node.accesskit_node().locate();
        harness.state_mut().1 = false;
        harness.run();
        assert!(
            harness.query_by_label("Target").is_none(),
            "disabled select left its popup open"
        );
        harness.event(egui::Event::AccessKitActionRequest(
            egui::accesskit::ActionRequest {
                action: egui::accesskit::Action::Click,
                target_node,
                target_tree,
                data: None,
            },
        ));
        harness.run();
        assert_eq!(harness.state().0, None);
    }
}

#[test]
fn popup_identity_survives_a_preceding_conditional_widget() {
    let mut harness = Harness::builder().build_ui_state(
        |ui, state: &mut (Option<egui::Id>, bool)| {
            if state.1 {
                ui.label("Extra");
            }
            let options = [SelectOption::new(egui::Id::new("target"), "Target")];
            Select::new(
                egui::Id::new("stable-select"),
                "Choices",
                "Choose",
                &options,
                &mut state.0,
            )
            .show(ui);
        },
        (None, false),
    );
    harness
        .get_by_role_and_label(egui::accesskit::Role::ComboBox, "Choices")
        .click();
    harness.run();
    harness.get_by_label("Target");
    harness.state_mut().1 = true;
    harness.run();
    harness.get_by_label("Target").click();
    harness.run();
    assert_eq!(harness.state().0, Some(egui::Id::new("target")));
}

#[test]
fn narrow_select_keeps_long_values_accessible_and_inside_viewport() {
    let long = "A very long localized selection value for this device";
    let option_id = egui::Id::new("long-option");
    let mut harness = Harness::builder()
        .with_size(egui::vec2(200.0, 350.0))
        .build_ui_state(
            move |ui, selected: &mut Option<egui::Id>| {
                let options = [SelectOption::new(option_id, long)];
                Select::new(
                    egui::Id::new("narrow-select"),
                    "Choice",
                    "Choose an option",
                    &options,
                    selected,
                )
                .show(ui);
            },
            Some(option_id),
        );
    harness.run();
    let field = harness.get_by_role_and_label(egui::accesskit::Role::ComboBox, "Choice");
    assert!(field.rect().right() <= 200.0, "{:?}", field.rect());
    assert_eq!(field.accesskit_node().value().as_deref(), Some(long));
    field.click();
    harness.run();
    let option = harness.get_by_label(long);
    assert!(option.rect().right() <= 200.0, "{:?}", option.rect());
}

#[test]
fn keyboard_dismissal_and_acceptance_restore_focus_to_select() {
    for key in [egui::Key::Escape, egui::Key::Enter] {
        let mut harness = Harness::builder().build_ui_state(
            |ui, selected: &mut Option<egui::Id>| {
                let options = [SelectOption::new(egui::Id::new("target"), "Choice one")];
                Select::new(
                    egui::Id::new("select"),
                    "Choices",
                    "Choose",
                    &options,
                    selected,
                )
                .show(ui);
                let _ = ui.button("Next");
            },
            None,
        );
        harness.run();
        harness.get_by_label("Choices").focus();
        harness.run();
        harness.key_press(egui::Key::Enter);
        harness.run();
        harness.get_by_label("Choice one").focus();
        harness.run();
        harness.key_press(key);
        harness.run();
        assert!(harness.query_by_label("Choice one").is_none());
        assert!(
            harness.get_by_label("Choices").is_focused(),
            "focus lost after {key:?}"
        );
    }
}

#[test]
fn search_filters_options_and_reports_no_results() {
    let mut harness = Harness::builder().build_ui_state(
        |ui, selected: &mut Option<egui::Id>| {
            let options = [
                SelectOption::new(egui::Id::new("kbd"), "Keyboard"),
                SelectOption::new(egui::Id::new("mouse"), "Mouse"),
            ];
            Select::new(
                egui::Id::new("search-select"),
                "Device",
                "Choose",
                &options,
                selected,
            )
            .searchable(true)
            .show(ui);
        },
        None,
    );
    harness.run();
    harness
        .get_by_role_and_label(egui::accesskit::Role::ComboBox, "Device")
        .click();
    harness.run();
    harness
        .get_by_role_and_label(egui::accesskit::Role::TextInput, "Search")
        .type_text("mouse");
    harness.run();
    harness.get_by_label("Mouse");
    assert!(
        harness.query_by_label("Keyboard").is_none(),
        "the search should filter non-matching options"
    );
    harness
        .get_by_role_and_label(egui::accesskit::Role::TextInput, "Search")
        .type_text("zzz");
    harness.run();
    harness.get_by_label("No results");
    assert!(harness.query_by_label("Mouse").is_none());
}

#[test]
fn a_filtered_option_can_be_selected_from_search() {
    let target = egui::Id::new("mouse");
    let mut harness = Harness::builder().build_ui_state(
        move |ui, selected: &mut Option<egui::Id>| {
            let options = [
                SelectOption::new(egui::Id::new("kbd"), "Keyboard"),
                SelectOption::new(target, "Mouse"),
            ];
            let response = Select::new(
                egui::Id::new("search-select-2"),
                "Device",
                "Choose",
                &options,
                selected,
            )
            .searchable(true)
            .show(ui);
            let _ = response;
        },
        None,
    );
    harness.run();
    harness
        .get_by_role_and_label(egui::accesskit::Role::ComboBox, "Device")
        .click();
    harness.run();
    harness
        .get_by_role_and_label(egui::accesskit::Role::TextInput, "Search")
        .type_text("mou");
    harness.run();
    harness.get_by_label("Mouse").click();
    harness.run();
    assert_eq!(*harness.state(), Some(target));
    assert_eq!(
        harness
            .get_by_role_and_label(egui::accesskit::Role::ComboBox, "Device")
            .accesskit_node()
            .value()
            .as_deref(),
        Some("Mouse")
    );
}

#[test]
fn escape_from_the_search_field_restores_focus_to_the_field() {
    let mut harness = Harness::builder().build_ui_state(
        |ui, selected: &mut Option<egui::Id>| {
            let options = [
                SelectOption::new(egui::Id::new("kbd"), "Keyboard"),
                SelectOption::new(egui::Id::new("mouse"), "Mouse"),
            ];
            Select::new(
                egui::Id::new("escape-search"),
                "Device",
                "Choose",
                &options,
                selected,
            )
            .searchable(true)
            .show(ui);
        },
        None,
    );
    harness.run();
    harness
        .get_by_role_and_label(egui::accesskit::Role::ComboBox, "Device")
        .click();
    harness.run();
    harness
        .get_by_role_and_label(egui::accesskit::Role::TextInput, "Search")
        .focus();
    harness.run();
    harness.key_press(egui::Key::Escape);
    harness.run();
    assert!(
        harness.query_by_label("Mouse").is_none(),
        "Escape should close the searchable popup"
    );
    assert!(
        harness
            .get_by_role_and_label(egui::accesskit::Role::ComboBox, "Device")
            .is_focused(),
        "Escape from the search field should restore focus to the field"
    );
}

#[test]
fn search_query_is_cleared_when_the_popup_reopens() {
    let mut harness = Harness::builder().build_ui_state(
        |ui, selected: &mut Option<egui::Id>| {
            let options = [
                SelectOption::new(egui::Id::new("kbd"), "Keyboard"),
                SelectOption::new(egui::Id::new("mouse"), "Mouse"),
            ];
            Select::new(
                egui::Id::new("reopen-search"),
                "Device",
                "Choose",
                &options,
                selected,
            )
            .searchable(true)
            .show(ui);
        },
        None,
    );
    harness.run();
    harness
        .get_by_role_and_label(egui::accesskit::Role::ComboBox, "Device")
        .click();
    harness.run();
    harness
        .get_by_role_and_label(egui::accesskit::Role::TextInput, "Search")
        .type_text("mouse");
    harness.run();
    assert!(harness.query_by_label("Keyboard").is_none());
    harness.key_press(egui::Key::Escape);
    harness.run();
    harness
        .get_by_role_and_label(egui::accesskit::Role::ComboBox, "Device")
        .click();
    harness.run();
    harness.get_by_label("Keyboard");
    assert_eq!(
        harness
            .get_by_role_and_label(egui::accesskit::Role::TextInput, "Search")
            .accesskit_node()
            .value()
            .as_deref(),
        Some(""),
        "reopening a searchable select should start from an empty query"
    );
}

#[test]
fn an_outside_click_dismisses_the_popup_without_stealing_focus() {
    let mut harness = Harness::builder().build_ui_state(
        |ui, selected: &mut Option<egui::Id>| {
            let options = [
                SelectOption::new(egui::Id::new("kbd"), "Keyboard"),
                SelectOption::new(egui::Id::new("mouse"), "Mouse"),
            ];
            Select::new(
                egui::Id::new("outside-search"),
                "Device",
                "Choose",
                &options,
                selected,
            )
            .searchable(true)
            .show(ui);
            ui.add_space(240.0);
            let _ = ui.button("Elsewhere");
        },
        None,
    );
    harness.run();
    harness
        .get_by_role_and_label(egui::accesskit::Role::ComboBox, "Device")
        .click();
    harness.run();
    harness
        .get_by_role_and_label(egui::accesskit::Role::TextInput, "Search")
        .focus();
    harness.run();
    harness.get_by_label("Elsewhere").click();
    harness.run();
    assert!(
        harness.query_by_label("Mouse").is_none(),
        "an outside click should dismiss the popup"
    );
    assert!(
        !harness
            .get_by_role_and_label(egui::accesskit::Role::ComboBox, "Device")
            .is_focused(),
        "an outside click must not move focus to the select field"
    );
}

#[test]
fn escape_with_focus_on_the_combo_restores_field_focus() {
    let mut harness = Harness::builder().build_ui_state(
        |ui, selected: &mut Option<egui::Id>| {
            let options = [
                SelectOption::new(egui::Id::new("kbd"), "Keyboard"),
                SelectOption::new(egui::Id::new("mouse"), "Mouse"),
            ];
            Select::new(
                egui::Id::new("combo-focus-escape"),
                "Device",
                "Choose",
                &options,
                selected,
            )
            .show(ui);
        },
        None,
    );
    harness.run();
    harness
        .get_by_role_and_label(egui::accesskit::Role::ComboBox, "Device")
        .click();
    harness.run();
    harness.get_by_label("Mouse");
    harness.key_press(egui::Key::Escape);
    harness.run();
    assert!(
        harness.query_by_label("Mouse").is_none(),
        "Escape should close the popup"
    );
    assert!(
        harness
            .get_by_role_and_label(egui::accesskit::Role::ComboBox, "Device")
            .is_focused(),
        "Escape with focus on the combo should restore field focus"
    );
}
