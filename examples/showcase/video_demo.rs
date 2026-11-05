use egui::{ColorImage, Context, Id, TextureHandle, TextureOptions, Ui, ViewportCommand};
use std::{
    fs,
    io::{self, BufReader, Read, Write},
    path::PathBuf,
    process::{Child, Command, Stdio},
    sync::{
        Arc, Mutex,
        mpsc::{self, SyncSender},
    },
    thread::{self, JoinHandle},
};
use temivu::{
    Button, ButtonVariant, TextField, Theme, VideoAction, VideoPlayer, VideoPlayerState,
    VideoStatus,
};

#[derive(Clone, Copy)]
struct CommandState {
    playing: bool,
    volume: f64,
    speed: f64,
    seek: Option<f64>,
}

impl Default for CommandState {
    fn default() -> Self {
        Self {
            playing: false,
            volume: 0.75,
            speed: 1.0,
            seek: None,
        }
    }
}

#[derive(Default)]
struct Inbox {
    frame: Option<ColorImage>,
    position: f64,
    duration: f64,
    ended: bool,
    failed: bool,
}

struct Decoder {
    process: Child,
    input: Option<SyncSender<CommandState>>,
    reader: Option<JoinHandle<()>>,
    writer: Option<JoinHandle<()>>,
    inbox: Arc<Mutex<Inbox>>,
}

impl Decoder {
    fn start(path: PathBuf, ctx: &Context) -> io::Result<Self> {
        let path = fs::canonicalize(path)?;
        if !fs::metadata(&path)?.is_file() {
            return Err(io::Error::new(
                io::ErrorKind::InvalidInput,
                "Choose a local video file.",
            ));
        }
        let mut process = Command::new("/usr/bin/python3")
            .args(["-I", "-c", include_str!("video_backend.py")])
            .arg(path)
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .spawn()?;
        let stdout = process.stdout.take().expect("piped video output");
        let mut stdin = process.stdin.take().expect("piped video input");
        let (input, commands) = mpsc::sync_channel::<CommandState>(8);
        let writer = thread::spawn(move || {
            for command in commands {
                if writeln!(
                    stdin,
                    "{} {} {} {}",
                    u8::from(command.playing),
                    command.volume,
                    command.speed,
                    command.seek.unwrap_or(-1.0)
                )
                .and_then(|()| stdin.flush())
                .is_err()
                {
                    break;
                }
            }
        });
        let inbox = Arc::new(Mutex::new(Inbox::default()));
        let mailbox = inbox.clone();
        let ctx = ctx.clone();
        let reader = thread::spawn(move || {
            let mut stdout = BufReader::new(stdout);
            while let Ok(status) = read_packet(&mut stdout, &mailbox) {
                ctx.request_repaint();
                if status == 2 {
                    return;
                }
            }
            if let Ok(mut data) = mailbox.lock() {
                data.failed = true;
            }
            ctx.request_repaint();
        });
        Ok(Self {
            process,
            input: Some(input),
            reader: Some(reader),
            writer: Some(writer),
            inbox,
        })
    }
}

impl Drop for Decoder {
    fn drop(&mut self) {
        self.input.take();
        let _ = self.process.kill();
        let _ = self.process.wait();
        if let Some(reader) = self.reader.take() {
            let _ = reader.join();
        }
        if let Some(writer) = self.writer.take() {
            let _ = writer.join();
        }
    }
}

fn read_packet(stream: &mut impl Read, mailbox: &Mutex<Inbox>) -> io::Result<u8> {
    let mut header = [0; 25];
    stream.read_exact(&mut header)?;
    let word = |i| u32::from_be_bytes(header[i..i + 4].try_into().expect("four bytes")) as usize;
    let seconds = |i| f64::from_be_bytes(header[i..i + 8].try_into().expect("eight bytes"));
    let (width, height) = (word(0), word(4));
    if width > 720 || height > 720 || (width == 0) != (height == 0) || header[24] > 2 {
        return Err(io::Error::new(
            io::ErrorKind::InvalidData,
            "Invalid video packet",
        ));
    }
    let frame = if width != 0 {
        let mut rgba = vec![0; width * height * 4];
        stream.read_exact(&mut rgba)?;
        Some(ColorImage::from_rgba_unmultiplied([width, height], &rgba))
    } else {
        None
    };
    let mut data = mailbox
        .lock()
        .map_err(|_| io::Error::other("Video mailbox closed"))?;
    if frame.is_some() {
        data.frame = frame;
    }
    data.position = seconds(8);
    data.duration = seconds(16);
    data.ended |= header[24] == 1;
    data.failed |= header[24] == 2;
    Ok(header[24])
}

#[derive(Default)]
pub struct DemoVideo {
    path: String,
    autoload: bool,
    decoder: Option<Decoder>,
    texture: Option<TextureHandle>,
    player: VideoPlayerState,
    status: VideoStatus,
    command: CommandState,
    dirty: bool,
    at_end: bool,
    message: Option<String>,
}

