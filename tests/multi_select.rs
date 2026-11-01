use egui_kittest::{
    Harness,
    kittest::{NodeT, Queryable},
};
use temivu::{MultiSelect, SelectOption};

fn view(ui: &mut egui::Ui, state: &mut (Vec<egui::Id>, bool)) {
    let options = [
        SelectOption::new(egui::Id::new("email"), "Email"),
        SelectOption::new(egui::Id::new("sms"), "SMS"),
        SelectOption::new(egui::Id::new("push"), "Push"),
        SelectOption::new(egui::Id::new("inbox"), "Inbox").enabled(false),
    ];
    let response = MultiSelect::new(
        egui::Id::new("channels"),
        "Channels",
        "Choose channels",
        &options,
        &mut state.0,
    )
    .show(ui);
    state.1 |= response.changed();
}

fn open(harness: &mut Harness<'static, (Vec<egui::Id>, bool)>) {
    harness
        .get_by_role_and_label(egui::accesskit::Role::ComboBox, "Channels")
        .click();
    harness.run();
}

#[test]
fn toggling_options_keeps_the_popup_open_and_summarises() {
    let mut harness = Harness::builder().build_ui_state(view, (Vec::new(), false));
    harness.run();
    open(&mut harness);
    harness.get_by_label("Email").click();
    harness.run_steps(2);
    harness.get_by_label("SMS").click();
    harness.run_steps(2);
    assert_eq!(
        harness.state().0,
        vec![egui::Id::new("email"), egui::Id::new("sms")]
    );
    harness.get_by_label("Push");
    assert_eq!(
        harness
            .get_by_role_and_label(egui::accesskit::Role::ComboBox, "Channels")
            .accesskit_node()
            .value()
            .as_deref(),
        Some("Email, SMS")
    );
    assert_eq!(
        harness.get_by_label("Email").accesskit_node().is_selected(),
        Some(true)
    );
    assert!(harness.state().1, "toggling marks the response changed");
}

#[test]
fn keyboard_space_toggles_the_focused_option() {
    let mut harness = Harness::builder().build_ui_state(view, (Vec::new(), false));
    harness.run();
    open(&mut harness);
    harness.get_by_label("SMS").focus();
    harness.run();
    harness.key_press(egui::Key::Space);
    harness.run();
    assert_eq!(harness.state().0, vec![egui::Id::new("sms")]);
    harness.key_press(egui::Key::Space);
    harness.run();
    assert!(harness.state().0.is_empty(), "a second Space deselects");
}

#[test]
fn escape_closes_the_popup_and_restores_field_focus() {
    let mut harness = Harness::builder().build_ui_state(view, (Vec::new(), false));
    harness.run();
    open(&mut harness);
    harness.get_by_label("Email").focus();
    harness.run();
    harness.key_press(egui::Key::Escape);
    harness.run();
    assert!(
        harness.query_by_label("Email").is_none(),
        "Escape should close the popup"
    );
    assert!(
        harness
            .get_by_role_and_label(egui::accesskit::Role::ComboBox, "Channels")
            .is_focused(),
        "Escape should restore focus to the field"
    );
}

#[test]
fn disabled_options_cannot_be_toggled() {
    let mut harness = Harness::builder().build_ui_state(view, (Vec::new(), false));
    harness.run();
    open(&mut harness);
    let inbox = harness.get_by_label("Inbox");
    assert!(inbox.accesskit_node().is_disabled());
    inbox.click();
    harness.run();
    assert!(harness.state().0.is_empty());
}

#[test]
fn more_than_three_selections_show_a_count() {
    let selected = vec![
        egui::Id::new("email"),
        egui::Id::new("sms"),
        egui::Id::new("push"),
        egui::Id::new("inbox"),
    ];
    let mut harness = Harness::builder().build_ui_state(view, (selected, false));
    harness.run();
    assert_eq!(
        harness
            .get_by_role_and_label(egui::accesskit::Role::ComboBox, "Channels")
            .accesskit_node()
            .value()
            .as_deref(),
        Some("4 selected")
    );
}

