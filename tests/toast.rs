use egui_kittest::{
    Harness,
    kittest::{NodeT, Queryable},
};
use temivu::{Theme, Toast, ToastState};

#[test]
fn persistent_toast_announces_and_can_be_dismissed_by_keyboard() {
    let mut harness = Harness::builder().build_ui_state(
        |ui, state: &mut ToastState| {
            Toast::new(state, &Theme::light(), "Saved", "Dismiss notification")
                .description("Your changes were saved")
                .show(ui);
        },
        ToastState::new(None),
    );
    harness.run();
    harness.get_by_role_and_label(
        egui::accesskit::Role::Alert,
        "Saved: Your changes were saved",
    );
    let dismiss_rect = harness.get_by_label("Dismiss notification").rect();
    assert!(dismiss_rect.width() >= 32.0 && dismiss_rect.height() >= 32.0);
    harness.get_by_label("Dismiss notification").focus();
    harness.run();
    harness.key_press(egui::Key::Enter);
    harness.run();
    assert!(!harness.state().is_visible());
    harness.run();
    assert!(
        harness
            .query_by_role(egui::accesskit::Role::Alert)
            .is_none()
    );
}

#[test]
fn timed_toast_expires_and_dismissal_marks_response_changed() {
    use std::time::Duration;
    let mut timed = Harness::builder().build_ui_state(
        |ui, state: &mut ToastState| {
            Toast::new(state, &Theme::light(), "Temporary", "Dismiss").show(ui);
        },
        ToastState::new(Some(Duration::from_secs(5))),
    );
    timed.run();
    assert!(timed.state().is_visible());
    let now = timed.ctx.input(|input| input.time);
    timed.input_mut().time = Some(now + 6.0);
    timed.step();
    assert!(!timed.state().is_visible());

    let mut dismissible = Harness::builder().build_ui_state(
        |ui, state: &mut (ToastState, bool)| {
            let result = Toast::new(&mut state.0, &Theme::light(), "Saved", "Dismiss").show(ui);
            state.1 |= result.is_some_and(|response| response.changed());
        },
        (ToastState::new(None), false),
    );
    dismissible.get_by_label("Dismiss").click();
    dismissible.run();
    assert!(!dismissible.state().0.is_visible());
    assert!(dismissible.state().1);
}

#[test]
fn narrow_toast_keeps_localized_dismiss_action_usable() {
    let mut harness = Harness::builder()
        .with_size(egui::vec2(200.0, 300.0))
        .build_ui_state(
            |ui, state: &mut ToastState| {
                Toast::new(
                    state,
                    &Theme::light(),
                    "Changes saved on this device",
                    "Dismiss notification",
                )
                .description(
                    "Your updated settings are available whenever this device is connected.",
                )
                .show(ui);
            },
            ToastState::new(None),
        );
    harness.run();
    let action = harness.get_by_label("Dismiss notification");
    assert!(action.rect().right() <= 200.0, "{:?}", action.rect());
    for shape in &harness.output().shapes {
        if let egui::Shape::Text(text) = &shape.shape {
            assert!(
                text.pos.x + text.galley.size().x <= 200.5,
                "{:?}",
                text.galley.job.text
            );
        }
    }
    action.click();
    harness.run();
    assert!(!harness.state().is_visible());
}

#[test]
fn timer_restarts_after_pointer_or_keyboard_interaction() {
    use std::time::Duration;
    for keyboard in [false, true] {
        let mut harness = Harness::builder().build_ui_state(
            |ui, state: &mut ToastState| {
                let _ = ui.button("Other action");
                Toast::new(state, &Theme::light(), "Temporary", "Dismiss").show(ui);
            },
            ToastState::new(Some(Duration::from_secs(5))),
        );
        harness.run();
        if keyboard {
            harness.get_by_label("Dismiss").focus();
        } else {
            let rect = harness.get_by_role(egui::accesskit::Role::Alert).rect();
            harness.hover_at(rect.center());
        }
        harness.run();
        let now = harness.ctx.input(|input| input.time);
        harness.input_mut().time = Some(now + 10.0);
        harness.step();
        assert!(harness.state().is_visible());
        if keyboard {
            harness.get_by_label("Other action").focus();
        } else {
            harness.remove_cursor();
        }
        harness.run();
        let resumed = harness.ctx.input(|input| input.time);
        harness.input_mut().time = Some(resumed + 4.0);
        harness.step();
        assert!(harness.state().is_visible());
        harness.input_mut().time = Some(resumed + 6.0);
        harness.step();
        assert!(!harness.state().is_visible());
    }
}

#[test]
fn zero_duration_never_exposes_a_toast() {
    let mut harness = Harness::builder().build_ui_state(
        |ui, state: &mut ToastState| {
            Toast::new(state, &Theme::light(), "Invisible", "Dismiss").show(ui);
        },
        ToastState::new(Some(std::time::Duration::ZERO)),
    );
    harness.run();
    assert!(!harness.state().is_visible());
    assert!(harness.query_by_label("Invisible").is_none());
}

/// The toast frame is the only rounded rectangle with a 10-point radius.
fn toast_card_alpha(harness: &Harness<'_, ToastState>) -> Option<u8> {
    harness
        .output()
        .shapes
        .iter()
        .find_map(|clipped| match &clipped.shape {
            egui::Shape::Rect(shape) if shape.corner_radius.nw == 10 => Some(shape.fill.a()),
            _ => None,
        })
}