impl DemoVideo {
    pub fn with_path(path: Option<String>) -> Self {
        Self {
            autoload: path.is_some(),
            path: path.unwrap_or_default(),
            ..Self::default()
        }
    }

    pub fn visible(&mut self, visible: bool, ctx: &Context) {
        if !visible && self.command.playing {
            self.command.playing = false;
            self.status.playing = false;
            self.dirty = true;
        }
        self.flush(ctx);
    }

    fn flush(&mut self, ctx: &Context) {
        if self.dirty
            && let Some(decoder) = &self.decoder
        {
            let input = decoder.input.as_ref().expect("live decoder input");
            match input.try_send(self.command) {
                Ok(()) => {
                    self.dirty = false;
                    self.command.seek = None;
                }
                Err(mpsc::TrySendError::Full(_)) => {
                    ctx.request_repaint_after(std::time::Duration::from_millis(16))
                }
                Err(mpsc::TrySendError::Disconnected(_)) => self.dirty = false,
            }
        }
    }

    fn load(&mut self, ctx: &Context) {
        self.decoder = None;
        self.texture = None;
        self.player = VideoPlayerState::default();
        self.status = VideoStatus::default();
        self.command = CommandState::default();
        self.dirty = false;
        self.at_end = false;
        match Decoder::start(PathBuf::from(self.path.trim()), ctx) {
            Ok(decoder) => {
                self.decoder = Some(decoder);
                self.message = None;
            }
            Err(error) => self.message = Some(error.to_string()),
        }
    }

    pub fn show(&mut self, ui: &mut Ui, theme: &Theme) {
        if self.autoload {
            self.autoload = false;
            self.load(ui.ctx());
        }
        if let Some(decoder) = &self.decoder {
            let (frame, position, duration, ended, failed) = {
                let mut data = decoder.inbox.lock().expect("video mailbox");
                (
                    data.frame.take(),
                    data.position,
                    data.duration,
                    std::mem::take(&mut data.ended),
                    data.failed,
                )
            };
            if let Some(frame) = frame {
                if let Some(texture) = &mut self.texture {
                    texture.set(frame, TextureOptions::LINEAR);
                } else {
                    self.texture = Some(ui.ctx().load_texture(
                        "temivu-preview-video",
                        frame,
                        TextureOptions::LINEAR,
                    ));
                }
            }
            self.status.position = position;
            self.status.duration = duration;
            self.at_end |= ended;
            if ended || failed {
                self.status.playing = false;
                self.command.playing = false;
            }
            if failed {
                self.status.duration = 0.0;
                self.message = Some(
                    "Video unavailable. Check the file and the Linux GStreamer installation."
                        .to_owned(),
                );
            }
        }
        let fullscreen = ui.input(|input| input.viewport().fullscreen.unwrap_or(false));
        self.status.fullscreen = fullscreen;
        let viewport_height = ui.ctx().content_rect().height();
        let height = (viewport_height * 0.43).clamp(160.0, 440.0);
        let mut view = VideoPlayer::new(Id::new("demo-video"), "Video preview", theme, self.status)
            .max_height(height);
        if let Some(texture) = &self.texture {
            view = view.frame(egui::load::SizedTexture::from_handle(texture));
        }
        let ctx = ui.ctx().clone();
        view.show(ui, &mut self.player, |action| {
            match action {
                VideoAction::Play => {
                    if self.at_end || self.status.position >= self.status.duration - 0.05 {
                        self.command.seek = Some(0.0);
                        self.at_end = false;
                    }
                    self.command.playing = true;
                    self.status.playing = true;
                }
                VideoAction::Pause => {
                    self.command.playing = false;
                    self.status.playing = false;
                }
                VideoAction::Seek(position) => {
                    self.at_end = false;
                    self.command.seek = Some(position);
                    self.status.position = position;
                }
                VideoAction::Volume(level) => self.command.volume = level,
                VideoAction::Speed(speed) => self.command.speed = speed,
                VideoAction::ToggleFullscreen => {
                    ctx.send_viewport_cmd(ViewportCommand::Fullscreen(!fullscreen));
                    return;
                }
            }
            self.dirty = true;
        });
        self.flush(ui.ctx());
        if fullscreen {
            return;
        }
        if let Some(message) = &self.message {
            ui.label(egui::RichText::new(message).color(theme.error));
        }
        ui.add_space(8.0);
        egui::CollapsingHeader::new("Local video source")
            .default_open(self.decoder.is_none())
            .show(ui, |ui| {
                TextField::new(&mut self.path, theme, "Local video")
                    .hint("/path/to/video.mp4")
                    .max_chars(4096)
                    .show(ui);
                if Button::new("Open video", theme)
                    .variant(ButtonVariant::Outline)
                    .enabled(!self.path.trim().is_empty())
                    .show(ui)
                    .clicked()
                {
                    self.load(ui.ctx());
                }
            });
    }
}
