use egui_kittest::{
    Harness,
    kittest::{NodeT, Queryable},
};
use temivu::{Sheet, Theme};

#[derive(Default)]
struct State {
    open: bool,
    applied: usize,
    background: usize,
}

fn view(ui: &mut egui::Ui, state: &mut State) {
    let theme = Theme::light();
    let State {
        open,
        applied,
        background,
    } = state;
    if ui.button("Background action").clicked() {
        *background += 1;
    }
    if ui.button("Open sheet").clicked() {
        *open = true;
    }
    Sheet::new(egui::Id::new("filters-sheet"), "Filters", &theme).show(ui.ctx(), open, |ui| {
        if ui.button("Apply filters").clicked() {
            *applied += 1;
        }
    });
}

#[test]
fn escape_closes_the_sheet_and_restores_trigger_focus() {
    let mut harness = Harness::builder()
        .with_size(egui::vec2(640.0, 480.0))
        .build_ui_state(view, State::default());
    harness.run();
    harness.get_by_label("Open sheet").focus();
    harness.run();
    harness.key_press(egui::Key::Enter);
    harness.run();
    assert!(harness.state().open);

    let dialog = harness.get_by_role_and_label(egui::accesskit::Role::Dialog, "Filters");
    assert!(dialog.accesskit_node().is_modal());
    assert!(
        dialog.is_focused(),
        "the sheet should receive focus when it opens"
    );
    harness.get_by_label("Close");

    let mut reached_content = false;
    for _ in 0..4 {
        harness.key_press(egui::Key::Tab);
        harness.run();
        assert!(
            !harness
                .get_by_label("Background action")
                .accesskit_node()
                .is_focused(),
            "focus must stay inside the modal sheet"
        );
        reached_content |= harness.get_by_label("Apply filters").is_focused();
    }
    assert!(reached_content, "Tab should reach the sheet's content");

    harness.key_press(egui::Key::Escape);
    harness.run();
    harness.run();
    assert!(!harness.state().open);
    assert!(harness.query_by_label("Apply filters").is_none());
    assert!(harness.get_by_label("Open sheet").is_focused());
}

#[test]
fn backdrop_blocks_background_input_and_respects_dismiss_policy() {
    for dismissible in [true, false] {
        let mut harness = Harness::builder()
            .with_size(egui::vec2(500.0, 300.0))
            .build_ui_state(
                move |ui, state: &mut State| {
                    let theme = Theme::light();
                    let State {
                        open,
                        applied,
                        background,
                    } = state;
                    if ui.button("Background action").clicked() {
                        *background += 1;
                    }
                    if ui.button("Open sheet").clicked() {
                        *open = true;
                    }
                    Sheet::new(egui::Id::new("backdrop-sheet"), "Filters", &theme)
                        .barrier_dismissible(dismissible)
                        .show(ui.ctx(), open, |ui| {
                            if ui.button("Apply filters").clicked() {
                                *applied += 1;
                            }
                        });
                },
                State::default(),
            );
        harness.run();
        harness.get_by_label("Open sheet").click();
        harness.run();
        assert!(harness.state().open);

        harness.get_by_label("Background action").click();
        harness.run();
        assert_eq!(
            harness.state().background,
            0,
            "the backdrop must block clicks on the background"
        );
        assert_eq!(harness.state().open, !dismissible);
    }
}

#[test]
fn dismiss_button_closes_the_sheet() {
    let mut harness = Harness::builder()
        .with_size(egui::vec2(640.0, 480.0))
        .build_ui_state(view, State::default());
    harness.run();
    harness.get_by_label("Open sheet").click();
    harness.run();
    assert!(harness.state().open);

    harness.get_by_label("Close").click();
    harness.run();
    assert!(!harness.state().open);
    assert!(harness.query_by_label("Apply filters").is_none());
}

#[test]
fn ui_close_inside_the_content_dismisses_the_sheet() {
    let mut harness = Harness::builder()
        .with_size(egui::vec2(640.0, 480.0))
        .build_ui_state(
            |ui, state: &mut State| {
                let theme = Theme::light();
                let State { open, applied, .. } = state;
                if ui.button("Open sheet").clicked() {
                    *open = true;
                }
                Sheet::new(egui::Id::new("apply-sheet"), "Filters", &theme).show(
                    ui.ctx(),
                    open,
                    |ui| {
                        if ui.button("Apply filters").clicked() {
                            *applied += 1;
                            ui.close();
                        }
                    },
                );
            },
            State::default(),
        );
    harness.run();
    harness.get_by_label("Open sheet").click();
    harness.run();

    harness.get_by_label("Apply filters").click();
    harness.run();
    assert_eq!(harness.state().applied, 1);
    assert!(!harness.state().open);
}

#[test]
fn width_is_clamped_on_a_narrow_window() {
    let mut harness = Harness::builder()
        .with_size(egui::vec2(200.0, 300.0))
        .build_ui_state(view, State::default());
    harness.run();
    harness.get_by_label("Open sheet").click();
    harness.run();

    let dialog = harness.get_by_role_and_label(egui::accesskit::Role::Dialog, "Filters");
    let width = dialog.rect().width();
    assert!(
        width <= 200.0 - 2.0 * 8.0 + 0.5,
        "sheet width {width} should be clamped to the window"
    );
    assert!(width > 100.0, "the sheet should keep a usable width");
    harness.get_by_label("Apply filters");
}

#[test]
fn compact_sheet_keeps_its_header_visible_and_dismisses_by_grip() {
    let mut harness = Harness::builder()
        .with_size(egui::vec2(340.0, 300.0))
        .build_ui_state(
            |ui, open| {
                Sheet::new(egui::Id::new("compact-sheet"), "Filters", &Theme::light())
                    .description("Choose which items are shown.")
                    .show(ui.ctx(), open, |ui| {
                        for index in 0..30 {
                            ui.label(format!("Filter {index}"));
                        }
                    });
            },
            true,
        );
    harness.run();
    harness.run_steps(3);
    let sheet = harness
        .get_by_role_and_label(egui::accesskit::Role::Dialog, "Filters")
        .rect();
    let close = harness.get_by_label("Close").rect();
    assert!(
        sheet.top() >= 0.0 && sheet.bottom() <= 300.5,
        "sheet={sheet:?}, close={close:?}"
    );
    assert!(sheet.height() < 300.0 && close.height() >= 44.0);
    let grip = egui::pos2(sheet.center().x, sheet.top() + 23.0);
    let destination = grip + egui::vec2(0.0, 85.0);
    harness.event(egui::Event::PointerMoved(grip));
    harness.drag_at(grip);
    harness.event(egui::Event::PointerMoved(destination));
    harness.run_steps(3);
    assert!(*harness.state());
    harness.drop_at(destination);
    harness.run();
    assert!(!harness.state());
}
