use egui_kittest::Harness;
use egui_kittest::kittest::{NodeT, Queryable};
use temivu::{Button, ButtonSize, InputMode, Theme};

#[test]
fn button_clicks_and_disabled_does_not_click() {
    let mut harness = Harness::builder().build_ui_state(
        |ui, clicked| {
            let theme = Theme::light();
            if Button::new("Save", &theme).show(ui).clicked() {
                *clicked = true;
            }
        },
        false,
    );
    harness.get_by_label("Save").click();
    harness.run();
    assert!(*harness.state());

    let mut disabled = Harness::builder().build_ui_state(
        |ui, clicked| {
            let theme = Theme::light();
            if Button::new("Disabled", &theme)
                .enabled(false)
                .show(ui)
                .clicked()
            {
                *clicked = true;
            }
        },
        false,
    );
    disabled.get_by_label("Disabled").click();
    disabled.run();
    assert!(!*disabled.state());
}

#[test]
fn sizes_match_upstream_tokens() {
    assert_eq!(ButtonSize::ExtraSmall.height(InputMode::Desktop), 24.0);
    assert_eq!(ButtonSize::Medium.height(InputMode::Touch), 44.0);
    assert_eq!(ButtonSize::Large.height(InputMode::Touch), 48.0);
}

#[test]
fn touch_theme_uses_touch_button_size_by_default() {
    let mut harness = Harness::builder().build_ui(|ui| {
        Button::new("Touch action", &Theme::mint_light().touch()).show(ui);
    });
    harness.run();
    assert!(harness.get_by_label("Touch action").rect().height() >= 44.0);
}

#[test]
fn keyboard_focus_and_enter_activate_button() {
    let mut harness = Harness::builder().build_ui_state(
        |ui, clicked| {
            if Button::new("Keyboard", &Theme::light()).show(ui).clicked() {
                *clicked = true;
            }
        },
        false,
    );
    harness.get_by_label("Keyboard").focus();
    harness.run();
    harness.key_press(egui::Key::Enter);
    harness.run();
    assert!(*harness.state());
}

#[test]
fn themed_frame_is_painted_when_global_button_frames_are_disabled() {
    let mut harness = Harness::builder().build_ui(|ui| {
        ui.visuals_mut().button_frame = false;
        Button::new("Frame", &Theme::light()).show(ui);
    });
    harness.run();
    let fill = Theme::light().primary;
    assert!(
        harness
            .output()
            .shapes
            .iter()
            .any(|shape| { matches!(&shape.shape, egui::Shape::Rect(rect) if rect.fill == fill) })
    );
}

#[test]
fn disabled_background_is_dimmed_once() {
    let mut harness = Harness::builder().build_ui(|ui| {
        Button::new("Muted", &Theme::light())
            .enabled(false)
            .show(ui);
    });
    harness.run();
    let expected = Theme::light().primary.gamma_multiply(0.5);
    assert!(
        harness.output().shapes.iter().any(|shape| {
            matches!(&shape.shape, egui::Shape::Rect(rect) if rect.fill == expected)
        })
    );
}

#[test]
fn pointer_hover_changes_button_background() {
    let mut harness = Harness::builder().build_ui(|ui| {
        Button::new("Hover", &Theme::light()).show(ui);
    });
    harness.run();
    let before = Theme::light().primary;
    assert!(
        harness.output().shapes.iter().any(|shape| {
            matches!(&shape.shape, egui::Shape::Rect(rect) if rect.fill == before)
        })
    );
    harness.get_by_label("Hover").hover();
    harness.run();
    assert!(
        !harness.output().shapes.iter().any(|shape| {
            matches!(&shape.shape, egui::Shape::Rect(rect) if rect.fill == before)
        })
    );
}

