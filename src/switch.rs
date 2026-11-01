//! Forui switch port. Drag gestures, labels with errors and animation parity
//! remain open; the visual geometry follows Forui's `CupertinoSwitch` defaults.
use crate::Theme;
use egui::{FontId, Rect, Response, Sense, Stroke, Ui, Vec2, Widget, WidgetInfo, WidgetType};

/// A switch whose value is owned by the caller, with one focusable semantic node.
pub struct Switch<'a> {
    value: &'a mut bool,
    theme: &'a Theme,
    label: &'a str,
    description: Option<&'a str>,
    enabled: bool,
    leading_label: bool,
}
impl<'a> Switch<'a> {
    pub fn new(value: &'a mut bool, theme: &'a Theme) -> Self {
        Self {
            value,
            theme,
            label: "Switch",
            description: None,
            enabled: true,
            leading_label: false,
        }
    }
    pub fn label(mut self, label: &'a str) -> Self {
        self.label = label;
        self
    }
    pub fn description(mut self, description: &'a str) -> Self {
        self.description = Some(description);
        self
    }
    pub fn enabled(mut self, enabled: bool) -> Self {
        self.enabled = enabled;
        self
    }
    pub fn leading_label(mut self, leading: bool) -> Self {
        self.leading_label = leading;
        self
    }
    pub fn show(self, ui: &mut Ui) -> Response {
        ui.add(self)
    }
}
impl Widget for Switch<'_> {
    fn ui(self, ui: &mut Ui) -> Response {
        let ui_enabled = ui.is_enabled();
        let enabled = self.enabled && ui_enabled;
        // Forui's CupertinoSwitch geometry: a 51x31 track, a 27 point thumb with
        // a 2 point inset and a pill radius.
        let track_size = Vec2::new(51.0, 31.0);
        let track_radius = track_size.y / 2.0;
        let thumb_radius = 13.5;
        let label_font = FontId::proportional(self.theme.typography.body.size);
        let detail_font = FontId::proportional(self.theme.typography.caption.size);
        let max_text_width = (ui.available_width() - track_size.x - 8.0).max(1.0);
        let label = ui.painter().layout(
            self.label.to_owned(),
            label_font,
            self.theme.foreground,
            max_text_width,
        );
        let detail = self.description.map(|text| {
            ui.painter().layout(
                text.to_owned(),
                detail_font,
                self.theme.muted_foreground,
                max_text_width,
            )
        });
        let text_width = detail
            .as_ref()
            .map_or(label.size().x, |g| label.size().x.max(g.size().x));
        let text_height = label.size().y + detail.as_ref().map_or(0.0, |g| 2.0 + g.size().y);
        let size = Vec2::new(
            track_size.x + 8.0 + text_width,
            track_size.y.max(text_height),
        );
        let sense = if enabled {
            Sense::click()
        } else {
            Sense::hover()
        };
        let (rect, mut response) = ui.allocate_exact_size(size, sense);
        response.widget_info(|| {
            WidgetInfo::selected(WidgetType::Checkbox, enabled, *self.value, self.label)
        });
        ui.ctx().accesskit_node_builder(response.id, |node| {
            node.set_role(egui::accesskit::Role::Switch);
            if let Some(description) = self.description {
                node.set_description(description);
            }
        });
        if ui.is_rect_visible(rect) {
            // The disabled variants follow `FColors.disable`, except when the
            // whole `Ui` is already dimmed by egui, in which case dimming again
            // would halve the contrast twice.
            let dim = |color: egui::Color32| {
                if !self.enabled && ui_enabled {
                    self.theme.disable(color)
                } else {
                    color
                }
            };
            let track_fill = dim(if *self.value {
                self.theme.primary
            } else {
                self.theme.secondary
            });
            let thumb_fill = dim(if self.theme.dark {
                self.theme.foreground
            } else {
                self.theme.background
            });
            let left_label = self.leading_label ^ ui.layout().prefer_right_to_left();
            let track_left = if left_label {
                rect.right() - track_size.x
            } else {
                rect.left()
            };
            let track = Rect::from_min_size(
                egui::pos2(track_left, rect.center().y - track_size.y / 2.0),
                track_size,
            );
            ui.painter().rect_filled(track, track_radius, track_fill);
            if !*self.value {
                // Upstream has no outline; the port keeps one so the unselected
                // track keeps 3:1 non-text contrast against the background.
                ui.painter().rect_stroke(
                    track,
                    track_radius,
                    Stroke::new(1.0, self.theme.muted_foreground),
                    egui::StrokeKind::Inside,
                );
            }
            let x = if *self.value ^ ui.layout().prefer_right_to_left() {
                track.right() - track_radius
            } else {
                track.left() + track_radius
            };
            ui.painter()
                .circle_filled(egui::pos2(x, track.center().y), thumb_radius, thumb_fill);
            if response.has_focus() {
                ui.painter().rect_stroke(
                    track.expand(2.0),
                    track_radius + 2.0,
                    Stroke::new(2.0, self.theme.primary),
                    egui::StrokeKind::Outside,
                );
            }
            let text_x = if left_label {
                rect.left()
            } else {
                track.right() + 8.0
            };
            let top = rect.center().y - text_height / 2.0;
            ui.painter().galley(
                egui::pos2(text_x, top),
                label.clone(),
                self.theme.foreground,
            );
            if let Some(detail) = detail {
                ui.painter().galley(
                    egui::pos2(text_x, top + label.size().y + 2.0),
                    detail,
                    self.theme.muted_foreground,
                );
            }
        }
        if response.clicked() && enabled {
            *self.value = !*self.value;
            response.mark_changed();
        }
        response
    }
}
