//! A backend-independent video surface and media controls. The host supplies
//! decoded textures, reports playback status, and applies returned actions.
use crate::{PopoverMenu, SelectMenuTile, Slider, Theme, motion};
use egui::{Align2, Color32, Id, Pos2, Rect, Response, Stroke, Ui, Vec2, vec2};

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Volume {
    level: f64,
    muted: bool,
}

impl Default for Volume {
    fn default() -> Self {
        Self {
            level: 0.75,
            muted: false,
        }
    }
}

impl Volume {
    /// Set a normalized level. The last nonzero level survives muting.
    pub fn set_level(&mut self, level: f64) {
        if level.is_finite() {
            if level > 0.0 {
                self.level = level.min(1.0);
                self.muted = false;
            } else {
                self.muted = true;
            }
        }
    }

    pub fn level(self) -> f64 {
        if self.muted { 0.0 } else { self.level }
    }

    pub fn toggle_mute(&mut self) {
        self.muted = !self.muted;
    }
}

/// A volume slider with a mute button and a percentage readout. Returns true
/// when the caller should apply the new volume.
pub fn volume_control(ui: &mut Ui, theme: &Theme, volume: &mut Volume) -> bool {
    let before = *volume;
    ui.horizontal(|ui| {
        volume_button(ui, theme, volume);
        // Reserve room for three digits plus the percent sign.
        let width = (ui.available_width() - 48.0).clamp(1.0, 260.0);
        ui.scope(|ui| {
            ui.set_max_width(width);
            volume_slider(ui, theme, volume);
        });
        ui.label(volume_readout(theme, *volume));
    });
    before != *volume
}

fn volume_readout(theme: &Theme, volume: Volume) -> egui::RichText {
    egui::RichText::new(format!("{:.0}%", volume.level() * 100.0))
        .monospace()
        .color(theme.foreground)
}

fn volume_slider(ui: &mut Ui, theme: &Theme, volume: &mut Volume) -> Response {
    let actual = volume.level();
    let motion_id = ui.next_auto_id().with("volume-travel");
    let mut level = f64::from(
        motion::spring_f32(ui.ctx(), motion_id, actual as f32, actual as f32, theme)
            .clamp(0.0, 1.0),
    );
    let response = Slider::new(&mut level, theme)
        .expect("normalized volume")
        .label("Volume")
        .show_label(false)
        .filled(true)
        .show(ui);
    if response.changed() {
        volume.set_level(level);
        motion::reset(ui.ctx(), motion_id);
    }
    ui.ctx()
        .accesskit_node_builder(response.id, |node| node.set_numeric_value(volume.level()));
    response
}

fn volume_button(ui: &mut Ui, theme: &Theme, volume: &mut Volume) -> Response {
    let muted = volume.level() == 0.0;
    let ctx = ui.ctx().clone();
    let response = media_button(
        ui,
        theme,
        if muted { "Unmute" } else { "Mute" },
        |painter, c, ink, id| {
            let audible = motion::spring_bool(&ctx, id.with("volume-waves"), !muted, theme);
            speaker_icon(painter, c, ink, audible);
        },
    );
    if response.clicked() {
        volume.toggle_mute();
    }
    response
}

fn speaker_icon(painter: &egui::Painter, c: Pos2, ink: Color32, audible: f32) {
    let points = [
        (-8.0, -3.0),
        (-4.0, -3.0),
        (1.0, -7.0),
        (1.0, 7.0),
        (-4.0, 3.0),
        (-8.0, 3.0),
        (-8.0, -3.0),
    ]
    .map(|(x, y)| c + vec2(x, y));
    painter.add(egui::Shape::line(points.to_vec(), Stroke::new(1.5, ink)));
    let amount = audible.clamp(0.0, 1.0);
    for radius in [4.0, 7.0] {
        let points = (0..=8)
            .map(|i| {
                let angle = (i as f32 / 8.0 - 0.5) * 1.8;
                c + vec2(2.5, 0.0) + Vec2::angled(angle) * radius * (0.5 + 0.5 * amount)
            })
            .collect();
        painter.add(egui::Shape::line(
            points,
            Stroke::new(1.4, ink.gamma_multiply(amount)),
        ));
    }
    for d in [-1.0, 1.0] {
        painter.line_segment(
            [c + vec2(4.0, -2.5 * d), c + vec2(9.0, 2.5 * d)],
            Stroke::new(1.5, ink.gamma_multiply(1.0 - amount)),
        );
    }
}

