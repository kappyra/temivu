//! Capsule and rail switches with spring-driven thumb travel.
use crate::{Theme, Vesper};
use egui::{FontId, Rect, Response, Sense, Stroke, Ui, Vec2, Widget, WidgetInfo, WidgetType};

/// Switch silhouette and thumb behavior.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum SwitchStyle {
    /// The thumb stretches across the capsule, then settles into a circle.
    #[default]
    Capsule,
    /// A circular thumb travels over a thin track.
    Rail,
}

/// A switch whose value is owned by the caller, with one focusable semantic node.
pub struct Switch<'a> {
    value: &'a mut bool,
    theme: &'a Theme,
    label: &'a str,
    description: Option<&'a str>,
    enabled: bool,
    leading_label: bool,
    active_color: Option<egui::Color32>,
    style: SwitchStyle,
    lighting: Option<Option<Vesper>>,
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
            active_color: None,
            style: SwitchStyle::Capsule,
            lighting: None,
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
    /// Override the selected track color.
    pub fn active_color(mut self, color: egui::Color32) -> Self {
        self.active_color = Some(color);
        self
    }
    pub fn leading_label(mut self, leading: bool) -> Self {
        self.leading_label = leading;
        self
    }
    pub fn style(mut self, style: SwitchStyle) -> Self {
        self.style = style;
        self
    }
    /// Override inherited lighting. Pass `None` to disable it for this switch.
    pub fn lighting(mut self, lighting: Option<Vesper>) -> Self {
        self.lighting = Some(lighting);
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
        // Both styles use the same layout and minimum touch target.
        let track_size = Vec2::new(51.0, 31.0);
        let track_radius = track_size.y / 2.0;

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
            track_size.y.max(text_height).max(44.0),
        );
        let sense = if enabled {
            Sense::click()
        } else {
            Sense::hover()
        };
        let (rect, mut response) = ui.allocate_exact_size(size, sense);
        if response.clicked() && enabled {
            *self.value = !*self.value;
            response.mark_changed();
            ui.ctx().request_repaint();
        }
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
            let amount = crate::motion::spring_bool(
                ui.ctx(),
                response.id.with("thumb"),
                *self.value,
                self.theme,
            )
            .clamp(0.0, 1.0);
            let pressed = crate::motion::spring_bool(
                ui.ctx(),
                response.id.with("thumb-size-press"),
                enabled
                    && (response.is_pointer_button_down_on()
                        || (response.has_focus()
                            && ui.input(|input| {
                                input.key_down(egui::Key::Space) || input.key_down(egui::Key::Enter)
                            }))),
                self.theme,
            )
            .clamp(0.0, 1.0);
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
            let rtl = ui.layout().prefer_right_to_left();
            let x = |offset| {
                if rtl {
                    track.right() - offset
                } else {
                    track.left() + offset
                }
            };
            let active = dim(self.active_color.unwrap_or(self.theme.primary));
            let thumb_fill = dim(self.theme.card);
            let painter = ui.painter();
            let lighting = enabled
                .then(|| self.lighting.unwrap_or_else(|| Vesper::context(ui.ctx())))
                .flatten();
            let light_track = |rect, radius, base: egui::Color32| {
                if let Some(style) = lighting {
                    let surface = base.lerp_to_gamma(active, amount);
                    style.paint_on(painter, self.theme, &response, rect, radius, surface);
                }
            };
            match self.style {
                SwitchStyle::Capsule => {
                    if amount < 1.0 {
                        painter.rect(
                            track,
                            track_radius,
                            dim(self.theme.secondary),
                            Stroke::new(
                                1.0,
                                dim(self.theme.muted_foreground.gamma_multiply(1.0 - amount)),
                            ),
                            egui::StrokeKind::Inside,
                        );
                    }
                    if amount > 0.0 {
                        let edge = x(track.width() * amount);
                        let clip = if rtl {
                            track.with_min_x(edge)
                        } else {
                            track.with_max_x(edge)
                        };
                        painter
                            .with_clip_rect(clip)
                            .rect_filled(track, track_radius, active);
                    }
                    light_track(
                        track,
                        (track_radius.round() as u8).into(),
                        dim(self.theme.secondary),
                    );
                    // Each edge travels separately, so the leading edge pulls
                    // the thumb into a capsule before the trailing edge catches up.
                    let ease = |start: f32, end: f32| {
                        let t: f32 = ((amount - start) / (end - start)).clamp(0.0, 1.0);
                        t * t * (3.0 - 2.0 * t)
                    };
                    let radius = 11.5;
                    let travel = track.width() - 2.0 * track_radius;
                    let near = x(track_radius - radius + travel * ease(0.36, 1.0));
                    let far = x(track_radius + radius + travel * ease(0.0, 0.64));
                    let half_height = radius - 1.2 * pressed;
                    if amount == 0.0 || amount == 1.0 {
                        let center = egui::pos2((near + far) / 2.0, track.center().y);
                        painter.circle(
                            center,
                            half_height,
                            thumb_fill,
                            Stroke::new(0.8, dim(self.theme.border)),
                        );
                    } else {
                        let thumb = Rect::from_center_size(
                            egui::pos2((near + far) / 2.0, track.center().y),
                            Vec2::new((near - far).abs(), 2.0 * half_height),
                        );
                        painter.rect_filled(thumb, half_height, thumb_fill);
                    }
                }
                SwitchStyle::Rail => {
                    let from = track_radius - 2.0;
                    let to = track.width() - from;
                    let knob = x(egui::lerp(from..=to, amount));
                    let rail = |start: f32, end: f32, color| {
                        let left = start.min(end);
                        painter.rect_filled(
                            Rect::from_min_size(
                                egui::pos2(left - 3.5, track.center().y - 3.5),
                                Vec2::new((start - end).abs() + 7.0, 7.0),
                            ),
                            4,
                            color,
                        );
                    };
                    rail(
                        x(from),
                        x(to),
                        dim(self.theme.muted_foreground.gamma_multiply(0.65)),
                    );
                    if amount > 0.0 {
                        rail(x(from), knob, active);
                    }
                    light_track(
                        Rect::from_min_max(
                            egui::pos2(x(from).min(x(to)) - 3.5, track.center().y - 3.5),
                            egui::pos2(x(from).max(x(to)) + 3.5, track.center().y + 3.5),
                        ),
                        4.into(),
                        dim(self.theme.muted_foreground.gamma_multiply(0.65)),
                    );
                    let center = egui::pos2(knob, track.center().y);
                    let radius = 13.0 - 1.3 * pressed;
                    painter.circle_filled(
                        center + egui::vec2(0.0, 1.5),
                        radius + 1.0,
                        egui::Color32::from_black_alpha(28),
                    );
                    painter.circle(
                        center,
                        radius,
                        thumb_fill,
                        Stroke::new(1.0, dim(self.theme.border)),
                    );
                }
            }
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
        response
    }
}
