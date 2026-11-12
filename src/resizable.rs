//! Preliminary two-pane resizable split. More than two panes, RTL, touch
//! gestures, animation and full parity remain open.
use crate::Theme;
use egui::{CursorIcon, EventFilter, Id, Key, Modifiers, Rect, Response, Sense, Stroke, Ui};

/// The axis along which the two panes are arranged.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum ResizableAxis {
    /// The first pane is on the left and the second on the right.
    #[default]
    Horizontal,
    /// The first pane is on the top and the second on the bottom.
    Vertical,
}

/// The result of showing a [`Resizable`].
pub struct ResizableResponse {
    /// The divider's response. It senses drags and keyboard focus.
    pub divider: Response,
    /// The first pane's extent along the main axis, in points.
    pub first_extent: f32,
    /// The second pane's extent along the main axis, in points.
    pub second_extent: f32,
    /// Whether a drag or arrow key changed the split this frame.
    pub changed: bool,
}

/// A caller-owned two-pane split with a draggable, keyboard-resizable divider.
///
/// `fraction` is the first pane's share of the space available to both panes
/// (excluding the divider), clamped to the configured minimum extents. Pass a
/// stable, globally unique ID. The divider is focusable; when focused, the
/// arrow keys on the main axis move it. Its accessibility value reports both
/// pane extents.
///
/// ```
/// # fn example(ui: &mut egui::Ui, theme: &temivu::Theme, fraction: &mut f32) {
/// use temivu::{Resizable, ResizableAxis};
/// Resizable::new(
///     egui::Id::new("editor-split"),
///     ResizableAxis::Horizontal,
///     fraction,
///     theme,
/// )
/// .min_extents(120.0, 160.0)
/// .show(
///     ui,
///     |ui| {
///         ui.label("Outline");
///     },
///     |ui| {
///         ui.label("Preview");
///     },
/// );
/// # }
/// ```
pub struct Resizable<'a> {
    id: Id,
    axis: ResizableAxis,
    fraction: &'a mut f32,
    min_first: f32,
    min_second: f32,
    divider_extent: f32,
    keyboard_step: Option<f32>,
    track: bool,
    label: &'a str,
    theme: &'a Theme,
}

impl<'a> Resizable<'a> {
    pub fn new(id: Id, axis: ResizableAxis, fraction: &'a mut f32, theme: &'a Theme) -> Self {
        Self {
            id,
            axis,
            fraction,
            min_first: 0.0,
            min_second: 0.0,
            divider_extent: 11.0,
            keyboard_step: None,
            track: true,
            label: "Resize panes",
            theme,
        }
    }

    /// The minimum extents of the two panes, in points. Defaults to zero.
    pub fn min_extents(mut self, first: f32, second: f32) -> Self {
        self.min_first = first;
        self.min_second = second;
        self
    }

    /// The divider's interactive thickness, in points. Defaults to 11.
    pub fn divider_extent(mut self, extent: f32) -> Self {
        self.divider_extent = extent;
        self
    }

    /// Show the line along the divider track. The handle is always available.
    pub fn track(mut self, visible: bool) -> Self {
        self.track = visible;
        self
    }

    /// How far an arrow key moves the divider in points. Defaults to half a
    /// percent of the panes' combined extent, but at least one point.
    ///
    /// Non-finite or non-positive values are ignored so they cannot poison the
    /// derived extents and rectangles.
    pub fn keyboard_step(mut self, step: f32) -> Self {
        if step.is_finite() && step > 0.0 {
            self.keyboard_step = Some(step);
        }
        self
    }

    /// The divider's accessible label. Defaults to "Resize panes".
    pub fn label(mut self, label: &'a str) -> Self {
        self.label = label;
        self
    }

