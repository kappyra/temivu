use egui::{ColorImage, Context, Id, TextureHandle, TextureOptions, Ui, ViewportCommand};
use std::{
    fs,
    io::{self, BufReader, Read, Write},
    path::{Path, PathBuf},
    process::{Child, Command, Stdio},
    sync::{
        Arc, Mutex,
        mpsc::{self, SyncSender},
    },
    task::Poll,
    thread::{self, JoinHandle},
    time::Duration,
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
    fn start(path: &Path, ctx: &Context) -> io::Result<Self> {
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

struct FilePicker(Child);

impl FilePicker {
    fn open() -> io::Result<Self> {
        Command::new("/usr/bin/zenity")
            .args(["--file-selection", "--title=Choose a video"])
            .stdin(Stdio::null())
            .stdout(Stdio::piped())
            .spawn()
            .map(Self)
    }

    fn selection(&mut self, ctx: &Context) -> io::Result<Poll<Option<PathBuf>>> {
        let Some(status) = self.0.try_wait()? else {
            ctx.request_repaint_after(Duration::from_millis(80));
            return Ok(Poll::Pending);
        };
        if !status.success() {
            return if status.code() == Some(1) {
                Ok(Poll::Ready(None))
            } else {
                Err(io::Error::other("The file picker could not open."))
            };
        }
        let mut path = String::new();
        self.0
            .stdout
            .take()
            .expect("piped file selection")
            .read_to_string(&mut path)?;
        Ok(Poll::Ready(Some(PathBuf::from(
            path.strip_suffix('\n').unwrap_or(&path),
        ))))
    }
}

impl Drop for FilePicker {
    fn drop(&mut self) {
        let _ = self.0.kill();
        let _ = self.0.wait();
    }
}

fn typed_path(input: &str) -> PathBuf {
    if Path::new(input).is_file() {
        return PathBuf::from(input);
    }
    let input = input.trim();
    let input = input
        .strip_prefix('"')
        .and_then(|path| path.strip_suffix('"'))
        .or_else(|| {
            input
                .strip_prefix('\'')
                .and_then(|path| path.strip_suffix('\''))
        })
        .unwrap_or(input);
    if let Some(relative) = input.strip_prefix("~/")
        && let Some(home) = std::env::home_dir()
    {
        return home.join(relative);
    }
    PathBuf::from(input)
}

#[derive(Default)]
pub struct DemoVideo {
    path: String,
    autoload: bool,
    picker: Option<FilePicker>,
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
                    ctx.request_repaint_after(Duration::from_millis(16))
                }
                Err(mpsc::TrySendError::Disconnected(_)) => self.dirty = false,
            }
        }
    }

    fn load(&mut self, path: &Path, ctx: &Context) {
        let path = match fs::canonicalize(path) {
            Ok(path) if path.is_file() => path,
            Ok(_) => {
                self.message = Some("Choose a video file, not a folder.".to_owned());
                return;
            }
            Err(error) => {
                self.message = Some(format!("Could not open that path. {error}"));
                return;
            }
        };
        match Decoder::start(&path, ctx) {
            Ok(decoder) => self.decoder = Some(decoder),
            Err(error) => {
                self.message = Some(format!("Could not start the video decoder. {error}"));
                return;
            }
        }
        self.texture = None;
        self.player = VideoPlayerState::default();
        self.status = VideoStatus::default();
        self.command = CommandState::default();
        self.dirty = false;
        self.at_end = false;
        self.message = None;
    }

    fn source_controls(&mut self, ui: &mut Ui, theme: &Theme) {
        if let Some(picker) = &mut self.picker {
            let selection = picker.selection(ui.ctx());
            if !matches!(selection, Ok(Poll::Pending)) {
                self.picker = None;
            }
            match selection {
                Ok(Poll::Ready(Some(path))) => {
                    self.path = path.to_string_lossy().into_owned();
                    self.load(&path, ui.ctx());
                }
                Ok(Poll::Pending | Poll::Ready(None)) => {}
                Err(error) => self.message = Some(error.to_string()),
            }
        }
        let path = TextField::new(&mut self.path, theme, "Local video path")
            .id(Id::new("demo-video-path"))
            .hint("/path/to/video.mp4")
            .show(ui)
            .response;
        let submitted = path.lost_focus() && ui.input(|input| input.key_pressed(egui::Key::Enter));
        ui.horizontal_wrapped(|ui| {
            if Button::new("Choose video…", theme)
                .enabled(self.picker.is_none())
                .show(ui)
                .clicked()
            {
                match FilePicker::open() {
                    Ok(picker) => {
                        self.picker = Some(picker);
                        ui.ctx().request_repaint();
                    }
                    Err(_) => {
                        self.message = Some(
                            "The file picker requires Zenity. Enter a local path to open a video."
                                .to_owned(),
                        )
                    }
                }
            }
            let open = Button::new("Open path", theme)
                .variant(ButtonVariant::Outline)
                .enabled(self.picker.is_none() && !self.path.trim().is_empty())
                .show(ui);
            if (open.clicked() || submitted) && open.enabled() {
                self.load(&typed_path(&self.path), ui.ctx());
            }
        });
        ui.add_space(8.0);
    }

    pub fn show(&mut self, ui: &mut Ui, theme: &Theme) {
        if self.autoload {
            self.autoload = false;
            self.load(&typed_path(&self.path), ui.ctx());
        }
        let fullscreen = ui.input(|input| input.viewport().fullscreen.unwrap_or(false));
        if !fullscreen {
            self.source_controls(ui, theme);
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
        if !fullscreen {
            if let Some(message) = &self.message {
                ui.label(egui::RichText::new(message).color(theme.error));
            } else if self.decoder.is_some() && self.texture.is_none() {
                ui.label("Loading video…");
            }
        }
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
    }
}
