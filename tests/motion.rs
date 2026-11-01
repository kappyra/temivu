//! Behaviour tests for the reduced-motion-aware animation helper.
use egui_kittest::Harness;
use temivu::Theme;
use temivu::motion::{animate_bool, animate_f32, reset};

fn animation_id() -> egui::Id {
    egui::Id::new("temivu-motion-test")
}

/// `egui_kittest` disables animation by default; enable a non-zero animation
/// time after building the harness to exercise the animated paths.
fn enable_animations<State>(harness: &Harness<'_, State>) {
    harness
        .ctx
        .all_styles_mut(|style| style.animation_time = 0.2);
}

#[test]
fn bool_animation_advances_then_stops_requesting_repaints() {
    let theme = Theme::light();
    let mut harness = Harness::builder().build_ui_state(
        |ui, state: &mut (bool, f32)| {
            state.1 = animate_bool(ui.ctx(), animation_id(), state.0, &theme);
            ui.label("animated");
        },
        (false, 0.0),
    );
    enable_animations(&harness);
    reset(&harness.ctx, animation_id());
    harness.input_mut().time = Some(0.0);
    harness.step();
    assert_eq!(harness.state().1, 0.0);
    assert!(
        !harness.ctx.has_requested_repaint(),
        "an idle animation must not request repaints"
    );

    // Start the transition. The first frame of a transition evaluates to the
    // previous value and must schedule a repaint.
    harness.state_mut().0 = true;
    harness.input_mut().time = Some(0.0);
    harness.step();
    assert_eq!(harness.state().1, 0.0);
    assert!(
        harness.ctx.has_requested_repaint(),
        "a running animation must request a repaint"
    );

    // Half of the 200 ms default duration.
    harness.input_mut().time = Some(0.1);
    harness.step();
    let halfway = harness.state().1;
    assert!(
        halfway > 0.0 && halfway < 1.0,
        "expected an intermediate value, got {halfway}"
    );
    assert!(harness.ctx.has_requested_repaint());

    harness.input_mut().time = Some(0.2);
    harness.step();
    assert_eq!(harness.state().1, 1.0);

    // Idle frames stay idle. egui repeats a requested repaint once, so the
    // first frame after the animation settles may still report a request; the
    // following frame must not.
    harness.input_mut().time = Some(0.6);
    harness.step();
    assert_eq!(harness.state().1, 1.0);
    assert!(
        !harness.ctx.has_requested_repaint(),
        "an idle animation must not request repaints"
    );

    // Animating back down uses the same machinery.
    harness.state_mut().0 = false;
    harness.input_mut().time = Some(0.6);
    harness.step();
    assert!(harness.ctx.has_requested_repaint());
    harness.input_mut().time = Some(0.7);
    harness.step();
    let descending = harness.state().1;
    assert!(
        descending > 0.0 && descending < 1.0,
        "expected an intermediate descending value, got {descending}"
    );
    harness.input_mut().time = Some(0.9);
    harness.step();
    assert_eq!(harness.state().1, 0.0);
    harness.input_mut().time = Some(1.0);
    harness.step();
    assert_eq!(harness.state().1, 0.0);
    assert!(!harness.ctx.has_requested_repaint());
}

#[test]
fn disabled_animation_time_snaps_to_target() {
    let theme = Theme::light();
    let mut harness = Harness::builder().build_ui_state(
        |ui, state: &mut f32| {
            *state = animate_bool(ui.ctx(), animation_id(), true, &theme);
            ui.label("snapped");
        },
        0.0,
    );
    harness
        .ctx
        .all_styles_mut(|style| style.animation_time = 0.0);
    harness.input_mut().time = Some(0.0);
    harness.step();
    assert_eq!(*harness.state(), 1.0);
    assert!(!harness.ctx.has_requested_repaint());
}

#[test]
fn reduced_motion_and_disabled_settings_snap_to_target() {
    for configure in [
        |theme: &mut Theme| theme.motion.reduced_motion = true,
        |theme: &mut Theme| theme.motion.enabled = false,
    ] {
        let mut theme = Theme::light();
        configure(&mut theme);
        let mut harness = Harness::builder().build_ui_state(
            |ui, state: &mut f32| {
                *state = animate_bool(ui.ctx(), animation_id(), true, &theme);
                ui.label("snapped");
            },
            0.0,
        );
        harness.input_mut().time = Some(0.0);
        harness.step();
        assert_eq!(*harness.state(), 1.0);
        assert!(!harness.ctx.has_requested_repaint());

        // A second frame does not start animating either.
        harness.input_mut().time = Some(0.05);
        harness.step();
        assert_eq!(*harness.state(), 1.0);
        assert!(!harness.ctx.has_requested_repaint());
    }
}