#[test]
fn toast_fade_in_respects_motion_settings() {
    let theme = Theme::light();
    let mut harness = Harness::builder().build_ui_state(
        |ui, state: &mut ToastState| {
            Toast::new(state, &theme, "Saved", "Dismiss").show(ui);
        },
        ToastState::new(None),
    );
    harness
        .ctx
        .all_styles_mut(|style| style.animation_time = 0.2);
    harness.input_mut().time = Some(0.0);
    harness.step();
    assert_eq!(
        toast_card_alpha(&harness),
        Some(255),
        "the first revealed frame is fully opaque"
    );

    // Hiding the toast resets the reveal state, so restarting it fades in again.
    harness.state_mut().dismiss();
    harness.step();
    harness.state_mut().restart();
    harness.input_mut().time = Some(10.0);
    harness.step();
    let start = toast_card_alpha(&harness).expect("toast card while fading in");
    assert_eq!(start, 0, "the fade starts transparent");
    assert!(
        harness.ctx.has_requested_repaint(),
        "a running fade must request repaints"
    );

    harness.input_mut().time = Some(10.1);
    harness.step();
    let halfway = toast_card_alpha(&harness).expect("toast card while fading in");
    assert!(
        halfway > 0 && halfway < 255,
        "expected a partially transparent toast, got alpha {halfway}"
    );

    harness.input_mut().time = Some(11.0);
    harness.step();
    assert_eq!(
        toast_card_alpha(&harness),
        Some(255),
        "the fade completes at full opacity"
    );

    // Reduced motion and zero animation time snap the toast fully opaque.
    for configure in [0, 1] {
        let mut theme = Theme::light();
        if configure == 1 {
            theme.motion.reduced_motion = true;
        }
        let mut snapped = Harness::builder().build_ui_state(
            move |ui, state: &mut ToastState| {
                Toast::new(state, &theme, "Saved", "Dismiss").show(ui);
            },
            ToastState::new(None),
        );
        snapped
            .ctx
            .all_styles_mut(|style| style.animation_time = 0.2);
        snapped.input_mut().time = Some(0.0);
        snapped.step();
        assert_eq!(
            toast_card_alpha(&snapped),
            Some(255),
            "reduced motion must snap the toast to full opacity"
        );
        assert!(!snapped.ctx.has_requested_repaint());
    }
}

#[test]
fn toast_uses_forui_padding_and_body_typography() {
    let theme = Theme::light();
    let card = theme.card;
    let body = theme.typography.body.size;
    let mut harness = Harness::builder().build_ui(move |ui| {
        let mut state = ToastState::new(None);
        Toast::new(&mut state, &theme, "Saved", "Dismiss")
            .description("Your changes are available.")
            .show(ui);
    });
    harness.run();
    let card_rect = harness
        .output()
        .shapes
        .iter()
        .find_map(|shape| match &shape.shape {
            egui::Shape::Rect(rect) if rect.fill == card && rect.corner_radius.nw == 10 => {
                Some(rect.rect)
            }
            _ => None,
        })
        .expect("toast card");
    let title = harness
        .output()
        .shapes
        .iter()
        .find_map(|shape| match &shape.shape {
            egui::Shape::Text(text) if text.galley.job.text == "Saved" => {
                Some((text.pos.x, text.galley.job.sections[0].format.font_id.size))
            }
            _ => None,
        })
        .expect("toast title");
    // Forui FToastStyle: 16 point horizontal padding and body.sm title size
    // (the 1 point border is drawn inside the frame).
    assert!((title.0 - card_rect.left() - 16.0).abs() <= 1.0);
    assert_eq!(title.1, body);
}

#[test]
fn destructive_variant_paints_the_title_in_the_destructive_colour() {
    let theme = Theme::light();
    let expected = theme.readable_on(theme.destructive, theme.card, 4.5);
    let mut harness = Harness::builder().build_ui_state(
        move |ui, state: &mut ToastState| {
            Toast::new(state, &theme, "Something went wrong", "Dismiss")
                .variant(temivu::ToastVariant::Destructive)
                .show(ui);
        },
        ToastState::new(None),
    );
    harness.run();
    assert!(
        harness
            .output()
            .shapes
            .iter()
            .any(|shape| match &shape.shape {
                egui::Shape::Text(text) => text
                    .galley
                    .job
                    .sections
                    .iter()
                    .any(|section| section.format.color == expected),
                _ => false,
            }),
        "the destructive title should use the contrast-checked destructive colour"
    );
}

#[test]
fn toast_icon_is_rendered_as_a_decorative_glyph() {
    let mut harness = Harness::builder().build_ui_state(
        |ui, state: &mut ToastState| {
            Toast::new(state, &Theme::light(), "Saved", "Dismiss")
                .icon("!")
                .show(ui);
        },
        ToastState::new(None),
    );
    harness.run();
    harness.get_by_role_and_label(egui::accesskit::Role::Alert, "Saved");
    assert!(
        harness.get_by_label("!").accesskit_node().is_hidden(),
        "the approximate icon glyph should be hidden from assistive technology"
    );
}
