use egui_kittest::{
    Harness,
    kittest::{NodeT, Queryable},
};
use temivu::{CircularProgress, CircularSize, Motion, Progress, Theme};

#[test]
fn rejects_invalid_progress_and_width() {
    let theme = Theme::light();
    for value in [-0.1, 1.1, f32::NAN, f32::INFINITY] {
        assert!(Progress::determinate(&theme, value).is_err());
    }
    assert!(Progress::determinate(&theme, 0.0).is_ok());
    assert!(Progress::determinate(&theme, 1.0).is_ok());
    assert!(Progress::indeterminate(&theme).width(-1.0).is_err());
}

#[test]
fn values_are_exposed_and_update_as_the_fill_changes() {
    let mut harness = Harness::builder().build_ui_state(
        |ui, value: &mut f32| {
            let theme = Theme::light();
            Progress::determinate(&theme, *value)
                .unwrap()
                .label("Download")
                .width(100.0)
                .unwrap()
                .show(ui);
        },
        0.5_f32,
    );
    harness.run();
    let node = harness.get_by_label("Download");
    assert_eq!(node.accesskit_node().numeric_value(), Some(50.0));
    let widths = |harness: &Harness<'_, f32>| {
        harness
            .output()
            .shapes
            .iter()
            .filter_map(|s| match &s.shape {
                egui::Shape::Rect(rect) if rect.fill == Theme::light().primary => {
                    Some(rect.rect.width())
                }
                _ => None,
            })
            .collect::<Vec<_>>()
    };
    assert!(widths(&harness).contains(&50.0));
    *harness.state_mut() = 0.9;
    harness.run();
    assert_eq!(
        harness
            .get_by_label("Download")
            .accesskit_node()
            .numeric_value(),
        Some(90.0)
    );
    assert!(widths(&harness).contains(&90.0));
}

#[test]
fn indeterminate_and_circular_expose_labels_without_numeric_values() {
    let mut harness = Harness::builder().build_ui(|ui| {
        let theme = Theme::dark();
        Progress::indeterminate(&theme)
            .label("Loading")
            .motion(Motion::Disabled)
            .width(100.0)
            .unwrap()
            .show(ui);
        CircularProgress::new(&theme)
            .label("Waiting")
            .size(CircularSize::Large, true)
            .motion(Motion::Disabled)
            .show(ui);
    });
    harness.run();
    assert_eq!(
        harness
            .get_by_label("Loading")
            .accesskit_node()
            .numeric_value(),
        None
    );
    assert_eq!(
        harness
            .get_by_label("Waiting")
            .accesskit_node()
            .numeric_value(),
        None
    );
    assert_eq!(
        harness.get_by_label("Waiting").rect().size(),
        egui::vec2(20.0, 20.0)
    );
    let fills: Vec<_> = harness
        .output()
        .shapes
        .iter()
        .filter_map(|s| match &s.shape {
            egui::Shape::Rect(rect) if rect.fill == Theme::dark().primary => {
                Some(rect.rect.width())
            }
            _ => None,
        })
        .collect();
    assert!(fills.contains(&40.0));
    for (size, expected) in [
        (CircularSize::ExtraSmall, 12.0),
        (CircularSize::Small, 14.0),
        (CircularSize::ExtraLarge, 20.0),
    ] {
        assert_eq!(size.points(false), expected);
    }
}

#[test]
fn indeterminate_progress_limits_repaints_during_pause() {
    let mut harness = Harness::builder().build_ui(|ui| {
        Progress::indeterminate(&Theme::light())
            .width(100.0)
            .unwrap()
            .show(ui);
    });
    for _ in 0..3 {
        harness.input_mut().time = Some(1.2);
        harness.step();
    }
    // egui may schedule an unrelated repaint earlier than the animation's 300 ms.
    let delay = harness.output().viewport_output[&egui::ViewportId::ROOT].repaint_delay;
    assert!(
        delay >= std::time::Duration::from_millis(30),
        "pause repaint delay was {delay:?}"
    );
}

#[test]
fn zero_animation_time_keeps_both_indicators_still() {
    let mut harness = Harness::builder().build_ui(|ui| {
        ui.style_mut().animation_time = 0.0;
        let theme = Theme::light();
        Progress::indeterminate(&theme)
            .width(100.0)
            .unwrap()
            .show(ui);
        CircularProgress::new(&theme).show(ui);
    });
    let shapes_at = |harness: &mut Harness<'_, ()>, time| {
        harness.input_mut().time = Some(time);
        harness.step();
        let mut fill_width = None;
        let mut arc = None;
        for shape in &harness.output().shapes {
            match &shape.shape {
                egui::Shape::Rect(rect) if rect.fill == Theme::light().primary => {
                    fill_width = Some(rect.rect.width());
                }
                egui::Shape::Path(path) => arc = Some(path.points.clone()),
                _ => {}
            }
        }
        (
            fill_width.expect("progress fill"),
            arc.expect("progress arc"),
        )
    };
    let first = shapes_at(&mut harness, 0.1);
    let later = shapes_at(&mut harness, 0.6);
    assert_eq!(first.0, 40.0);
    assert_eq!(first, later);
}
