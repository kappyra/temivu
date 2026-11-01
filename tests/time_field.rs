use egui_kittest::{
    Harness,
    kittest::{NodeT, Queryable},
};
use temivu::{Messages, Theme, Time, TimeField, TimeFieldState};

struct State {
    field: TimeFieldState,
    changed: bool,
    error: Option<String>,
}

#[derive(Clone, Copy)]
struct Options {
    hint: Option<&'static str>,
    invalid: Option<&'static str>,
    messages: Messages,
    enabled: bool,
}

impl Default for Options {
    fn default() -> Self {
        Self {
            hint: None,
            invalid: None,
            messages: Messages::for_locale("en"),
            enabled: true,
        }
    }
}

fn build(options: Options) -> Harness<'static, State> {
    Harness::builder()
        .with_size(egui::vec2(380.0, 200.0))
        .build_ui_state(
            move |ui, state: &mut State| {
                let theme = Theme::light();
                let mut field = TimeField::new(&mut state.field, &theme, "Start time")
                    .messages(options.messages)
                    .enabled(options.enabled);
                if let Some(hint) = options.hint {
                    field = field.hint(hint);
                }
                if let Some(invalid) = options.invalid {
                    field = field.invalid_message(invalid);
                }
                let output = field.show(ui);
                state.changed |= output.changed;
                state.error = output.error;
            },
            State {
                field: TimeFieldState::new(),
                changed: false,
                error: None,
            },
        )
}

fn focus(harness: &mut Harness<'static, State>) {
    harness
        .get_by_role_and_label(egui::accesskit::Role::TextInput, "Start time")
        .click();
    harness.run();
}

#[test]
fn typing_a_valid_time_updates_the_caller_value() {
    let mut harness = build(Options::default());
    harness.run();
    focus(&mut harness);

    harness
        .get_by_role_and_label(egui::accesskit::Role::TextInput, "Start time")
        .type_text("09:30");
    harness.run();

    assert_eq!(harness.state().field.text(), "09:30");
    assert_eq!(
        harness.state().field.value(),
        Time::with_seconds(9, 30, 0).ok()
    );
    assert_eq!(harness.state().error, None);
    assert!(harness.state().changed);
    let node = harness.get_by_role_and_label(egui::accesskit::Role::TextInput, "Start time");
    assert_eq!(node.value().as_deref(), Some("09:30"));
    assert_eq!(node.accesskit_node().data().invalid(), None);
}

#[test]
fn invalid_input_shows_an_error_and_clears_the_value() {
    let mut harness = build(Options::default());
    harness.run();
    focus(&mut harness);

    harness
        .get_by_role_and_label(egui::accesskit::Role::TextInput, "Start time")
        .type_text("25:00");
    harness.run();

    assert_eq!(harness.state().field.text(), "25:00");
    assert_eq!(harness.state().field.value(), None);
    assert_eq!(harness.state().error.as_deref(), Some("Invalid time."));
    let node = harness.get_by_role_and_label(egui::accesskit::Role::TextInput, "Start time");
    assert_eq!(
        node.accesskit_node().data().invalid(),
        Some(egui::accesskit::Invalid::True)
    );
    assert_eq!(
        node.accesskit_node().description().as_deref(),
        Some("Invalid time.")
    );
}

#[test]
fn seconds_and_short_forms_are_accepted() {
    let mut harness = build(Options::default());
    harness.run();
    focus(&mut harness);

    harness
        .get_by_role_and_label(egui::accesskit::Role::TextInput, "Start time")
        .type_text("9:5:7");
    harness.run();
    assert_eq!(
        harness.state().field.value(),
        Time::with_seconds(9, 5, 7).ok()
    );
    assert_eq!(harness.state().error, None);

    // Clear and use a two-digit seconds form.
    harness
        .get_by_role_and_label(egui::accesskit::Role::TextInput, "Start time")
        .type_text(":00");
    harness.run();
    // Appending turns "9:5:7" into an invalid value until corrected.
    assert_eq!(harness.state().error.as_deref(), Some("Invalid time."));
}

#[test]
fn custom_hint_and_invalid_message_are_used() {
    let mut harness = build(Options {
        hint: Some("HH:MM"),
        invalid: Some("Use HH:MM"),
        ..Options::default()
    });
    harness.run();
    focus(&mut harness);
    let node = harness.get_by_role_and_label(egui::accesskit::Role::TextInput, "Start time");
    assert_eq!(node.accesskit_node().placeholder(), Some("HH:MM"));

    node.type_text("nope");
    harness.run();
    assert_eq!(harness.state().error.as_deref(), Some("Use HH:MM"));
}

#[test]
fn localized_messages_change_the_hint() {
    let mut harness = build(Options {
        messages: Messages::for_locale("de"),
        ..Options::default()
    });
    harness.run();
    let node = harness.get_by_role_and_label(egui::accesskit::Role::TextInput, "Start time");
    let placeholder = node
        .accesskit_node()
        .placeholder()
        .unwrap_or_default()
        .to_owned();
    assert_eq!(placeholder, "Wähle eine Uhrzeit");
}

#[test]
fn empty_text_has_no_error_and_no_value() {
    let mut harness = build(Options::default());
    harness.run();
    assert_eq!(harness.state().field.value(), None);
    assert_eq!(harness.state().error, None);
    let node = harness.get_by_role_and_label(egui::accesskit::Role::TextInput, "Start time");
    assert_eq!(node.accesskit_node().data().invalid(), None);
}

#[test]
fn disabled_field_ignores_input() {
    let mut harness = build(Options {
        enabled: false,
        ..Options::default()
    });
    harness.run();
    let node = harness.get_by_role_and_label(egui::accesskit::Role::TextInput, "Start time");
    assert!(node.accesskit_node().is_disabled());

    harness
        .input_mut()
        .events
        .push(egui::Event::Text("09:30".to_owned()));
    harness.run();
    assert_eq!(harness.state().field.text(), "");
    assert_eq!(harness.state().field.value(), None);
}
