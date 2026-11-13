use egui_kittest::{
    Harness,
    kittest::{NodeT, Queryable},
};
use temivu::{Slider, SliderMark, Theme};

#[test]
fn invalid_values_and_steps_are_rejected() {
    let theme = Theme::light();
    for mut value in [f64::NAN, -0.1, 1.1] {
        assert!(Slider::new(&mut value, &theme).is_err());
    }
    for step in [0.0, -0.1, 1.1, f64::INFINITY] {
        let mut value = 0.0;
        assert!(Slider::new(&mut value, &theme).unwrap().step(step).is_err());
    }
}

#[test]
fn pointer_keyboard_and_semantics_update_value() {
    let mut harness = Harness::builder().build_ui_state(
        |ui, value| {
            Slider::new(value, &Theme::light())
                .unwrap()
                .label("Volume")
                .step(0.1)
                .unwrap()
                .show(ui);
        },
        0.2_f64,
    );
    harness.run();
    let node = harness.get_by_label("Volume");
    assert_eq!(node.accesskit_node().role(), egui::accesskit::Role::Slider);
    assert_eq!(node.accesskit_node().numeric_value(), Some(0.2));
    node.click();
    harness.run();
    assert!(*harness.state() > 0.2);
    let after_click = *harness.state();
    harness.get_by_label("Volume").focus();
    harness.run();
    harness.key_press(egui::Key::ArrowRight);
    harness.run();
    assert!(*harness.state() > after_click);
}

#[test]
fn disabled_slider_cannot_be_changed_and_has_no_actions() {
    let mut harness = Harness::builder().build_ui_state(
        |ui, value| {
            Slider::new(value, &Theme::dark())
                .unwrap()
                .label("Disabled slider")
                .vertical(true)
                .enabled(false)
                .show(ui);
        },
        0.5_f64,
    );
    harness.run();
    let node = harness.get_by_label("Disabled slider");
    assert!(node.accesskit_node().is_disabled());
    assert!(
        !node
            .accesskit_node()
            .data()
            .supports_action(egui::accesskit::Action::Increment)
    );
    node.click();
    harness.run();
    assert_eq!(*harness.state(), 0.5);
}

#[test]
fn stale_accessibility_actions_do_not_change_disabled_values() {
    for inherited in [false, true] {
        let mut harness = Harness::builder().build_ui_state(
            move |ui, state: &mut (f64, bool)| {
                if inherited {
                    ui.disable();
                }
                state.1 |= Slider::new(&mut state.0, &Theme::light())
                    .unwrap()
                    .label("Volume")
                    .enabled(inherited)
                    .show(ui)
                    .changed();
            },
            (0.5_f64, false),
        );
        harness.run();
        let node = harness.get_by_label("Volume");
        assert!(
            !node
                .accesskit_node()
                .data()
                .supports_action(egui::accesskit::Action::Increment)
        );
        let (target_node, target_tree) = node.accesskit_node().locate();
        for (action, data) in [
            (egui::accesskit::Action::Increment, None),
            (
                egui::accesskit::Action::SetValue,
                Some(egui::accesskit::ActionData::NumericValue(0.9)),
            ),
        ] {
            harness.event(egui::Event::AccessKitActionRequest(
                egui::accesskit::ActionRequest {
                    action,
                    target_node,
                    target_tree,
                    data,
                },
            ));
            harness.run();
            assert_eq!(*harness.state(), (0.5, false));
        }
    }
}

#[test]
fn marks_reject_invalid_positions() {
    for value in [f64::NAN, -0.1, 1.1, f64::INFINITY] {
        assert!(SliderMark::new(value).is_err());
    }
}

#[test]
fn horizontal_marks_paint_ticks_and_labels() {
    let marks = [
        SliderMark::new(0.0).unwrap().label("Min"),
        SliderMark::new(0.5).unwrap().label("Mid"),
        SliderMark::new(1.0).unwrap().label("Max"),
    ];
    let mut harness = Harness::builder().build_ui_state(
        move |ui, value: &mut f64| {
            Slider::new(value, &Theme::light())
                .unwrap()
                .label("Level")
                .marks(&marks)
                .show(ui);
        },
        0.4_f64,
    );
    harness.run();
    let texts: Vec<String> = harness
        .output()
        .shapes
        .iter()
        .filter_map(|shape| match &shape.shape {
            egui::Shape::Text(text) => Some(text.galley.text().to_owned()),
            _ => None,
        })
        .collect();
    for label in ["Min", "Mid", "Max"] {
        assert!(
            texts.iter().any(|text| text == label),
            "mark label {label} should be painted, got {texts:?}"
        );
    }
    let tick_count = harness
        .output()
        .shapes
        .iter()
        .filter(|shape| matches!(shape.shape, egui::Shape::Circle(circle) if (circle.radius - 1.5).abs() < f32::EPSILON))
        .count();
    assert!(
        tick_count >= 3,
        "expected three tick dots, got {tick_count}"
    );
}

#[test]
fn vertical_marks_paint_ticks_without_labels() {
    let marks = [
        SliderMark::new(0.25).unwrap().label("Low"),
        SliderMark::new(0.75).unwrap().label("High"),
    ];
    let mut harness = Harness::builder().build_ui_state(
        move |ui, value: &mut f64| {
            Slider::new(value, &Theme::light())
                .unwrap()
                .label("Level")
                .vertical(true)
                .marks(&marks)
                .show(ui);
        },
        0.5_f64,
    );
    harness.run();
    let texts: Vec<String> = harness
        .output()
        .shapes
        .iter()
        .filter_map(|shape| match &shape.shape {
            egui::Shape::Text(text) => Some(text.galley.text().to_owned()),
            _ => None,
        })
        .collect();
    assert!(!texts.iter().any(|text| text == "Low" || text == "High"));
    let tick_count = harness
        .output()
        .shapes
        .iter()
        .filter(|shape| matches!(shape.shape, egui::Shape::Circle(circle) if (circle.radius - 1.5).abs() < f32::EPSILON))
        .count();
    assert!(tick_count >= 2, "expected two tick dots, got {tick_count}");
}

