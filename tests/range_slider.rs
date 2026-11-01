use egui_kittest::{
    Harness,
    kittest::{NodeT, Queryable},
};
use temivu::{RangeSlider, Theme};

fn harness(initial: (f64, f64)) -> Harness<'static, (f64, f64)> {
    let mut harness = Harness::builder().build_ui_state(
        |ui, value: &mut (f64, f64)| {
            RangeSlider::new(value, &Theme::light())
                .expect("valid range")
                .label("Price range")
                .step(0.05)
                .expect("valid step")
                .show(ui);
        },
        initial,
    );
    harness.run();
    harness
}

#[test]
fn invalid_values_and_steps_are_rejected() {
    let theme = Theme::light();
    for mut value in [
        (f64::NAN, 0.5),
        (0.0, 1.5),
        (0.8, 0.2),
        (-0.1, 0.5),
        (0.2, f64::INFINITY),
    ] {
        assert!(RangeSlider::new(&mut value, &theme).is_err());
    }
    let mut value = (0.2, 0.8);
    for step in [0.0, -0.1, 1.1, f64::INFINITY] {
        assert!(
            RangeSlider::new(&mut value, &theme)
                .unwrap()
                .step(step)
                .is_err()
        );
    }
}

#[test]
fn exposes_two_slider_semantics_with_numeric_values() {
    let harness = harness((0.2, 0.8));
    let start = harness.get_by_label("Price range start");
    assert_eq!(start.accesskit_node().role(), egui::accesskit::Role::Slider);
    assert_eq!(start.accesskit_node().numeric_value(), Some(0.2));
    let end = harness.get_by_label("Price range end");
    assert_eq!(end.accesskit_node().role(), egui::accesskit::Role::Slider);
    assert_eq!(end.accesskit_node().numeric_value(), Some(0.8));
}

#[test]
fn keyboard_adjusts_the_focused_thumb_and_keeps_the_order() {
    let mut harness = harness((0.2, 0.8));
    harness.get_by_label("Price range end").focus();
    harness.run();
    harness.key_press(egui::Key::ArrowRight);
    harness.run();
    assert!((harness.state().1 - 0.85).abs() < 1e-9);
    harness.key_press(egui::Key::End);
    harness.run();
    assert!((harness.state().1 - 1.0).abs() < 1e-9);
    // The start thumb cannot pass the end thumb.
    harness.get_by_label("Price range start").focus();
    harness.run();
    harness.key_press(egui::Key::End);
    harness.run();
    assert!((harness.state().0 - 1.0).abs() < 1e-9);
    assert!((harness.state().1 - 1.0).abs() < 1e-9);
    harness.key_press(egui::Key::Home);
    harness.run();
    assert!((harness.state().0).abs() < 1e-9);
}

#[test]
fn dragging_the_start_thumb_updates_the_range() {
    let mut harness = harness((0.2, 0.8));
    let center = harness.get_by_label("Price range start").rect().center();
    harness.input_mut().events.push(egui::Event::PointerButton {
        pos: center,
        button: egui::PointerButton::Primary,
        pressed: true,
        modifiers: egui::Modifiers::default(),
    });
    harness.step();
    harness
        .input_mut()
        .events
        .push(egui::Event::PointerMoved(center + egui::vec2(40.0, 0.0)));
    harness.step();
    harness.input_mut().events.push(egui::Event::PointerButton {
        pos: center + egui::vec2(40.0, 0.0),
        button: egui::PointerButton::Primary,
        pressed: false,
        modifiers: egui::Modifiers::default(),
    });
    harness.step();
    assert!(
        harness.state().0 > 0.2,
        "dragging right should increase the start value, got {}",
        harness.state().0
    );
    assert!(harness.state().0 <= harness.state().1);
}

#[test]
fn disabled_range_slider_cannot_change_or_expose_actions() {
    let mut harness = Harness::builder().build_ui_state(
        |ui, value: &mut (f64, f64)| {
            RangeSlider::new(value, &Theme::dark())
                .expect("valid range")
                .label("Budget")
                .enabled(false)
                .show(ui);
        },
        (0.25, 0.75),
    );
    harness.run();
    let start = harness.get_by_label("Budget start");
    assert!(start.accesskit_node().is_disabled());
    assert!(
        !start
            .accesskit_node()
            .data()
            .supports_action(egui::accesskit::Action::Increment)
    );
    harness.get_by_label("Budget end").focus();
    harness.run();
    assert!(
        !harness.get_by_label("Budget end").is_focused(),
        "disabled thumbs should not be focusable"
    );
    harness.get_by_label("Budget start").click();
    harness.run();
    assert_eq!(*harness.state(), (0.25, 0.75));
}

