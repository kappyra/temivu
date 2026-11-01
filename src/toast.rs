//! Preliminary single-toast lifecycle with a reduced-motion-aware fade-in.
//! Overlay queues, gestures, exit animation and parity remain open.
use crate::Theme;
use egui::{Frame, Margin, Response, RichText, Stroke, Ui, WidgetInfo, WidgetType};
use std::time::Duration;

/// Caller-owned visibility and expiry. The clock begins when the toast is first shown
/// and restarts after pointer hover or keyboard focus on its dismiss control.
#[derive(Clone, Debug)]
pub struct ToastState {
    visible: bool,
    timeout: Option<Duration>,
    started_at: Option<f64>,
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
    ///
    /// The toast fades in through the reduced-motion-aware helper in
    /// [`crate::motion`]: it snaps when animations are disabled, and requests a
    /// repaint only while the fade is in progress.
    pub fn show(self, ui: &mut Ui) -> Option<Response> {
        let now = ui.input(|input| input.time);
        let motion_id = ui.make_persistent_id("temivu-toast-reveal");
        if !self.state.visible {
            // Reset so a restarted toast fades in again instead of appearing.
            crate::motion::reset(ui.ctx(), motion_id);
            return None;
        }
        if self.state.timeout == Some(Duration::ZERO) {
            self.state.dismiss();
            crate::motion::reset(ui.ctx(), motion_id);
            return None;
        }
        let reveal = crate::motion::animate_f32(ui.ctx(), motion_id, 0.0, 1.0, self.theme);
        let scope = ui.scope(|ui| {
            if reveal < 1.0 {
                ui.multiply_opacity(reveal.max(0.0));
            }
            let padding = if self.theme.touch {
                Margin::symmetric(16, 14)
            } else {
                Margin::symmetric(16, 12)
            };
            let body_size = self.theme.typography.body.size;
            let (title_color, description_color) = match self.variant {
                ToastVariant::Primary => (self.theme.foreground, self.theme.muted_foreground),
                ToastVariant::Destructive => {
                    let destructive =
                        self.theme
                            .readable_on(self.theme.destructive, self.theme.card, 4.5);
                    (destructive, destructive)
                }
            };
            Frame::new()
                .fill(self.theme.card)
                .stroke(Stroke::new(1.0, self.theme.border))
                .corner_radius(10)
                .inner_margin(padding)
                .show(ui, |ui| {
                    // Forui `FToastStyle`: titleSpacing 2 (desktop) / 4 (touch).
                    ui.spacing_mut().item_spacing.y = if self.theme.touch { 4.0 } else { 2.0 };
                    ui.horizontal_wrapped(|ui| {
                        if let Some(icon) = self.icon {
                            let glyph = ui.add(
                                egui::Label::new(
                                    RichText::new(icon).size(body_size).color(title_color),
                                )
                                .selectable(false),
                            );
                            // The glyph approximates a decorative vector icon; hide its
                            // own text node so assistive technology does not announce it.
                            ui.ctx().accesskit_node_builder(glyph.id, |node| {
                                node.set_hidden();
                            });
                        }
                        ui.vertical(|ui| {
                            ui.label(RichText::new(self.title).size(body_size).color(title_color));
                            if let Some(description) = self.description {
                                ui.label(
                                    RichText::new(description)
                                        .size(body_size)
                                        .color(description_color),
                                );
                            }
                        });
                        let dismiss = ui.add(
                            egui::Button::new(self.dismiss_label).min_size(egui::Vec2::splat(32.0)),
                        );
                        if let Some(label) = self.dismiss_accessible_label {
                            ui.ctx().accesskit_node_builder(dismiss.id, |node| {
                                node.set_label(label);
                            });
                        }
                        if dismiss.clicked() {
                            self.state.dismiss();
                        }
                        dismiss.has_focus()
                    })
                    .inner
                })
        });
        let frame = scope.inner;
        let mut response = frame.response;
        let hovered = ui.input(|input| {
            input
                .pointer
                .hover_pos()
                .is_some_and(|pos| response.rect.contains(pos))
        });
        if let Some(delay) = self.state.tick(now, hovered || frame.inner) {
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
