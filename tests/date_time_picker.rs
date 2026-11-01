use egui_kittest::{
    Harness,
    kittest::{NodeT, Queryable},
};
use temivu::{
    Date, DateTime, DateTimePicker, DateTimePickerLabels, DateTimePickerState, Theme, Time,
};

fn date(year: i32, month: u8, day: u8) -> Date {
    Date::new(year, month, day).expect("a valid date")
}

fn time(hour: u8, minute: u8) -> Time {
    Time::new(hour, minute).expect("a valid time")
}

type PickerState = (DateTimePickerState, bool, Option<DateTime>);

#[test]
fn clicking_a_day_updates_the_combined_date_time_value() {
    let mut harness = Harness::builder()
        .with_size(egui::vec2(520.0, 720.0))
        .build_ui_state(
            |ui, state: &mut PickerState| {
                let output = DateTimePicker::new(&mut state.0, &Theme::light()).show(ui);
                state.1 |= output.changed;
                state.2 = output.selected;
            },
            (
                DateTimePickerState::new(date(2024, 7, 1), None),
                false,
                None,
            ),
        );
    harness.run();
    harness.get_by_role_and_label(egui::accesskit::Role::Heading, "July 2024");
    harness
        .get_by_role_and_label(egui::accesskit::Role::GridCell, "Monday, July 15, 2024")
        .click();
    harness.run();
    assert_eq!(
        harness.state().0.selected(),
        Some(DateTime::new(date(2024, 7, 15), Time::MIDNIGHT))
    );
    assert!(harness.state().1);
    let group = harness.get_by_role_and_label(egui::accesskit::Role::Group, "Date and time");
    assert_eq!(
        group.accesskit_node().value().as_deref(),
        Some("Monday, July 15, 2024, 00:00")
    );
    assert_eq!(
        harness
            .get_by_role_and_label(egui::accesskit::Role::GridCell, "Monday, July 15, 2024")
            .accesskit_node()
            .is_selected(),
        Some(true)
    );
    // The time wheels are part of the same component and stay caller-owned.
    harness.get_by_role_and_label(egui::accesskit::Role::ListBox, "Hour");
    harness.get_by_role_and_label(egui::accesskit::Role::ListBox, "Minute");
}

#[test]
fn keyboard_time_selection_keeps_the_date_and_updates_the_group() {
    let selected = DateTime::new(date(2024, 7, 15), time(9, 30));
    let mut harness = Harness::builder()
        .with_size(egui::vec2(520.0, 720.0))
        .build_ui_state(
            |ui, state: &mut PickerState| {
                let output = DateTimePicker::new(&mut state.0, &Theme::light()).show(ui);
                state.1 |= output.changed;
                state.2 = output.selected;
            },
            (DateTimePickerState::from_selected(selected), false, None),
        );
    harness.run();
    harness
        .get_by_role_and_label(egui::accesskit::Role::ListBox, "Minute")
        .focus();
    harness.run();
    harness.key_press(egui::Key::ArrowDown);
    harness.run();
    assert_eq!(
        harness.state().0.selected(),
        Some(DateTime::new(date(2024, 7, 15), time(9, 31)))
    );
    assert_eq!(harness.state().2, harness.state().0.selected());
    let group = harness.get_by_role_and_label(egui::accesskit::Role::Group, "Date and time");
    assert_eq!(
        group.accesskit_node().value().as_deref(),
        Some("Monday, July 15, 2024, 09:31")
    );
}

#[test]
fn bounds_are_forwarded_to_the_calendar() {
    let mut harness = Harness::builder()
        .with_size(egui::vec2(520.0, 720.0))
        .build_ui_state(
            |ui, state: &mut PickerState| {
                let output = DateTimePicker::new(&mut state.0, &Theme::light())
                    .bounds(date(2024, 7, 10), date(2024, 7, 20))
                    .show(ui);
                state.1 |= output.changed;
                state.2 = output.selected;
            },
            (
                DateTimePickerState::new(date(2024, 7, 1), None),
                false,
                None,
            ),
        );
    harness.run();
    let out_of_bounds =
        harness.get_by_role_and_label(egui::accesskit::Role::GridCell, "Monday, July 1, 2024");
    assert!(out_of_bounds.accesskit_node().is_disabled());
    out_of_bounds.click();
    harness.run();
    assert_eq!(harness.state().0.selected(), None);

    // June is out of bounds, so the previous-month button is disabled.
    let previous = harness.get_by_role_and_label(egui::accesskit::Role::Button, "Previous month");
    assert!(previous.accesskit_node().is_disabled());
    previous.click();
    harness.run();
    harness.get_by_role_and_label(egui::accesskit::Role::Heading, "July 2024");

    harness
        .get_by_role_and_label(egui::accesskit::Role::GridCell, "Monday, July 15, 2024")
        .click();
    harness.run();
    assert_eq!(
        harness.state().0.selected(),
        Some(DateTime::new(date(2024, 7, 15), Time::MIDNIGHT))
    );
}

