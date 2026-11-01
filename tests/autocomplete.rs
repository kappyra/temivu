use egui_kittest::{
    Harness,
    kittest::{NodeT, Queryable},
};
use temivu::{Autocomplete, AutocompleteOption, Messages, Theme};

#[derive(Default)]
struct AutocompleteState {
    query: String,
    changed: bool,
    accepted: Option<egui::Id>,
    open: bool,
}

fn fruit_options(disabled: Option<&'static str>) -> Vec<AutocompleteOption<'static>> {
    [
        ("apple", "Apple"),
        ("apricot", "Apricot"),
        ("banana", "Banana"),
        ("cherry", "Cherry"),
    ]
    .into_iter()
    .map(|(id, label)| {
        let option = AutocompleteOption::new(egui::Id::new(id), label);
        match disabled {
            Some(disabled) if disabled == label => option.enabled(false),
            _ => option,
        }
    })
    .collect()
}

#[test]
fn typing_filters_suggestions_and_exposes_combobox_roles() {
    let mut harness = Harness::builder()
        .with_size(egui::vec2(360.0, 320.0))
        .build_ui_state(
            |ui, state: &mut AutocompleteState| {
                let candidates = fruit_options(None);
                let output =
                    Autocomplete::new(&mut state.query, &Theme::light(), "Fruit", &candidates)
                        .hint("Type a fruit")
                        .show(ui);
                state.changed |= output.changed;
                state.accepted = output.accepted.or(state.accepted);
                state.open = output.open;
            },
            AutocompleteState::default(),
        );
    harness.run();
    let field = harness.get_by_role_and_label(egui::accesskit::Role::ComboBox, "Fruit");
    assert_eq!(field.accesskit_node().value().as_deref(), Some(""));
    field.click();
    harness.run();
    assert!(
        harness.state().open,
        "focusing the field should open the list"
    );

    let field = harness.get_by_role_and_label(egui::accesskit::Role::ComboBox, "Fruit");
    field.type_text("ap");
    harness.run();
    assert_eq!(harness.state().query, "ap");
    let field = harness.get_by_role_and_label(egui::accesskit::Role::ComboBox, "Fruit");
    assert_eq!(field.accesskit_node().value().as_deref(), Some("ap"));
    assert_eq!(field.accesskit_node().data().is_expanded(), Some(true));
    harness.get_by_role_and_label(egui::accesskit::Role::ListBox, "Fruit");
    assert!(harness.query_by_label("Banana").is_none());
    assert!(harness.query_by_label("Cherry").is_none());
    assert_eq!(
        harness.get_by_label("Apple").accesskit_node().role(),
        egui::accesskit::Role::ListBoxOption
    );
    harness.get_by_label("Apricot");
}

#[test]
fn keyboard_navigation_skips_disabled_and_accepts() {
    let mut harness = Harness::builder()
        .with_size(egui::vec2(360.0, 320.0))
        .build_ui_state(
            |ui, state: &mut AutocompleteState| {
                let candidates = fruit_options(Some("Apricot"));
                let output =
                    Autocomplete::new(&mut state.query, &Theme::light(), "Fruit", &candidates)
                        .show(ui);
                state.changed |= output.changed;
                state.accepted = output.accepted.or(state.accepted);
                state.open = output.open;
            },
            AutocompleteState::default(),
        );
    harness.run();
    harness
        .get_by_role_and_label(egui::accesskit::Role::ComboBox, "Fruit")
        .click();
    harness.run();
    harness
        .get_by_role_and_label(egui::accesskit::Role::ComboBox, "Fruit")
        .type_text("ap");
    harness.run();
    assert!(
        harness
            .get_by_label("Apricot")
            .accesskit_node()
            .is_disabled(),
        "disabled suggestions stay listed but disabled"
    );

    harness.key_press(egui::Key::ArrowDown);
    harness.run();
    harness.key_press(egui::Key::ArrowDown);
    harness.run();
    harness.key_press(egui::Key::Enter);
    harness.run();
    assert_eq!(
        harness.state().query,
        "Apple",
        "disabled Apricot is skipped by keyboard navigation"
    );
    assert_eq!(harness.state().accepted, Some(egui::Id::new("apple")));
    assert!(harness.state().changed);
    assert!(
        !harness.state().open,
        "accepting closes the suggestion list"
    );
    assert!(
        harness
            .get_by_role_and_label(egui::accesskit::Role::ComboBox, "Fruit")
            .is_focused(),
        "accepting restores or keeps focus on the field"
    );
}

#[test]
fn escape_dismisses_without_changing_the_query() {
    let mut harness = Harness::builder()
        .with_size(egui::vec2(360.0, 320.0))
        .build_ui_state(
            |ui, state: &mut AutocompleteState| {
                let candidates = fruit_options(None);
                let output =
                    Autocomplete::new(&mut state.query, &Theme::light(), "Fruit", &candidates)
                        .show(ui);
                state.changed |= output.changed;
                state.accepted = output.accepted.or(state.accepted);
                state.open = output.open;
            },
            AutocompleteState::default(),
        );
    harness.run();
    harness
        .get_by_role_and_label(egui::accesskit::Role::ComboBox, "Fruit")
        .click();
    harness.run();
    harness
        .get_by_role_and_label(egui::accesskit::Role::ComboBox, "Fruit")
        .type_text("ap");
    harness.run();
    harness.get_by_label("Apple");
    harness.key_press(egui::Key::Escape);
    harness.run();
    assert_eq!(harness.state().query, "ap");
    assert!(!harness.state().open);
    assert!(
        harness.query_by_label("Apple").is_none(),
        "Escape closes the list"
    );
    assert!(
        harness
            .get_by_role_and_label(egui::accesskit::Role::ComboBox, "Fruit")
            .is_focused(),
        "Escape keeps focus on the field"
    );
}

#[test]
fn pointer_click_accepts_a_suggestion_and_returns_focus() {
    let mut harness = Harness::builder()
        .with_size(egui::vec2(360.0, 320.0))
        .build_ui_state(
            |ui, state: &mut AutocompleteState| {
                let candidates = fruit_options(None);
                let output =
                    Autocomplete::new(&mut state.query, &Theme::light(), "Fruit", &candidates)
                        .show(ui);
                state.changed |= output.changed;
                state.accepted = output.accepted.or(state.accepted);
                state.open = output.open;
            },
            AutocompleteState::default(),
        );
    harness.run();
    harness
        .get_by_role_and_label(egui::accesskit::Role::ComboBox, "Fruit")
        .click();
    harness.run();
    harness
        .get_by_role_and_label(egui::accesskit::Role::ComboBox, "Fruit")
        .type_text("ap");
    harness.run();
    harness.get_by_label("Apricot").click();
    harness.run();
    assert_eq!(harness.state().query, "Apricot");
    assert_eq!(harness.state().accepted, Some(egui::Id::new("apricot")));
    assert!(!harness.state().open);
    assert_eq!(
        harness
            .get_by_role_and_label(egui::accesskit::Role::ComboBox, "Fruit")
            .accesskit_node()
            .value()
            .as_deref(),
        Some("Apricot")
    );
    assert!(
        harness
            .get_by_role_and_label(egui::accesskit::Role::ComboBox, "Fruit")
            .is_focused(),
        "a pointer acceptance returns focus to the field"
    );
}

#[test]
fn no_results_is_a_localized_polite_alert() {
    let messages = Messages::for_locale("de");
    let expected = messages.autocomplete_no_results;
    let mut harness = Harness::builder()
        .with_size(egui::vec2(360.0, 320.0))
        .build_ui_state(
            move |ui, state: &mut AutocompleteState| {
                let candidates = fruit_options(None);
                let output =
                    Autocomplete::new(&mut state.query, &Theme::light(), "Fruit", &candidates)
                        .messages(messages)
                        .show(ui);
                state.changed |= output.changed;
                state.accepted = output.accepted.or(state.accepted);
                state.open = output.open;
            },
            AutocompleteState::default(),
        );
    harness.run();
    harness
        .get_by_role_and_label(egui::accesskit::Role::ComboBox, "Fruit")
        .click();
    harness.run();
    harness
        .get_by_role_and_label(egui::accesskit::Role::ComboBox, "Fruit")
        .type_text("zzz");
    harness.run();
    let alert = harness.get_by_role_and_label(egui::accesskit::Role::Alert, expected);
    assert!(alert.rect().is_positive());
    assert_eq!(alert.accesskit_node().live(), egui::accesskit::Live::Polite);
    assert!(harness.query_by_label("Apple").is_none());
    assert!(
        harness.state().open,
        "the empty result stays open as an alert"
    );
}

#[test]
fn disabled_autocomplete_ignores_input() {
    let mut harness = Harness::builder().build_ui_state(
        |ui, state: &mut AutocompleteState| {
            let candidates = fruit_options(None);
            let output = Autocomplete::new(&mut state.query, &Theme::light(), "Fruit", &candidates)
                .enabled(false)
                .show(ui);
            state.changed |= output.changed;
            state.open = output.open;
        },
        AutocompleteState {
            query: "existing".to_owned(),
            ..Default::default()
        },
    );
    harness.run();
    let field = harness.get_by_role_and_label(egui::accesskit::Role::ComboBox, "Fruit");
    assert!(field.accesskit_node().is_disabled());
    assert!(!harness.state().open);
    field.click();
    harness.run();
    harness
        .get_by_role_and_label(egui::accesskit::Role::ComboBox, "Fruit")
        .type_text("more");
    harness.run();
    assert_eq!(harness.state().query, "existing");
    assert!(harness.query_by_label("Apple").is_none());
}
