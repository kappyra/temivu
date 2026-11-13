//! Responsive notification card with a paint-only exit.
use crate::{
    InputMode, Theme,
    glass::GlassTransition,
    liquid::{self, DropletAnchor, LiquidStyle, SurfaceTransition, fade_shape},
    motion,
};
use egui::{Id, Rect, Response, RichText, Role, Ui, WidgetInfo, vec2};
use std::time::Duration;

#[derive(Clone, Debug)]
struct ToastSnapshot {
    rect: Rect,
    frame: egui::Frame,
    shapes: Vec<egui::epaint::ClippedShape>,
}

/// Caller-owned visibility and expiry. The clock begins when the toast is first shown
/// and restarts after pointer hover or keyboard focus on its dismiss control.
#[derive(Clone, Debug)]
pub struct ToastState {
    visible: bool,
    timeout: Option<Duration>,
    started_at: Option<f64>,
    snapshot: Option<ToastSnapshot>,
}

impl Default for ToastState {
    fn default() -> Self {
        Self::new(Some(Duration::from_secs(5)))
    }
}

/// Visual lifecycle of a notification.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ToastStatus {
    Visible,
    Closing,
    Closed,
}

impl ToastState {
    /// `None` keeps the toast visible until explicitly dismissed.
    pub fn new(timeout: Option<Duration>) -> Self {
        Self {
            visible: true,
            timeout,
            started_at: None,
            snapshot: None,
        }
    }
    pub fn is_visible(&self) -> bool {
        self.visible
    }
    pub fn dismiss(&mut self) {
        self.visible = false;
    }
    pub fn restart(&mut self) {
        self.visible = true;
        self.started_at = None;
    }

    pub(crate) fn pause_timer(&mut self) {
        self.started_at = None;
    }

    pub fn status(&self) -> ToastStatus {
        if self.visible {
            ToastStatus::Visible
        } else if self.snapshot.is_some() {
            ToastStatus::Closing
        } else {
            ToastStatus::Closed
        }
    }

    /// Whether the card or its closing animation still needs to be rendered.
    pub fn needs_render(&self) -> bool {
        self.visible || self.snapshot.is_some()
    }

    // Returns the next timer wakeup; bounds the duration passed to egui.
    fn tick(&mut self, now: f64, paused: bool) -> Option<Duration> {
        if !self.visible || !now.is_finite() {
            return None;
        }
        if paused {
            self.pause_timer();
            return None;
        }
        let timeout = self.timeout?;
        let start = *self.started_at.get_or_insert(now);
        let elapsed = (now - start).max(0.0);
        let remaining = timeout.as_secs_f64() - elapsed;
        if remaining <= 0.0 {
            self.visible = false;
            return None;
        }
        Some(Duration::from_secs_f64(remaining.min(86_400.0)))
    }
}

/// Notification severity. Destructive notices use a warning mark and a tinted
/// status tile, with standard theme colors for the message.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum ToastVariant {
    #[default]
    Primary,
    Destructive,
}

/// A single inline toast. The dismissal label is supplied by the caller for localization.
pub struct Toast<'a> {
    state: &'a mut ToastState,
    theme: &'a Theme,
    title: &'a str,
    description: Option<&'a str>,
    dismiss_label: &'a str,
    variant: ToastVariant,
    dismiss_accessible_label: Option<&'a str>,
    input_mode: Option<InputMode>,
    icon: Option<&'a str>,
    inset: bool,
}

