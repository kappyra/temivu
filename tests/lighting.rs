use egui::{Event, Modifiers, PointerButton, Pos2, Rect, Sense, Shape, vec2};
use egui_kittest::{Harness, kittest::Queryable};
use temivu::{Button, LightEffects, LightTracking, Switch, Theme, Vesper};

fn light_center(harness: &Harness<'_, impl Sized>) -> Option<f32> {
    fn accumulate(shape: &Shape, alpha: &mut f32, weighted_x: &mut f32) {
        match shape {
            Shape::Mesh(mesh) => {
                assert!(mesh.is_valid());
                for vertex in &mesh.vertices {
                    let weight = f32::from(vertex.color.a());
                    *alpha += weight;
                    *weighted_x += vertex.pos.x * weight;
                }
            }
            Shape::Vec(shapes) => {
                for shape in shapes {
                    accumulate(shape, alpha, weighted_x);
                }
            }
            _ => {}
        }
    }

    let mut alpha = 0.0;
    let mut weighted_x = 0.0;
    for entry in &harness.output().shapes {
        accumulate(&entry.shape, &mut alpha, &mut weighted_x);
    }
    (alpha > 0.0).then(|| weighted_x / alpha)
}

fn step(harness: &mut Harness<'_, impl Sized>, time: f64, events: impl IntoIterator<Item = Event>) {
    harness.input_mut().time = Some(time);
    harness.input_mut().events.extend(events);
    harness.step();
}

fn primary(pos: Pos2, pressed: bool) -> Event {
    Event::PointerButton {
        pos,
        button: PointerButton::Primary,
        pressed,
        modifiers: Modifiers::NONE,
    }
}

fn touch(pos: Pos2, phase: egui::TouchPhase) -> Event {
    Event::Touch {
        device_id: egui::TouchDeviceId(0),
        id: egui::TouchId(1),
        phase,
        pos,
        force: None,
    }
}

#[test]
fn lighting_is_opt_in_and_widget_overrides_take_precedence() {
    let mut harness = Harness::builder().build_ui_state(
        |ui, state: &mut (Option<Vesper>, bool)| {
            let theme = Theme::light();
            Vesper::set_context(ui.ctx(), state.0);
            Button::new("Inherited", &theme).show(ui);
            Button::new("Off", &theme).lighting(None).show(ui);
            Switch::new(&mut state.1, &theme)
                .label("Local")
                .lighting(Some(Vesper::subtle()))
                .show(ui);
        },
        (None, false),
    );
    let inherited = harness.get_by_label("Inherited").rect().center();
    let off = harness.get_by_label("Off").rect().center();
    let local = harness.get_by_label("Local").rect().center();

    step(&mut harness, 1.0, [Event::PointerMoved(inherited)]);
    assert_eq!(Vesper::context(&harness.ctx), None);
    assert!(light_center(&harness).is_none());

    harness.state_mut().0 = Some(Vesper::subtle());
    step(&mut harness, 2.0, [Event::PointerMoved(inherited)]);
    assert_eq!(Vesper::context(&harness.ctx), Some(Vesper::subtle()));
    assert!(light_center(&harness).is_some());
    step(&mut harness, 3.1, [Event::PointerMoved(off)]);
    assert!(light_center(&harness).is_none());

    harness.state_mut().0 = None;
    step(&mut harness, 4.2, [Event::PointerMoved(local)]);
    assert_eq!(Vesper::context(&harness.ctx), None);
    assert!(light_center(&harness).is_some());
}

#[test]
fn direct_light_tracks_input_and_fades_after_release() {
    let mut harness = Harness::builder().build_ui_state(
        |ui, rect| {
            let theme = Theme::light();
            let (bounds, response) =
                ui.allocate_exact_size(vec2(210.0, 54.0), Sense::click_and_drag());
            *rect = bounds;
            response.surrender_focus();
            Vesper {
                tracking: LightTracking::Direct,
                effects: LightEffects {
                    follow: true,
                    ..LightEffects::default()
                },
                ..Vesper::subtle()
            }
            .paint(ui.painter(), &theme, &response, bounds, 27.into());
        },
        Rect::NOTHING,
    );
    harness
        .ctx
        .all_styles_mut(|style| style.animation_time = 0.2);
    let rect = *harness.state();
    let left = Pos2::new(rect.left() + 25.0, rect.center().y);
    let right = Pos2::new(rect.right() - 25.0, rect.center().y);

    step(
        &mut harness,
        1.0,
        [Event::PointerMoved(left), primary(left, true)],
    );
    assert!(light_center(&harness).expect("light at press") < rect.center().x);
    step(&mut harness, 1.04, [Event::PointerMoved(right)]);
    assert!(light_center(&harness).expect("light follows drag") > rect.center().x);

    step(
        &mut harness,
        1.08,
        [primary(right, false), Event::PointerGone],
    );
    assert!(
        light_center(&harness).is_some(),
        "release keeps a short tail"
    );
    for frame in 1..=8 {
        step(&mut harness, 1.08 + f64::from(frame) * 0.05, []);
    }
    assert!(light_center(&harness).is_none());
}

