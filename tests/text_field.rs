use egui_kittest::{
    Harness,
    kittest::{NodeT, Queryable},
};
use temivu::{TextField, Theme};

#[test]
fn editing_updates_owned_value_and_validation_semantics() {
    let validate = |value: &str| {
        if value.contains('@') {
            None
        } else {
            Some("Invalid email".to_owned())
        }
    };
    let mut harness = Harness::builder().build_ui_state(
        |ui, value| {
            let result = TextField::new(value, &Theme::light(), "Email")
                .hint("name@example.com")
                .description("Work email")
                .validator(&validate)
                .max_chars(30)
                .show(ui);
            assert!(result.response.rect.is_positive());
            assert_eq!(result.error, validate(value));
        },
        String::new(),
    );
    harness.run();
    let node = harness.get_by_role_and_label(egui::accesskit::Role::TextInput, "Email");
    assert_eq!(
        node.accesskit_node().description().as_deref(),
        Some("Work email\nInvalid email")
    );
    assert_eq!(
        node.accesskit_node().data().invalid(),
        Some(egui::accesskit::Invalid::True)
    );
    node.click();
    node.type_text("a@b");
    harness.run();
    assert_eq!(harness.state(), "a@b");
    let node = harness.get_by_role_and_label(egui::accesskit::Role::TextInput, "Email");
    assert_eq!(
        node.accesskit_node().description().as_deref(),
        Some("Work email")
    );
    assert_eq!(node.accesskit_node().data().invalid(), None);
}

#[test]
fn disabled_field_remains_unchanged_and_password_is_masked() {
    let mut disabled = Harness::builder().build_ui_state(
        |ui, value| {
            let _ = TextField::new(value, &Theme::light(), "Disabled")
                .enabled(false)
                .read_only(true)
                .multiline(true)
                .show(ui);
        },
        "existing".to_owned(),
    );
    disabled.run();
    let node =
        disabled.get_by_role_and_label(egui::accesskit::Role::MultilineTextInput, "Disabled");
    assert!(node.accesskit_node().is_disabled());
    node.click();
    node.type_text("more");
    disabled.run();
    assert_eq!(disabled.state(), "existing");
    let mut password = Harness::builder().build_ui(|ui| {
        let mut secret = "secret".to_owned();
        let _ = TextField::new(&mut secret, &Theme::light(), "Password")
            .password(true)
            .show(ui);
    });
    password.run();
    let node = password.get_by_role_and_label(egui::accesskit::Role::PasswordInput, "Password");
    assert_ne!(node.accesskit_node().value().as_deref(), Some("secret"));
}

#[test]
fn read_only_field_is_selectable_and_cannot_be_edited() {
    let mut harness = Harness::builder().build_ui_state(
        |ui, value| {
            TextField::new(value, &Theme::light(), "Reference")
                .read_only(true)
                .show(ui);
        },
        "selectable".to_owned(),
    );
    harness.run();
    harness
        .get_by_role_and_label(egui::accesskit::Role::TextInput, "Reference")
        .focus();
    harness.run();
    assert!(
        harness
            .get_by_role_and_label(egui::accesskit::Role::TextInput, "Reference")
            .accesskit_node()
            .is_focused()
    );
    harness.key_press_modifiers(egui::Modifiers::COMMAND, egui::Key::A);
    harness.step();
    harness.event(egui::Event::Copy);
    harness.step();
    assert!(
        harness
            .output()
            .platform_output
            .commands
            .iter()
            .any(|cmd| matches!(cmd, egui::OutputCommand::CopyText(text) if text == "selectable"))
    );
    harness
        .get_by_role_and_label(egui::accesskit::Role::TextInput, "Reference")
        .type_text("changed");
    harness.run();
    assert_eq!(harness.state(), "selectable");
}

