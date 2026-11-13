use egui_kittest::{
    Harness,
    kittest::{NodeT, Queryable},
};
use temivu::{OtpField, OtpLabels, Theme};

type OtpState = (String, bool, bool);

fn build() -> Harness<'static, OtpState> {
    build_with(6, true, None)
}

fn build_with(
    length: usize,
    enabled: bool,
    error: Option<&'static str>,
) -> Harness<'static, OtpState> {
    Harness::builder()
        .with_size(egui::vec2(420.0, 160.0))
        .build_ui_state(
            move |ui, state: &mut OtpState| {
                let theme = Theme::light();
                let mut field = OtpField::new(&mut state.0, &theme, "One-time password")
                    .length(length)
                    .enabled(enabled);
                if let Some(error) = error {
                    field = field.error(error);
                }
                let output = field.show(ui);
                state.1 |= output.changed;
                state.2 = output.completed;
            },
            (String::new(), false, false),
        )
}

fn focus(harness: &mut Harness<'static, OtpState>) {
    harness
        .get_by_role_and_label(egui::accesskit::Role::TextInput, "One-time password")
        .focus();
    harness.run();
}

fn box_label(index: usize, length: usize) -> String {
    format!("Digit {} of {length}", index + 1)
}

#[test]
fn typing_digits_fills_boxes_and_updates_caller_state() {
    let mut harness = build();
    harness.run();
    focus(&mut harness);

    harness
        .get_by_role_and_label(egui::accesskit::Role::TextInput, "One-time password")
        .type_text("123");
    harness.run();

    assert_eq!(harness.state().0, "123");
    assert!(harness.state().1, "typing should mark the response changed");
    assert!(!harness.state().2);
    assert_eq!(
        harness
            .get_by_role_and_label(egui::accesskit::Role::TextInput, "One-time password")
            .value()
            .as_deref(),
        Some("123")
    );
    for (index, digit) in ["1", "2", "3"].into_iter().enumerate() {
        let label = box_label(index, 6);
        let box_node = harness.get_by_role_and_label(egui::accesskit::Role::TextInput, &label);
        assert_eq!(box_node.value().as_deref(), Some(digit));
    }
    let empty_label = box_label(3, 6);
    let empty = harness.get_by_role_and_label(egui::accesskit::Role::TextInput, &empty_label);
    assert_eq!(empty.value(), None);
}

#[test]
fn non_digit_input_is_filtered() {
    let mut harness = build();
    harness.run();
    focus(&mut harness);

    harness
        .get_by_role_and_label(egui::accesskit::Role::TextInput, "One-time password")
        .type_text("1a2b3!");
    harness.run();

    assert_eq!(harness.state().0, "123");
}

#[test]
fn typing_more_than_the_length_keeps_the_first_digits() {
    let mut harness = build();
    harness.run();
    focus(&mut harness);

    harness
        .get_by_role_and_label(egui::accesskit::Role::TextInput, "One-time password")
        .type_text("1234567890");
    harness.run();

    assert_eq!(harness.state().0, "123456");
    assert!(harness.state().2, "a full code should report completed");
}

#[test]
fn backspace_removes_digits_and_moves_the_caret_back() {
    let mut harness = build();
    harness.run();
    focus(&mut harness);

    harness
        .get_by_role_and_label(egui::accesskit::Role::TextInput, "One-time password")
        .type_text("123");
    harness.run();

    harness.key_press(egui::Key::Backspace);
    harness.run();
    assert_eq!(harness.state().0, "12");

    harness.key_press(egui::Key::Backspace);
    harness.run();
    assert_eq!(harness.state().0, "1");

    // Backspace at the first box deletes its digit and stays put.
    harness.key_press(egui::Key::ArrowLeft);
    harness.run();
    harness.key_press(egui::Key::Backspace);
    harness.run();
    assert_eq!(harness.state().0, "");
}

