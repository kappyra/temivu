use egui::{Color32, Context, Pos2, RawInput, Rect, Shape, TextureId, pos2, vec2};
use temivu::{
    Card, SharedElementDrive as Drive, SharedElementEndpoint as Endpoint, SharedElementFade,
    SharedElementFit, SharedElementPose, SharedElementSide as Side, SharedElementSnapshot,
    SharedElementStyle, SharedElementTransition, Theme,
};

fn setup() -> (Context, Theme, SharedElementTransition, Endpoint, Endpoint) {
    let ctx = Context::default();
    ctx.all_styles_mut(|style| style.animation_time = 0.2);
    let motion = SharedElementTransition::new(egui::Id::unique("story"));
    let source = Endpoint::new(
        Rect::from_min_size(pos2(44.0, 62.0), vec2(84.0, 47.0)),
        16.0,
        Color32::from_gray(30),
    );
    let destination = Endpoint::new(
        Rect::from_min_size(pos2(90.0, 29.0), vec2(266.0, 198.0)),
        30.0,
        Color32::from_gray(60),
    );
    (ctx, Theme::dark(), motion, source, destination)
}

fn frame<T>(
    ctx: &Context,
    t: f64,
    mut paint: impl FnMut(&mut egui::Ui) -> T,
) -> (T, Vec<egui::epaint::ClippedShape>) {
    let mut value = None;
    let mut output = ctx.run_ui(
        RawInput {
            time: Some(t),
            screen_rect: Some(Rect::from_min_size(Pos2::ZERO, vec2(600.0, 500.0))),
            ..Default::default()
        },
        |ui| value = Some(paint(ui)),
    );
    output.textures_delta.clear();
    (value.unwrap(), output.shapes)
}

#[test]
fn reversal_preserves_pose_and_velocity_at_the_event_time() {
    let (ctx, theme, motion, source, destination) = setup();
    let sample = |t, side| {
        frame(&ctx, t, |_| {
            motion
                .sample(&ctx, &theme, source, destination, Drive::SettleTo(side))
                .unwrap()
        })
        .0
    };
    assert_eq!(sample(0.0, Side::Source).rect, source.rect);
    sample(0.01, Side::Destination);
    let moving = sample(0.09, Side::Destination);
    assert!(moving.progress > 0.0 && moving.progress < 1.0 && moving.velocity > 0.0);

    let reversed = sample(0.09, Side::Source);
    assert_eq!(reversed.rect, moving.rect);
    assert_eq!(reversed.corners, moving.corners);
    assert_eq!(reversed.opacity, moving.opacity);
    assert_eq!(reversed.velocity, moving.velocity);
    assert!(sample(0.22, Side::Source).velocity < 0.0);
    let restored = sample(1.2, Side::Source);
    assert_eq!(restored.settled, Some(Side::Source));
    assert_eq!(restored.rect, source.rect);
}

#[test]
fn gesture_can_grab_midflight_and_release_into_a_retargeted_destination() {
    let (ctx, theme, motion, source, destination) = setup();
    let sample = |t, to, drive| {
        frame(&ctx, t, |_| {
            motion.sample(&ctx, &theme, source, to, drive).unwrap()
        })
        .0
    };
    sample(0.01, destination, Drive::SettleTo(Side::Destination));
    let mid = sample(0.09, destination, Drive::SettleTo(Side::Destination));
    let grabbed = sample(
        0.09,
        destination,
        Drive::Follow {
            progress: mid.progress,
            velocity: mid.velocity,
        },
    );
    assert!(grabbed.rect.min.distance(mid.rect.min) < 0.0001);
    assert!(grabbed.rect.max.distance(mid.rect.max) < 0.0001);
    let scrubbed = sample(
        0.12,
        destination,
        Drive::Follow {
            progress: 0.62,
            velocity: 1.13,
        },
    );
    let shifted = Endpoint::new(
        destination.rect.translate(vec2(37.0, 73.0)),
        10.0,
        destination.fill,
    );
    let released = sample(0.12, shifted, Drive::SettleTo(Side::Destination));
    assert_eq!(released.rect, scrubbed.rect);
    assert_eq!(released.corners, scrubbed.corners);
    assert_eq!(released.velocity, 1.13);
    let rest = sample(1.12, shifted, Drive::SettleTo(Side::Destination));
    assert_eq!(rest.rect, shifted.rect);
    assert_eq!(rest.corners.nw, 10.0);
    assert_eq!(rest.settled, Some(Side::Destination));
}

#[test]
fn spring_sampling_depends_on_elapsed_time_and_ids_are_independent() {
    fn simulate(fps: u32) -> SharedElementPose {
        let (ctx, theme, motion, from, to) = setup();
        let other = SharedElementTransition::new(egui::Id::unique("other story"));
        let mut last = None;
        for index in 0..=fps {
            let t = 0.16 * f64::from(index) / f64::from(fps);
            last = Some(
                frame(&ctx, t, |_| {
                    let second = other
                        .sample(
                            &ctx,
                            &theme,
                            from,
                            to,
                            Drive::Follow {
                                progress: 0.7,
                                velocity: 0.0,
                            },
                        )
                        .unwrap();
                    assert_eq!(second.progress, 0.7);
                    motion
                        .sample(&ctx, &theme, from, to, Drive::SettleTo(Side::Destination))
                        .unwrap()
                })
                .0,
            );
        }
        last.unwrap()
    }
    let slow = simulate(6);
    let fast = simulate(24);
    assert_eq!(slow, fast);
}