impl<'a> Toast<'a> {
    pub fn new(
        state: &'a mut ToastState,
        theme: &'a Theme,
        title: &'a str,
        dismiss_label: &'a str,
    ) -> Self {
        Self {
            state,
            theme,
            title,
            description: None,
            dismiss_label,
            variant: ToastVariant::Primary,
            dismiss_accessible_label: None,
            input_mode: None,
            icon: None,
            inset: false,
        }
    }
    pub fn description(mut self, description: &'a str) -> Self {
        self.description = Some(description);
        self
    }
    /// An optional leading icon rendered as a text glyph. This is an
    /// approximation of the upstream widget icon (Forui uses vector icons). The
    /// glyph is decorative and is not part of the toast's accessible label.
    pub fn icon(mut self, icon: &'a str) -> Self {
        self.icon = Some(icon);
        self
    }
    /// The upstream `FToastVariant` (primary or destructive).
    pub fn variant(mut self, variant: ToastVariant) -> Self {
        self.variant = variant;
        self
    }
    /// An accessible name for the dismiss control, for callers that render
    /// several toasts whose visible labels would otherwise be identical.
    pub fn dismiss_accessible_label(mut self, label: &'a str) -> Self {
        self.dismiss_accessible_label = Some(label);
        self
    }
    /// Select the touch or desktop layout explicitly.
    pub fn input_mode(mut self, input_mode: InputMode) -> Self {
        self.input_mode = Some(input_mode);
        self
    }

    pub(crate) fn inset(mut self, inset: bool) -> Self {
        self.inset = inset;
        self
    }

    /// Show the toast and return its response while it is visible.
    /// Keep calling this after dismissal to finish the paint-only exit.
    pub fn show(mut self, ui: &mut Ui) -> Option<Response> {
        let motion_id = ui.make_persistent_id("temivu-toast-reveal");
        if self.state.timeout == Some(Duration::ZERO) {
            self.state.dismiss();
        }
        if !self.state.visible {
            self.paint_exit(ui, motion_id);
            return None;
        }

        let reveal = liquid::surface_progress(
            ui.ctx(),
            motion_id,
            true,
            LiquidStyle::droplet(),
            self.theme,
        )
        .min(1.0);
        let animate = self.theme.motion.animations_enabled(ui.ctx()) && !ui.is_sizing_pass();
        let glass = GlassTransition::new(ui.ctx(), reveal, !self.inset);
        let layer = ui.layer_id();
        let start = (animate || glass.has_backdrop()).then(|| {
            ui.ctx()
                .graphics(|graphics| graphics.get(layer).map_or(0, |list| list.next_idx().0))
        });
        let mut child = ui.new_child(egui::UiBuilder::new().id_salt(motion_id));
        let compact = self.input_mode.map_or_else(
            || self.theme.touch || ui.ctx().content_rect().width() < 600.0,
            |mode| mode == InputMode::Touch,
        );
        let width = ui
            .available_width()
            .min(if compact { 392.0 } else { 360.0 });
        child.set_max_width(width);
        let frame = if self.inset {
            notice_frame(self.theme)
        } else {
            crate::surface::popup_frame(self.theme, if compact { 16 } else { 14 })
        };
        let notice = Notice {
            title: self.title,
            description: self.description,
            icon: self.icon,
            variant: self.variant,
        };
        let touch_footer = compact && !self.inset;
        let label = self.dismiss_accessible_label.unwrap_or(self.dismiss_label);
        let card = frame.show(&mut child, |ui| {
            ui.set_width((width - frame.total_margin().sum().x).max(0.0));
            notice.show(ui, self.theme, compact, touch_footer, |ui, size| {
                ui.add_enabled_ui(reveal >= 0.97, |ui| {
                    notice_close(
                        ui,
                        self.theme,
                        self.dismiss_label,
                        label,
                        size,
                        touch_footer,
                    )
                })
                .inner
            })
        });

        let mut response = card.response;
        let layout = toast_layout(ui, response.rect, reveal);
        if let Some(start) = start {
            let clip = ui.clip_rect();
            if animate {
                let snapshot = self.state.snapshot.get_or_insert_with(|| ToastSnapshot {
                    rect: response.rect,
                    frame,
                    shapes: Vec::new(),
                });
                snapshot.rect = response.rect;
                snapshot.frame = frame;
                snapshot.shapes.clear();
                ui.ctx().graphics(|graphics| {
                    if let Some(list) = graphics.get(layer) {
                        snapshot
                            .shapes
                            .extend(list.all_entries().skip(start + 1).cloned());
                    }
                });
            }
            if reveal < 1.0 || glass.has_backdrop() {
                let surface =
                    toast_surface(response.rect, layout, reveal, frame, self.theme, &glass);
                ui.ctx().graphics_mut(|graphics| {
                    let list = graphics.entry(layer);
                    list.set(egui::layers::ShapeIdx(start), clip, surface);
                    if reveal < 1.0 {
                        let pose = toast_pose(response.rect, layout, clip, reveal, frame);
                        for index in start + 1..list.next_idx().0 {
                            list.mutate_shape(egui::layers::ShapeIdx(index), |entry| {
                                pose_toast(entry, pose);
                            });
                        }
                    }
                });
            }
        }
        if !animate {
            self.state.snapshot = None;
        }

        if card.inner.clicked() {
            self.state.dismiss();
        }
        let now = ui.input(|input| input.time);
        let hovered = ui.rect_contains_pointer(layout);
        if let Some(delay) = self.state.tick(now, hovered || card.inner.has_focus()) {
            ui.ctx().request_repaint_after(delay);
        }
        if !self.state.visible {
            response.mark_changed();
            ui.ctx().request_repaint();
        }
        let label = self.description.map_or_else(
            || self.title.to_owned(),
            |text| format!("{}: {text}", self.title),
        );
        response.widget_info(|| WidgetInfo::labeled(Role::Unknown, ui.is_enabled(), &label));
        ui.ctx().accesskit_node_builder(response.id, |node| {
            node.set_role(egui::accesskit::Role::Alert);
            node.set_label(label);
            node.set_live(egui::accesskit::Live::Polite);
        });
        Some(response)
    }