#[test]
fn search_filters_multi_select_options() {
    let mut harness = Harness::builder().build_ui_state(
        |ui, selected: &mut Vec<egui::Id>| {
            let options = [
                SelectOption::new(egui::Id::new("email"), "Email"),
                SelectOption::new(egui::Id::new("sms"), "SMS"),
            ];
            MultiSelect::new(
                egui::Id::new("searchable-channels"),
                "Channels",
                "Choose channels",
                &options,
                selected,
            )
            .searchable(true)
            .show(ui);
        },
        Vec::new(),
    );
    harness.run();
    harness
        .get_by_role_and_label(egui::accesskit::Role::ComboBox, "Channels")
        .click();
    harness.run();
    harness
        .get_by_role_and_label(egui::accesskit::Role::TextInput, "Search")
        .type_text("sms");
    harness.run();
    harness.get_by_label("SMS");
    assert!(harness.query_by_label("Email").is_none());
}

#[test]
fn escape_from_search_restores_field_focus() {
    let mut harness = Harness::builder().build_ui_state(
        |ui, selected: &mut Vec<egui::Id>| {
            let options = [
                SelectOption::new(egui::Id::new("email"), "Email"),
                SelectOption::new(egui::Id::new("sms"), "SMS"),
            ];
            MultiSelect::new(
                egui::Id::new("search-escape-channels"),
                "Channels",
                "Choose channels",
                &options,
                selected,
            )
            .searchable(true)
            .show(ui);
        },
        Vec::new(),
    );
    harness.run();
    harness
        .get_by_role_and_label(egui::accesskit::Role::ComboBox, "Channels")
        .click();
    harness.run();
    harness
        .get_by_role_and_label(egui::accesskit::Role::TextInput, "Search")
        .focus();
    harness.run();
    harness.key_press(egui::Key::Escape);
    harness.run();
    assert!(harness.query_by_label("Email").is_none());
    assert!(
        harness
            .get_by_role_and_label(egui::accesskit::Role::ComboBox, "Channels")
            .is_focused(),
        "Escape from the search field should restore field focus"
    );
}

#[test]
fn an_outside_click_dismisses_the_multi_select_popup() {
    let mut harness = Harness::builder().build_ui_state(
        |ui, selected: &mut Vec<egui::Id>| {
            let options = [
                SelectOption::new(egui::Id::new("email"), "Email"),
                SelectOption::new(egui::Id::new("sms"), "SMS"),
            ];
            MultiSelect::new(
                egui::Id::new("outside-channels"),
                "Channels",
                "Choose channels",
                &options,
                selected,
            )
            .show(ui);
            ui.add_space(240.0);
            let _ = ui.button("Elsewhere");
        },
        Vec::new(),
    );
    harness.run();
    harness
        .get_by_role_and_label(egui::accesskit::Role::ComboBox, "Channels")
        .click();
    harness.run();
    harness.get_by_label("SMS");
    harness.get_by_label("Elsewhere").click();
    harness.run();
    assert!(
        harness.query_by_label("SMS").is_none(),
        "an outside click should dismiss the multi-select popup"
    );
}

#[test]
fn custom_count_summary_is_used() {
    let summary = |count: usize| format!("{count} ausgewählt");
    let selected = vec![
        egui::Id::new("email"),
        egui::Id::new("sms"),
        egui::Id::new("push"),
        egui::Id::new("inbox"),
    ];
    let mut harness = Harness::builder().build_ui_state(
        move |ui, selected: &mut Vec<egui::Id>| {
            let options = [
                SelectOption::new(egui::Id::new("email"), "Email"),
                SelectOption::new(egui::Id::new("sms"), "SMS"),
                SelectOption::new(egui::Id::new("push"), "Push"),
                SelectOption::new(egui::Id::new("inbox"), "Inbox"),
            ];
            MultiSelect::new(
                egui::Id::new("localized-channels"),
                "Channels",
                "Choose channels",
                &options,
                selected,
            )
            .count_summary(&summary)
            .show(ui);
        },
        selected,
    );
    harness.run();
    assert_eq!(
        harness
            .get_by_role_and_label(egui::accesskit::Role::ComboBox, "Channels")
            .accesskit_node()
            .value()
            .as_deref(),
        Some("4 ausgewählt")
    );
}

#[test]
fn escape_with_focus_on_the_combo_restores_field_focus() {
    let mut harness = Harness::builder().build_ui_state(view, (Vec::new(), false));
    harness.run();
    open(&mut harness);
    harness.get_by_label("SMS");
    harness.key_press(egui::Key::Escape);
    harness.run();
    assert!(harness.query_by_label("SMS").is_none());
    assert!(
        harness
            .get_by_role_and_label(egui::accesskit::Role::ComboBox, "Channels")
            .is_focused(),
        "Escape with focus on the combo should restore field focus"
    );
}
