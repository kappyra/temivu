use egui_kittest::{Harness, kittest::Queryable};
use temivu::{
    Theme, VideoAction, VideoPlayer, VideoPlayerState, VideoStatus, Volume, volume_control,
};

#[test]
fn volume_button_restores_the_last_audible_value() {
    let mut harness = Harness::builder().build_ui_state(
        |ui, volume| {
            volume_control(ui, &Theme::light(), volume);
        },
        Volume::default(),
    );
    harness.state_mut().set_level(0.43);
    harness.run_steps(3);
    harness.get_by_label("Mute").click();
    harness.run_steps(3);
    assert_eq!(harness.state().level(), 0.0);
    harness.get_by_label("Unmute").click();
    harness.run_steps(3);
    assert_eq!(harness.state().level(), 0.43);
}

#[test]
fn player_emits_play_seek_speed_and_fullscreen_actions() {
    let mut harness = Harness::builder()
        .with_size(egui::vec2(550.0, 670.0))
        .build_ui_state(
            |ui, state: &mut (VideoPlayerState, Vec<VideoAction>)| {
                let status = VideoStatus {
                    position: 0.0,
                    duration: 20.0,
                    ..Default::default()
                };
                VideoPlayer::new(egui::Id::unique("video"), "Clip", &Theme::light(), status)
                    .max_height(220.0)
                    .show(ui, &mut state.0, |action| state.1.push(action));
            },
            (VideoPlayerState::default(), Vec::new()),
        );
    let play = harness.get_by_label("Play").rect();
    let fullscreen = harness.get_by_label("Fullscreen").rect();
    assert!((play.center().y - fullscreen.center().y).abs() < 1.0);
    assert!(fullscreen.right() < 550.0 && fullscreen.left() > play.right());
    harness.get_by_label("Play").click();
    harness.run_steps(3);
    assert!(harness.state().1.contains(&VideoAction::Play));

    harness.get_by_label("Playback position").click();
    harness.run_steps(3);
    assert!(harness.state().1.iter().any(
        |action| matches!(action, VideoAction::Seek(seconds) if (9.0..11.0).contains(seconds))
    ));

    harness.get_by_label("Options").click();
    harness.run_steps(3);
    harness.get_by_label("1.5×").click();
    harness.run_steps(3);
    assert!(harness.state().1.contains(&VideoAction::Speed(1.5)));
    harness.get_by_label("Fullscreen").click();
    harness.run_steps(3);
    assert!(harness.state().1.contains(&VideoAction::ToggleFullscreen));
}

#[test]
fn focused_video_surface_supports_seek_shortcuts() {
    let mut harness = Harness::builder()
        .with_size(egui::vec2(390.0, 600.0))
        .build_ui_state(
            |ui, state: &mut (VideoPlayerState, Vec<VideoAction>)| {
                VideoPlayer::new(
                    egui::Id::unique("keyboard-video"),
                    "Clip",
                    &Theme::light(),
                    VideoStatus {
                        position: 8.0,
                        duration: 20.0,
                        ..Default::default()
                    },
                )
                .max_height(220.0)
                .show(ui, &mut state.0, |action| state.1.push(action));
            },
            (VideoPlayerState::default(), Vec::new()),
        );
    harness.get_by_label("Play Clip").focus();
    harness.run_steps(3);
    harness.key_press(egui::Key::ArrowRight);
    harness.run_steps(3);
    assert!(harness.state().1.contains(&VideoAction::Seek(13.0)));
    assert!(harness.get_by_label("Fullscreen").rect().right() <= 390.0);
}

#[test]
fn default_player_is_monochrome_and_mint_is_opt_in() {
    fn has_tint(shape: &egui::Shape) -> bool {
        let tint = |color: egui::Color32| color.r() != color.g() || color.g() != color.b();
        match shape {
            egui::Shape::Rect(rect) => tint(rect.fill) || tint(rect.stroke.color),
            egui::Shape::Path(path) => tint(path.fill),
            egui::Shape::LineSegment { stroke, .. } => tint(stroke.color),
            egui::Shape::Vec(shapes) => shapes.iter().any(has_tint),
            _ => false,
        }
    }
    for (theme, expected_tint) in [
        (Theme::light(), false),
        (Theme::dark(), false),
        (Theme::mint_light(), true),
    ] {
        let harness = Harness::builder().build_ui_state(
            |ui, state| {
                VideoPlayer::new(
                    egui::Id::unique("palette"),
                    "Clip",
                    &theme,
                    VideoStatus {
                        position: 10.0,
                        duration: 20.0,
                        ..Default::default()
                    },
                )
                .show(ui, state, |_| {});
            },
            VideoPlayerState::default(),
        );
        assert_eq!(
            harness
                .output()
                .shapes
                .iter()
                .any(|entry| has_tint(&entry.shape)),
            expected_tint,
        );
    }
}