    fn paint_exit(&mut self, ui: &mut Ui, id: Id) {
        let Some(snapshot) = &self.state.snapshot else {
            motion::reset(ui.ctx(), id);
            return;
        };
        let reveal =
            liquid::surface_progress(ui.ctx(), id, false, LiquidStyle::droplet(), self.theme)
                .min(1.0);
        if reveal <= 0.001 {
            self.state.snapshot = None;
            motion::reset(ui.ctx(), id);
            return;
        }
        let rect = Rect::from_min_size(ui.available_rect_before_wrap().min, snapshot.rect.size());
        let layout = toast_layout(ui, rect, reveal);
        let pose = toast_pose(
            snapshot.rect,
            layout,
            ui.clip_rect(),
            reveal,
            snapshot.frame,
        );
        let glass = GlassTransition::new(ui.ctx(), reveal, !self.inset);
        ui.painter().add(toast_surface(
            snapshot.rect,
            layout,
            reveal,
            snapshot.frame,
            self.theme,
            &glass,
        ));
        for entry in &snapshot.shapes {
            let mut entry = entry.clone();
            pose_toast(&mut entry, pose);
            ui.painter()
                .with_clip_rect(entry.clip_rect)
                .add(entry.shape);
        }
    }
}

pub(crate) fn notice_frame(theme: &Theme) -> egui::Frame {
    egui::Frame::new()
        .fill(theme.muted)
        .corner_radius(18)
        .inner_margin(8)
}

pub(crate) struct Notice<'a> {
    pub title: &'a str,
    pub description: Option<&'a str>,
    pub icon: Option<&'a str>,
    pub variant: ToastVariant,
}