#[test]
fn arrow_keys_move_the_caret_and_typing_replaces_digits() {
    let mut harness = build();
    harness.run();
    focus(&mut harness);

    harness
        .get_by_role_and_label(egui::accesskit::Role::TextInput, "One-time password")
        .type_text("12");
    harness.run();
    assert_eq!(harness.state().0, "12");

    harness.key_press(egui::Key::ArrowLeft);
    harness.run();
    harness
        .get_by_role_and_label(egui::accesskit::Role::TextInput, "One-time password")
        .type_text("9");
    harness.run();
    assert_eq!(harness.state().0, "19");

    harness.key_press(egui::Key::ArrowRight);
    harness.run();
    harness
        .get_by_role_and_label(egui::accesskit::Role::TextInput, "One-time password")
        .type_text("8");
    harness.run();
    assert_eq!(harness.state().0, "198");
}

#[test]
fn pasting_filters_digits_and_fills_the_code() {
    let mut harness = build();
    harness.run();
    focus(&mut harness);

    harness
        .input_mut()
        .events
        .push(egui::Event::Paste("12a3456".to_owned()));
    harness.run();

    assert_eq!(harness.state().0, "123456");
    assert!(harness.state().2);
}

#[test]
fn delete_removes_the_digit_under_the_caret() {
    let mut harness = build();
    harness.run();
    focus(&mut harness);

    harness
        .get_by_role_and_label(egui::accesskit::Role::TextInput, "One-time password")
        .type_text("123");
    harness.run();
    // The caret rests on the empty box after the code, so Delete is a no-op.
    harness.key_press(egui::Key::Delete);
    harness.run();
    assert_eq!(harness.state().0, "123");

    harness.key_press(egui::Key::ArrowLeft);
    harness.run();
    harness.key_press(egui::Key::Delete);
    harness.run();
    assert_eq!(harness.state().0, "12");

    harness.key_press(egui::Key::ArrowLeft);
    harness.run();
    harness.key_press(egui::Key::Delete);
    harness.run();
    assert_eq!(harness.state().0, "1");
}

#[test]
fn clicking_a_box_focuses_the_group_and_places_the_caret() {
    let mut harness = Harness::builder()
        .with_size(egui::vec2(420.0, 160.0))
        .build_ui_state(
            |ui, value: &mut String| {
                let theme = Theme::light();
                OtpField::new(value, &theme, "One-time password").show(ui);
            },
            "12".to_owned(),
        );
    harness.run();
    // Clicking the third box targets the first empty box after the code.
    let third_label = box_label(2, 6);
    let third_box = harness.get_by_role_and_label(egui::accesskit::Role::TextInput, &third_label);
    third_box.click();
    harness.run();
    harness
        .get_by_role_and_label(egui::accesskit::Role::TextInput, "One-time password")
        .type_text("7");
    harness.run();
    assert_eq!(harness.state(), "127");
}

#[test]
fn disabled_field_exposes_state_and_ignores_input() {
    let mut harness = build_with(6, false, None);
    harness.run();

    let disabled =
        harness.get_by_role_and_label(egui::accesskit::Role::TextInput, "One-time password");
    assert!(disabled.accesskit_node().is_disabled());
    disabled.click();
    harness.run();
    harness
        .get_by_role_and_label(egui::accesskit::Role::TextInput, "One-time password")
        .focus();
    harness.run();

    harness
        .input_mut()
        .events
        .push(egui::Event::Text("1".to_owned()));
    harness.run();
    assert_eq!(harness.state().0, "");
}

#[test]
fn invalid_state_is_exposed_on_the_group_node() {
    let mut harness = build_with(6, true, Some("Invalid code"));
    harness.run();

    let node = harness.get_by_role_and_label(egui::accesskit::Role::TextInput, "One-time password");
    assert_eq!(
        node.accesskit_node().data().invalid(),
        Some(egui::accesskit::Invalid::True)
    );
    harness.get_by_label("Invalid code");
}

#[test]
fn length_is_configurable_and_labels_reflect_it() {
    let mut harness = build_with(4, true, None);
    harness.run();
    focus(&mut harness);
    harness
        .get_by_role_and_label(egui::accesskit::Role::TextInput, "One-time password")
        .type_text("123456");
    harness.run();
    assert_eq!(harness.state().0, "1234");
    assert!(harness.state().2);
    harness.get_by_role_and_label(egui::accesskit::Role::TextInput, "Digit 4 of 4");
    assert!(
        harness
            .query_by_role_and_label(egui::accesskit::Role::TextInput, "Digit 5 of 4")
            .is_none()
    );
}

