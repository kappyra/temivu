use egui::{Color32, Context, Frame, Rect, Shape, TextureId, pos2, vec2};
use std::sync::Arc;
use temivu::{Dialog, Glass, GlassBackdrop, GlassMode, GlassRegion, GlassRenderer, Theme};
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
        Shape::Vec(shapes) => shapes.iter().find_map(region),
        _ => None,
    }
}

#[test]
fn glass_lens_uses_a_bounded_mesh_and_warps_backdrop_coordinates() {
    let ctx = Context::default();
    let bounds = Rect::from_min_size(pos2(0.0, 0.0), vec2(256.0, 128.0));
    let rect = Rect::from_min_size(pos2(40.0, 24.0), vec2(170.0, 70.0));
    let texture = TextureId::User(1);
    for mode in [temivu::GlassMode::Frost, temivu::GlassMode::Prism] {
        let mut shapes = ctx.run_ui(Default::default(), |ui| {
            let painter = ui.painter();
            Glass::new(0.0).mode(mode).paint(
                painter,
                &Theme::light(),
                rect,
                Frame::new().fill(Color32::WHITE).corner_radius(28),
                Some(GlassBackdrop { texture, bounds }),
            );
        });
        shapes.textures_delta.clear();
        let lens = shapes
            .shapes
            .iter()
            .find_map(|entry| match &entry.shape {
                Shape::Mesh(mesh) if mesh.texture_id == texture => Some(mesh),
                _ => None,
            })
            .expect("the lens samples the supplied texture");
        assert!(lens.is_valid());
        assert_eq!(lens.vertices.len(), 144);
        assert!(lens.vertices.iter().all(|vertex| {
            rect.expand(0.001).contains(vertex.pos)
                && (0.0..=1.0).contains(&vertex.uv.x)
                && (0.0..=1.0).contains(&vertex.uv.y)
        }));
        let warped = lens.vertices.iter().any(|vertex| {
            let undistorted = (vertex.pos - bounds.min) / bounds.size();
            vertex.uv.distance(undistorted.to_pos2()) > 0.02
        });
        assert_eq!(warped, mode == temivu::GlassMode::Prism);
    }
}

#[test]
fn live_glass_adds_a_paint_callback_without_moving_the_dialog_content() {
    use egui_kittest::{Harness, kittest::Queryable};

    let mut harness = Harness::builder().build_ui_state(
        |ui, glass: &mut bool| {
            let theme = Theme::light();
            Glass::set_context(
                ui.ctx(),
                glass.then(|| Glass::default().mode(GlassMode::Frost)),
            );
            let mut open = true;
            Dialog::new(egui::Id::new("glass-dialog"), "Glass dialog")
                .theme(&theme)
                .show(ui.ctx(), &mut open, |ui| ui.label("Stable content"));
        },
        false,
    );
    Glass::set_renderer(&harness.ctx, Some(Arc::new(TestRenderer)));
    harness.run();
    let original = harness.get_by_label("Stable content").rect();
    assert!(
        !harness
            .output()
            .shapes
            .iter()
            .any(|entry| region(&entry.shape).is_some())
    );

    *harness.state_mut() = true;
    harness.run();
    let backdrop = harness
        .output()
        .shapes
        .iter()
        .find_map(|entry| region(&entry.shape))
        .unwrap();
    assert_eq!(backdrop.mode, GlassMode::Frost);
    assert!(backdrop.rect.contains_rect(original));
    assert_eq!(harness.get_by_label("Stable content").rect(), original);
}

#[test]
fn droplet_retains_a_solid_surface_until_the_filtered_backdrop_is_ready() {
    fn fill_alpha(shape: &Shape) -> Option<u8> {
        let color = match shape {
            Shape::Rect(rect) => rect.fill,
            Shape::Path(path) => path.fill,
            Shape::Vec(shapes) => return shapes.iter().find_map(fill_alpha),
            _ => return None,
        };
        let [r, g, b, a] = color.to_srgba_unmultiplied();
        (r > 180 && g < 40 && b < 40).then_some(a)
    }

    let ctx = Context::default();
    let theme = Theme::light();
    theme.apply(&ctx);
    ctx.all_styles_mut(|style| style.animation_time = 0.2);
    Glass::set_context(&ctx, Some(Glass::new(0.2)));
    Glass::set_renderer(&ctx, Some(Arc::new(TestRenderer)));
    let frame = Frame::new()
        .fill(Color32::from_rgb(210, 12, 20))
        .inner_margin(16)
        .corner_radius(24);
    let render = |time, mut open| {
        let mut output = ctx.run_ui(
            egui::RawInput {
                time: Some(time),
                screen_rect: Some(Rect::from_min_size(pos2(0.0, 0.0), vec2(480.0, 420.0))),
                ..Default::default()
            },
            |ui| {
                Dialog::new(egui::Id::new("solid-droplet"), "Example")
                    .theme(&theme)
                    .source(Rect::from_center_size(pos2(230.0, 210.0), vec2(44.0, 44.0)))
                    .frame(frame)
                    .show(ui.ctx(), &mut open, |ui| {
                        ui.set_width(160.0);
                        ui.label("Apply");
                    });
            },
        );
        output.textures_delta.clear();
        (
            output
                .shapes
                .iter()
                .find_map(|entry| fill_alpha(&entry.shape)),
            output
                .shapes
                .iter()
                .find_map(|entry| region(&entry.shape))
                .map(|glass| glass.strength),
        )
    };

    render(0.0, false);
    render(0.01, true);
    assert_eq!(render(0.10, true), (Some(255), None));
    let (settled, backdrop) = render(1.0, true);
    assert!(settled.is_some_and(|alpha| alpha < 220));
    assert_eq!(backdrop, Some(1.0));

    render(1.01, false);
    assert_eq!(render(1.10, false), (Some(255), None));
    assert_eq!(render(2.0, false), (None, None));

    render(2.01, true);
    render(3.0, true);
    Glass::set_renderer(&ctx, None);
    assert_eq!(render(3.02, true), (Some(255), None));
}