fn tick_centers(harness: &Harness<'_, impl std::any::Any>) -> Vec<egui::Pos2> {
    harness
        .output()
        .shapes
        .iter()
        .filter_map(|shape| match &shape.shape {
            egui::Shape::Circle(circle) if (circle.radius - 1.5).abs() < f32::EPSILON => {
                Some(circle.center)
            }
            _ => None,
        })
        .collect()
}

#[test]
fn horizontal_marks_align_with_the_rail_and_labels_stay_centred() {
    let marks = [
        SliderMark::new(0.0).unwrap().label("Min"),
        SliderMark::new(0.5).unwrap().label("Mid"),
        SliderMark::new(1.0).unwrap().label("Max"),
    ];
    let mut harness = Harness::builder().build_ui_state(
        move |ui, state: &mut (f64, Option<egui::Rect>)| {
            let response = Slider::new(&mut state.0, &Theme::light())
                .unwrap()
                .label("Level")
                .marks(&marks)
                .show(ui);
            state.1 = Some(response.rect);
        },
        (0.4_f64, None),
    );
    harness.run();
    let rect = harness.state().1.expect("response rect");
    // The default egui style uses the rectangular handle with aspect ratio 0.75,
    // so the rail is inset by (height / 2.5) * 0.75 = 0.3 * height on each side.
    assert!(
        (rect.height() - 20.0).abs() < 0.5,
        "expected the default 20-point slider thickness, got {}",
        rect.height()
    );
    // Independent absolute expectation: egui's default inset is 6 pt for the
    // 20 pt desktop thickness (8 * 0.75).
    let expected = [rect.left() + 6.0, rect.center().x, rect.right() - 6.0];
    let mut centers = tick_centers(&harness);
    centers.sort_by(|a, b| a.x.partial_cmp(&b.x).unwrap());
    assert_eq!(centers.len(), 3, "three ticks expected, got {centers:?}");
    for (center, expected_x) in centers.iter().zip(expected) {
        assert!(
            (center.x - expected_x).abs() <= 0.5,
            "tick at {} should be at {expected_x}",
            center.x
        );
    }
    // Painted labels are centred under their ticks (TextShape::pos is the
    // top-left corner after alignment, so add half the galley width).
    let labels: Vec<f32> = harness
        .output()
        .shapes
        .iter()
        .filter_map(|shape| match &shape.shape {
            egui::Shape::Text(text) if matches!(text.galley.text(), "Min" | "Mid" | "Max") => {
                Some(text.pos.x + text.galley.size().x / 2.0)
            }
            _ => None,
        })
        .collect();
    assert_eq!(labels.len(), 3);
    for x in labels {
        assert!(
            centers.iter().any(|center| (center.x - x).abs() <= 1.5),
            "label at {x} should be centred under a tick"
        );
    }
}

#[test]
fn vertical_marks_follow_the_flipped_rail() {
    let marks = [
        SliderMark::new(0.25).unwrap(),
        SliderMark::new(0.75).unwrap(),
    ];
    let mut harness = Harness::builder().build_ui_state(
        move |ui, state: &mut (f64, Option<egui::Rect>)| {
            let response = Slider::new(&mut state.0, &Theme::light())
                .unwrap()
                .label("Level")
                .vertical(true)
                .marks(&marks)
                .show(ui);
            state.1 = Some(response.rect);
        },
        (0.5_f64, None),
    );
    harness.run();
    let rect = harness.state().1.expect("response rect");
    // Same default rectangular handle: the vertical rail is inset by 0.3 * width.
    let inset = rect.width() * 0.3;
    let top = rect.top() + inset;
    let bottom = rect.bottom() - inset;
    let expected_025 = bottom - 0.25 * (bottom - top);
    let expected_075 = bottom - 0.75 * (bottom - top);
    let mut centers = tick_centers(&harness);
    centers.sort_by(|a, b| b.y.partial_cmp(&a.y).unwrap());
    assert_eq!(centers.len(), 2, "two ticks expected, got {centers:?}");
    assert!((centers[0].y - expected_025).abs() <= 0.5);
    assert!((centers[1].y - expected_075).abs() <= 0.5);
}

#[test]
fn circular_handle_uses_the_full_radius_inset() {
    let marks = [SliderMark::new(0.0).unwrap(), SliderMark::new(1.0).unwrap()];
    let mut harness = Harness::builder().build_ui_state(
        move |ui, state: &mut (f64, Option<egui::Rect>)| {
            ui.visuals_mut().handle_shape = egui::style::HandleShape::Circle;
            let response = Slider::new(&mut state.0, &Theme::light())
                .unwrap()
                .label("Level")
                .marks(&marks)
                .show(ui);
            state.1 = Some(response.rect);
        },
        (0.5_f64, None),
    );
    harness.run();
    let rect = harness.state().1.expect("response rect");
    let inset = rect.height() / 2.5;
    let mut centers = tick_centers(&harness);
    centers.sort_by(|a, b| a.x.partial_cmp(&b.x).unwrap());
    assert_eq!(centers.len(), 2);
    assert!((centers[0].x - (rect.left() + inset)).abs() <= 0.5);
    assert!((centers[1].x - (rect.right() - inset)).abs() <= 0.5);
}