    pub fn show(
        self,
        ui: &mut Ui,
        first: impl FnOnce(&mut Ui),
        second: impl FnOnce(&mut Ui),
    ) -> ResizableResponse {
        let mut rect = ui.available_rect_before_wrap();
        // Use the clip only as a fallback for unbounded layouts. A caller's
        // finite pane size must stay stable as it scrolls past the clip.
        let clip = ui.clip_rect();
        if !rect.max.x.is_finite() {
            rect.max.x = clip.max.x.max(rect.min.x);
        }
        if !rect.max.y.is_finite() {
            rect.max.y = clip.max.y.max(rect.min.y);
        }
        let total = match self.axis {
            ResizableAxis::Horizontal => rect.width(),
            ResizableAxis::Vertical => rect.height(),
        };
        let divider_extent = self.divider_extent.max(1.0).min(total.max(1.0));
        let range = (total - divider_extent).max(0.0);
        // Scale both minimums proportionally when the window is too small to
        // satisfy them, instead of collapsing one pane entirely.
        let requested_min = self.min_first.max(0.0) + self.min_second.max(0.0);
        let scale = if requested_min > range && requested_min > 0.0 {
            range / requested_min
        } else {
            1.0
        };
        let min_first = (self.min_first.max(0.0) * scale).min(range);
        let min_second = (self.min_second.max(0.0) * scale).min(range - min_first);
        let max_first = (range - min_second).max(min_first);
        let fraction = if self.fraction.is_finite() {
            *self.fraction
        } else {
            0.5
        };
        let mut first_extent = (fraction * range).clamp(min_first, max_first);

        let (rect, _) = ui.allocate_exact_size(rect.size(), Sense::hover());
        let (_, divider_rect, _) = split(rect, self.axis, first_extent, divider_extent);
        let divider_id = self.id.with("temivu-resizable-divider");
        let mut divider = ui.interact(divider_rect, divider_id, Sense::click_and_drag());
        if divider.hovered() || divider.dragged() {
            ui.ctx().set_cursor_icon(match self.axis {
                ResizableAxis::Horizontal => CursorIcon::ResizeHorizontal,
                ResizableAxis::Vertical => CursorIcon::ResizeVertical,
            });
        }

        let mut changed = false;
        if divider.dragged() {
            let delta = match self.axis {
                ResizableAxis::Horizontal => divider.drag_delta().x,
                ResizableAxis::Vertical => divider.drag_delta().y,
            };
            if delta != 0.0 && range > 0.0 {
                first_extent = (first_extent + delta).clamp(min_first, max_first);
                *self.fraction = first_extent / range;
                changed = true;
            }
        }

        if divider.has_focus() {
            ui.memory_mut(|memory| {
                memory.set_focus_lock_filter(
                    divider_id,
                    EventFilter {
                        horizontal_arrows: self.axis == ResizableAxis::Horizontal,
                        vertical_arrows: self.axis == ResizableAxis::Vertical,
                        ..Default::default()
                    },
                );
            });
            let step = self.keyboard_step.unwrap_or((range * 0.005).max(1.0));
            let delta = ui.input_mut(|input| match self.axis {
                ResizableAxis::Horizontal => {
                    if input.consume_key(Modifiers::NONE, Key::ArrowLeft) {
                        Some(-step)
                    } else if input.consume_key(Modifiers::NONE, Key::ArrowRight) {
                        Some(step)
                    } else {
                        None
                    }
                }
                ResizableAxis::Vertical => {
                    if input.consume_key(Modifiers::NONE, Key::ArrowUp) {
                        Some(-step)
                    } else if input.consume_key(Modifiers::NONE, Key::ArrowDown) {
                        Some(step)
                    } else {
                        None
                    }
                }
            });
            if let Some(delta) = delta
                && range > 0.0
            {
                first_extent = (first_extent + delta).clamp(min_first, max_first);
                *self.fraction = first_extent / range;
                changed = true;
            }
        }
        if changed {
            divider.mark_changed();
        }

        let (first_rect, divider_rect, second_rect) =
            split(rect, self.axis, first_extent, divider_extent);
        ui.scope_builder(egui::UiBuilder::new().max_rect(first_rect), |ui| {
            ui.set_clip_rect(ui.clip_rect().intersect(first_rect));
            ui.set_min_size(first_rect.size());
            first(ui);
        });
        ui.scope_builder(egui::UiBuilder::new().max_rect(second_rect), |ui| {
            ui.set_clip_rect(ui.clip_rect().intersect(second_rect));
            ui.set_min_size(second_rect.size());
            second(ui);
        });

        self.paint_divider(ui, divider_rect, &divider);
        ui.ctx().accesskit_node_builder(divider.id, |node| {
            node.set_role(egui::accesskit::Role::Slider);
            node.set_label(self.label);
            node.set_value(format!("{first_extent:.0}, {:.0}", range - first_extent));
            node.set_numeric_value(first_extent as f64);
            node.set_min_numeric_value(min_first as f64);
            node.set_max_numeric_value(max_first as f64);
        });

        ResizableResponse {
            divider,
            first_extent,
            second_extent: range - first_extent,
            changed,
        }
    }

