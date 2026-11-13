use std::sync::Arc;

use egui::{Color32, Context, Frame, Margin, Rect, Sense, Shape, vec2};
use egui_kittest::{Harness, kittest::Queryable};
use temivu::{
    EffectSurface, Glass, GlassProfile, GlassRegion, GlassRenderer, LightSpill, Theme, Vesper,
};

struct TestRenderer;

impl GlassRenderer for TestRenderer {
    fn callback(&self, region: GlassRegion) -> egui::PaintCallback {
        egui::PaintCallback {
            rect: region.rect,
            callback: Arc::new(region),
        }
    }
}

fn region(shape: &Shape) -> Option<&GlassRegion> {
    match shape {
        Shape::Callback(callback) => callback.callback.downcast_ref(),
        _ => None,
    }
}

#[test]
fn effects_follow_measured_frame_and_leave_child_clicks_intact() {
    let frame = Frame::new()
        .fill(Color32::from_gray(35))
        .corner_radius(16)
        .inner_margin(18)
        .outer_margin(6);
    let mut harness = Harness::builder().build_ui_state(
        |ui, (child_clicks, surface_clicks, surface_rect)| {
            let response = EffectSurface::new(&Theme::dark())
                .id_salt("interactive-surface")
                .frame(frame)
                .glass(Some(Glass::default()))
                .glass_profile(GlassProfile::Dock)
                .lighting(Some(Vesper::subtle()))
                .sense(Sense::click())
                .show(ui, |ui| ui.button("Child action"));
            *surface_rect = response.response.rect;
            *child_clicks += usize::from(response.inner.clicked());
            *surface_clicks += usize::from(response.response.clicked());
        },
        (0, 0, Rect::NOTHING),
    );
    Glass::set_renderer(&harness.ctx, Some(Arc::new(TestRenderer)));
    harness.run_steps(3);

    let child = harness.get_by_label("Child action");
    harness.event(egui::Event::PointerMoved(child.rect().center()));
    harness.run_steps(3);
    let shapes = &harness.output().shapes;
    let (backdrop_index, backdrop) = shapes
        .iter()
        .enumerate()
        .find_map(|(index, entry)| region(&entry.shape).map(|region| (index, region)))
        .expect("a live backdrop");
    assert_eq!(backdrop.rect, harness.state().2 - Margin::same(6));
    assert_eq!(backdrop.profile, GlassProfile::Dock);
    let light_index = shapes
        .iter()
        .position(|entry| matches!(entry.shape, Shape::Mesh(_)))
        .expect("lighting under the hovered child");
    let foreground_index = shapes
        .iter()
        .position(|entry| matches!(&entry.shape, Shape::Text(text) if text.galley.job.text == "Child action"))
        .expect("child foreground");
    assert!(backdrop_index < light_index && light_index < foreground_index);

    harness.get_by_label("Child action").click();
    harness.run_steps(3);
    assert_eq!((harness.state().0, harness.state().1), (1, 0));

    let padding = harness.state().2.min + vec2(13.0, 13.0);
    harness.hover_at(padding);
    harness.drag_at(padding);
    harness.run_steps(1);
    harness.drop_at(padding);
    harness.run_steps(3);
    assert_eq!((harness.state().0, harness.state().1), (1, 1));
}

#[test]
fn default_disabled_and_clipped_surfaces_skip_effects() {
    let ctx = Context::default();
    Glass::set_renderer(&ctx, Some(Arc::new(TestRenderer)));
    Glass::set_context(&ctx, Some(Glass::default()));
    Vesper::set_context(&ctx, Some(Vesper::subtle()));
    let mut output = ctx.run_ui(Default::default(), |ui| {
        let theme = Theme::light();
        EffectSurface::new(&theme).show(ui, |ui| ui.label("Plain"));
        ui.add_enabled_ui(false, |ui| {
            EffectSurface::new(&theme)
                .glass(Some(Glass::default()))
                .lighting(Some(Vesper::subtle()))
                .show(ui, |ui| ui.label("Disabled"));
        });
        ui.scope(|ui| {
            ui.set_clip_rect(Rect::from_min_size(
                ui.next_widget_position(),
                vec2(0.0, 0.0),
            ));
            EffectSurface::new(&theme)
                .glass(Some(Glass::default()))
                .lighting(Some(Vesper::subtle()))
                .show(ui, |ui| ui.label("Clipped"));
        });
    });
    output.textures_delta.clear();
    assert!(
        !output
            .shapes
            .iter()
            .any(|entry| region(&entry.shape).is_some() || matches!(entry.shape, Shape::Mesh(_)))
    );
}

#[test]
fn spill_reaches_the_wrapped_frame_behind_its_child() {
    let mut harness = Harness::builder().build_ui_state(
        |ui, (source, receiver)| {
            let theme = Theme::dark();
            let style = Vesper {
                spill: Some(LightSpill::default()),
                ..Vesper::subtle()
            };
            ui.horizontal(|ui| {
                let (rect, response) = ui.allocate_exact_size(vec2(52.0, 42.0), Sense::hover());
                *source = rect;
                style.paint(ui.painter(), &theme, &response, rect, 12.into());
                *receiver = EffectSurface::new(&theme)
                    .lighting(Some(style))
                    .show(ui, |ui| ui.label("Spill receiver"))
                    .response
                    .rect;
            });
        },
        (Rect::NOTHING, Rect::NOTHING),
    );
    harness.run_steps(2);
    harness.hover_at(harness.state().0.center());
    harness.run_steps(3);

    let shapes = &harness.output().shapes;
    let text = shapes
        .iter()
        .position(|entry| {
            matches!(&entry.shape, Shape::Text(text) if text.galley.job.text == "Spill receiver")
        })
        .expect("receiver foreground");
    let receiver = harness.state().1.expand(0.01);
    assert!(shapes[..text].iter().any(|entry| {
        matches!(&entry.shape, Shape::Mesh(mesh) if receiver.contains_rect(mesh.calc_bounds()))
    }));
}

#[test]
fn floating_surface_travels_and_disables_its_children_during_exit() {
    let ctx = Context::default();
    let mut theme = Theme::light();
    let run = |time, open, theme: &Theme| {
        let mut result = None;
        let mut output = ctx.run_ui(
            egui::RawInput {
                screen_rect: Some(Rect::from_min_size(egui::Pos2::ZERO, vec2(390.0, 680.0))),
                time: Some(time),
                ..Default::default()
            },
            |ui| {
                result = temivu::FloatingMotion::flight(vec2(0.0, 52.0)).show(
                    ui.ctx(),
                    egui::Id::unique("floating-preview-test"),
                    (egui::Align2::LEFT_TOP, vec2(16.0, 20.0)),
                    open,
                    EffectSurface::new(theme),
                    |ui| ui.button("Action").enabled(),
                );
            },
        );
        output.textures_delta.clear();
        result
    };
    assert!(run(0.0, false, &theme).is_none());
    run(0.1, true, &theme);
    run(0.13, true, &theme);
    let traveling = run(0.15, true, &theme).expect("animated panel");
    let settled = run(1.0, true, &theme).expect("settled panel");
    assert!(traveling.response.rect.top() > settled.response.rect.top() + 2.0);
    assert!(settled.inner);
    assert!(!run(1.01, false, &theme).expect("exit frame").inner);
    theme.motion.reduced_motion = true;
    assert!(run(1.02, false, &theme).is_none());
}