#[test]
fn float_animation_starts_at_initial_and_reset_starts_over() {
    let theme = Theme::light();
    let mut harness = Harness::builder().build_ui_state(
        |ui, state: &mut f32| {
            *state = animate_f32(ui.ctx(), animation_id(), 0.0, 1.0, &theme);
            ui.label("entrance");
        },
        1.0,
    );
    enable_animations(&harness);
    reset(&harness.ctx, animation_id());
    harness.input_mut().time = Some(0.0);
    harness.step();
    assert_eq!(*harness.state(), 0.0);
    assert!(harness.ctx.has_requested_repaint());

    harness.input_mut().time = Some(0.05);
    harness.step();
    let growing = *harness.state();
    assert!(
        growing > 0.0 && growing < 1.0,
        "expected a growing entrance value, got {growing}"
    );

    // Reset discards the stored progress, so the next frame starts at `initial`.
    reset(&harness.ctx, animation_id());
    harness.input_mut().time = Some(5.0);
    harness.step();
    assert_eq!(*harness.state(), 0.0);
    assert!(harness.ctx.has_requested_repaint());
}

#[test]
fn custom_duration_and_easing_are_honoured() {
    let mut theme = Theme::light();
    theme.motion.normal = 0.4;
    theme.motion.easing = temivu::Easing::EaseInOut;
    let mut harness = Harness::builder().build_ui_state(
        |ui, state: &mut (bool, f32)| {
            state.1 = animate_bool(ui.ctx(), animation_id(), state.0, &theme);
            ui.label("custom");
        },
        (true, 1.0),
    );
    enable_animations(&harness);
    reset(&harness.ctx, animation_id());
    harness.input_mut().time = Some(0.0);
    harness.step();
    // A first `animate_bool` call jumps to the target.
    assert_eq!(harness.state().1, 1.0);

    harness.state_mut().0 = false;
    harness.input_mut().time = Some(0.0);
    harness.step();
    harness.input_mut().time = Some(0.2);
    harness.step();
    let halfway = harness.state().1;
    // Ease-in-out at t = 0.5 is 0.5, so the value is halfway between 1 and 0.
    assert!(
        (halfway - 0.5).abs() < 0.01,
        "expected eased halfway value, got {halfway}"
    );
    harness.input_mut().time = Some(0.4);
    harness.step();
    assert_eq!(harness.state().1, 0.0);
}

#[test]
fn non_finite_values_snap_instead_of_panicking() {
    let theme = Theme::light();
    let mut harness = Harness::builder().build_ui_state(
        |ui, state: &mut (f32, f32)| {
            state.1 = animate_f32(ui.ctx(), animation_id(), state.0, f32::NAN, &theme);
            ui.label("non-finite");
        },
        (0.0, 0.0),
    );
    harness.input_mut().time = Some(0.0);
    harness.step();
    assert!(harness.state().1.is_nan());
    assert!(!harness.ctx.has_requested_repaint());
}

#[test]
fn float_animation_supports_values_outside_the_unit_range() {
    let theme = Theme::light();
    let mut harness = Harness::builder().build_ui_state(
        |ui, state: &mut f32| {
            *state = animate_f32(ui.ctx(), animation_id(), 0.0, 10.0, &theme);
            ui.label("range");
        },
        0.0,
    );
    enable_animations(&harness);
    reset(&harness.ctx, animation_id());
    harness.input_mut().time = Some(0.0);
    harness.step();
    assert_eq!(*harness.state(), 0.0);
    harness.input_mut().time = Some(0.1);
    harness.step();
    let halfway = *harness.state();
    assert!(
        (halfway - 5.0).abs() < 0.001,
        "expected an unclamped intermediate value, got {halfway}"
    );
    harness.input_mut().time = Some(1.0);
    harness.step();
    assert_eq!(*harness.state(), 10.0);
}