#[test]
fn accesskit_increment_and_decrement_actions_change_the_value() {
    use egui::accesskit::{Action, ActionRequest, TreeId};

    let slider_id = egui::Id::new("at-range");
    let mut harness = Harness::builder().build_ui_state(
        move |ui, value: &mut (f64, f64)| {
            RangeSlider::new(value, &Theme::light())
                .expect("valid range")
                .id(slider_id)
                .label("Price range")
                .step(0.05)
                .expect("valid step")
                .show(ui);
        },
        (0.2, 0.8),
    );
    harness.run();
    let target = slider_id.with("range-end").accesskit_id();
    harness
        .input_mut()
        .events
        .push(egui::Event::AccessKitActionRequest(ActionRequest {
            action: Action::Increment,
            target_tree: TreeId::ROOT,
            target_node: target,
            data: None,
        }));
    harness.step();
    assert!((harness.state().1 - 0.85).abs() < 1e-9);
    harness
        .input_mut()
        .events
        .push(egui::Event::AccessKitActionRequest(ActionRequest {
            action: Action::Decrement,
            target_tree: TreeId::ROOT,
            target_node: target,
            data: None,
        }));
    harness.step();
    assert!((harness.state().1 - 0.8).abs() < 1e-9);
}

#[test]
fn arrow_keys_do_not_move_focus_away_from_the_focused_thumb() {
    let mut harness = Harness::builder().build_ui_state(
        |ui, state: &mut ((f64, f64), (f64, f64))| {
            let theme = Theme::light();
            let _ = ui.button("Before");
            RangeSlider::new(&mut state.0, &theme)
                .expect("valid range")
                .label("First")
                .show(ui);
            RangeSlider::new(&mut state.1, &theme)
                .expect("valid range")
                .label("Second")
                .show(ui);
        },
        ((0.2, 0.8), (0.2, 0.8)),
    );
    harness.run();
    harness.get_by_label("First start").focus();
    harness.run();
    harness.key_press(egui::Key::ArrowUp);
    harness.run();
    assert!(
        harness.get_by_label("First start").is_focused(),
        "the focus lock should keep arrow keys from moving focus"
    );
    assert!((harness.state().0.0 - 0.25).abs() < 1e-9);
}

#[test]
fn same_label_sliders_get_distinct_default_ids() {
    let mut harness = Harness::builder().build_ui_state(
        |ui, state: &mut ((f64, f64), (f64, f64))| {
            let theme = Theme::light();
            RangeSlider::new(&mut state.0, &theme)
                .expect("valid range")
                .label("Range")
                .show(ui);
            RangeSlider::new(&mut state.1, &theme)
                .expect("valid range")
                .label("Range")
                .show(ui);
        },
        ((0.2, 0.8), (0.2, 0.8)),
    );
    harness.run();
    assert_eq!(
        harness.query_all_by_label("Range start").count(),
        2,
        "both same-label sliders should expose their own thumb"
    );
    {
        let first = harness
            .query_all_by_label("Range start")
            .next()
            .expect("first slider thumb");
        first.focus();
    }
    harness.run();
    harness.key_press(egui::Key::End);
    harness.run();
    assert!((harness.state().0.0 - 0.8).abs() < 1e-9);
    assert!(
        (harness.state().1.0 - 0.2).abs() < 1e-9,
        "same-label sliders must not share state through a colliding ID"
    );
}

#[test]
fn clicking_the_track_at_a_place_that_snaps_to_the_current_value_does_not_mark_changed() {
    let mut harness = Harness::builder().build_ui_state(
        |ui, state: &mut ((f64, f64), bool)| {
            let response = RangeSlider::new(&mut state.0, &Theme::light())
                .expect("valid range")
                .label("Price range")
                .step(0.2)
                .expect("valid step")
                .show(ui);
            state.1 |= response.changed();
        },
        ((0.2, 0.8), false),
    );
    harness.run();
    let start = harness.get_by_label("Price range start").rect().center();
    // 0.05 of the 254-point track to the right of the start thumb: the snapped
    // value is still 0.2, and the point lies outside the thumb so the track's
    // click path (outer response) is the one that runs.
    let position = start + egui::vec2(12.7, 0.0);
    for pressed in [true, false] {
        harness.input_mut().events.push(egui::Event::PointerButton {
            pos: position,
            button: egui::PointerButton::Primary,
            pressed,
            modifiers: egui::Modifiers::default(),
        });
    }
    harness.run();
    assert_eq!(harness.state().0, (0.2, 0.8));
    assert!(
        !harness.state().1,
        "a track click that snaps to the current value must not report a change"
    );
}