#[test]
fn a_held_contact_stays_lit_across_the_touch_long_press_and_on_passive_surfaces() {
    for (sense, touchscreen) in [
        (Sense::click(), true),
        (Sense::hover(), true),
        (Sense::hover(), false),
    ] {
        let mut harness = Harness::builder().build_ui_state(
            |ui, rect| {
                let theme = Theme::light();
                let (bounds, response) = ui.allocate_exact_size(vec2(210.0, 54.0), sense);
                *rect = bounds;
                Vesper::expressive().paint(ui.painter(), &theme, &response, bounds, 27.into());
            },
            Rect::NOTHING,
        );
        harness
            .ctx
            .all_styles_mut(|style| style.animation_time = 0.2);
        let rect = *harness.state();
        let left = Pos2::new(rect.left() + 26.0, rect.center().y);
        let right = Pos2::new(rect.right() - 26.0, rect.center().y);
        let mut events = vec![Event::PointerMoved(left), primary(left, true)];
        if touchscreen {
            events.push(touch(left, egui::TouchPhase::Start));
        }
        step(&mut harness, 1.0, events);
        for time in [1.15, 1.38, 1.62, 1.78, 2.0, 2.18, 2.4] {
            step(&mut harness, time, []);
        }
        let label = format!("sense={sense:?}, touchscreen={touchscreen}");
        assert!(
            light_center(&harness).is_some_and(|x| x < rect.center().x),
            "{label}"
        );

        let mut events = vec![Event::PointerMoved(right)];
        if touchscreen {
            events.push(touch(right, egui::TouchPhase::Move));
        }
        step(&mut harness, 2.43, events);
        assert!(
            light_center(&harness).is_some_and(|x| x > rect.center().x),
            "{label}"
        );

        let mut events = vec![primary(right, false), Event::PointerGone];
        if touchscreen {
            events.push(touch(right, egui::TouchPhase::End));
        }
        step(&mut harness, 2.47, events);
        for frame in 1..=10 {
            step(&mut harness, 2.47 + f64::from(frame) * 0.05, []);
        }
        assert!(light_center(&harness).is_none(), "{label}");
    }
}

#[test]
fn reduced_motion_keeps_the_steady_light_and_suppresses_the_pulse() {
    let mut harness = Harness::builder().build_ui_state(
        |ui, state: &mut (Rect, bool, bool)| {
            let mut theme = Theme::light();
            theme.motion.reduced_motion = state.1;
            let (bounds, response) = ui.allocate_exact_size(vec2(210.0, 54.0), Sense::click());
            state.0 = bounds;
            Vesper {
                effects: LightEffects {
                    scatter: true,
                    follow: state.2,
                    ..LightEffects::default()
                },
                ..Vesper::subtle()
            }
            .paint(ui.painter(), &theme, &response, bounds, 27.into());
        },
        (Rect::NOTHING, true, false),
    );
    harness
        .ctx
        .all_styles_mut(|style| style.animation_time = 0.2);
    let center = harness.state().0.center();
    step(
        &mut harness,
        1.0,
        [Event::PointerMoved(center), primary(center, true)],
    );
    assert!(light_center(&harness).is_none(), "the pulse is suppressed");

    harness.state_mut().2 = true;
    step(&mut harness, 1.08, []);
    assert!(
        light_center(&harness).is_some(),
        "steady feedback is visible"
    );
    step(&mut harness, 1.12, [primary(center, false)]);

    harness.state_mut().1 = false;
    harness.state_mut().2 = false;
    step(&mut harness, 1.2, [primary(center, true)]);
    assert!(
        light_center(&harness).is_some(),
        "a normal press shows the pulse"
    );
}

#[test]
fn pointer_focus_does_not_keep_the_contact_glow_after_release() {
    let mut harness = Harness::builder().build_ui_state(
        |ui, rect| {
            let theme = Theme::light().touch();
            let (bounds, response) = ui.allocate_exact_size(vec2(160.0, 54.0), Sense::click());
            *rect = bounds;
            if response.clicked() {
                response.request_focus();
            }
            Vesper::subtle().paint(ui.painter(), &theme, &response, bounds, 26.into());
        },
        Rect::NOTHING,
    );
    harness
        .ctx
        .all_styles_mut(|style| style.animation_time = 0.2);
    let point = harness.state().center();
    step(
        &mut harness,
        1.0,
        [Event::PointerMoved(point), primary(point, true)],
    );
    assert!(light_center(&harness).is_some());
    step(&mut harness, 1.03, [primary(point, false)]);
    step(&mut harness, 1.06, [Event::PointerGone]);
    assert!(harness.ctx.memory(|memory| memory.focused().is_some()));
    for frame in 1..=10 {
        step(&mut harness, 1.06 + f64::from(frame) * 0.05, []);
    }
    assert!(light_center(&harness).is_none());
}
