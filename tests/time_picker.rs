use egui_kittest::{
    Harness,
    kittest::{NodeT, Queryable},
};
use temivu::{Theme, Time, TimePicker, TimePickerLabels};

#[test]
fn keyboard_changes_time_and_announces_wheel_values() {
    let mut harness = Harness::builder().build_ui_state(
        |ui, time: &mut Time| {
            TimePicker::new(time, &Theme::light()).show(ui);
        },
        Time::new(9, 30).expect("a valid time"),
    );
    harness.run();
    let group = harness.get_by_role_and_label(egui::accesskit::Role::Group, "Time");
    assert_eq!(group.accesskit_node().value().as_deref(), Some("09:30"));

    harness
        .get_by_role_and_label(egui::accesskit::Role::ListBox, "Hour")
        .focus();
    harness.run();
    let hour_wheel = harness.get_by_role_and_label(egui::accesskit::Role::ListBox, "Hour");
    assert!(hour_wheel.is_focused());
    assert_eq!(hour_wheel.accesskit_node().value().as_deref(), Some("09"));
    harness.key_press(egui::Key::ArrowDown);
    harness.run();
    assert_eq!(harness.state().hour(), 10);
    let group = harness.get_by_role_and_label(egui::accesskit::Role::Group, "Time");
    assert_eq!(group.accesskit_node().value().as_deref(), Some("10:30"));
    assert_eq!(
        harness
            .get_by_role_and_label(egui::accesskit::Role::ListBox, "Hour")
            .accesskit_node()
            .value()
            .as_deref(),
        Some("10")
    );

    harness.key_press(egui::Key::ArrowUp);
    harness.run();
    assert_eq!(harness.state().hour(), 9);
    assert_eq!(harness.state().minute(), 30);

    // Hours clamp at the wheel ends rather than wrapping.
    harness
        .get_by_role_and_label(egui::accesskit::Role::ListBox, "Hour")
        .focus();
    harness.run();
    for _ in 0..24 {
        harness.key_press(egui::Key::ArrowUp);
        harness.run();
    }
    assert_eq!(harness.state().hour(), 0);
}

#[test]
fn left_and_right_keys_move_focus_between_wheels() {
    let mut harness = Harness::builder().build_ui_state(
        |ui, time: &mut Time| {
            TimePicker::new(time, &Theme::light())
                .seconds(true)
                .show(ui);
        },
        Time::new(9, 30).expect("a valid time"),
    );
    harness.run();
    harness
        .get_by_role_and_label(egui::accesskit::Role::ListBox, "Hour")
        .focus();
    harness.run();
    harness.key_press(egui::Key::ArrowRight);
    harness.run();
    assert!(
        harness
            .get_by_role_and_label(egui::accesskit::Role::ListBox, "Minute")
            .is_focused()
    );
    harness.key_press(egui::Key::ArrowRight);
    harness.run();
    assert!(
        harness
            .get_by_role_and_label(egui::accesskit::Role::ListBox, "Second")
            .is_focused()
    );
    harness.key_press(egui::Key::ArrowLeft);
    harness.run();
    assert!(
        harness
            .get_by_role_and_label(egui::accesskit::Role::ListBox, "Minute")
            .is_focused()
    );
    harness.key_press(egui::Key::ArrowLeft);
    harness.run();
    assert!(
        harness
            .get_by_role_and_label(egui::accesskit::Role::ListBox, "Hour")
            .is_focused()
    );
}

#[test]
fn pointer_selection_updates_minutes_and_focus() {
    let mut harness = Harness::builder()
        .with_size(egui::vec2(420.0, 320.0))
        .build_ui_state(
            |ui, time: &mut Time| {
                TimePicker::new(time, &Theme::light())
                    .labels(TimePickerLabels {
                        group: "Appointment time",
                        hour: "Hour",
                        minute: "Minute",
                        second: "Second",
                    })
                    .show(ui);
            },
            Time::new(9, 30).expect("a valid time"),
        );
    harness.run();
    let option = harness.get_by_role_and_label(egui::accesskit::Role::ListBoxOption, "32");
    option.click();
    harness.run();
    assert_eq!(harness.state().minute(), 32);
    assert_eq!(harness.state().hour(), 9);
    let group = harness.get_by_role_and_label(egui::accesskit::Role::Group, "Appointment time");
    assert_eq!(group.accesskit_node().value().as_deref(), Some("09:32"));
    assert!(
        harness
            .get_by_role_and_label(egui::accesskit::Role::ListBox, "Minute")
            .is_focused(),
        "selecting a wheel row should move focus to its wheel"
    );
}

#[test]
fn seconds_wheel_reports_full_value_and_clamps() {
    let mut harness = Harness::builder().build_ui_state(
        |ui, time: &mut Time| {
            TimePicker::new(time, &Theme::light())
                .seconds(true)
                .show(ui);
        },
        Time::with_seconds(23, 59, 58).expect("a valid time"),
    );
    harness.run();
    let group = harness.get_by_role_and_label(egui::accesskit::Role::Group, "Time");
    assert_eq!(group.accesskit_node().value().as_deref(), Some("23:59:58"));
    assert_eq!(
        harness
            .get_by_role_and_label(egui::accesskit::Role::ListBox, "Second")
            .accesskit_node()
            .value()
            .as_deref(),
        Some("58")
    );
    harness
        .get_by_role_and_label(egui::accesskit::Role::ListBox, "Second")
        .focus();
    harness.run();
    harness.key_press(egui::Key::ArrowDown);
    harness.run();
    assert_eq!(harness.state().second(), 59);
    harness.key_press(egui::Key::ArrowDown);
    harness.run();
    assert_eq!(harness.state().second(), 59, "seconds clamp at 59");
    assert_eq!(harness.state().hour(), 23);
    assert_eq!(harness.state().minute(), 59);
}

#[test]
fn disabled_time_picker_keeps_caller_value() {
    let mut harness = Harness::builder().build_ui_state(
        |ui, time: &mut Time| {
            TimePicker::new(time, &Theme::light())
                .enabled(false)
                .show(ui);
        },
        Time::new(12, 0).expect("a valid time"),
    );
    harness.run();
    assert!(
        harness
            .get_by_role_and_label(egui::accesskit::Role::ListBox, "Hour")
            .accesskit_node()
            .is_disabled()
    );
    harness
        .get_by_role_and_label(egui::accesskit::Role::ListBox, "Hour")
        .focus();
    harness.run();
    harness.key_press(egui::Key::ArrowDown);
    harness.run();
    assert_eq!(*harness.state(), Time::new(12, 0).expect("a valid time"));
}
