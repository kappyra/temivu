use egui_kittest::{
    Harness,
    kittest::{NodeT, Queryable},
};
use temivu::{Select, SelectGroup, SelectOption, Theme, inherited_select_error};

struct State {
    color: Option<egui::Id>,
    enabled: bool,
    error: Option<&'static str>,
    saw_enabled: bool,
    inherited_error: Option<String>,
    changed: bool,
}

impl Default for State {
    fn default() -> Self {
        Self {
            color: None,
            enabled: true,
            error: None,
            saw_enabled: true,
            inherited_error: None,
            changed: false,
        }
    }
}

fn view(ui: &mut egui::Ui, state: &mut State) {
    let theme = Theme::light();
    let options = [
        SelectOption::new(egui::Id::unique("red"), "Red"),
        SelectOption::new(egui::Id::unique("blue"), "Blue"),
    ];
    let State {
        color,
        enabled,
        error,
        saw_enabled,
        inherited_error,
        changed,
    } = state;
    let mut group = SelectGroup::new(egui::Id::unique("shipping"), "Shipping color", &theme)
        .description("Used on the label")
        .enabled(*enabled);
    if let Some(error) = *error {
        group = group.error(error);
    }
    let response = group.show(ui, |ui| {
        *inherited_error = inherited_select_error(ui);
        *saw_enabled = ui.is_enabled();
        Select::new(
            egui::Id::unique("shipping-color"),
            "Color",
            "Choose a color",
            &options,
            color,
        )
        .show(ui)
    });
    *changed = response.invalid && response.inner.changed();
}

fn harness(state: State) -> Harness<'static, State> {
    Harness::builder()
        .with_size(egui::vec2(360.0, 320.0))
        .build_ui_state(view, state)
}

#[test]
fn group_exposes_its_role_label_and_description() {
    let mut harness = harness(State::default());
    harness.run();

    let group = harness.get_by_role_and_label(egui::accesskit::Role::Group, "Shipping color");
    assert_eq!(
        group.accesskit_node().description().as_deref(),
        Some("Used on the label")
    );
    harness.get_by_label("Shipping color");
    // The nested select is still usable and named.
    let field = harness.get_by_role_and_label(egui::accesskit::Role::ComboBox, "Color");
    assert!(!field.accesskit_node().is_disabled());
}

#[test]
fn pointer_selection_still_works_inside_an_enabled_group() {
    let mut harness = harness(State::default());
    harness.run();
    harness
        .get_by_role_and_label(egui::accesskit::Role::ComboBox, "Color")
        .click();
    harness.run();
    harness.get_by_label("Blue").click();
    harness.run();
    assert_eq!(harness.state().color, Some(egui::Id::unique("blue")));
}

#[test]
fn disabled_group_disables_its_fields_and_blocks_activation() {
    let mut harness = harness(State {
        enabled: false,
        ..State::default()
    });
    harness.run();
    assert!(
        !harness.state().saw_enabled,
        "the content callback should observe the inherited disabled state"
    );
    let field = harness.get_by_role_and_label(egui::accesskit::Role::ComboBox, "Color");
    assert!(field.accesskit_node().is_disabled());
    field.click();
    harness.run();
    assert!(harness.query_by_label("Red").is_none());
    assert_eq!(harness.state().color, None);
}

#[test]
fn validation_error_marks_the_group_and_its_fields_invalid() {
    let mut harness = harness(State {
        error: Some("Choose a color"),
        ..State::default()
    });
    harness.run();

    assert_eq!(
        harness.state().inherited_error.as_deref(),
        Some("Choose a color"),
        "the group should publish its validation message to child fields"
    );
    let group = harness.get_by_role_and_label(egui::accesskit::Role::Group, "Shipping color");
    assert_eq!(
        group.accesskit_node().data().invalid(),
        Some(egui::accesskit::Invalid::True)
    );
    assert!(
        group
            .accesskit_node()
            .description()
            .is_some_and(|description| description.contains("Choose a color"))
    );
    harness.get_by_label("Choose a color");

    let field = harness.get_by_role_and_label(egui::accesskit::Role::ComboBox, "Color");
    assert_eq!(
        field.accesskit_node().data().invalid(),
        Some(egui::accesskit::Invalid::True),
        "the select field should inherit the group's invalid state"
    );
    assert!(
        field
            .accesskit_node()
            .description()
            .is_some_and(|description| description.contains("Choose a color"))
    );
}

#[test]
fn a_valid_sibling_group_does_not_inherit_the_previous_error() {
    let mut harness = Harness::builder()
        .with_size(egui::vec2(360.0, 320.0))
        .build_ui_state(
            |ui, color: &mut Option<egui::Id>| {
                let theme = Theme::light();
                let options = [SelectOption::new(egui::Id::unique("red"), "Red")];
                SelectGroup::new(egui::Id::unique("invalid-group"), "Invalid group", &theme)
                    .error("Must choose")
                    .show(ui, |ui| {
                        Select::new(
                            egui::Id::unique("invalid-field"),
                            "First",
                            "Choose",
                            &options,
                            color,
                        )
                        .show(ui);
                    });
                SelectGroup::new(egui::Id::unique("valid-group"), "Valid group", &theme).show(
                    ui,
                    |ui| {
                        let _ = Select::new(
                            egui::Id::unique("valid-field"),
                            "Second",
                            "Choose",
                            &options,
                            &mut None,
                        )
                        .show(ui);
                    },
                );
            },
            None,
        );
    harness.run();
    let field = harness.get_by_role_and_label(egui::accesskit::Role::ComboBox, "Second");
    assert_ne!(
        field.accesskit_node().data().invalid(),
        Some(egui::accesskit::Invalid::True),
        "a sibling group must not inherit a previous group's error"
    );
}
