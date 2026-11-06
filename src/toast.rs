//! Responsive notification card with a paint-only exit.
use crate::{Theme, liquid::fade_shape, motion};
use egui::{Id, Rect, Response, RichText, Ui, WidgetInfo, WidgetType, vec2};
use std::time::Duration;

#[derive(Clone, Debug)]
struct ToastSnapshot {
    rect: Rect,
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
            self.started_at = None;
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

/// The upstream `FToastVariant`: the primary style uses the regular text colours,
/// the destructive style paints the title and description in the destructive
/// colour (contrast-adjusted against the card so it meets AA).
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
    icon: Option<&'a str>,
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
            icon: None,
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

        let reveal = motion::surface_reveal(ui.ctx(), motion_id, true, self.theme);
        let animate = self.theme.motion.animations_enabled(ui.ctx()) && !ui.is_sizing_pass();
        let layer = ui.layer_id();
        let start = animate.then(|| {
            ui.ctx()
                .graphics(|graphics| graphics.get(layer).map_or(0, |list| list.next_idx().0))
        });
        let mut child = ui.new_child(egui::UiBuilder::new().id_salt(motion_id));
        let compact = self.theme.touch || ui.ctx().content_rect().width() < 600.0;
        let width = ui.available_width().min(392.0);
        child.set_max_width(width);
        let mut frame = crate::surface::popup_frame(self.theme, if compact { 14 } else { 16 })
            .corner_radius(22);
        frame.shadow.offset = [0, 4];
        frame.shadow.blur = 14;
        let colors = match self.variant {
            ToastVariant::Primary => (self.theme.foreground, self.theme.muted_foreground),
            ToastVariant::Destructive => {
                let destructive =
                    self.theme
                        .readable_on(self.theme.destructive, self.theme.card, 4.5);
                (destructive, destructive)
            }
        };
        let card = frame.show(&mut child, |ui| {
            ui.set_width((width - frame.total_margin().sum().x).max(0.0));
            ui.spacing_mut().item_spacing.x = 10.0;
            ui.with_layout(egui::Layout::right_to_left(egui::Align::TOP), |ui| {
                let size = if compact { 44.0 } else { 34.0 };
                let dismiss = ui
                    .add_enabled_ui(reveal >= 0.97, |ui| {
                        crate::button::press_native(ui, self.theme, |ui| {
                            ui.add(
                                egui::Button::new(
                                    RichText::new("×").size(21.0).color(self.theme.foreground),
                                )
                                .fill(self.theme.muted)
                                .stroke(egui::Stroke::NONE)
                                .corner_radius(255)
                                .min_size(vec2(size, size)),
                            )
                        })
                    })
                    .inner
                    .on_hover_text(self.dismiss_label);
                let accessible = self.dismiss_accessible_label.unwrap_or(self.dismiss_label);
                ui.ctx()
                    .accesskit_node_builder(dismiss.id, |node| node.set_label(accessible));

                ui.with_layout(egui::Layout::left_to_right(egui::Align::TOP), |ui| {
                    if ui.available_width() >= 114.0 {
                        let (rect, _) =
                            ui.allocate_exact_size(vec2(30.0, 30.0), egui::Sense::hover());
                        ui.painter()
                            .circle_filled(rect.center(), 15.0, self.theme.muted);
                        let icon = self.icon.unwrap_or(match self.variant {
                            ToastVariant::Primary => "i",
                            ToastVariant::Destructive => "!",
                        });
                        let glyph = ui.put(
                            rect,
                            egui::Label::new(RichText::new(icon).strong().color(colors.0))
                                .selectable(false)
                                .truncate(),
                        );
                        ui.ctx()
                            .accesskit_node_builder(glyph.id, |node| node.set_hidden());
                    } else if let Some(icon) = self.icon {
                        // Preserve explicit decorative icons even in very narrow cards.
                        let glyph = ui.add(
                            egui::Label::new(RichText::new(icon).color(colors.0)).selectable(false),
                        );
                        ui.ctx()
                            .accesskit_node_builder(glyph.id, |node| node.set_hidden());
                    }
                    ui.vertical(|ui| {
                        ui.set_width(ui.available_width());
                        ui.spacing_mut().item_spacing.y = 3.0;
                        ui.add(
                            egui::Label::new(
                                RichText::new(self.title)
                                    .size(self.theme.typography.body.size)
                                    .strong()
                                    .color(colors.0),
                            )
                            .wrap(),
                        );
                        if let Some(description) = self.description {
                            ui.add(
                                egui::Label::new(
                                    RichText::new(description)
                                        .size(self.theme.typography.body.size)
                                        .color(colors.1),
                                )
                                .wrap(),
                            );
                        }
                    });
                });
                dismiss
            })
            .inner
        });

        let mut response = card.response;
        let layout = toast_layout(ui, response.rect, reveal);
        if let Some(start) = start {
            let clip = ui.clip_rect();
            let snapshot = self.state.snapshot.get_or_insert_with(|| ToastSnapshot {
                rect: response.rect,
                shapes: Vec::new(),
            });
            snapshot.rect = response.rect;
            snapshot.shapes.clear();
            ui.ctx().graphics(|graphics| {
                if let Some(list) = graphics.get(layer) {
                    snapshot
                        .shapes
                        .extend(list.all_entries().skip(start).cloned());
                }
            });
            if reveal < 1.0 {
                ui.ctx().graphics_mut(|graphics| {
                    let list = graphics.entry(layer);
                    for index in start..list.next_idx().0 {
                        list.mutate_shape(egui::layers::ShapeIdx(index), |entry| {
                            pose_toast(entry, response.rect, layout, clip, reveal);
                        });
                    }
                });
            }
        } else {
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
        response.widget_info(|| WidgetInfo::labeled(WidgetType::Other, ui.is_enabled(), &label));
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
        let reveal = motion::surface_reveal(ui.ctx(), id, false, self.theme);
        if reveal <= 0.001 {
            self.state.snapshot = None;
            motion::reset(ui.ctx(), id);
            return;
        }
        let rect = Rect::from_min_size(ui.available_rect_before_wrap().min, snapshot.rect.size());
        let layout = toast_layout(ui, rect, reveal);
        for entry in snapshot.shapes.iter() {
            let mut entry = entry.clone();
            pose_toast(&mut entry, snapshot.rect, layout, ui.clip_rect(), reveal);
            ui.painter()
                .with_clip_rect(entry.clip_rect)
                .add(entry.shape);
        }
    }
}

fn toast_layout(ui: &mut Ui, rect: Rect, reveal: f32) -> Rect {
    let mut layout = rect;
    if ui.layout().is_vertical() {
        layout.max.y = layout.min.y + rect.height() * reveal;
    }
    ui.advance_cursor_after_rect(layout);
    layout
}

fn pose_toast(
    entry: &mut egui::epaint::ClippedShape,
    rect: Rect,
    layout: Rect,
    clip: Rect,
    reveal: f32,
) {
    let transform = egui::emath::TSTransform::from_translation(
        layout.center_bottom() - rect.center_bottom() + vec2(0.0, 8.0 * (1.0 - reveal)),
    ) * motion::scale_around(rect.center_bottom(), egui::lerp(0.92..=1.0, reveal));
    entry.shape.transform(transform);
    entry.clip_rect = transform
        .mul_rect(entry.clip_rect)
        .intersect(layout.expand(12.0))
        .intersect(clip);
    fade_shape(&mut entry.shape, reveal);
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
