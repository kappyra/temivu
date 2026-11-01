use egui_kittest::{
    Harness,
    kittest::{NodeT, Queryable},
};
use temivu::{Switch, Theme};

#[test]
fn pointer_and_keyboard_change_the_caller_owned_value() {
    let mut harness = Harness::builder().build_ui_state(
        |ui, value| {
            Switch::new(value, &Theme::light())
                .label("Wi-Fi")
                .description("Nearby networks")
                .show(ui);
        },
        false,
    );
    harness.run();
    let node = harness.get_by_label("Wi-Fi");
    assert_eq!(node.accesskit_node().role(), egui::accesskit::Role::Switch);
    assert_eq!(
        node.accesskit_node().description().as_deref(),
        Some("Nearby networks")
    );
    assert_eq!(
        node.accesskit_node().toggled(),
        Some(egui::accesskit::Toggled::False)
    );
    node.click();
    harness.run();
    assert!(*harness.state());
    assert_eq!(
        harness.get_by_label("Wi-Fi").accesskit_node().toggled(),
        Some(egui::accesskit::Toggled::True)
    );
    harness.get_by_label("Wi-Fi").focus();
    harness.run();
    harness.key_press(egui::Key::Space);
    harness.run();
    assert!(!*harness.state());
}

#[test]
fn disabled_switch_cannot_be_toggled() {
    let mut harness = Harness::builder().build_ui_state(
        |ui, value| {
            Switch::new(value, &Theme::dark())
                .label("Disabled")
                .enabled(false)
                .leading_label(true)
                .show(ui);
        },
        true,
    );
    harness.run();
    let node = harness.get_by_label("Disabled");
    assert!(node.accesskit_node().is_disabled());
    node.click();
    harness.run();
    assert!(*harness.state());
}

#[test]
fn disabled_switch_track_is_dimmed_once() {
    for inherited in [false, true] {
        let mut harness = Harness::builder().build_ui_state(
            move |ui, value| {
                if inherited {
                    ui.disable();
                }
                Switch::new(value, &Theme::light())
                    .label("Quiet")
                    .enabled(inherited)
                    .show(ui);
            },
            false,
        );
        harness.run();
        let expected = Theme::light().secondary.gamma_multiply(0.5);
        assert!(harness.output().shapes.iter().any(|shape| {
            matches!(&shape.shape, egui::Shape::Rect(rect) if rect.fill == expected)
        }));
    }
}

#[test]
fn enabled_off_track_has_visible_outline_in_both_themes() {
    fn luminance(color: egui::Color32) -> f64 {
        let channel = |value: u8| {
            let value = f64::from(value) / 255.0;
            if value <= 0.04045 {
                value / 12.92
            } else {
                ((value + 0.055) / 1.055).powf(2.4)
            }
        };
        0.2126 * channel(color.r()) + 0.7152 * channel(color.g()) + 0.0722 * channel(color.b())
    }
    for theme in [Theme::light(), Theme::dark()] {
        let expected = theme.muted_foreground;
        let (a, b) = (luminance(expected), luminance(theme.background));
        assert!((a.max(b) + 0.05) / (a.min(b) + 0.05) >= 3.0);
        let mut harness = Harness::builder().build_ui(move |ui| {
            let mut value = false;
            Switch::new(&mut value, &theme).label("Option").show(ui);
        });
        harness.run();
        assert!(harness.output().shapes.iter().any(|shape| matches!(
            &shape.shape, egui::Shape::Rect(rect) if rect.stroke.color == expected && rect.stroke.width > 0.0
        )));
    }
}

#[test]
fn long_labels_wrap_without_clipping_switch_at_narrow_widths_and_zoom() {
    for (width, zoom) in [(200.0, 1.0), (300.0, 1.5)] {
        for leading in [false, true] {
            let mut harness = Harness::builder()
                .with_size(egui::vec2(width, 450.0))
                .build_ui_state(
                    move |ui, value| {
                        Switch::new(value, &Theme::light())
                            .label("Automatically synchronize every update")
                            .description("Updates this device when connected")
                            .leading_label(leading)
                            .show(ui);
                    },
                    false,
                );
            harness.ctx.set_zoom_factor(zoom);
            harness.run();
            let node = harness.get_by_label("Automatically synchronize every update");
            assert!(node.rect().right() <= width / zoom, "{:?}", node.rect());
            assert_eq!(
                node.accesskit_node().description().as_deref(),
                Some("Updates this device when connected")
            );
            node.click();
            harness.run();
            assert!(*harness.state());
        }
    }
}

#[test]
fn unbroken_long_label_does_not_push_switch_offscreen() {
    let label = "ABCDEFGHIJKLMNOPQRSTUVWXYZABCDEFGHIJKLMNOPQRSTUVWXYZ";
    for leading in [false, true] {
        let mut harness = Harness::builder()
            .with_size(egui::vec2(200.0, 600.0))
            .build_ui_state(
                move |ui, value| {
                    Switch::new(value, &Theme::light())
                        .label(label)
                        .leading_label(leading)
                        .show(ui);
                },
                false,
            );
        harness.run();
        let node = harness.get_by_label(label);
        assert!(node.rect().right() <= 200.0, "{:?}", node.rect());
        node.click();
        harness.run();
        assert!(*harness.state());
    }
}

#[test]
fn switch_track_and_thumb_follow_forui_cupertino_geometry() {
    for (theme, track_fill) in [
        (Theme::light(), Theme::light().secondary),
        (Theme::light(), Theme::light().primary),
    ] {
        let checked = track_fill == theme.primary;
        let mut harness = Harness::builder().build_ui(move |ui| {
            let mut value = checked;
            Switch::new(&mut value, &theme).show(ui);
        });
        harness.run();
        let track = harness
            .output()
            .shapes
            .iter()
            .find_map(|shape| match &shape.shape {
                egui::Shape::Rect(rect) if rect.fill == track_fill => Some(rect.rect),
                _ => None,
            })
            .expect("switch track");
        assert_eq!(track.size(), egui::vec2(51.0, 31.0));
        let thumb = harness
            .output()
            .shapes
            .iter()
            .find_map(|shape| match &shape.shape {
                egui::Shape::Circle(circle) if circle.radius == 13.5 => Some(circle.center),
                _ => None,
            })
            .expect("27 point thumb");
        assert!(track.contains(thumb));
    }
}