#[test]
fn accesskit_actions_at_bounds_and_in_the_middle() {
    use egui::accesskit::{Action, ActionRequest, TreeId};

    let slider_id = egui::Id::new("clamped-range");
    let mut harness = Harness::builder().build_ui_state(
        move |ui, value: &mut (f64, f64)| {
            RangeSlider::new(value, &Theme::light())
                .expect("valid range")
                .id(slider_id)
                .label("Budget")
                .step(0.05)
                .expect("valid step")
                .show(ui);
        },
        (0.0, 1.0),
    );
    harness.run();
    let start = slider_id.with("range-start").accesskit_id();
    let end = slider_id.with("range-end").accesskit_id();
    let request = |action, target| {
        egui::Event::AccessKitActionRequest(ActionRequest {
            action,
            target_tree: TreeId::ROOT,
            target_node: target,
            data: None,
        })
    };
    // Decrementing the start thumb at the lower bound stays at zero.
    harness
        .input_mut()
        .events
        .push(request(Action::Decrement, start));
    harness.step();
    assert_eq!(harness.state().0, 0.0);
    // Incrementing the end thumb at the upper bound stays at one.
    harness
        .input_mut()
        .events
        .push(request(Action::Increment, end));
    harness.step();
    assert_eq!(harness.state().1, 1.0);

    // Two action requests in one frame are both applied in the middle range.
    let mid_id = egui::Id::new("mid-range");
    let mut mid = Harness::builder().build_ui_state(
        move |ui, value: &mut (f64, f64)| {
            RangeSlider::new(value, &Theme::light())
                .expect("valid range")
                .id(mid_id)
                .label("Mood")
                .step(0.05)
                .expect("valid step")
                .show(ui);
        },
        (0.2, 0.8),
    );
    mid.run();
    let mid_end = mid_id.with("range-end").accesskit_id();
    mid.input_mut()
        .events
        .push(request(Action::Increment, mid_end));
    mid.input_mut()
        .events
        .push(request(Action::Increment, mid_end));
    mid.step();
    assert!((mid.state().1 - 0.9).abs() < 1e-9);
}

#[test]
fn vertical_slider_reports_vertical_orientation_and_maps_downwards() {
    let mut harness = Harness::builder().build_ui_state(
        |ui, value: &mut (f64, f64)| {
            RangeSlider::new(value, &Theme::light())
                .expect("valid range")
                .label("Level")
                .vertical(true)
                .show(ui);
        },
        (0.2, 0.8),
    );
    harness.run();
    let start = harness.get_by_label("Level start");
    assert_eq!(
        start.accesskit_node().orientation(),
        Some(egui::accesskit::Orientation::Vertical)
    );
    assert_eq!(start.accesskit_node().numeric_value(), Some(0.2));
    harness.get_by_label("Level start").focus();
    harness.run();
    harness.key_press(egui::Key::ArrowUp);
    harness.run();
    assert!(
        (harness.state().0 - 0.25).abs() < 1e-9,
        "up should increase"
    );
    // Dragging downwards decreases the value.
    let center = harness.get_by_label("Level start").rect().center();
    harness.input_mut().events.push(egui::Event::PointerButton {
        pos: center,
        button: egui::PointerButton::Primary,
        pressed: true,
        modifiers: egui::Modifiers::default(),
    });
    harness.step();
    harness
        .input_mut()
        .events
        .push(egui::Event::PointerMoved(center + egui::vec2(0.0, 40.0)));
    harness.step();
    harness.input_mut().events.push(egui::Event::PointerButton {
        pos: center + egui::vec2(0.0, 40.0),
        button: egui::PointerButton::Primary,
        pressed: false,
        modifiers: egui::Modifiers::default(),
    });
    harness.step();
    assert!(
        harness.state().0 < 0.25,
        "dragging down should decrease the value, got {}",
        harness.state().0
    );
    assert!(harness.state().0 <= harness.state().1);
}

