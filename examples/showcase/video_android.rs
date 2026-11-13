use egui::{Context, Id, Ui};
use temivu::{Theme, VideoPlayer, VideoPlayerState, VideoStatus};

#[derive(Default)]
pub struct DemoVideo {
    player: VideoPlayerState,
}

impl DemoVideo {
    pub fn visible(&mut self, _visible: bool, _ctx: &Context) {}

    pub fn show(&mut self, ui: &mut Ui, theme: &Theme) {
        VideoPlayer::new(
            Id::unique("demo-video"),
            "Video preview",
            theme,
            VideoStatus::default(),
        )
        .max_height(240.0)
        .show(ui, &mut self.player, |_| {});
        ui.label("Local video playback is available in the Linux showcase.");
    }
}