fn media_button(
    ui: &mut Ui,
    theme: &Theme,
    label: &str,
    icon: impl FnOnce(&egui::Painter, Pos2, Color32, Id),
) -> Response {
    let size = if theme.touch { 48.0 } else { 42.0 };
    let response = crate::motion_controls::button_response(ui, Vec2::splat(size));
    response.widget_info(|| {
        egui::WidgetInfo::labeled(egui::WidgetType::Button, response.enabled(), label)
    });
    let press = crate::motion_controls::press_amount(&response, theme);
    let (rect, ink) = crate::motion_controls::paint_pill(ui, theme, &response, press, 0.0);
    icon(ui.painter(), rect.center(), ink, response.id);
    response
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum VideoAction {
    Play,
    Pause,
    Seek(f64),
    Volume(f64),
    Speed(f64),
    ToggleFullscreen,
}

/// Latest playback state from the host. Time values are in seconds.
#[derive(Clone, Copy, Debug, Default)]
pub struct VideoStatus {
    pub position: f64,
    pub duration: f64,
    pub playing: bool,
    pub fullscreen: bool,
}

/// UI preferences and interaction state for one player.
pub struct VideoPlayerState {
    pub volume: Volume,
    speed: f64,
    options_open: bool,
    audio_open: bool,
    scrub: Option<f64>,
    fullscreen: Option<FullscreenControls>,
}

struct FullscreenControls {
    visible_until: f64,
    keyboard: bool,
}

impl Default for VideoPlayerState {
    fn default() -> Self {
        Self {
            volume: Volume::default(),
            speed: 1.0,
            options_open: false,
            audio_open: false,
            scrub: None,
            fullscreen: None,
        }
    }
}

/// Video viewport and controls. No decoder or platform-specific window API
/// is required by this widget. The host handles the emitted actions.
pub struct VideoPlayer<'a> {
    id: Id,
    title: &'a str,
    theme: &'a Theme,
    status: VideoStatus,
    frame: Option<egui::load::SizedTexture>,
    height: f32,
}

impl<'a> VideoPlayer<'a> {
    pub fn new(id: Id, title: &'a str, theme: &'a Theme, status: VideoStatus) -> Self {
        Self {
            id,
            title,
            theme,
            status,
            frame: None,
            height: 440.0,
        }
    }

    pub fn frame(mut self, frame: egui::load::SizedTexture) -> Self {
        self.frame = Some(frame);
        self
    }

    pub fn max_height(mut self, height: f32) -> Self {
        if height.is_finite() {
            self.height = height.max(100.0);
        }
        self
    }