#[test]
fn invalid_geometry_and_reduced_motion_have_clean_settled_states() {
    let (ctx, mut theme, motion, from, to) = setup();
    theme.motion.reduced_motion = true;
    let (pose, _) = frame(&ctx, 0.0, |_| {
        let invalid = Endpoint::new(Rect::NOTHING, 0.0, Color32::WHITE);
        assert!(
            motion
                .sample(
                    &ctx,
                    &theme,
                    from,
                    invalid,
                    Drive::SettleTo(Side::Destination)
                )
                .is_none()
        );
        motion
            .sample(&ctx, &theme, from, to, Drive::SettleTo(Side::Destination))
            .unwrap()
    });
    assert_eq!(pose.rect, to.rect);
    assert_eq!(pose.opacity, [0.0, 1.0]);
    assert_eq!(pose.velocity, 0.0);
    assert_eq!(pose.settled, Some(Side::Destination));
    let shifted = Endpoint::new(to.rect.translate(vec2(12.0, 0.0)), 22.0, to.fill);
    theme.motion.reduced_motion = false;
    let (resized, _) = frame(&ctx, 0.1, |_| {
        motion
            .sample(
                &ctx,
                &theme,
                from,
                shifted,
                Drive::SettleTo(Side::Destination),
            )
            .unwrap()
    });
    assert_eq!(resized.rect, shifted.rect);
    assert_eq!(resized.settled, Some(Side::Destination));
}

#[test]
fn shared_image_is_painted_once_and_clipped_to_the_rounded_mask() {
    let (ctx, theme, motion, from, to) = setup();
    let texture = TextureId::User(12);
    let motion = motion.style(SharedElementStyle {
        fade: SharedElementFade::Through,
        fit: SharedElementFit::Cover,
        scrim: Color32::from_black_alpha(20),
        ..Default::default()
    });
    let (pose, shapes) = frame(&ctx, 0.1, |ui| {
        let bounds = Rect::from_min_size(Pos2::ZERO, vec2(90.0, 53.0));
        let image = SharedElementSnapshot::from_shapes(
            ui.ctx(),
            bounds,
            [Shape::image(
                texture,
                bounds,
                Rect::from_min_max(Pos2::ZERO, pos2(1.0, 1.0)),
                Color32::WHITE,
            )],
        );
        let pose = motion
            .sample(
                &ctx,
                &theme,
                from,
                to,
                Drive::Follow {
                    progress: 0.45,
                    velocity: 0.0,
                },
            )
            .unwrap();
        assert_eq!(pose.opacity, [0.0, 0.0]);
        let clip = Rect::from_min_max(pos2(50.0, 0.0), pos2(500.0, 270.0));
        motion.paint(
            &ui.painter().with_clip_rect(clip),
            pose,
            [Some(&image), Some(&image)],
        );
        pose
    });
    let pieces: Vec<_> = shapes
        .iter()
        .filter_map(|shape| match &shape.shape {
            Shape::Mesh(mesh) if mesh.texture_id == texture => Some((shape.clip_rect, mesh)),
            _ => None,
        })
        .collect();
    assert_eq!(pieces.len(), 1);
    let (clip, mesh) = pieces[0];
    assert!(mesh.is_valid());
    assert!(pose.rect.expand(0.001).contains_rect(clip));
    let corner_test = [
        (pose.rect.left_top(), vec2(1.0, 1.0), pose.corners.nw),
        (pose.rect.right_top(), vec2(-1.0, 1.0), pose.corners.ne),
        (pose.rect.right_bottom(), vec2(-1.0, -1.0), pose.corners.se),
        (pose.rect.left_bottom(), vec2(1.0, -1.0), pose.corners.sw),
    ];
    for v in &mesh.vertices {
        assert!(pose.rect.expand(0.001).contains(v.pos));
        assert!(v.uv.x >= -0.001 && v.uv.x <= 1.001 && v.uv.y >= -0.001 && v.uv.y <= 1.001);
        assert_eq!(v.color, Color32::WHITE);
        for (corner, dir, r) in corner_test {
            let local = (v.pos - corner) * dir;
            if local.x < r && local.y < r {
                assert!((local - vec2(r, r)).length() <= r + 0.001);
            }
        }
    }
}

#[test]
fn measured_card_can_supply_both_its_bounds_and_cached_paint() {
    let (ctx, theme, motion, _, destination) = setup();
    let (bounds, shapes) = frame(&ctx, 0.0, |ui| {
        let (card, snapshot) = SharedElementSnapshot::capture(ui, |ui| {
            Card::new(&theme).show(ui, |ui, _| ui.label("Live card"))
        });
        let source = Endpoint::from_response(&card.response, 16.0, theme.card);
        let pose = motion
            .sample(
                &ctx,
                &theme,
                source,
                destination,
                Drive::Follow {
                    progress: 0.2,
                    velocity: 0.0,
                },
            )
            .unwrap();
        motion.paint(ui.painter(), pose, [Some(&snapshot), None]);
        source.rect
    });
    assert!(bounds.is_positive());
    assert!(
        shapes
            .iter()
            .any(|entry| matches!(entry.shape, Shape::Mesh(_)))
    );
}
