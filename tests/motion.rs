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
fn interaction_spring_retargets_and_respects_reduced_motion() {
    let mut harness = Harness::builder().build_ui_state(
        |ui, state: &mut (bool, f32, bool)| {
            let mut theme = Theme::mint_light();
            theme.motion.reduced_motion = state.2;
            state.1 = temivu::motion::spring_bool(ui.ctx(), animation_id(), state.0, &theme);
        },
        (false, 0.0, false),
    );
    enable_animations(&harness);
    harness.input_mut().time = Some(0.0);
    harness.step();
    harness.state_mut().0 = true;
    harness.input_mut().time = Some(0.05);
    harness.step();
    assert_eq!(
        harness.state().1,
        0.0,
        "the new target starts at the input event"
    );
    harness.input_mut().time = Some(0.075);
    harness.step();
    assert!(harness.state().1 > 0.0 && harness.state().1 < 1.0);
    harness.state_mut().0 = false;
    harness.input_mut().time = Some(0.1);
    harness.step();
    assert!(harness.state().1.is_finite());
    harness.state_mut().2 = true;
    harness.step();
    assert_eq!(harness.state().1, 0.0);
}

#[test]
fn changing_spring_speed_preserves_the_pose_at_the_retarget_event() {
    let mut harness = Harness::builder().build_ui_state(
        |ui, state: &mut (bool, bool, [f32; 2])| {
            let mut theme = Theme::light();
            let reference = if state.0 { 1.0 } else { 0.0 };
            let target = if state.1 { 0.0 } else { reference };
            state.2[0] =
                temivu::motion::spring_f32(ui.ctx(), animation_id(), 0.0, reference, &theme);
            if state.1 {
                theme.motion.normal *= 0.75;
            }
            state.2[1] = temivu::motion::spring_f32(
                ui.ctx(),
                animation_id().with("retarget"),
                0.0,
                target,
                &theme,
            );
        },
        (false, false, [0.0; 2]),
    );
    enable_animations(&harness);
    for (time, open, close) in [(0.0, false, false), (0.05, true, false), (0.08, true, true)] {
        harness.state_mut().0 = open;
        harness.state_mut().1 = close;
        harness.input_mut().time = Some(time);
        harness.step();
    }
    let [reference, retarget] = harness.state().2;
    assert!(reference > 0.0);
    assert!((reference - retarget).abs() < 0.0001);
    harness.input_mut().time = Some(0.083);
    harness.step();
    assert!(
        harness.state().2[1] > retarget,
        "the spring carries its velocity through reversal"
    );
    harness.input_mut().time = Some(0.2);
    harness.step();
    assert!(harness.state().2[1] < retarget);
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

#[test]
fn spring_settles_after_a_long_frame_instead_of_stretching_time() {
    let theme = Theme::light();
    let mut harness = Harness::builder().build_ui_state(
        |ui, value: &mut f32| {
            *value = temivu::motion::spring_f32(ui.ctx(), animation_id(), 0.0, 1.0, &theme);
        },
        0.0,
    );
    enable_animations(&harness);
    harness.input_mut().time = Some(0.0);
    harness.step();
    harness.input_mut().time = Some(2.0);
    harness.step();
    assert_eq!(*harness.state(), 1.0);
    harness.input_mut().time = Some(2.1);
    harness.step();
    assert!(!harness.ctx.has_requested_repaint());
}

#[test]
fn spring_does_not_skip_an_interaction_after_idle() {
    let theme = Theme::light();
    let mut harness = Harness::builder().build_ui_state(
        |ui, state: &mut (bool, f32)| {
            state.1 = temivu::motion::spring_bool(ui.ctx(), animation_id(), state.0, &theme);
        },
        (false, 0.0),
    );
    enable_animations(&harness);
    harness.input_mut().time = Some(0.0);
    harness.step();
    harness.state_mut().0 = true;
    harness.input_mut().time = Some(30.0);
    harness.step();
    assert_eq!(
        harness.state().1,
        0.0,
        "idle time must not count toward the new interaction"
    );
    harness.input_mut().time = Some(30.04);
    harness.step();
    assert!(harness.state().1 > 0.0 && harness.state().1 < 1.0);
}

#[test]
fn spring_pose_is_independent_of_frame_spacing() {
    fn sample(times: &[f64]) -> f32 {
        let ctx = egui::Context::default();
        let theme = Theme::light();
        let mut value = 0.0;
        for &time in times {
            let mut output = ctx.run_ui(
                egui::RawInput {
                    time: Some(time),
                    ..Default::default()
                },
                |ui| {
                    value = temivu::motion::spring_f32(ui.ctx(), animation_id(), 0.0, 1.0, &theme);
                },
            );
            output.textures_delta.clear();
        }
        value
    }
    let regular: Vec<_> = (0..=12).map(|frame| frame as f64 / 60.0).collect();
    assert!((sample(&regular) - sample(&[0.0, 0.013, 0.057, 0.14, 0.2])).abs() < 0.0001);
}

#[test]
fn size_spring_synchronizes_axes_and_reverses_without_jumping() {
    use egui::vec2;
    use temivu::motion::spring_size;

    let mut harness = Harness::builder().build_ui_state(
        |ui, state: &mut (egui::Vec2, egui::Vec2, bool)| {
            let mut theme = Theme::light();
            theme.motion.reduced_motion = state.2;
            state.1 = spring_size(ui.ctx(), animation_id(), state.0, &theme);
        },
        (vec2(100.0, 50.0), egui::Vec2::ZERO, false),
    );
    enable_animations(&harness);
    for (time, target) in [(0.0, vec2(100.0, 50.0)), (0.02, vec2(200.0, 100.0))] {
        harness.state_mut().0 = target;
        harness.input_mut().time = Some(time);
        harness.step();
        assert_eq!(harness.state().1, vec2(100.0, 50.0));
    }
    harness.input_mut().time = Some(0.06);
    harness.step();
    let growing = harness.state().1;
    assert!(growing.x > 100.0 && growing.x < 200.0);
    assert!((growing.x - 2.0 * growing.y).abs() < 0.001);

    harness.state_mut().0 = vec2(80.0, 40.0);
    harness.input_mut().time = Some(0.06);
    harness.step();
    assert_eq!(harness.state().1, growing);
    harness.input_mut().time = Some(0.2);
    harness.step();
    assert!(harness.state().1.x < growing.x);
    harness.state_mut().2 = true;
    harness.step();
    assert_eq!(harness.state().1, vec2(80.0, 40.0));
}