#[test]
fn stable_id_preserves_focus_when_a_preceding_sibling_appears() {
    let mut harness = Harness::builder().build_ui_state(
        |ui, state: &mut (String, bool)| {
            if state.1 {
                ui.label("Additional information");
            }
            TextField::new(&mut state.0, &Theme::light(), "Name")
                .id(egui::Id::unique("stable-name"))
                .show(ui);
        },
        (String::new(), false),
    );
    harness.run();
    harness
        .get_by_role_and_label(egui::accesskit::Role::TextInput, "Name")
        .focus();
    harness.run();
    assert!(
        harness
            .get_by_role_and_label(egui::accesskit::Role::TextInput, "Name")
            .accesskit_node()
            .is_focused()
    );
    harness.state_mut().1 = true;
    harness.run();
    let field = harness.get_by_role_and_label(egui::accesskit::Role::TextInput, "Name");
    assert!(field.accesskit_node().is_focused());
    field.type_text("Ada");
    harness.run();
    assert_eq!(harness.state().0, "Ada");
}

#[test]
fn touch_field_paints_one_outline_at_the_same_bounds_after_focus() {
    let mut harness = Harness::builder().build_ui(|ui| {
        let mut theme = Theme::light().touch();
        theme.motion.reduced_motion = true;
        let mut value = String::new();
        TextField::new(&mut value, &theme, "Name")
            .radius(19)
            .focus_stroke(egui::Stroke::new(3.0, egui::Color32::RED))
            .show(ui);
    });
    harness.run();
    let node = harness.get_by_role_and_label(egui::accesskit::Role::TextInput, "Name");
    let bounds = node.rect();
    assert!(bounds.height() >= 48.0);
    node.focus();
    harness.run();
    let focused = harness.get_by_role_and_label(egui::accesskit::Role::TextInput, "Name");
    assert_eq!(focused.rect(), bounds);
    let outlines: Vec<_> = harness
        .output()
        .shapes
        .iter()
        .filter_map(|entry| match &entry.shape {
            egui::Shape::Rect(rect) if rect.rect == bounds && rect.stroke.width > 0.0 => Some(rect),
            _ => None,
        })
        .collect();
    assert_eq!(outlines.len(), 1);
    assert_eq!(outlines[0].corner_radius.nw, 19);
    assert_eq!(
        outlines[0].stroke,
        egui::Stroke::new(3.0, egui::Color32::RED)
    );
    assert_eq!(outlines[0].stroke_kind, egui::StrokeKind::Inside);
}

#[test]
fn single_line_hint_and_value_share_the_configured_alignment() {
    let mut harness = Harness::builder()
        .with_size(egui::vec2(320.0, 240.0))
        .build_ui_state(
            |ui, (plain, centered): &mut (String, String)| {
                let theme = Theme::light();
                TextField::new(plain, &theme, "Plain")
                    .hint("Plain hint")
                    .show(ui);
                TextField::new(centered, &theme, "Centered")
                    .hint("Centered hint")
                    .alignment(egui::Align2::CENTER_CENTER)
                    .show(ui);
            },
            (String::new(), String::new()),
        );
    for (plain, centered) in [
        ("Plain hint", "Centered hint"),
        ("Plain value", "Centered value"),
    ] {
        harness.run();
        for (label, value) in [("Plain", plain), ("Centered", centered)] {
            let field = harness
                .get_by_role_and_label(egui::accesskit::Role::TextInput, label)
                .rect();
            let text = harness
                .output()
                .shapes
                .iter()
                .find_map(|entry| match &entry.shape {
                    egui::Shape::Text(text) if text.galley.job.text == value => {
                        Some(text.galley.rect.translate(text.pos.to_vec2()))
                    }
                    _ => None,
                })
                .expect("field text");
            assert!((text.center().y - field.center().y).abs() <= 1.0);
            if label == "Centered" {
                assert!((text.center().x - field.center().x).abs() <= 1.0);
            } else {
                assert!(text.center().x < field.center().x);
            }
        }
        *harness.state_mut() = ("Plain value".to_owned(), "Centered value".to_owned());
    }
}
