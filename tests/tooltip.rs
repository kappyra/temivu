use egui_kittest::{
    Harness,
    kittest::{NodeT, Queryable},
};
use temivu::{Dialog, tooltip};

#[test]
fn description_is_attached_without_waiting_for_hover() {
    let mut harness = Harness::builder().build_ui(|ui| {
        let response = ui.button("Help");
        ui.ctx()
            .accesskit_node_builder(response.id, |node| node.set_description("Existing detail"));
        tooltip::tooltip(response, "Extra detail");
    });
    harness.run();
    let node = harness.get_by_label("Help");
    assert_eq!(
        node.accesskit_node().description().as_deref(),
        Some("Existing detail\nExtra detail")
    );
    assert!(harness.query_by_label("Extra detail").is_none());
}

#[test]
fn keyboard_focus_shows_tooltip_and_escape_hides_it() {
    let mut harness = Harness::builder().build_ui(|ui| {
        tooltip::tooltip(ui.button("Focused help"), "Tooltip contents");
    });
    harness.run();
    harness.get_by_label("Focused help").focus();
    harness.run();
    assert!(harness.query_by_label("Tooltip contents").is_some());
    harness.key_press(egui::Key::Escape);
    harness.run();
    assert!(harness.query_by_label("Tooltip contents").is_none());
}

#[test]
fn hover_shows_tooltip_after_delay() {
    let mut harness = Harness::builder().build_ui(|ui| {
        tooltip::tooltip(ui.button("Hover help"), "Hover contents");
    });
    harness.run();
    harness.get_by_label("Hover help").hover();
    harness.run_steps(60);
    assert!(harness.query_by_label("Hover contents").is_some());
}

#[test]
fn hidden_tooltip_does_not_consume_escape() {
    let mut harness = Harness::builder().build_ui_state(
        |ui, seen: &mut bool| {
            tooltip::tooltip(ui.button("Unfocused help"), "Hidden");
            if ui.input(|input| input.key_pressed(egui::Key::Escape)) {
                *seen = true;
            }
        },
        false,
    );
    harness.run();
    harness.key_press(egui::Key::Escape);
    harness.run();
    assert!(*harness.state());
}

#[test]
fn escape_inside_a_dialog_closes_it_instead_of_hiding_the_tooltip() {
    let mut harness = Harness::builder().build_ui_state(
        |ui, open: &mut bool| {
            Dialog::new(egui::Id::new("tooltip-dialog"), "Confirm").show(ui.ctx(), open, |ui| {
                let response = ui.button("Inside dialog");
                tooltip::tooltip(response, "Dialog tooltip");
            });
        },
        true,
    );
    harness.run();
    harness.get_by_label("Inside dialog").focus();
    harness.run();
    assert!(harness.query_by_label("Dialog tooltip").is_some());
    harness.key_press(egui::Key::Escape);
    harness.run();
    assert!(
        !*harness.state(),
        "Escape should close the dialog that contains the focused tooltip"
    );
}

#[test]
fn tooltip_uses_forui_inverted_surface() {
    let theme = temivu::Theme::light();
    let fill = theme.foreground;
    let text_color = theme.background;
    let mut harness = Harness::builder().build_ui(move |ui| {
        theme.apply(ui.ctx());
        tooltip::tooltip(ui.button("Help"), "Tooltip contents");
    });
    harness.run();
    harness.get_by_label("Help").focus();
    harness.run();
    harness.get_by_label("Tooltip contents");
    let shapes = &harness.output().shapes;
    assert!(
        shapes.iter().any(|shape| matches!(
            &shape.shape,
            egui::Shape::Rect(rect) if rect.fill == fill && rect.corner_radius.nw == 8
        )),
        "tooltip uses the inverted foreground fill and sm radius"
    );
    let text = shapes
        .iter()
        .find_map(|shape| match &shape.shape {
            egui::Shape::Text(text) if text.galley.job.text == "Tooltip contents" => {
                Some(text.galley.job.sections[0].format.color)
            }
            _ => None,
        })
        .expect("tooltip text");
    assert_eq!(text, text_color);
}
