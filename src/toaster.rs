//! Caller-owned queue for responsive notification cards.
//! Visible entries are limited, and closing cards release their space smoothly.
use crate::toast::{Notice, notice_close, notice_frame};
use crate::{Button, ButtonVariant, Dialog, Theme, Toast, ToastState, ToastStatus, ToastVariant};
use egui::{Align2, Area, Id, Order, Response, Ui, Vec2};
use std::time::Duration;

/// A toast to queue in a [`ToasterState`].
#[derive(Clone, Debug)]
pub struct ToastEntry {
    id: Id,
    title: String,
    description: Option<String>,
    timeout: Option<Duration>,
    variant: ToastVariant,
    icon: Option<String>,
}

impl ToastEntry {
    /// Create an entry with the default timeout of five seconds.
    pub fn new(id: Id, title: impl Into<String>) -> Self {
        Self {
            id,
            title: title.into(),
            description: None,
            timeout: Some(Duration::from_secs(5)),
            variant: ToastVariant::Primary,
            icon: None,
        }
    }
    pub fn description(mut self, description: impl Into<String>) -> Self {
        self.description = Some(description.into());
        self
    }
    /// `None` keeps the toast visible until it is dismissed.
    pub fn timeout(mut self, timeout: Option<Duration>) -> Self {
        self.timeout = timeout;
        self
    }
    /// The upstream `FToastVariant` (primary or destructive).
    pub fn variant(mut self, variant: ToastVariant) -> Self {
        self.variant = variant;
        self
    }
    /// An optional decorative leading icon (text glyph approximation).
    pub fn icon(mut self, icon: impl Into<String>) -> Self {
        self.icon = Some(icon.into());
        self
    }
    fn notice(&self) -> Notice<'_> {
        Notice {
            title: &self.title,
            description: self.description.as_deref(),
            icon: self.icon.as_deref(),
            variant: self.variant,
        }
    }
    pub fn id(&self) -> Id {
        self.id
    }
    pub fn title(&self) -> &str {
        &self.title
    }
}

/// Localized labels for the notification center.
#[derive(Clone, Copy, Debug)]
pub struct NotificationCenterLabels<'a> {
    pub title: &'a str,
    pub view_all: &'a str,
    pub close: &'a str,
    pub clear_all: &'a str,
    pub dismiss: &'a str,
    pub empty: &'a str,
}

impl Default for NotificationCenterLabels<'_> {
    fn default() -> Self {
        Self {
            title: "Notifications",
            view_all: "View all notifications",
            close: "Close notifications",
            clear_all: "Clear all",
            dismiss: "Dismiss",
            empty: "No notifications",
        }
    }
}

struct QueuedToast {
    entry: ToastEntry,
    state: ToastState,
}

/// Caller-owned queue of toasts.
pub struct ToasterState {
    entries: Vec<QueuedToast>,
    limit: usize,
}

impl Default for ToasterState {
    fn default() -> Self {
        Self::new(3)
    }
}

impl ToasterState {
    /// A queue that renders at most `limit` toasts at once (clamped to 1..=8).
    pub fn new(limit: usize) -> Self {
        Self {
            entries: Vec::new(),
            limit: limit.clamp(1, 8),
        }
    }

    pub fn limit(&self) -> usize {
        self.limit
    }

    /// Queue a toast. Re-pushing an existing ID replaces and restarts it.
    pub fn push(&mut self, entry: ToastEntry) {
        self.entries.retain(|queued| queued.entry.id != entry.id);
        self.entries.push(QueuedToast {
            state: ToastState::new(entry.timeout),
            entry,
        });
    }

    /// Dismiss a queued toast by ID; unknown IDs are ignored.
    pub fn dismiss(&mut self, id: Id) {
        if let Some(queued) = self.entries.iter_mut().find(|queued| queued.entry.id == id) {
            queued.state.dismiss();
        }
        self.prune();
    }

    pub fn clear(&mut self) {
        for queued in &mut self.entries {
            queued.state.dismiss();
        }
        self.prune();
    }

    pub fn len(&self) -> usize {
        self.entries
            .iter()
            .filter(|queued| queued.state.is_visible())
            .count()
    }

    pub fn is_empty(&self) -> bool {
        !self.entries.iter().any(|queued| queued.state.is_visible())
    }

    /// Whether there are visible cards, queued cards, or exit animations to render.
    /// Use this instead of `is_empty` to skip rendering an idle overlay.
    pub fn needs_render(&self) -> bool {
        !self.entries.is_empty()
    }