#[test]
fn long_vertical_labels_are_truncated_with_an_ellipsis() {
    let mut harness = Harness::builder().build_ui_state(
        |ui, value: &mut (f64, f64)| {
            RangeSlider::new(value, &Theme::light())
                .expect("valid range")
                .label("A very long vertical range label")
                .vertical(true)
                .show(ui);
        },
        (0.2, 0.8),
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
    assert!(
        texts
            .iter()
            .any(|text| text.starts_with('A') && text.ends_with('…')),
        "the truncated label should be painted, got {texts:?}"
    );
    assert!(
        !texts
            .iter()
            .any(|text| text == "A very long vertical range label"),
        "the full label must not be painted in the narrow column"
    );
}

#[test]
fn clicks_on_the_label_strip_do_not_change_the_values() {
    let mut harness = Harness::builder().build_ui_state(
        |ui, state: &mut ((f64, f64), Option<egui::Rect>)| {
            let response = RangeSlider::new(&mut state.0, &Theme::light())
                .expect("valid range")
                .label("Level")
                .vertical(true)
                .show(ui);
            state.1 = Some(response.rect);
        },
        ((0.2, 0.8), None),
    );
    harness.run();
    let rect = harness.state().1.expect("response rect");
    assert!((64.0..=240.0).contains(&rect.height()));
    for pressed in [true, false] {
        harness.input_mut().events.push(egui::Event::PointerButton {
            pos: egui::pos2(rect.center().x, rect.top() + 3.0),
            button: egui::PointerButton::Primary,
            pressed,
            modifiers: egui::Modifiers::default(),
        });
    }
    harness.run();
    assert_eq!(
        harness.state().0,
        (0.2, 0.8),
        "clicking the label strip must not move a thumb"
    );
}

#[test]
fn vertical_slider_fits_a_shallow_parent() {
    let mut harness = Harness::builder().build_ui_state(
        |ui, state: &mut ((f64, f64), Option<egui::Rect>)| {
            let theme = Theme::light();
            ui.allocate_ui(egui::vec2(120.0, 80.0), |ui| {
                let response = RangeSlider::new(&mut state.0, &theme)
                    .expect("valid range")
                    .label("Level")
                    .vertical(true)
                    .show(ui);
                state.1 = Some(response.rect);
            });
        },
        ((0.2, 0.8), None),
    );
    harness.run();
    let rect = harness.state().1.expect("response rect");
    assert!(
        rect.height() <= 80.0 + 1.0,
        "the slider should fit the 80-point parent, got {}",
        rect.height()
    );
}

#[test]
fn horizontal_label_strip_clicks_are_ignored() {
    let mut harness = Harness::builder().build_ui_state(
        |ui, state: &mut ((f64, f64), Option<egui::Rect>)| {
            let response = RangeSlider::new(&mut state.0, &Theme::light())
                .expect("valid range")
                .label("Price range")
                .show(ui);
            state.1 = Some(response.rect);
        },
        ((0.2, 0.8), None),
    );
    harness.run();
    let rect = harness.state().1.expect("response rect");
    for pressed in [true, false] {
        harness.input_mut().events.push(egui::Event::PointerButton {
            pos: egui::pos2(rect.left() + 4.0, rect.top() + 2.0),
            button: egui::PointerButton::Primary,
            pressed,
            modifiers: egui::Modifiers::default(),
        });
    }
    harness.run();
    assert_eq!(harness.state().0, (0.2, 0.8));
}

#[test]
fn vertical_slider_overflows_a_parent_shorter_than_the_minimum() {
    let mut harness = Harness::builder().build_ui_state(
        |ui, state: &mut ((f64, f64), Option<egui::Rect>)| {
            let theme = Theme::light();
            ui.allocate_ui(egui::vec2(120.0, 40.0), |ui| {
                let response = RangeSlider::new(&mut state.0, &theme)
                    .expect("valid range")
                    .label("Level")
                    .vertical(true)
                    .show(ui);
                state.1 = Some(response.rect);
            });
        },
        ((0.2, 0.8), None),
    );
    harness.run();
    assert!(
        harness.state().1.expect("response rect").height() >= 64.0 - 1.0,
        "the documented minimum height applies even when the parent is shorter"
    );
}
