//! Experimental normalized range slider for Forui's range slider controls.
//! Marks, arbitrary ranges and full parity remain open. Values are normalized
//! to `0.0..=1.0` and owned by the caller.
use crate::Theme;
use egui::{
    Align2, Color32, FontId, Key, Rect, Response, Sense, Stroke, Ui, Vec2, WidgetInfo, pos2, vec2,
};

/// A two-thumb normalized range slider.
pub struct RangeSlider<'a> {
    value: &'a mut (f64, f64),
    theme: &'a Theme,
    label: &'a str,
    enabled: bool,
    step: Option<f64>,
    id: Option<egui::Id>,
    vertical: bool,
}

impl<'a> RangeSlider<'a> {
    pub fn new(value: &'a mut (f64, f64), theme: &'a Theme) -> Result<Self, &'static str> {
        let (start, end) = *value;
        let inside = |value: f64| value.is_finite() && (0.0..=1.0).contains(&value);
        if !inside(start) || !inside(end) || start > end {
            return Err("range slider values must be finite, ordered and between zero and one");
        }
        Ok(Self {
            value,
            theme,
            label: "Range",
            enabled: true,
            step: None,
            id: None,
            vertical: false,
        })
    }

    pub fn label(mut self, label: &'a str) -> Self {
        self.label = label;
        self
    }

    pub fn enabled(mut self, enabled: bool) -> Self {
        self.enabled = enabled;
        self
    }

    /// Give this slider a stable egui ID. The default is derived from the label
    /// and the widget's position, so it stays unique for same-label sliders in
    /// one parent; an explicit ID is still required to address the widget from
    /// application state across layout changes.
    pub fn id(mut self, id: egui::Id) -> Self {
        self.id = Some(id);
        self
    }

    /// Lay the slider out vertically (minimum at the bottom, maximum at the top).
    pub fn vertical(mut self, vertical: bool) -> Self {
        self.vertical = vertical;
        self
    }

    /// Constrain values to uniform discrete intervals. Does not support marks.
    pub fn step(mut self, step: f64) -> Result<Self, &'static str> {
        if !step.is_finite() || !(0.0 < step && step <= 1.0) {
            return Err("range slider step must be positive and at most one");
        }
        self.step = Some(step);
        Ok(self)
    }

    /// Vertical columns are narrow: truncate a label that would overflow with an
    /// ellipsis so it cannot overlap neighbouring widgets.
    fn fit_label(&self, ui: &Ui, size: f32, color: Color32, max_width: f32) -> String {
        if !self.vertical || max_width <= 0.0 {
            return self.label.to_owned();
        }
        let font = FontId::proportional(size);
        let fits = |text: &str| {
            ui.fonts_mut(|fonts| {
                fonts
                    .layout_no_wrap(text.to_owned(), font.clone(), color)
                    .size()
                    .x
                    <= max_width
            })
        };
        if fits(self.label) {
            return self.label.to_owned();
        }
        // Binary-search the longest prefix that still fits with the ellipsis.
        let characters: Vec<char> = self.label.chars().collect();
        let (mut low, mut high) = (0usize, characters.len());
        let mut best = 0usize;
        while low < high {
            let middle = (low + high).div_ceil(2);
            let mut candidate: String = characters[..middle].iter().collect();
            candidate.push('…');
            if fits(&candidate) {
                best = middle;
                low = middle;
            } else {
                high = middle - 1;
            }
        }
        let mut truncated: String = characters[..best].iter().collect();
        truncated.push('…');
        truncated
    }

    pub fn show(self, ui: &mut Ui) -> Response {
        let theme = self.theme;
        let enabled = self.enabled && ui.is_enabled();
        let label_size = theme.typography.caption.size;
        let label_height = label_size + 6.0;
        let track_thickness = if theme.touch { 6.0 } else { 4.0 };
        let thumb_radius = if theme.touch { 11.0 } else { 9.0 };
        let (width, height) = if self.vertical {
            let height = ui.available_height().clamp(64.0, 240.0);
            let width = if theme.touch { 48.0 } else { 40.0 };
            (width, height)
        } else {
            let width = ui.available_width().clamp(96.0, 280.0);
            let height = label_height + track_thickness + 2.0 * (thumb_radius + 4.0);
            (width, height)
        };
        let (rect, mut response) = ui.allocate_exact_size(vec2(width, height), Sense::click());
        let id = self
            .id
            .unwrap_or_else(|| ui.auto_id_with(("temivu-range-slider", self.label)));
        let body = Rect::from_min_max(pos2(rect.left(), rect.top() + label_height), rect.max);
        let thumb_size = Vec2::splat(thumb_radius * 2.0 + 4.0);
        let track_rect = if self.vertical {
            Rect::from_center_size(
                body.center(),
                vec2(
                    track_thickness,
                    (body.height() - 2.0 * (thumb_radius + 4.0)).max(8.0),
                ),
            )
        } else {
            Rect::from_center_size(
                pos2(body.center().x, body.center().y),
                vec2(
                    (width - 2.0 * (thumb_radius + 4.0)).max(8.0),
                    track_thickness,
                ),
            )
        };
        let to_position = |value: f64| {
            if self.vertical {
                track_rect.bottom() - track_rect.height() * value as f32
            } else {
                track_rect.left() + track_rect.width() * value as f32
            }
        };
        let pointer_value = |position: egui::Pos2| {
            if self.vertical {
                let fraction =
                    ((track_rect.bottom() - position.y) / track_rect.height()).clamp(0.0, 1.0);
                f64::from(fraction)
            } else {
                let fraction =
                    ((position.x - track_rect.left()) / track_rect.width()).clamp(0.0, 1.0);
                f64::from(fraction)
            }
        };
        let snap = |value: f64| {
            let value = value.clamp(0.0, 1.0);
            self.step.map_or(value, |step| {
                ((value / step).round() * step).clamp(0.0, 1.0)
            })
        };
        let thumb_center = |value: f64| {
            if self.vertical {
                pos2(track_rect.center().x, to_position(value))
            } else {
                pos2(to_position(value), track_rect.center().y)
            }
        };

        let (mut start, mut end) = *self.value;
        let original = (start, end);
        let start_rect = Rect::from_center_size(thumb_center(start), thumb_size);
        let end_rect = Rect::from_center_size(thumb_center(end), thumb_size);
        let thumb_sense = if enabled {
            Sense::click_and_drag()
        } else {
            Sense::hover()
        };
        let start_response = ui.interact(start_rect, id.with("range-start"), thumb_sense);
        let end_response = ui.interact(end_rect, id.with("range-end"), thumb_sense);

        if enabled {
            // Clicking the track moves the nearer thumb to that position.
            if response.clicked()
                && let Some(position) = response.interact_pointer_pos()
                && body.contains(position)
            {
                let target = snap(pointer_value(position));
                let distance_start = (position - start_rect.center()).length();
                let distance_end = (position - end_rect.center()).length();
                if distance_start <= distance_end {
                    start = target.min(end);
                } else {
                    end = target.max(start);
                }
            }
            for (thumb, is_start) in [(&start_response, true), (&end_response, false)] {
                if thumb.dragged()
                    && let Some(position) = thumb.interact_pointer_pos()
                {
                    let target = snap(pointer_value(position));
                    if is_start {
                        start = target.min(end);
                    } else {
                        end = target.max(start);
                    }
                }
                let mut increase = 0usize;
                let mut decrease = 0usize;
                let mut home = false;
                let mut last = false;
                if thumb.has_focus() {
                    // Arrow keys adjust the value; they must not also move focus.
                    ui.memory_mut(|memory| {
                        memory.set_focus_lock_filter(
                            thumb.id,
                            egui::EventFilter {
                                horizontal_arrows: true,
                                vertical_arrows: true,
                                ..Default::default()
                            },
                        );
                    });
                    ui.input(|input| {
                        if input.key_pressed(Key::ArrowRight) || input.key_pressed(Key::ArrowUp) {
                            increase += 1;
                        }
                        if input.key_pressed(Key::ArrowLeft) || input.key_pressed(Key::ArrowDown) {
                            decrease += 1;
                        }
                        home = input.key_pressed(Key::Home);
                        last = input.key_pressed(Key::End);
                    });
                }
                ui.input(|input| {
                    increase += input.num_accesskit_action_requests(
                        thumb.id,
                        egui::accesskit::Action::Increment,
                    );
                    decrease += input.num_accesskit_action_requests(
                        thumb.id,
                        egui::accesskit::Action::Decrement,
                    );
                });
                if increase > 0 || decrease > 0 || home || last {
                    let step = self.step.unwrap_or(0.05);
                    let delta = increase as i64 - decrease as i64;
                    let mut updated = if is_start { start } else { end } + delta as f64 * step;
                    if home {
                        updated = 0.0;
                    }
                    if last {
                        updated = 1.0;
                    }
                    if is_start {
                        start = snap(updated).min(end);
                    } else {
                        end = snap(updated).max(start);
                    }
                }
            }
        }
        *self.value = (start, end);
        if (start, end) != original {
            response.mark_changed();
        }

        let painter = ui.painter();
        if !self.label.is_empty() {
            let label_color = if enabled {
                theme.foreground
            } else {
                theme.disable_blend(theme.foreground, theme.background)
            };
            let text = self.fit_label(ui, label_size, label_color, width - 8.0);
            painter.text(
                pos2(rect.left(), rect.top()),
                Align2::LEFT_TOP,
                text,
                FontId::proportional(label_size),
                label_color,
            );
        }
        let track_color = if enabled {
            theme.secondary
        } else {
            theme.disable_blend(theme.secondary, theme.background)
        };
        painter.rect_filled(track_rect, track_thickness / 2.0, track_color);
        let span_color = if enabled {
            theme.primary
        } else {
            theme.disable(theme.primary)
        };
        let (span_start, span_end) = (to_position(start), to_position(end));
        let span = if self.vertical {
            Rect::from_min_max(
                pos2(track_rect.left(), span_start.min(span_end)),
                pos2(track_rect.right(), span_start.max(span_end)),
            )
        } else {
            Rect::from_min_max(
                pos2(span_start.min(span_end), track_rect.top()),
                pos2(span_start.max(span_end), track_rect.bottom()),
            )
        };
        painter.rect_filled(span, track_thickness / 2.0, span_color);
        let focused = [
            (start_response.has_focus(), start),
            (end_response.has_focus(), end),
        ];
        for value in [start, end] {
            let center = thumb_center(value);
            painter.circle_filled(center, thumb_radius, theme.card);
            painter.circle_stroke(center, thumb_radius, Stroke::new(2.0, span_color));
        }
        if enabled {
            for (is_focused, value) in focused {
                if is_focused {
                    painter.circle_stroke(
                        thumb_center(value),
                        thumb_radius + 3.0,
                        Stroke::new(1.5, theme.primary),
                    );
                }
            }
        }

        let start_label = format!("{} start", self.label);
        let end_label = format!("{} end", self.label);
        start_response.widget_info(|| WidgetInfo::slider(enabled, start, start_label.as_str()));
        end_response.widget_info(|| WidgetInfo::slider(enabled, end, end_label.as_str()));
        for (thumb, value, label) in [
            (&start_response, start, start_label),
            (&end_response, end, end_label),
        ] {
            ui.ctx().accesskit_node_builder(thumb.id, |node| {
                node.set_label(label.as_str());
                node.set_min_numeric_value(0.0);
                node.set_max_numeric_value(1.0);
                node.set_numeric_value(value);
                node.set_orientation(if self.vertical {
                    egui::accesskit::Orientation::Vertical
                } else {
                    egui::accesskit::Orientation::Horizontal
                });
                if enabled {
                    node.add_action(egui::accesskit::Action::Increment);
                    node.add_action(egui::accesskit::Action::Decrement);
                } else {
                    node.clear_actions();
                }
            });
        }
        response
    }
}