    fn paint_divider(&self, ui: &Ui, rect: Rect, divider: &Response) {
        let active = divider.has_focus() || divider.hovered() || divider.dragged();
        let color = if active {
            self.theme.foreground
        } else {
            self.theme.border
        };
        let painter = ui.painter();
        let reveal =
            crate::motion::spring_bool(ui.ctx(), divider.id.with("handle"), active, self.theme)
                .clamp(0.0, 1.0);
        let short = egui::lerp(3.0..=5.0, reveal);
        let long = egui::lerp(22.0..=29.0, reveal);
        let (track_size, handle_size) = match self.axis {
            ResizableAxis::Horizontal => (egui::vec2(1.0, rect.height()), egui::vec2(short, long)),
            ResizableAxis::Vertical => (egui::vec2(rect.width(), 1.0), egui::vec2(long, short)),
        };
        let painter = painter.with_clip_rect(painter.clip_rect().intersect(rect));
        if self.track {
            painter.rect_filled(
                Rect::from_center_size(rect.center(), track_size),
                0,
                self.theme.border,
            );
        }
        let handle = Rect::from_center_size(rect.center(), handle_size);
        painter.rect_filled(handle, 3, color);
        if let Some(light) = crate::Vesper::context(ui.ctx()) {
            light.paint(&painter, self.theme, divider, handle, 3.into());
        }
        if divider.has_focus() {
            painter.rect_stroke(
                rect.shrink(1.0),
                2,
                Stroke::new(2.0, self.theme.foreground),
                egui::StrokeKind::Inside,
            );
        }
    }
}

/// Split `rect` into first pane, divider and second pane.
fn split(
    rect: Rect,
    axis: ResizableAxis,
    first_extent: f32,
    divider_extent: f32,
) -> (Rect, Rect, Rect) {
    match axis {
        ResizableAxis::Horizontal => {
            let split = rect.min.x + first_extent;
            let first = Rect::from_min_max(rect.min, egui::pos2(split, rect.max.y));
            let divider = Rect::from_min_max(
                egui::pos2(split, rect.min.y),
                egui::pos2(split + divider_extent, rect.max.y),
            );
            let second =
                Rect::from_min_max(egui::pos2(split + divider_extent, rect.min.y), rect.max);
            (first, divider, second)
        }
        ResizableAxis::Vertical => {
            let split = rect.min.y + first_extent;
            let first = Rect::from_min_max(rect.min, egui::pos2(rect.max.x, split));
            let divider = Rect::from_min_max(
                egui::pos2(rect.min.x, split),
                egui::pos2(rect.max.x, split + divider_extent),
            );
            let second =
                Rect::from_min_max(egui::pos2(rect.min.x, split + divider_extent), rect.max);
            (first, divider, second)
        }
    }
}
