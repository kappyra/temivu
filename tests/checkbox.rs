use egui_kittest::{
    Harness,
    kittest::{NodeT, Queryable},
};
use temivu::{Checkbox, Theme};

#[test]
fn touch_theme_expands_checkbox_hit_target() {
    let mut harness = Harness::builder().build_ui_state(
        |ui, value| {
            Checkbox::new(value, &Theme::mint_light().touch(), "Touch checkbox").show(ui);
        },
        false,
    );
    harness.run();
    assert!(harness.get_by_label("Touch checkbox").rect().height() >= 44.0);
}

#[test]
fn pointer_toggles_and_disabled_ignores_input() {
    let mut harness = Harness::builder().build_ui_state(
        |ui, values: &mut [bool; 2]| {
            let theme = Theme::light();
            Checkbox::new(&mut values[0], &theme, "Enable").show(ui);
            Checkbox::new(&mut values[1], &theme, "Disabled")
                .enabled(false)
                .touch(true)
                .error(true)
                .show(ui);
        },
        [false; 2],
    );
    let node = harness.get_by_label("Enable");
    assert_eq!(
        node.accesskit_node().role(),
        egui::accesskit::Role::CheckBox
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
        harness.get_by_label("Enable").accesskit_node().toggled(),
        Some(egui::accesskit::Toggled::True)
    );
    assert!(
        harness
            .get_by_label("Disabled")
            .accesskit_node()
            .is_disabled()
    );
}

#[test]
fn keyboard_space_toggles_checkbox() {
    let mut harness = Harness::builder().build_ui_state(
        |ui, value: &mut bool| {
            Checkbox::new(value, &Theme::light(), "Keyboard").show(ui);
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
fn checkbox_has_a_rounded_box_and_visible_border() {
    for (theme, touch, expected) in [
        (Theme::light(), false, 20.0),
        (Theme::light().touch(), true, 22.0),
    ] {
        let fill = theme.card;
        let mut harness = Harness::builder().build_ui(move |ui| {
            let mut value = false;
            Checkbox::new(&mut value, &theme, "Box")
                .touch(touch)
                .show(ui);
        });
        harness.run();
        let rect = harness
            .output()
            .shapes
            .iter()
            .find_map(|shape| match &shape.shape {
                egui::Shape::Rect(rect)
                    if rect.fill == fill
                        && rect.stroke.width > 0.0
                        && rect.corner_radius.nw == 6 =>
                {
                    Some(rect.rect)
                }
                _ => None,
            })
            .expect("checkbox box");
        assert_eq!(rect.width(), expected, "checkbox box width");
        assert_eq!(rect.height(), expected, "checkbox box height");
    }
}