    pub fn show(
        self,
        ui: &mut Ui,
        state: &mut VideoPlayerState,
        mut action: impl FnMut(VideoAction),
    ) {
        let mut chrome = self.theme.clone();
        chrome.dark = true;
        chrome.card = Color32::from_gray(22);
        chrome.muted = Color32::from_gray(33);
        chrome.secondary = Color32::from_gray(51);
        chrome.border = Color32::from_gray(72);
        chrome.foreground = Color32::from_gray(248);
        chrome.secondary_foreground = chrome.foreground;
        chrome.muted_foreground = Color32::from_gray(183);
        let [r, g, b, _] = self.theme.primary.to_srgba_unmultiplied();
        chrome.primary = if r == g && g == b {
            Color32::from_gray(235)
        } else {
            self.theme.readable_on(self.theme.primary, chrome.card, 4.5)
        };
        chrome.primary_foreground = chrome.readable_on(Color32::BLACK, chrome.primary, 4.5);
        let chrome = &chrome;
        if self.status.fullscreen {
            self.show_fullscreen(ui, state, chrome, &mut action);
            return;
        }
        if state.fullscreen.take().is_some() {
            state.audio_open = false;
            state.options_open = false;
            state.scrub = None;
        }
        ui.push_id(self.id, |ui| {
            egui::Frame::new()
                .fill(chrome.card)
                .stroke(Stroke::new(1.0, chrome.border))
                .corner_radius(24)
                .inner_margin(8)
                .show(ui, |ui| {
                    ui.visuals_mut().override_text_color = Some(chrome.foreground);
                    let (_, duration) = self.playback_time();
                    let width = ui.available_width().max(1.0);
                    let height = (width * 0.58).clamp(100.0, self.height);
                    let (surface, response) =
                        ui.allocate_exact_size(vec2(width, height), egui::Sense::click());
                    let play_label = if self.status.playing { "Pause" } else { "Play" };
                    response.widget_info(|| {
                        egui::WidgetInfo::labeled(
                            egui::WidgetType::Button,
                            response.enabled(),
                            format!("{play_label} {}", self.title),
                        )
                    });
                    let painter = ui.painter();
                    painter.rect_filled(surface, 18, Color32::from_gray(10));
                    if !self.paint_frame(painter, surface.shrink(6.0)) {
                        painter.text(
                            surface.center() + vec2(0.0, 40.0),
                            Align2::CENTER_CENTER,
                            self.title,
                            egui::FontId::proportional(14.0),
                            Color32::LIGHT_GRAY,
                        );
                    }
                    let affordance = motion::animate_bool(
                        ui.ctx(),
                        self.id.with("play-affordance"),
                        !self.status.playing || response.has_focus(),
                        chrome,
                    );
                    if affordance > 0.001 {
                        let center = surface.center();
                        let press = crate::motion_controls::press_amount(&response, chrome);
                        let size = 54.0 * motion::press_scale(press);
                        let surface = Rect::from_center_size(center, Vec2::splat(size));
                        painter.add(motion::press_shape(
                            egui::epaint::RectShape::new(
                                surface,
                                27,
                                chrome.card.gamma_multiply(affordance * 0.95),
                                Stroke::new(
                                    1.0,
                                    chrome.muted_foreground.gamma_multiply(affordance * 0.5),
                                ),
                                egui::StrokeKind::Inside,
                            ),
                            press,
                        ));
                        play_icon(
                            painter,
                            center,
                            Color32::WHITE.gamma_multiply(affordance),
                            self.status.playing,
                        );
                    }
                    if response.has_focus() {
                        painter.rect_stroke(
                            surface,
                            18,
                            Stroke::new(2.0, chrome.primary),
                            egui::StrokeKind::Inside,
                        );
                    }
                    if response.clicked() && duration > 0.0 {
                        action(self.play_action());
                    }
                    if response.has_focus() && duration > 0.0 && !ui.ctx().any_popup_open() {
                        ui.memory_mut(|memory| {
                            memory.set_focus_lock_filter(
                                response.id,
                                egui::EventFilter {
                                    horizontal_arrows: true,
                                    vertical_arrows: true,
                                    ..Default::default()
                                },
                            )
                        });
                        self.playback_shortcuts(ui, state, &mut action);
                        if consume_key(ui, egui::Key::F) {
                            action(VideoAction::ToggleFullscreen);
                        }
                    }
                    self.show_controls(ui, state, chrome, &mut action);
                });
        });
    }

    fn playback_time(&self) -> (f64, f64) {
        let duration = if self.status.duration.is_finite() {
            self.status.duration.max(0.0)
        } else {
            0.0
        };
        let position = if self.status.position.is_finite() {
            self.status.position.clamp(0.0, duration)
        } else {
            0.0
        };
        (position, duration)
    }

    fn play_action(&self) -> VideoAction {
        if self.status.playing {
            VideoAction::Pause
        } else {
            VideoAction::Play
        }
    }

    fn playback_shortcuts(
        &self,
        ui: &Ui,
        state: &mut VideoPlayerState,
        action: &mut impl FnMut(VideoAction),
    ) {
        let (position, duration) = self.playback_time();
        let key = |key| consume_key(ui, key);
        let seek = if key(egui::Key::ArrowLeft) {
            -5.0
        } else if key(egui::Key::ArrowRight) {
            5.0
        } else {
            0.0
        };
        if seek != 0.0 && duration > 0.0 {
            action(VideoAction::Seek((position + seek).clamp(0.0, duration)));
        }
        let volume = if key(egui::Key::ArrowDown) {
            -0.05
        } else if key(egui::Key::ArrowUp) {
            0.05
        } else {
            0.0
        };
        if volume != 0.0 {
            state
                .volume
                .set_level((state.volume.level() + volume).clamp(0.0, 1.0));
            action(VideoAction::Volume(state.volume.level()));
        } else if key(egui::Key::M) {
            state.volume.toggle_mute();
            action(VideoAction::Volume(state.volume.level()));
        }
    }

