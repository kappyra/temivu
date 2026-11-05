use egui_kittest::{
    Harness,
    kittest::{NodeT, Queryable},
};
use temivu::{Tabs, Theme};

#[test]
fn rejects_empty_and_out_of_bounds() {
    let theme = Theme::light();
    let mut index = 0;
    assert!(Tabs::new(&mut index, &[], &theme).is_err());
    index = 1;
    assert!(Tabs::new(&mut index, &["Only"], &theme).is_err());
}

#[test]
fn click_selects_content_and_reports_tab_role() {
    let mut harness = Harness::builder().build_ui_state(
        |ui, selected: &mut usize| {
            Tabs::new(selected, &["Account", "Settings"], &Theme::light())
                .unwrap()
                .show(ui, |ui, index| {
                    ui.label(if index == 0 {
                        "Account content"
                    } else {
                        "Settings content"
                    });
                });
        },
        0,
    );
    assert!(harness.get_by_label("Account content").rect().is_positive());
    let node = harness.get_by_label("Account");
    assert_eq!(node.accesskit_node().role(), egui::accesskit::Role::Tab);
    assert_eq!(node.accesskit_node().is_selected(), Some(true));
    harness.get_by_label("Settings").click();
    harness.run();
    assert_eq!(*harness.state(), 1);
    assert!(
        harness
            .get_by_label("Settings content")
            .rect()
            .is_positive()
    );
    assert!(harness.query_by_label("Account content").is_none());
    assert_eq!(
        harness
            .get_by_label("Settings")
            .accesskit_node()
            .is_selected(),
        Some(true)
    );
}

#[test]
fn arrow_keys_select_next_tab() {
    let mut harness = Harness::builder().build_ui_state(
        |ui, selected: &mut usize| {
            Tabs::new(selected, &["First", "Second"], &Theme::light())
                .unwrap()
                .show(ui, |_, _| ());
        },
        0,
    );
    harness.get_by_label("First").focus();
    harness.run();
    harness.key_press(egui::Key::ArrowRight);
    harness.run();
    assert_eq!(*harness.state(), 1);
}

#[test]
fn arrow_navigation_wraps_without_focusing_adjacent_controls() {
    let mut harness = Harness::builder().build_ui_state(
        |ui, selected| {
            let _ = ui.button("Before");
            Tabs::new(selected, &["First", "Second", "Third"], &Theme::light())
                .unwrap()
                .show(ui, |_, _| ());
            let _ = ui.button("After");
        },
        0usize,
    );
    harness.get_by_label("First").focus();
    harness.run();
    harness.key_press(egui::Key::ArrowLeft);
    harness.run();
    assert_eq!(*harness.state(), 2);
    assert!(harness.get_by_label("Third").accesskit_node().is_focused());
    harness.key_press(egui::Key::ArrowRight);
    harness.run();
    assert_eq!(*harness.state(), 0);
    assert!(harness.get_by_label("First").accesskit_node().is_focused());
}

#[test]
fn tabs_container_uses_forui_muted_surface_border_and_radius() {
    let theme = Theme::light();
    let fill = theme.muted;
    let border = theme.muted;
    let mut selected = 0;
    let mut harness = Harness::builder().build_ui(move |ui| {
        Tabs::new(&mut selected, &["One", "Two"], &theme)
            .unwrap()
            .show(ui, |ui, _| {
                ui.label("content");
            });
    });
    harness.run();
    let container = harness
        .output()
        .shapes
        .iter()
        .find_map(|shape| match &shape.shape {
            egui::Shape::Rect(rect)
                if rect.fill == fill
                    && rect.stroke.color == border
                    && rect.stroke.width == 1.0
                    && rect.corner_radius.nw == 10 =>
            {
                Some(rect.rect)
            }
            _ => None,
        })
        .expect("tabs container");
    // 36 point tab minimum height plus 4 points of padding on each side.
    assert!(
        container.height() >= 44.0,
        "tabs height: {}",
        container.height()
    );
}

#[test]
fn dragging_reorders_tabs_without_losing_content_identity() {
    let mut harness = Harness::builder()
        .with_size(egui::vec2(600.0, 200.0))
        .build_ui_state(
            |ui, state: &mut (usize, [usize; 3])| {
                Tabs::new(
                    &mut state.0,
                    &["General", "Privacy", "About"],
                    &Theme::light(),
                )
                .unwrap()
                .reorderable(&mut state.1)
                .unwrap()
                .show(ui, |_, _| {});
            },
            (0, [0, 1, 2]),
        );
    harness.run();
    let from = harness.get_by_label("General").rect().center();
    let to = harness.get_by_label("About").rect().center();
    harness.hover_at(from);
    harness.step();
    harness.drag_at(from);
    harness.step();
    harness.hover_at(to);
    harness.step();
    harness.drop_at(to);
    harness.run();
    assert_eq!(harness.state().1, [1, 2, 0]);
    assert_eq!(harness.state().0, 0);
    assert!(
        harness.get_by_label("General").rect().center().x
            > harness.get_by_label("About").rect().center().x
    );
}

#[test]
fn invalid_tab_orders_are_rejected() {
    let theme = Theme::light();
    let mut selected = 0;
    assert!(
        Tabs::new(&mut selected, &["A", "B"], &theme)
            .unwrap()
            .reorderable(&mut [0, 0])
            .is_err()
    );
    assert!(
        Tabs::new(&mut selected, &["A", "B"], &theme)
            .unwrap()
            .reorderable(&mut [0, 2])
            .is_err()
    );
}

#[test]
fn reordered_tab_label_moves_with_the_selection_indicator() {
    let mut harness = Harness::builder()
        .with_size(egui::vec2(600.0, 200.0))
        .with_step_dt(0.02)
        .build_ui_state(
            |ui, state: &mut (usize, [usize; 3])| {
                Tabs::new(
                    &mut state.0,
                    &["General", "Privacy", "About"],
                    &Theme::light(),
                )
                .unwrap()
                .reorderable(&mut state.1)
                .unwrap()
                .show(ui, |_, _| {});
            },
            (0, [0, 1, 2]),
        );
    harness
        .ctx
        .all_styles_mut(|style| style.animation_time = 0.2);
    let from = harness.get_by_label("General").rect().center();
    let to = harness.get_by_label("About").rect().center();
    harness.hover_at(from);
    harness.step();
    harness.drag_at(from);
    harness.step();
    harness.hover_at(to);
    harness.step();
    harness.drop_at(to);
    harness.run_steps(2);

    let card = Theme::light().card;
    let indicator = harness
        .output()
        .shapes
        .iter()
        .find_map(|entry| match &entry.shape {
            egui::Shape::Rect(rect) if rect.fill == card && rect.corner_radius.nw == 8 => {
                Some(rect.rect.center().x)
            }
            _ => None,
        })
        .expect("selection indicator");
    let text = harness
        .output()
        .shapes
        .iter()
        .find_map(|entry| match &entry.shape {
            egui::Shape::Text(text) if text.galley.job.text == "General" => {
                Some(text.pos.x + text.galley.size().x / 2.0)
            }
            _ => None,
        })
        .expect("moving label");
    assert_eq!(harness.state().1, [1, 2, 0]);
    assert!(from.x < indicator && indicator < to.x);
    assert!(
        (text - indicator).abs() < 2.0,
        "label={text}, indicator={indicator}"
    );
}
