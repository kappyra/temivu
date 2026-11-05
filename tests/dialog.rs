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
fn themed_dialog_draws_card_border_rounded_corners_and_barrier() {
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
                    && rect.corner_radius.nw == 32
        )),
        "the themed dialog uses the card, border and rounded corners"
    );
}

#[test]
fn anchored_droplet_dialog_morphs_and_its_exit_does_not_run_content() {
    fn has_mesh(shape: &egui::Shape) -> bool {
        match shape {
            egui::Shape::Mesh(_) => true,
            egui::Shape::Vec(shapes) => shapes.iter().any(has_mesh),
            _ => false,
        }
    }
    let ctx = egui::Context::default();
    let theme = temivu::Theme::light();
    theme.apply(&ctx);
    let mut calls = 0;
    for (time, state) in [(0.0, false), (0.04, true), (0.12, true), (0.2, false)] {
        let previous_calls = calls;
        let mut open = state;
        let mut output = ctx.run_ui(
            egui::RawInput {
                time: Some(time),
                screen_rect: Some(egui::Rect::from_min_size(
                    egui::Pos2::ZERO,
                    egui::vec2(600.0, 600.0),
                )),
                ..Default::default()
            },
            |ui| {
                Dialog::new(egui::Id::new("droplet-modal"), "Example")
                    .theme(&theme)
                    .liquid(temivu::liquid::LiquidStyle::droplet())
                    .source(egui::Rect::from_min_size(
                        egui::pos2(278.0, 210.0),
                        egui::vec2(44.0, 44.0),
                    ))
                    .show(ui.ctx(), &mut open, |ui| {
                        calls += 1;
                        ui.label("Content");
                    });
            },
        );
        if time == 0.12 {
            assert!(
                output.shapes.iter().any(|entry| has_mesh(&entry.shape)),
                "the modal frame should use Droplet geometry"
            );
        }
        if !state {
            assert_eq!(calls, previous_calls);
        }
        output.textures_delta.clear();
    }
}

#[test]
fn modal_frame_and_input_stay_in_sync_with_custom_area() {
    let theme = temivu::Theme::light();
    let card = theme.card;
    let mut harness = egui_kittest::Harness::builder()
        .with_size(egui::vec2(390.0, 640.0))
        .build_ui_state(
            move |ui, open| {
                let ctx = ui.ctx();
                let area = egui::Modal::default_area(egui::Id::new("physical-modal-area"));
                Dialog::new(egui::Id::new("logical-modal"), "Example")
                    .theme(&theme)
                    .area(area)
                    .source(egui::Rect::from_min_size(
                        egui::pos2(20.0, 52.0),
                        egui::vec2(110.0, 44.0),
                    ))
                    .show(ctx, open, |ui| {
                        ui.set_width(250.0);
                        let _ = ui.button("Apply changes");
                    });
            },
            false,
        );
    harness
        .ctx
        .all_styles_mut(|style| style.animation_time = 0.2);
    *harness.state_mut() = true;
    for time in [1.0, 1.08] {
        harness.input_mut().time = Some(time);
        harness.step();
    }
    let moving_frame = harness
        .output()
        .shapes
        .iter()
        .find_map(|shape| match &shape.shape {
            egui::Shape::Rect(rect) if rect.fill == card && rect.corner_radius.nw >= 20 => {
                Some(rect.rect)
            }
            _ => None,
        })
        .expect("visible moving card");
    assert!(moving_frame.width() < 250.0);
    assert!(
        harness
            .get_by_label("Apply changes")
            .accesskit_node()
            .is_disabled()
    );

    harness.input_mut().time = Some(1.6);
    harness.step();
    assert!(
        !harness
            .get_by_label("Apply changes")
            .accesskit_node()
            .is_disabled()
    );
    let settled = harness
        .get_by_role_and_label(egui::accesskit::Role::Dialog, "Example")
        .rect();
    assert!(settled.width() > 250.0 && settled.right() < 390.0);

    *harness.state_mut() = false;
    harness.input_mut().time = Some(1.62);
    harness.step();
    assert!(harness.query_by_label("Apply changes").is_none());
    assert!(
        harness
            .output()
            .shapes
            .iter()
            .any(|shape| { matches!(&shape.shape, egui::Shape::Rect(rect) if rect.fill == card) }),
        "the outgoing card should still be visible"
    );
}

#[test]
fn centered_droplet_dialog_starts_as_a_small_surface() {
    let ctx = egui::Context::default();
    let theme = temivu::Theme::light();
    theme.apply(&ctx);
    ctx.all_styles_mut(|style| style.animation_time = 0.2);
    let fill = egui::Color32::from_rgb(212, 227, 241);
    let screen = egui::Rect::from_min_size(egui::Pos2::ZERO, egui::vec2(720.0, 500.0));
    let render_width = |time, mut open| {
        let mut output = ctx.run_ui(
            egui::RawInput {
                time: Some(time),
                screen_rect: Some(screen),
                ..Default::default()
            },
            |ui| {
                Dialog::new(egui::Id::new("center-expansion"), "Example")
                    .theme(&theme)
                    .liquid(temivu::liquid::LiquidStyle::droplet())
                    .frame(egui::Frame::new().fill(fill).corner_radius(24))
                    .show(ui.ctx(), &mut open, |ui| {
                        ui.set_width(360.0);
                        ui.label("Confirmation");
                    });
            },
        );
        let width = output.shapes.iter().find_map(|entry| match &entry.shape {
            egui::Shape::Rect(rect) if rect.fill == fill => Some(rect.rect.width()),
            _ => None,
        });
        output.textures_delta.clear();
        width
    };
    render_width(0.0, false);
    render_width(0.05, true);
    let early = render_width(0.14, true).expect("opening surface");
    let settled = render_width(0.8, true).expect("settled surface");
    assert!(early < settled * 0.7, "early={early}, settled={settled}");
}