    /// Number of preview cards still animating out.
    pub fn closing_count(&self) -> usize {
        self.entries
            .iter()
            .filter(|queued| queued.state.status() == ToastStatus::Closing)
            .count()
    }

    /// Render a modal with all active and pending notifications. Call every frame,
    /// including after closing, and hide the floating preview while `open` is true.
    /// The trigger response supplies the origin for the opening and closing motion.
    pub fn show_center(
        &mut self,
        trigger: &Response,
        open: &mut bool,
        theme: &Theme,
        labels: NotificationCenterLabels<'_>,
    ) {
        self.center(
            &trigger.ctx,
            trigger.id.with("notification-center"),
            Some(trigger),
            open,
            theme,
            labels,
        );
    }

    fn center(
        &mut self,
        ctx: &egui::Context,
        id: Id,
        trigger: Option<&Response>,
        open: &mut bool,
        theme: &Theme,
        labels: NotificationCenterLabels<'_>,
    ) {
        if *open {
            for queued in &mut self.entries {
                queued.state.pause_timer();
            }
        }

        let viewport = ctx.content_rect();
        let touch = theme.touch || viewport.width() < 600.0;
        let inset = if touch { 24.0 } else { 64.0 };
        let padding = if touch { 12 } else { 18 };
        let width = (viewport.width() - inset - 2.0 * f32::from(padding) - 2.0).clamp(1.0, 456.0);
        let max_height = (viewport.height() * if touch { 0.84 } else { 0.72 }
            - 2.0 * f32::from(padding))
        .max(1.0);
        let mut close = false;
        let dialog = Dialog::new(id, labels.title)
            .theme(theme)
            .frame(crate::surface::popup_frame(theme, padding));
        let dialog = match trigger {
            Some(trigger) => dialog.source_response(trigger),
            None => dialog,
        };
        dialog.show(ctx, open, |ui| {
            ui.set_width(width);
            ui.set_max_height(max_height);
            ui.spacing_mut().item_spacing.y = 8.0;
            let count = self.len();
            let count_key = id.with("center-count");
            let count_changed = ctx.data_mut(|data| {
                let previous = data.get_temp::<usize>(count_key);
                data.insert_temp(count_key, count);
                previous != Some(count)
            });
            if count_changed {
                ctx.request_repaint();
            }
            close = center_header(ui, theme, labels, count, touch).clicked();

            if count == 0 {
                notice_frame(theme).show(ui, |ui| {
                    ui.label(egui::RichText::new(labels.empty).color(theme.muted_foreground));
                });
                return;
            }

            let footer_height = if count > 1 {
                if touch { 64.0 } else { 54.0 }
            } else {
                0.0
            };
            let remaining = (max_height - ui.min_rect().height() - footer_height - 8.0).max(1.0);
            let mut dismissed = None;
            egui::ScrollArea::vertical()
                .max_height(remaining)
                .min_scrolled_height(0.0)
                .auto_shrink([false, true])
                .show(ui, |ui| {
                    egui::Frame::NONE.inner_margin(3).show(ui, |ui| {
                        for queued in self
                            .entries
                            .iter()
                            .filter(|queued| queued.state.is_visible())
                        {
                            let entry = &queued.entry;
                            ui.push_id(entry.id, |ui| {
                                let accessible = format!("{}: {}", labels.dismiss, entry.title);
                                notice_frame(theme).show(ui, |ui| {
                                    let response =
                                        entry.notice().show(ui, theme, touch, false, |ui, size| {
                                            notice_close(
                                                ui,
                                                theme,
                                                labels.dismiss,
                                                &accessible,
                                                size,
                                                false,
                                            )
                                        });
                                    if response.clicked() {
                                        dismissed = Some(entry.id);
                                    }
                                });
                            });
                        }
                    });
                });
            if let Some(id) = dismissed {
                self.entries.retain(|queued| queued.entry.id != id);
                ctx.request_repaint();
            }
            if self.len() > 1 {
                ui.separator();
                ui.vertical_centered(|ui| {
                    if Button::new(labels.clear_all, theme)
                        .variant(ButtonVariant::Ghost)
                        .input_mode(notice_input(touch))
                        .show(ui)
                        .clicked()
                    {
                        self.entries.clear();
                        ctx.request_repaint();
                    }
                });
            }
        });
        if close {
            *open = false;
            ctx.request_repaint();
        }
    }

    pub fn contains(&self, id: Id) -> bool {
        self.entries
            .iter()
            .any(|queued| queued.entry.id == id && queued.state.is_visible())
    }

