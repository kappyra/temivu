use egui_kittest::{
    Harness,
    kittest::{NodeT, Queryable},
};
use temivu::{Radio, Theme};

#[test]
fn pointer_changes_selection_and_disabled_does_not() {
    let mut harness = Harness::builder().build_ui_state(
        |ui, values: &mut [bool; 2]| {
            let theme = Theme::light();
            Radio::new(&mut values[0], &theme, "Option").show(ui);
            Radio::new(&mut values[1], &theme, "Disabled")
                .enabled(false)
                .touch(true)
                .error(true)
                .show(ui);
        },
        [false; 2],
    );
    let node = harness.get_by_label("Option");
    assert_eq!(
        node.accesskit_node().role(),
        egui::accesskit::Role::RadioButton
    );
    assert_eq!(
        node.accesskit_node().toggled(),
        Some(egui::accesskit::Toggled::False)
    );
    node.click();
    harness.get_by_label("Disabled").click();
    harness.run();
    assert_eq!(*harness.state(), [true, false]);
    assert_eq!(
        harness.get_by_label("Option").accesskit_node().toggled(),
        Some(egui::accesskit::Toggled::True)
    );
    assert!(
        harness
            .get_by_label("Disabled")
            .accesskit_node()
            .is_disabled()
    );
    harness.get_by_label("Option").click();
    harness.run();
    assert!(!harness.state()[0]);
}

#[test]
fn keyboard_space_changes_selection() {
    let mut harness = Harness::builder().build_ui_state(
        |ui, value: &mut bool| {
            Radio::new(value, &Theme::light(), "Keyboard").show(ui);
        },
        false,
    );
    harness.get_by_label("Keyboard").focus();
    harness.run();
    harness.key_press(egui::Key::Space);
    harness.run();
    assert!(*harness.state());
}

#[test]
fn radio_uses_forui_circle_and_indicator_sizes() {
    for (theme, touch, expected) in [
        (Theme::light(), false, 16.0),
        (Theme::light().touch(), true, 20.0),
    ] {
        let border = theme.muted_foreground;
        let mut harness = Harness::builder().build_ui(move |ui| {
            let mut value = false;
            Radio::new(&mut value, &theme, "Option")
                .touch(touch)
                .show(ui);
        });
        harness.run();
        let outer = harness
            .output()
            .shapes
            .iter()
            .find_map(|shape| match &shape.shape {
                egui::Shape::Circle(circle)
                    if circle.stroke.color == border && circle.stroke.width > 0.0 =>
                {
                    Some(circle.radius)
                }
                _ => None,
            })
            .expect("radio outer circle");
        assert_eq!(outer * 2.0, expected, "radio diameter");
    }
}