#[test]
fn fullscreen_overlays_the_controls_and_restores_them_after_idle() {
    let mut harness = Harness::builder()
        .with_size(egui::vec2(800.0, 450.0))
        .build_ui_state(
            |ui, state: &mut (VideoPlayerState, Vec<VideoAction>)| {
                VideoPlayer::new(
                    egui::Id::unique("fullscreen"),
                    "Clip",
                    &Theme::light(),
                    VideoStatus {
                        duration: 20.0,
                        playing: true,
                        fullscreen: true,
                        ..Default::default()
                    },
                )
                .frame(egui::load::SizedTexture::new(
                    egui::TextureId::User(42),
                    [400.0, 300.0],
                ))
                .show(ui, &mut state.0, |action| state.1.push(action));
            },
            (VideoPlayerState::default(), Vec::new()),
        );
    harness.run_steps(3);
    let viewport = harness
        .output()
        .shapes
        .iter()
        .find_map(|entry| match &entry.shape {
            egui::Shape::Rect(rect)
                if rect.fill == egui::Color32::BLACK && rect.rect.width() > 700.0 =>
            {
                Some(rect.rect)
            }
            _ => None,
        })
        .expect("fullscreen background");
    let image = harness
        .output()
        .shapes
        .iter()
        .find_map(|entry| match &entry.shape {
            egui::Shape::Mesh(mesh) if mesh.texture_id == egui::TextureId::User(42) => {
                Some(mesh.calc_bounds())
            }
            _ => None,
        })
        .expect("video texture");
    assert_eq!(image.center(), viewport.center());
    assert_eq!(image.height(), viewport.height());
    assert!((image.width() / image.height() - 4.0 / 3.0).abs() < 0.001);
    let exit = harness.get_by_label("Exit fullscreen").rect();
    assert!(viewport.contains_rect(exit) && exit.bottom() > 420.0);
    let timeline = harness.get_by_label("Playback position").rect();
    assert!(timeline.width() > 740.0 && timeline.bottom() <= exit.top());

    harness.get_by_label("Pause").click();
    harness.run_steps(3);
    assert_eq!(harness.state().1, [VideoAction::Pause]);
    harness.get_by_label("Playback position").click();
    harness.run_steps(3);
    assert!(harness.state().1.iter().any(
        |action| matches!(action, VideoAction::Seek(seconds) if (9.0..11.0).contains(seconds))
    ));

    harness.get_by_label("Audio").click();
    harness.run_steps(3);
    let volume = harness.get_by_label("Volume").rect();
    assert!(
        volume.width() >= 280.0 && volume.height() >= 44.0,
        "{volume:?}"
    );
    assert!(viewport.contains_rect(volume));
    harness.get_by_label("Volume").click();
    harness.run_steps(3);
    assert!(harness.state().1.iter().any(
        |action| matches!(action, VideoAction::Volume(level) if (0.45..0.55).contains(level))
    ));
    harness.get_by_label("Mute").click();
    harness.run_steps(3);
    assert!(harness.state().1.contains(&VideoAction::Volume(0.0)));
    harness.key_press(egui::Key::Escape);
    harness.run_steps(3);
    assert!(!harness.state().1.contains(&VideoAction::ToggleFullscreen));
    harness.get_by_label("Options").click();
    harness.run_steps(3);
    harness.get_by_label("1.5×").click();
    harness.run_steps(3);
    assert!(harness.state().1.contains(&VideoAction::Speed(1.5)));

    harness.hover_at(viewport.center());
    harness.input_mut().time = Some(20.0);
    harness.step();
    for time in [23.0, 23.5, 24.0] {
        harness.input_mut().time = Some(time);
        harness.step();
    }
    assert!(harness.query_by_label("Playback position").is_none());
    assert!(harness.query_by_label("Exit fullscreen").is_none());

    harness.hover_at(viewport.center() + egui::vec2(1.0, 1.0));
    for time in [24.01, 24.3, 24.6] {
        harness.input_mut().time = Some(time);
        harness.step();
    }
    harness.get_by_label("Playback position");
    harness.state_mut().1.clear();
    harness.get_by_label("Exit fullscreen").click();
    harness.run_steps(3);
    assert_eq!(harness.state().1, [VideoAction::ToggleFullscreen]);
    harness.key_press(egui::Key::Escape);
    harness.run_steps(3);
    assert_eq!(harness.state().1, [VideoAction::ToggleFullscreen; 2]);
}