    fn prune(&mut self) {
        self.entries.retain(|queued| queued.state.needs_render());
    }

    /// Render the queue at the bottom trailing corner on desktop or the top
    /// center on touch layouts. Expired toasts are pruned after rendering.
    pub fn show(&mut self, ui: &mut Ui, theme: &Theme, dismiss_label: &str) -> ToasterResponse {
        let id = ui.make_persistent_id("temivu-toaster");
        let anchor = default_anchor(ui, theme);
        self.show_with(ui, theme, dismiss_label, id, anchor, default_offset(anchor))
    }

    /// Render the queue with an explicit overlay ID and anchor. Give each
    /// toaster in one parent its own ID.
    pub fn show_with(
        &mut self,
        ui: &mut Ui,
        theme: &Theme,
        dismiss_label: &str,
        id: Id,
        anchor: Align2,
        offset: Vec2,
    ) -> ToasterResponse {
        self.preview(ui, theme, dismiss_label, (id, anchor, offset), None)
            .0
    }

    fn preview(
        &mut self,
        ui: &mut Ui,
        theme: &Theme,
        dismiss_label: &str,
        (id, anchor, offset): (Id, Align2, Vec2),
        labels: Option<NotificationCenterLabels<'_>>,
    ) -> (ToasterResponse, Option<Response>) {
        let viewport = ui.ctx().content_rect();
        let compact = theme.touch || viewport.width() < 600.0;
        const GUTTER: i8 = 14;
        let max_width = if compact { 392.0 } else { 360.0 };
        let width = (viewport.width() - 2.0 * offset.x.abs().max(4.0))
            .clamp(0.0, max_width + 2.0 * f32::from(GUTTER));
        let height =
            (viewport.height() - offset.y.abs() - 16.0).clamp(0.0, viewport.height() * 0.62);
        let inset = labels.is_some();
        let footer = labels.filter(|_| self.len() > 1);
        let mut view_all = None;
        let area = Area::new(id)
            .order(Order::Foreground)
            .anchor(anchor, offset)
            .constrain_to(viewport)
            .default_size(egui::vec2(width, height))
            .fade_in(false)
            .show(ui.ctx(), |ui| {
                if self.entries.is_empty() {
                    return 0;
                }
                ui.set_width(width);
                ui.set_max_height(height);
                let mut shown = 0;
                egui::Frame::NONE
                    .inner_margin(egui::Margin {
                        left: GUTTER,
                        right: GUTTER,
                        ..egui::Margin::ZERO
                    })
                    .show(ui, |ui| {
                        let (tray, backdrop) = if inset {
                            let alpha = crate::motion::animate_bool(
                                ui.ctx(),
                                id.with("tray-chrome"),
                                !self.is_empty(),
                                theme,
                            );
                            let glass = crate::glass::GlassTransition::new(ui.ctx(), alpha, true);
                            let base = crate::surface::popup_frame(theme, 8);
                            let tray = if glass.enabled() {
                                glass.frame(theme, base)
                            } else {
                                base.multiply_with_opacity(alpha)
                            };
                            (
                                tray,
                                glass
                                    .has_backdrop()
                                    .then_some(crate::GlassProfile::Floating),
                            )
                        } else {
                            (egui::Frame::NONE, None)
                        };
                        crate::glass::show_frame(ui, tray, backdrop, |ui| {
                            let overflow = self.len().saturating_sub(self.limit);
                            let action_space = if footer.is_some() {
                                if compact && width < 350.0 {
                                    122.0
                                } else {
                                    68.0
                                }
                            } else {
                                0.0
                            };
                            let count_space = if inset && overflow > 0 { 22.0 } else { 0.0 };
                            egui::ScrollArea::vertical()
                                .max_height(
                                    (height
                                        - action_space
                                        - count_space
                                        - tray.total_margin().sum().y)
                                        .max(1.0),
                                )
                                .min_scrolled_height(0.0)
                                .auto_shrink([false, true])
                                .show(ui, |ui| {
                                    ui.spacing_mut().item_spacing.y = 6.0;
                                    let padding = if inset { 2.0 } else { f32::from(GUTTER) };
                                    ui.add_space(padding);
                                    for queued in &mut self.entries {
                                        let QueuedToast { entry, state } = queued;
                                        if state.is_visible() && shown >= self.limit {
                                            continue;
                                        }
                                        let rendered = ui
                                            .push_id(entry.id, |ui| {
                                                show_toast(
                                                    ui,
                                                    theme,
                                                    dismiss_label,
                                                    entry,
                                                    state,
                                                    inset,
                                                )
                                            })
                                            .inner;
                                        shown += usize::from(rendered && state.is_visible());
                                    }
                                    ui.add_space(padding);
                                });

                            let actions = footer.filter(|_| self.len() > 1);
                            let opacity = labels.map_or(1.0, |_| {
                                let target = f32::from(actions.is_some());
                                crate::motion::MotionTrack::new(
                                    ui.ctx(),
                                    id.with("queue-actions"),
                                    theme.motion,
                                )
                                .sample(
                                    target,
                                    target,
                                    crate::motion::MotionRole::Feedback.tween(theme.motion),
                                )
                                .value
                            });
                            if let Some(labels) = actions {
                                let (clear, view) = ui
                                    .scope(|ui| {
                                        ui.multiply_opacity(opacity);
                                        ui.spacing_mut().item_spacing = egui::vec2(6.0, 6.0);
                                        ui.separator();
                                        let overflow = self.len().saturating_sub(shown);
                                        if overflow > 0 {
                                            ui.vertical_centered(|ui| {
                                                ui.label(
                                                    egui::RichText::new(format!("+{overflow}"))
                                                        .strong()
                                                        .color(theme.muted_foreground),
                                                );
                                            });
                                        }
                                        ui.with_layout(
                                            egui::Layout::right_to_left(egui::Align::TOP)
                                                .with_main_wrap(true),
                                            |ui| {
                                                let view = Button::new(labels.view_all, theme)
                                                    .input_mode(notice_input(compact))
                                                    .show(ui);
                                                let clear = Button::new(labels.clear_all, theme)
                                                    .input_mode(notice_input(compact))
                                                    .variant(ButtonVariant::Ghost)
                                                    .show(ui);
                                                (clear.clicked(), view)
                                            },
                                        )
                                        .inner
                                    })
                                    .inner;
                                view_all = Some(view);
                                if clear {
                                    self.clear();
                                    ui.ctx().request_repaint();
                                }
                            }
                        });
                    });
                shown
            });
        self.prune();
        (
            ToasterResponse {
                response: area.response,
                visible: area.inner,
            },
            view_all,
        )
    }
}