#[test]
fn keyboard_grid_navigation_selects_and_keeps_focus() {
    let today = date(2024, 7, 1);
    let mut harness = Harness::builder()
        .with_size(egui::vec2(520.0, 720.0))
        .build_ui_state(
            move |ui, state: &mut PickerState| {
                let output = DateTimePicker::new(&mut state.0, &Theme::light())
                    .today(today)
                    .show(ui);
                state.1 |= output.changed;
                state.2 = output.selected;
            },
            (DateTimePickerState::new(today, None), false, None),
        );
    harness.run();
    harness
        .get_by_role_and_label(egui::accesskit::Role::Grid, "Calendar")
        .focus();
    harness.run();
    harness.key_press(egui::Key::ArrowRight);
    harness.run();
    harness.key_press(egui::Key::Enter);
    harness.run();
    assert_eq!(
        harness.state().0.selected(),
        Some(DateTime::new(date(2024, 7, 2), Time::MIDNIGHT))
    );
    // The selected day is the grid's AccessKit active descendant, so keyboard
    // navigation keeps driving the same grid.
    harness.key_press(egui::Key::ArrowRight);
    harness.run();
    harness.key_press(egui::Key::Space);
    harness.run();
    assert_eq!(
        harness.state().0.selected(),
        Some(DateTime::new(date(2024, 7, 3), Time::MIDNIGHT))
    );
}

#[test]
fn seconds_and_custom_labels_are_forwarded() {
    let selected = DateTime::new(
        date(2024, 7, 15),
        Time::with_seconds(9, 30, 5).expect("a valid time"),
    );
    let labels = DateTimePickerLabels {
        group: "Appointment",
        empty_value: "Nothing scheduled",
        ..DateTimePickerLabels::english()
    };
    let mut harness = Harness::builder()
        .with_size(egui::vec2(520.0, 720.0))
        .build_ui_state(
            move |ui, state: &mut PickerState| {
                let output = DateTimePicker::new(&mut state.0, &Theme::light())
                    .labels(labels)
                    .seconds(true)
                    .show(ui);
                state.1 |= output.changed;
                state.2 = output.selected;
            },
            (DateTimePickerState::from_selected(selected), false, None),
        );
    harness.run();
    assert_eq!(
        harness
            .get_by_role_and_label(egui::accesskit::Role::ListBox, "Second")
            .accesskit_node()
            .value()
            .as_deref(),
        Some("05")
    );
    let group = harness.get_by_role_and_label(egui::accesskit::Role::Group, "Appointment");
    assert_eq!(
        group.accesskit_node().value().as_deref(),
        Some("Monday, July 15, 2024, 09:30:05")
    );
}

#[test]
fn empty_selection_reports_the_configured_empty_value() {
    let labels = DateTimePickerLabels {
        group: "Meeting",
        empty_value: "Nothing scheduled",
        ..DateTimePickerLabels::english()
    };
    let mut harness = Harness::builder()
        .with_size(egui::vec2(520.0, 720.0))
        .build_ui_state(
            move |ui, state: &mut PickerState| {
                let output = DateTimePicker::new(&mut state.0, &Theme::light())
                    .labels(labels)
                    .enabled(false)
                    .show(ui);
                state.1 |= output.changed;
                state.2 = output.selected;
            },
            (
                DateTimePickerState::new(date(2024, 7, 1), None),
                false,
                None,
            ),
        );
    harness.run();
    let group = harness.get_by_role_and_label(egui::accesskit::Role::Group, "Meeting");
    assert_eq!(
        group.accesskit_node().value().as_deref(),
        Some("Nothing scheduled")
    );
    assert!(group.accesskit_node().is_disabled());
    assert!(
        harness
            .get_by_role_and_label(egui::accesskit::Role::GridCell, "Monday, July 15, 2024")
            .accesskit_node()
            .is_disabled()
    );
    assert!(
        harness
            .get_by_role_and_label(egui::accesskit::Role::ListBox, "Hour")
            .accesskit_node()
            .is_disabled()
    );
    harness
        .get_by_role_and_label(egui::accesskit::Role::GridCell, "Monday, July 15, 2024")
        .click();
    harness.run();
    assert_eq!(harness.state().0.selected(), None);
}