impl Notice<'_> {
    fn marker(&self, ui: &mut Ui, rect: Rect, theme: &Theme) {
        let tint = match self.variant {
            ToastVariant::Primary => theme.foreground,
            ToastVariant::Destructive => theme.readable_on(theme.destructive, theme.card, 4.5),
        };
        let fill = if self.variant == ToastVariant::Destructive {
            tint.gamma_multiply(if theme.dark { 0.18 } else { 0.09 })
        } else {
            theme
                .foreground
                .gamma_multiply(if theme.dark { 0.13 } else { 0.07 })
        };
        ui.painter().rect_filled(rect, 11, fill);
        if let Some(icon) = self
            .icon
            .or((self.variant == ToastVariant::Destructive).then_some("!"))
        {
            let mut icon_ui = ui.new_child(
                egui::UiBuilder::new()
                    .id_salt("notice-marker")
                    .max_rect(rect)
                    .layout(egui::Layout::centered_and_justified(
                        egui::Direction::LeftToRight,
                    )),
            );
            let response = icon_ui.add(
                egui::Label::new(RichText::new(icon).strong().color(tint))
                    .selectable(false)
                    .truncate(),
            );
            ui.ctx()
                .accesskit_node_builder(response.id, |node| node.set_hidden());
        } else {
            ui.painter().circle_filled(rect.center(), 3.0, tint);
        }
    }

    pub fn show(
        &self,
        ui: &mut Ui,
        theme: &Theme,
        touch: bool,
        footer: bool,
        dismiss: impl FnOnce(&mut Ui, f32) -> Response,
    ) -> Response {
        let width = ui.available_width();
        let size = if touch { 44.0 } else { 32.0 };
        let marker_size = 28.0_f32;
        let text_left = (marker_size + 10.0).min(width * 0.30);
        let text_right = if footer { 0.0 } else { size + 6.0 };
        let start = ui.cursor().min;
        let text_rect = Rect::from_min_size(
            start + vec2(text_left, 0.0),
            vec2((width - text_left - text_right).max(1.0), marker_size),
        );
        let mut text_ui = ui.new_child(
            egui::UiBuilder::new()
                .id_salt("message")
                .max_rect(text_rect)
                .layout(egui::Layout::top_down(egui::Align::Min)),
        );
        text_ui.set_width(text_rect.width());
        text_ui.spacing_mut().item_spacing.y = 3.0;
        for (text, heading) in [(Some(self.title), true), (self.description, false)] {
            if let Some(text) = text {
                let text = if heading {
                    RichText::new(text)
                        .size(theme.typography.body.size)
                        .strong()
                        .color(theme.foreground)
                } else {
                    RichText::new(text)
                        .size(theme.typography.body.size)
                        .color(theme.muted_foreground)
                };
                text_ui.add(egui::Label::new(text).wrap().selectable(false));
            }
        }
        let row_height = text_ui
            .min_rect()
            .height()
            .max(if footer { marker_size } else { size });
        let row = Rect::from_min_size(start, vec2(width, row_height));
        let mark_width = marker_size.min((text_left - 4.0).max(0.0));
        self.marker(
            ui,
            Rect::from_center_size(
                egui::pos2(row.left() + mark_width / 2.0, row.center().y),
                vec2(mark_width, marker_size),
            ),
            theme,
        );
        ui.advance_cursor_after_rect(row);

        if footer {
            ui.add_space(4.0);
            ui.with_layout(egui::Layout::right_to_left(egui::Align::TOP), |ui| {
                dismiss(ui, size)
            })
            .inner
        } else {
            let action = Rect::from_center_size(
                egui::pos2(row.right() - size / 2.0, row.center().y),
                egui::Vec2::splat(size),
            );
            let mut action_ui = ui.new_child(
                egui::UiBuilder::new()
                    .id_salt("close")
                    .max_rect(action)
                    .layout(egui::Layout::top_down(egui::Align::Center)),
            );
            dismiss(&mut action_ui, size)
        }
    }
}

pub(crate) fn notice_close(
    ui: &mut Ui,
    theme: &Theme,
    label: &str,
    accessible: &str,
    size: f32,
    show_label: bool,
) -> Response {
    let text = if show_label { label } else { "" };
    let response = crate::button::press_native(ui, theme, |ui| {
        let response = ui.add(
            egui::Button::new(RichText::new(text).color(theme.foreground))
                .wrap()
                .fill(if show_label {
                    theme.muted
                } else {
                    theme.foreground.gamma_multiply(0.07)
                })
                .stroke(egui::Stroke::NONE)
                .corner_radius(255)
                .min_size(egui::Vec2::splat(size)),
        );
        if !show_label && ui.is_rect_visible(response.rect) {
            let center = response.rect.center();
            let stroke = egui::Stroke::new(1.5, theme.foreground);
            for dy in [-3.0, 3.0] {
                ui.painter()
                    .line_segment([center + vec2(-3.0, dy), center + vec2(3.0, -dy)], stroke);
            }
        }
        response
    })
    .on_hover_text(label);
    ui.ctx()
        .accesskit_node_builder(response.id, |node| node.set_label(accessible));
    response
}