fn show_toast(
    ui: &mut Ui,
    theme: &Theme,
    dismiss_label: &str,
    entry: &ToastEntry,
    state: &mut ToastState,
    inset: bool,
) -> bool {
    let mut toast = Toast::new(state, theme, &entry.title, dismiss_label)
        .variant(entry.variant)
        .inset(inset);
    if let Some(description) = &entry.description {
        toast = toast.description(description);
    }
    if let Some(icon) = &entry.icon {
        toast = toast.icon(icon);
    }
    let accessible = format!("{dismiss_label}: {}", entry.title);
    toast
        .dismiss_accessible_label(&accessible)
        .show(ui)
        .is_some()
}

fn center_header(
    ui: &mut Ui,
    theme: &Theme,
    labels: NotificationCenterLabels<'_>,
    count: usize,
    touch: bool,
) -> Response {
    let size = if touch { 44.0 } else { 32.0 };
    let width = ui.available_width();
    let lane = (size + 6.0_f32).min(width * 0.33);
    let origin = ui.cursor().min;
    let title_rect = egui::Rect::from_min_size(
        origin + egui::vec2(lane, 0.0),
        egui::vec2((width - 2.0 * lane).max(1.0), size),
    );
    let mut title = ui.new_child(
        egui::UiBuilder::new()
            .id_salt("title")
            .max_rect(title_rect)
            .layout(egui::Layout::left_to_right(egui::Align::Center).with_main_justify(true)),
    );
    title.add(
        egui::Label::new(
            egui::RichText::new(labels.title)
                .size(20.0)
                .strong()
                .color(theme.foreground),
        )
        .wrap()
        .halign(egui::Align::Center)
        .selectable(false),
    );

    let height = size.max(title.min_rect().height());
    let row = egui::Rect::from_min_size(origin, egui::vec2(width, height));
    let action_size = Vec2::splat(size.min(lane));
    let slot = |x| egui::Rect::from_center_size(egui::pos2(x, row.center().y), action_size);
    let badge_rect = slot(row.left() + lane / 2.0);
    let mut badge = ui.new_child(
        egui::UiBuilder::new()
            .id_salt("count")
            .max_rect(badge_rect)
            .layout(egui::Layout::centered_and_justified(
                egui::Direction::LeftToRight,
            )),
    );
    badge
        .painter()
        .rect_filled(badge_rect.shrink(2.0), 255, theme.muted);
    badge.add(
        egui::Label::new(egui::RichText::new(count.to_string()).color(theme.muted_foreground))
            .truncate()
            .selectable(false),
    );

    let close_rect = slot(row.right() - lane / 2.0);
    let mut action = ui.new_child(
        egui::UiBuilder::new()
            .id_salt("dismiss")
            .max_rect(close_rect)
            .layout(egui::Layout::top_down(egui::Align::Center)),
    );
    let close = notice_close(
        &mut action,
        theme,
        labels.close,
        labels.close,
        action_size.x,
        false,
    );
    ui.advance_cursor_after_rect(row);
    close
}