#[test]
fn caller_labels_override_the_english_defaults() {
    let labels = OtpLabels {
        digit: &|index: usize, length: usize| format!("Code {} of {length}", index + 1),
    };
    let mut harness = Harness::builder()
        .with_size(egui::vec2(420.0, 160.0))
        .build_ui_state(
            move |ui, value: &mut String| {
                let theme = Theme::light();
                OtpField::new(value, &theme, "Security code")
                    .labels(labels)
                    .show(ui);
            },
            String::new(),
        );
    harness.run();
    harness.get_by_role_and_label(egui::accesskit::Role::TextInput, "Code 1 of 6");
    assert!(
        harness
            .query_by_role_and_label(egui::accesskit::Role::TextInput, "Digit 1 of 6")
            .is_none()
    );
}

#[test]
fn a_prefilled_string_is_normalized_to_digits() {
    let mut harness = Harness::builder()
        .with_size(egui::vec2(420.0, 160.0))
        .build_ui_state(
            |ui, value: &mut String| {
                let theme = Theme::light();
                OtpField::new(value, &theme, "One-time password").show(ui);
            },
            "a1b2c3".to_owned(),
        );
    harness.run();
    assert_eq!(harness.state(), "123");
    assert_eq!(group_value(&harness), Some("123".to_owned()));
}

fn group_value(harness: &Harness<'static, String>) -> Option<String> {
    harness
        .get_by_role_and_label(egui::accesskit::Role::TextInput, "One-time password")
        .value()
}

#[test]
fn explicit_ids_keep_two_fields_independent_and_stateful() {
    let mut harness = Harness::builder()
        .with_size(egui::vec2(420.0, 240.0))
        .build_ui_state(
            |ui, state: &mut (String, String)| {
                let theme = Theme::light();
                OtpField::new(&mut state.0, &theme, "First code")
                    .id(egui::Id::unique("first-code"))
                    .show(ui);
                OtpField::new(&mut state.1, &theme, "Second code")
                    .id(egui::Id::unique("second-code"))
                    .show(ui);
            },
            (String::new(), String::new()),
        );
    harness.run();
    harness
        .get_by_role_and_label(egui::accesskit::Role::TextInput, "First code")
        .focus();
    harness.run();
    harness
        .get_by_role_and_label(egui::accesskit::Role::TextInput, "First code")
        .type_text("1");
    harness.run();
    // The focused box must persist across frames, so the next digit appends.
    harness
        .get_by_role_and_label(egui::accesskit::Role::TextInput, "First code")
        .type_text("2");
    harness.run();

    assert_eq!(harness.state().0, "12");
    assert_eq!(harness.state().1, "");
    assert_eq!(
        harness
            .get_by_role_and_label(egui::accesskit::Role::TextInput, "Second code")
            .value()
            .as_deref(),
        Some("")
    );
}

#[test]
fn excessive_lengths_are_clamped_to_the_supported_maximum() {
    let mut harness = build_with(usize::MAX, true, None);
    harness.run();
    harness
        .get_by_role_and_label(egui::accesskit::Role::TextInput, "One-time password")
        .focus();
    harness.run();
    assert_eq!(
        harness
            .get_by_role_and_label(egui::accesskit::Role::TextInput, "One-time password")
            .accesskit_node()
            .label()
            .as_deref(),
        Some("One-time password")
    );
    let last = format!("Digit {} of {}", OtpField::MAX_LENGTH, OtpField::MAX_LENGTH);
    harness.get_by_role_and_label(egui::accesskit::Role::TextInput, &last);
    assert!(
        harness
            .query_by_role_and_label(
                egui::accesskit::Role::TextInput,
                &format!(
                    "Digit {} of {}",
                    OtpField::MAX_LENGTH + 1,
                    OtpField::MAX_LENGTH + 1
                ),
            )
            .is_none(),
        "no box beyond the supported maximum may be rendered"
    );
}