fn toast_layout(ui: &mut Ui, rect: Rect, reveal: f32) -> Rect {
    let mut layout = rect;
    if ui.layout().is_vertical() {
        layout.max.y = layout.min.y + rect.height() * reveal;
    }
    ui.advance_cursor_after_rect(layout);
    layout
}

fn toast_bounds(rect: Rect, layout: Rect) -> (Rect, Rect) {
    let target = rect.translate(layout.center_bottom() - rect.center_bottom());
    let size = target.size().min(vec2(52.0, 18.0));
    let source = Rect::from_center_size(target.center_bottom() - vec2(0.0, size.y * 0.5), size);
    (source, target)
}

fn toast_surface(
    rect: Rect,
    layout: Rect,
    reveal: f32,
    frame: egui::Frame,
    theme: &Theme,
    glass: &GlassTransition,
) -> egui::Shape {
    let (source, target) = toast_bounds(rect, layout);
    let frame = glass.frame(theme, frame);
    let mut surface = liquid::matched_shape_with_opacity(
        source,
        target,
        reveal,
        frame,
        None,
        glass.seed_opacity(reveal),
    );
    if !glass.enabled() {
        fade_shape(&mut surface, reveal);
    }
    if !glass.has_backdrop() {
        return surface;
    }
    let (rect, corners) = liquid::anchored_glass_geometry(
        DropletAnchor {
            rect: source,
            contact: None,
        },
        target,
        reveal,
        frame,
        SurfaceTransition::Droplet,
        true,
    );
    glass.with_backdrop(surface, rect, corners)
}

fn toast_pose(
    rect: Rect,
    layout: Rect,
    clip: Rect,
    reveal: f32,
    frame: egui::Frame,
) -> (egui::emath::TSTransform, Rect, f32) {
    let (source, target) = toast_bounds(rect, layout);
    let (transform, visible, alpha) =
        liquid::droplet_content_pose(source, target, reveal, frame, true);
    let shift = egui::emath::TSTransform::from_translation(target.min - rect.min);
    (transform * shift, visible.intersect(clip), alpha * reveal)
}

fn pose_toast(
    entry: &mut egui::epaint::ClippedShape,
    (transform, visible, alpha): (egui::emath::TSTransform, Rect, f32),
) {
    entry.shape.transform(transform);
    entry.clip_rect = transform.mul_rect(entry.clip_rect).intersect(visible);
    fade_shape(&mut entry.shape, alpha);
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn timed_persistent_and_restarted_lifecycles() {
        let mut state = ToastState::new(Some(Duration::from_secs(5)));
        assert_eq!(state.tick(10.0, false), Some(Duration::from_secs(5)));
        assert_eq!(state.tick(12.0, false), Some(Duration::from_secs(3)));
        assert_eq!(state.tick(15.0, false), None);
        assert!(!state.is_visible());
        state.restart();
        assert_eq!(state.tick(20.0, false), Some(Duration::from_secs(5)));
        assert_eq!(state.tick(22.0, true), None);
        assert_eq!(state.tick(30.0, false), Some(Duration::from_secs(5)));
        let mut persistent = ToastState::new(None);
        assert_eq!(persistent.tick(100.0, false), None);
        assert!(persistent.is_visible());
        persistent.dismiss();
        assert!(!persistent.is_visible());
        let mut immediate = ToastState::new(Some(Duration::ZERO));
        assert_eq!(immediate.tick(0.0, false), None);
        assert!(!immediate.is_visible());
        let mut long = ToastState::new(Some(Duration::MAX));
        assert_eq!(long.tick(f64::NAN, false), None);
        assert!(long.is_visible());
        assert_eq!(long.tick(0.0, false), Some(Duration::from_secs(86_400)));
    }
}
