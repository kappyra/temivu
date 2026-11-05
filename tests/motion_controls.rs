use egui_kittest::{
    Harness,
    kittest::{NodeT, Queryable},
};
use temivu::{
    Direction, Theme, ViewMode, direction_pad, layout_toggle, quantity_stepper, search_box,
    toggle_button,
};

#[test]
fn stepper_reaches_both_bounds_and_folds_at_zero() {
    let mut harness = Harness::builder().build_ui_state(
        |ui, count| {
            quantity_stepper(ui, &Theme::light(), count, 0..=1, "Less", "More");
        },
        0,
    );
    harness.run_steps(5);
    assert!(harness.query_by_label("Less").is_none());
    harness.get_by_label("More").click();
    harness.run_steps(5);
    assert_eq!(*harness.state(), 1);
    assert!(harness.get_by_label("More").accesskit_node().is_disabled());
    harness.get_by_label("Less").click();
    harness.run_steps(5);
    assert_eq!(*harness.state(), 0);
    assert!(harness.query_by_label("Less").is_none());
}

#[test]
fn search_expands_focuses_and_closes_without_losing_query() {
    let mut harness = Harness::builder()
        .with_size(egui::vec2(240.0, 160.0))
        .build_ui_state(
            |ui, state: &mut (bool, String)| {
                search_box(
                    ui,
                    &Theme::light(),
                    &mut state.0,
                    &mut state.1,
                    "Find",
                    "Close",
                );
            },
            (false, String::new()),
        );
    harness.run_steps(3);
    let trigger = harness.get_by_role_and_label(egui::accesskit::Role::Button, "Find");
    trigger.focus();
    harness.run();
    harness.key_press(egui::Key::Enter);
    harness.run_steps(5);
    let field = harness.get_by_role_and_label(egui::accesskit::Role::TextInput, "Find");
    assert!(field.accesskit_node().is_focused());
    assert!(field.rect().right() < 240.0);
    harness.event(egui::Event::Text("calendar".to_owned()));
    harness.run();
    harness.key_press(egui::Key::Escape);
    harness.run_steps(5);
    assert_eq!(harness.state(), &(false, "calendar".to_owned()));
    assert!(
        harness
            .get_by_role_and_label(egui::accesskit::Role::Button, "Find")
            .accesskit_node()
            .is_focused()
    );
}

#[test]
fn state_controls_report_changes_and_direction_pad_accepts_arrow_keys() {
    let mut harness = Harness::builder().build_ui_state(
        |ui, state: &mut (ViewMode, bool, Option<Direction>)| {
            let theme = Theme::light();
            layout_toggle(ui, &theme, &mut state.0, ["Columns", "Grid", "List"]);
            toggle_button(ui, &theme, &mut state.1, ["Follow", "Following"]);
            if let Some(direction) = direction_pad(ui, &theme, ["Up", "Left", "Right", "Down"]) {
                state.2 = Some(direction);
            }
        },
        (ViewMode::Column, false, None),
    );
    harness.run();
    harness.get_by_label("Columns").click();
    harness.run();
    harness.get_by_label("Follow").click();
    harness.run();
    assert_eq!(
        harness.get_by_label("Following").accesskit_node().toggled(),
        Some(egui::accesskit::Toggled::True)
    );
    harness.get_by_label("Up").focus();
    harness.run();
    harness.key_press(egui::Key::ArrowLeft);
    harness.run();
    assert_eq!(
        harness.state(),
        &(ViewMode::Grid, true, Some(Direction::Left))
    );
    assert!(harness.get_by_label("Left").accesskit_node().is_focused());
}