#[test]
fn button_content_padding_matches_forui_size_tokens() {
    for (size, desktop, touch) in [
        (ButtonSize::ExtraSmall, (8.0, 6.0), (10.0, 9.0)),
        (ButtonSize::Small, (10.0, 9.0), (12.0, 12.0)),
        (ButtonSize::Medium, (10.0, 11.0), (12.0, 14.0)),
        (ButtonSize::Large, (10.0, 13.0), (12.0, 16.0)),
    ] {
        assert_eq!(
            size.padding(InputMode::Desktop),
            egui::vec2(desktop.0, desktop.1)
        );
        assert_eq!(size.padding(InputMode::Touch), egui::vec2(touch.0, touch.1));
    }
}

#[test]
fn icon_glyph_is_rendered_but_the_accessible_name_stays_the_label() {
    let mut harness = Harness::builder().build_ui_state(
        |ui, clicks: &mut usize| {
            let response = Button::new("Continue", &Theme::light()).icon("+").show(ui);
            if response.clicked() {
                *clicks += 1;
            }
        },
        0usize,
    );
    harness.run();
    let button = harness.get_by_label("Continue");
    assert_eq!(
        button.accesskit_node().label().as_deref(),
        Some("Continue"),
        "the accessible name must not include the icon glyph"
    );
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
        texts.iter().any(|text| text.contains("+")),
        "the icon glyph should be painted, got {texts:?}"
    );
    harness.get_by_label("Continue").click();
    harness.run();
    assert_eq!(*harness.state(), 1);
}

#[test]
fn icon_button_remains_clickable_in_a_narrow_parent() {
    let mut harness = Harness::builder().build_ui_state(
        |ui, state: &mut (usize, Option<egui::Rect>)| {
            ui.allocate_ui(egui::vec2(80.0, 40.0), |ui| {
                let response = Button::new("Open settings", &Theme::light())
                    .icon("+")
                    .show(ui);
                if response.clicked() {
                    state.0 += 1;
                }
                state.1 = Some(response.rect);
            });
        },
        (0usize, None),
    );
    harness.run();
    harness.get_by_label("Open settings").click();
    harness.run();
    assert_eq!(harness.state().0, 1);
    let rect = harness.state().1.expect("response rect");
    assert!(
        rect.width() <= 80.0,
        "the icon button should fit the narrow parent by truncating its single text job, got {}",
        rect.width()
    );
}

#[test]
fn button_deforms_while_held_without_shrinking_the_hit_area() {
    let theme = Theme::light();
    let mut harness = Harness::builder().build_ui(|ui| {
        Button::new("Spring", &theme).show(ui);
    });
    harness
        .ctx
        .all_styles_mut(|style| style.animation_time = 0.2);
    let size = |harness: &Harness<'_, ()>| {
        harness
            .output()
            .shapes
            .iter()
            .find_map(|shape| match &shape.shape {
                egui::Shape::Rect(rect) if rect.fill.a() == 255 && rect.corner_radius.nw > 0 => {
                    Some((rect.rect.width(), f32::from(rect.corner_radius.nw)))
                }
                egui::Shape::Path(path) if path.fill.a() == 255 => {
                    let bounds = egui::Rect::from_points(&path.points);
                    let left_at_top = path
                        .points
                        .iter()
                        .filter(|point| point.y == bounds.top())
                        .map(|point| point.x)
                        .fold(f32::INFINITY, f32::min);
                    Some((bounds.width(), left_at_top - bounds.left()))
                }
                _ => None,
            })
            .unwrap()
    };
    let before = size(&harness);
    let hit = harness.get_by_label("Spring").rect();
    let center = hit.center();
    harness.hover_at(center);
    harness.input_mut().time = Some(2.0);
    harness.step();
    harness.drag_at(center);
    harness.input_mut().time = Some(2.01);
    harness.step();
    harness.input_mut().time = Some(2.12);
    harness.step();
    let pressed = size(&harness);
    harness.input_mut().time = Some(2.121);
    harness.step();
    let next = size(&harness);
    assert!((next.1 - pressed.1).abs() > 0.0 && (next.1 - pressed.1).abs() < 1.0);
    assert!(pressed.0 < before.0 - 1.0 && pressed.1 < before.1 - 2.0);
    assert_eq!(harness.get_by_label("Spring").rect(), hit);
}
