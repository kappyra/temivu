use egui_kittest::{
    Harness,
    kittest::{NodeT, Queryable},
};
use temivu::Dialog;

#[test]
fn escape_closes_open_dialog_and_closed_dialog_is_absent() {
    let mut harness = Harness::builder().build_ui_state(
        |ui, open: &mut bool| {
            Dialog::new(egui::Id::new("example-dialog"), "Example").show(ui.ctx(), open, |ui| {
                ui.label("Dialog content");
            });
        },
        true,
    );
    harness.run();
    harness.get_by_label("Dialog content");
    assert!(
        harness
            .get_by_role_and_label(egui::accesskit::Role::Dialog, "Example")
            .accesskit_node()
            .is_modal()
    );
    harness.key_press(egui::Key::Escape);
    harness.run();
    assert!(!*harness.state());
    harness.run();
    assert!(harness.query_by_label("Dialog content").is_none());
}

#[test]
fn backdrop_blocks_background_and_respects_dismiss_policy() {
    for dismissible in [true, false] {
        let mut harness = Harness::builder()
            .with_size(egui::vec2(500.0, 300.0))
            .build_ui_state(
                move |ui, state: &mut (bool, bool)| {
                    if ui.button("Background action").clicked() {
                        state.1 = true;
                    }
                    Dialog::new(egui::Id::new("backdrop-dialog"), "Example")
                        .barrier_dismissible(dismissible)
                        .show(ui.ctx(), &mut state.0, |ui| {
                            ui.label("Modal content");
                        });
                },
                (true, false),
            );
        harness.run();
        harness.get_by_label("Background action").click();
        harness.run();
        assert!(!harness.state().1, "background received a modal click");
        assert_eq!(harness.state().0, !dismissible);
    }
}

#[test]
fn keyboard_navigation_stays_in_modal_while_open() {
    let mut harness = Harness::builder().build_ui(|ui| {
        let _ = ui.button("Background");
        let mut open = true;
        Dialog::new(egui::Id::new("focus-dialog"), "Example").show(ui.ctx(), &mut open, |ui| {
            let _ = ui.button("Dialog action");
        });
    });
    harness.run();
    let mut reached_action = false;
    for _ in 0..3 {
        harness.key_press(egui::Key::Tab);
        harness.run();
        assert!(
            !harness
                .get_by_label("Background")
                .accesskit_node()
                .is_focused()
        );
        reached_action |= harness
            .get_by_label("Dialog action")
            .accesskit_node()
            .is_focused();
    }
    assert!(reached_action);
}

#[test]
fn dismissal_restores_focus_to_the_trigger() {
    let mut harness = Harness::builder().build_ui_state(
        |ui, open: &mut bool| {
            if ui.button("Open dialog").clicked() {
                *open = true;
            }
            Dialog::new(egui::Id::new("restore-focus"), "Example").show(ui.ctx(), open, |ui| {
                let _ = ui.button("Dialog action");
            });
        },
        false,
    );
    harness.get_by_label("Open dialog").focus();
    harness.key_press(egui::Key::Enter);
    harness.run();
    assert!(*harness.state());
    assert!(
        harness
            .get_by_role_and_label(egui::accesskit::Role::Dialog, "Example")
            .accesskit_node()
            .is_focused()
    );
    harness.key_press(egui::Key::Tab);
    harness.run();
    harness.key_press(egui::Key::Escape);
    harness.run();
    harness.run();
    assert!(!*harness.state());
    assert!(
        harness
            .get_by_label("Open dialog")
            .accesskit_node()
            .is_focused()
    );
}

#[test]
fn themed_dialog_draws_forui_card_border_radius_and_barrier() {
    use temivu::Theme;
    let theme = Theme::light();
    let barrier = theme.barrier;
    let card = theme.card;
    let border = theme.border;
    let mut harness = Harness::builder().build_ui_state(
        |ui, open: &mut bool| {
            Dialog::new(egui::Id::new("themed-dialog"), "Example")
                .theme(&theme)
                .show(ui.ctx(), open, |ui| {
                    ui.label("Themed content");
                });
        },
        true,
    );
    harness.run();
    harness.get_by_label("Themed content");
    let shapes = &harness.output().shapes;
    assert!(
        shapes.iter().any(|shape| matches!(
            &shape.shape,
            egui::Shape::Rect(rect) if rect.fill == barrier
        )),
        "the themed dialog uses the theme barrier color"
    );
    assert!(
        shapes.iter().any(|shape| matches!(
            &shape.shape,
            egui::Shape::Rect(rect)
                if rect.fill == card
                    && rect.stroke.color == border
                    && rect.stroke.width == 1.0
                    && rect.corner_radius.nw == 10
        )),
        "the themed dialog uses the card, border and md radius"
    );
}