    fn show_controls(
        &self,
        ui: &mut Ui,
        state: &mut VideoPlayerState,
        chrome: &Theme,
        action: &mut impl FnMut(VideoAction),
    ) {
        let (position, duration) = self.playback_time();
        let width_hint = ui.available_width();
        let play_label = if self.status.playing { "Pause" } else { "Play" };
        let mut seek = state.scrub.unwrap_or(if duration > 0.0 {
            position / duration
        } else {
            0.0
        });
        let timeline = Slider::new(&mut seek, chrome)
            .expect("normalized timeline")
            .label("Playback position")
            .show_label(false)
            .filled(true)
            .enabled(duration > 0.0)
            .show(ui);
        if timeline.dragged() {
            state.scrub = Some(seek);
        } else if timeline.drag_stopped() || timeline.changed() {
            state.scrub = None;
            action(VideoAction::Seek(seek * duration));
        } else if !timeline.is_pointer_button_down_on() {
            state.scrub = None;
        }
        let mut toggle_play = false;
        let mut transport = |ui: &mut Ui| {
            ui.spacing_mut().item_spacing.x = 6.0;
            toggle_play = ui
                .add_enabled_ui(duration > 0.0, |ui| {
                    media_button(ui, chrome, play_label, |p, c, ink, _| {
                        play_icon(p, c, ink, self.status.playing)
                    })
                })
                .inner
                .clicked();
            ui.label(
                egui::RichText::new(format!(
                    "{} / {}",
                    time_label(seek * duration),
                    time_label(duration)
                ))
                .monospace()
                .color(chrome.foreground),
            );
        };
        let mut settings = |ui: &mut Ui| {
            ui.spacing_mut().item_spacing.x = 6.0;
            let volume_before = state.volume;
            let audio = media_button(ui, chrome, "Audio", |p, c, ink, _| {
                speaker_icon(
                    p,
                    c,
                    ink,
                    if volume_before.level() > 0.0 {
                        1.0
                    } else {
                        0.0
                    },
                );
            })
            .on_hover_text("Audio");
            if audio.clicked() {
                state.audio_open = !state.audio_open;
            }
            let panel_width = (ui.ctx().content_rect().width() - 48.0).clamp(1.0, 304.0);
            crate::Popover::new(self.id.with("audio"), "Audio", &mut state.audio_open)
                .theme(chrome)
                .width(panel_width)
                .focus_on_open(true)
                .show(&audio, |ui| {
                    // Give the panel a measured width even when the popup's
                    // sizing pass starts with the narrow audio button.
                    ui.set_width(panel_width);
                    ui.horizontal(|ui| {
                        volume_button(ui, chrome, &mut state.volume);
                        ui.label(
                            egui::RichText::new("Audio volume")
                                .strong()
                                .color(chrome.foreground),
                        );
                        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                            ui.label(volume_readout(chrome, state.volume).strong());
                        });
                    });
                    ui.add_space(4.0);
                    ui.spacing_mut().interact_size.y = 44.0;
                    volume_slider(ui, chrome, &mut state.volume)
                });
            if state.volume != volume_before {
                action(VideoAction::Volume(state.volume.level()));
            }
            let options = media_button(ui, chrome, "Options", |p, c, ink, _| {
                for x in [-5.0, 0.0, 5.0] {
                    p.circle_filled(c + vec2(x, 0.0), 1.5, ink);
                }
            })
            .on_hover_text("Playback options");
            if options.clicked() {
                state.options_open = !state.options_open;
            }
            PopoverMenu::new(
                self.id.with("options"),
                "Video options",
                &mut state.options_open,
            )
            .theme(chrome)
            .width(184.0)
            .show(&options, |ui, menu| {
                ui.label(
                    egui::RichText::new("Playback speed")
                        .strong()
                        .color(chrome.foreground),
                );
                for speed in [0.25, 0.5, 0.75, 1.0, 1.25, 1.5, 1.75, 2.0] {
                    let label = format!("{speed}×");
                    if menu
                        .select_menu_tile(
                            ui,
                            SelectMenuTile::new(&label, chrome)
                                .id(self.id.with(("speed", (speed * 100.0) as u32)))
                                .selected(state.speed == speed),
                        )
                        .clicked()
                    {
                        state.speed = speed;
                        action(VideoAction::Speed(speed));
                        ui.close();
                    }
                }
            });
            let label = if self.status.fullscreen {
                "Exit fullscreen"
            } else {
                "Fullscreen"
            };
            if media_button(ui, chrome, label, |p, c, ink, _| {
                let stroke = Stroke::new(1.6, ink);
                for (x, y) in [(-1.0, -1.0), (1.0, -1.0), (1.0, 1.0), (-1.0, 1.0)] {
                    let direction = vec2(x, y);
                    let (distance, reach) = if self.status.fullscreen {
                        (3.0, 4.0)
                    } else {
                        (7.0, -4.0)
                    };
                    let corner = c + direction * distance;
                    p.line_segment([corner, corner + vec2(x * reach, 0.0)], stroke);
                    p.line_segment([corner, corner + vec2(0.0, y * reach)], stroke);
                }
            })
            .clicked()
            {
                action(VideoAction::ToggleFullscreen);
            }
        };
        if width_hint >= if self.status.fullscreen { 336.0 } else { 450.0 } {
            let side = if chrome.touch { 48.0 } else { 42.0 };
            egui::Sides::new().height(side).show(ui, transport, |ui| {
                ui.allocate_ui_with_layout(
                    vec2(3.0 * side + 12.0, side),
                    egui::Layout::left_to_right(egui::Align::Center),
                    settings,
                );
            });
        } else {
            ui.horizontal_wrapped(|ui| {
                transport(ui);
                settings(ui);
            });
        }
        if toggle_play {
            action(self.play_action());
        }
    }

    fn paint_frame(&self, painter: &egui::Painter, area: Rect) -> bool {
        let Some(frame) = self
            .frame
            .filter(|frame| frame.size.is_finite() && frame.size.x > 0.0 && frame.size.y > 0.0)
        else {
            return false;
        };
        let scale = (area.width() / frame.size.x).min(area.height() / frame.size.y);
        if scale > 0.0 {
            painter.image(
                frame.id,
                Rect::from_center_size(area.center(), frame.size * scale),
                Rect::from_min_max(Pos2::ZERO, Pos2::new(1.0, 1.0)),
                Color32::WHITE,
            );
        }
        true
    }

    fn show_fullscreen(
        &self,
        ui: &mut Ui,
        state: &mut VideoPlayerState,
        chrome: &Theme,
        action: &mut impl FnMut(VideoAction),
    ) {
        const HIDE_DELAY: f64 = 2.5;
        let surface = ui.available_rect_before_wrap();
        let response = ui.interact(
            surface,
            self.id.with("fullscreen-video"),
            egui::Sense::click(),
        );
        let (_, duration) = self.playback_time();
        let painter = ui.painter_at(surface);
        painter.rect_filled(surface, 0, Color32::BLACK);
        self.paint_frame(&painter, surface);

        let (now, moved, over_controls, touch, pressed, tab, typing) = ui.input(|input| {
            let touch = input.any_touches()
                || input
                    .events
                    .iter()
                    .any(|event| matches!(event, egui::Event::Touch { .. }));
            let mouse = input
                .pointer
                .hover_pos()
                .filter(|point| surface.contains(*point));
            (
                input.time,
                !touch
                    && mouse.is_some()
                    && input
                        .events
                        .iter()
                        .any(|event| matches!(event, egui::Event::PointerMoved(_))),
                !touch && mouse.is_some_and(|point| point.y > surface.bottom() - 102.0),
                touch,
                input.pointer.any_pressed(),
                input.key_pressed(egui::Key::Tab),
                input
                    .events
                    .iter()
                    .any(|event| matches!(event, egui::Event::Key { pressed: true, .. })),
            )
        });
        let controls = state.fullscreen.get_or_insert(FullscreenControls {
            visible_until: now + HIDE_DELAY,
            keyboard: false,
        });
        let was_visible = !self.status.playing || now < controls.visible_until;
        let menu_open = state.audio_open || state.options_open || ui.ctx().any_popup_open();
        if moved || pressed {
            controls.keyboard = false;
        }
        controls.keyboard |= tab;
        let hold = !self.status.playing
            || menu_open
            || state.scrub.is_some()
            || controls.keyboard
            || (over_controls && was_visible);
        if moved || typing || response.clicked() || hold {
            controls.visible_until = now + HIDE_DELAY;
        }
        let keyboard = controls.keyboard;
        let visible = hold || now < controls.visible_until;
        if visible && !hold {
            ui.ctx()
                .request_repaint_after(std::time::Duration::from_secs_f64(
                    (controls.visible_until - now).max(0.0),
                ));
        }
        response.widget_info(|| {
            let label = if touch || !was_visible {
                "Show video controls"
            } else if self.status.playing {
                "Pause video"
            } else {
                "Play video"
            };
            egui::WidgetInfo::labeled(egui::WidgetType::Button, response.enabled(), label)
        });

        if !menu_open {
            if consume_key(ui, egui::Key::Escape) || consume_key(ui, egui::Key::F) {
                action(VideoAction::ToggleFullscreen);
                return;
            }
            let play_key = consume_key(ui, egui::Key::K)
                || ((!keyboard || response.has_focus()) && consume_key(ui, egui::Key::Space));
            if duration > 0.0 && (play_key || (response.clicked() && was_visible && !touch)) {
                action(self.play_action());
            }
            if !keyboard || response.has_focus() {
                self.playback_shortcuts(ui, state, action);
            }
        }

        let alpha = motion::animate_bool(
            ui.ctx(),
            self.id.with("fullscreen-controls"),
            visible,
            chrome,
        );
        if alpha <= 0.001 {
            return;
        }
        let width = (surface.width() - 24.0).max(1.0);
        let controls = egui::Area::new(self.id.with("fullscreen-toolbar"))
            .order(egui::Order::Foreground)
            .interactable(visible)
            .pivot(Align2::CENTER_BOTTOM)
            .fixed_pos(surface.center_bottom() - vec2(0.0, 8.0))
            .default_width(width)
            .show(ui.ctx(), |ui| {
                ui.set_width(width);
                ui.set_opacity(alpha);
                if !visible {
                    ui.disable();
                }
                ui.visuals_mut().override_text_color = Some(chrome.foreground);
                self.show_controls(ui, state, chrome, action);
            });
        let top = (controls.response.rect.top() - 48.0).clamp(surface.top(), surface.bottom());
        let mut scrim = egui::Mesh::default();
        let tint = Color32::from_black_alpha((210.0 * alpha).round() as u8);
        scrim.colored_vertex(egui::pos2(surface.left(), top), Color32::TRANSPARENT);
        scrim.colored_vertex(egui::pos2(surface.right(), top), Color32::TRANSPARENT);
        scrim.colored_vertex(surface.right_bottom(), tint);
        scrim.colored_vertex(surface.left_bottom(), tint);
        scrim.add_triangle(0, 1, 2);
        scrim.add_triangle(0, 2, 3);
        painter.add(egui::Shape::mesh(scrim));
    }
}

fn consume_key(ui: &Ui, key: egui::Key) -> bool {
    ui.input_mut(|input| input.consume_key(egui::Modifiers::NONE, key))
}

fn play_icon(painter: &egui::Painter, center: Pos2, ink: Color32, playing: bool) {
    if playing {
        for x in [-3.0, 3.0] {
            painter.rect_filled(
                Rect::from_center_size(center + vec2(x, 0.0), vec2(3.0, 13.0)),
                1,
                ink,
            );
        }
    } else {
        let points = [(-4.0, -7.0), (7.0, 0.0), (-4.0, 7.0)].map(|(x, y)| center + vec2(x, y));
        painter.add(egui::Shape::convex_polygon(
            points.to_vec(),
            ink,
            Stroke::NONE,
        ));
    }
}

fn time_label(value: f64) -> String {
    let seconds = value.max(0.0) as u64;
    if seconds >= 3600 {
        format!(
            "{}:{:02}:{:02}",
            seconds / 3600,
            seconds / 60 % 60,
            seconds % 60
        )
    } else {
        format!("{}:{:02}", seconds / 60, seconds % 60)
    }
}