fn notice_input(touch: bool) -> crate::InputMode {
    if touch {
        crate::InputMode::Touch
    } else {
        crate::InputMode::Desktop
    }
}

fn default_anchor(ui: &Ui, theme: &Theme) -> Align2 {
    if theme.touch || ui.ctx().content_rect().width() < 600.0 {
        Align2::CENTER_TOP
    } else {
        Align2::RIGHT_BOTTOM
    }
}

/// Keep the overlay 16 points away from whichever screen edge(s) the anchor
/// touches, and centred otherwise, so every [`Align2`] maps to a sane offset.
fn default_offset(anchor: Align2) -> Vec2 {
    let x = match anchor.x() {
        egui::Align::Min => 16.0,
        egui::Align::Center => 0.0,
        egui::Align::Max => -16.0,
    };
    let y = match anchor.y() {
        egui::Align::Min => 16.0,
        egui::Align::Center => 0.0,
        egui::Align::Max => -16.0,
    };
    egui::vec2(x, y)
}

/// The result of rendering a [`ToasterState`]. The area may be used to attach
/// hover or keyboard behaviour; `visible` is the number of toasts on screen.
pub struct ToasterResponse {
    pub response: Response,
    pub visible: usize,
}

/// Renders a [`ToasterState`] as a stacked foreground overlay.
pub struct Toaster<'a> {
    state: &'a mut ToasterState,
    theme: &'a Theme,
    dismiss_label: &'a str,
    id: Option<Id>,
    anchor: Option<Align2>,
    offset: Option<Vec2>,
}

impl<'a> Toaster<'a> {
    pub fn new(state: &'a mut ToasterState, theme: &'a Theme, dismiss_label: &'a str) -> Self {
        Self {
            state,
            theme,
            dismiss_label,
            id: None,
            anchor: None,
            offset: None,
        }
    }

    /// Use a caller-supplied overlay ID (one per toaster under a parent).
    pub fn id(mut self, id: Id) -> Self {
        self.id = Some(id);
        self
    }

    /// Override the default anchor (bottom-end on desktop, top-centre on touch).
    pub fn anchor(mut self, anchor: Align2) -> Self {
        self.anchor = Some(anchor);
        self
    }

    /// Offset from the chosen anchor, in egui points.
    pub fn offset(mut self, offset: Vec2) -> Self {
        self.offset = Some(offset);
        self
    }

    /// Render a grouped preview with queue actions under the cards when
    /// at least two notices remain. The center lists the entire queue.
    /// Call every frame on the same parent so the modal can animate closed.
    pub fn show_with_center(
        self,
        ui: &mut Ui,
        open: &mut bool,
        labels: NotificationCenterLabels<'_>,
    ) {
        let id = self
            .id
            .unwrap_or_else(|| ui.make_persistent_id("temivu-toaster"));
        let anchor = self
            .anchor
            .unwrap_or_else(|| default_anchor(ui, self.theme));
        let offset = self.offset.unwrap_or_else(|| default_offset(anchor));
        let trigger = if !*open && self.state.needs_render() {
            self.state
                .preview(
                    ui,
                    self.theme,
                    self.dismiss_label,
                    (id, anchor, offset),
                    Some(labels),
                )
                .1
        } else {
            None
        };
        *open |= trigger.as_ref().is_some_and(Response::clicked);
        self.state.center(
            ui.ctx(),
            id.with("notification-center"),
            trigger.as_ref(),
            open,
            self.theme,
            labels,
        );
    }

    pub fn show(self, ui: &mut Ui) -> ToasterResponse {
        let id = self
            .id
            .unwrap_or_else(|| ui.make_persistent_id("temivu-toaster"));
        let anchor = self
            .anchor
            .unwrap_or_else(|| default_anchor(ui, self.theme));
        self.state.show_with(
            ui,
            self.theme,
            self.dismiss_label,
            id,
            anchor,
            self.offset.unwrap_or_else(|| default_offset(anchor)),
        )
    }
}
