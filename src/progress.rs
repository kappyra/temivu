//! Experimental Forui progress indicators. See coverage notes for missing parity.
use crate::Theme;
use egui::{Pos2, Rect, Response, Sense, Shape, Stroke, Ui, Vec2, WidgetInfo, WidgetType};

/// Whether indeterminate animations should run.
///
/// A zero egui `animation_time` also disables these animations, even when
/// `Enabled` is selected. Applications must set that preference themselves;
/// egui does not automatically read the operating system setting.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum Motion {
    #[default]
    Enabled,
    Disabled,
}

/// Linear progress, either determinate or indeterminate.
pub struct Progress<'a> {
    theme: &'a Theme,
    value: Option<f32>,
    label: &'a str,
    motion: Motion,
    width: Option<f32>,
    thickness: f32,
}

impl<'a> Progress<'a> {
    /// Reject values outside 0..=1 instead of silently clamping.
    pub fn determinate(theme: &'a Theme, value: f32) -> Result<Self, &'static str> {
        if !value.is_finite() || !(0.0..=1.0).contains(&value) {
            return Err("progress must be between zero and one");
        }
        Ok(Self {
            theme,
            value: Some(value),
            label: "Progress",
            motion: Motion::Enabled,
            width: None,
            thickness: 6.0,
        })
    }
    pub fn indeterminate(theme: &'a Theme) -> Self {
        Self {
            theme,
            value: None,
            label: "Progress",
            motion: Motion::Enabled,
            width: None,
            thickness: 6.0,
        }
    }
    pub fn label(mut self, label: &'a str) -> Self {
        self.label = label;
        self
    }
    pub fn motion(mut self, motion: Motion) -> Self {
        self.motion = motion;
        self
    }
    /// Set the track thickness in logical points.
    pub fn thickness(mut self, thickness: f32) -> Result<Self, &'static str> {
        if !thickness.is_finite() || !(1.0..=64.0).contains(&thickness) {
            return Err("thickness must be between 1 and 64 points");
        }
        self.thickness = thickness;
        Ok(self)
    }
    /// Override available width. Values must be finite and non-negative.
    pub fn width(mut self, width: f32) -> Result<Self, &'static str> {
        if !width.is_finite() || width < 0.0 {
            return Err("width must be finite and non-negative");
        }
        self.width = Some(width);
        Ok(self)
    }
    pub fn show(self, ui: &mut Ui) -> Response {
        let available = ui.available_width();
        let width = self.width.unwrap_or(if available.is_finite() {
            available.max(0.0)
        } else {
            0.0
        });
        let (rect, response) =
            ui.allocate_exact_size(Vec2::new(width, self.thickness), Sense::hover());
        response.widget_info(|| {
            let mut info =
                WidgetInfo::labeled(WidgetType::ProgressIndicator, ui.is_enabled(), self.label);
            info.value = self.value.map(|value| f64::from((value * 100.0).round()));
            info
        });
        if ui.is_rect_visible(rect) {
            let radius = (self.thickness / 2.0).round() as u8;
            ui.painter().rect_filled(rect, radius, self.theme.muted);
            let (start, end) = if let Some(value) = self.value {
                let displayed = if self.motion == Motion::Disabled {
                    crate::motion::reset(ui.ctx(), response.id.with("progress-fill"));
                    value
                } else {
                    crate::motion::animate_f32(
                        ui.ctx(),
                        response.id.with("progress-fill"),
                        value,
                        value,
                        self.theme,
                    )
                };
                (0.0, displayed.clamp(0.0, 1.0))
            } else if self.motion == Motion::Disabled || ui.style().animation_time <= 0.0 {
                (0.0, 0.4)
            } else {
                let phase = ui.input(|input| input.time.rem_euclid(1.5)) as f32;
                let start = (-0.4 + 1.4 * phase.min(1.0)).clamp(-0.4, 1.0);
                if width > 0.0 {
                    let wait = if phase < 1.0 {
                        std::time::Duration::from_millis(16)
                    } else {
                        std::time::Duration::from_secs_f32(1.5 - phase)
                    };
                    ui.ctx().request_repaint_after(wait);
                }
                (start.max(0.0), (start + 0.4).min(1.0))
            };
            if end > start {
                let rtl = ui.layout().prefer_right_to_left();
                let left = if rtl { 1.0 - end } else { start };
                let right = if rtl { 1.0 - start } else { end };
                let fill = Rect::from_min_max(
                    Pos2::new(rect.left() + width * left, rect.top()),
                    Pos2::new(rect.left() + width * right, rect.bottom()),
                );
                ui.painter().rect_filled(fill, radius, self.theme.primary);
            }
        }
        response
    }
}

/// Desktop/touch circular indicator sizes from the upstream typography defaults.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum CircularSize {
    ExtraSmall,
    Small,
    #[default]
    Medium,
    Large,
    ExtraLarge,
}
impl CircularSize {
    pub fn points(self, touch: bool) -> f32 {
        let desktop = match self {
            Self::ExtraSmall => 12.0,
            Self::Small => 14.0,
            Self::Medium => 16.0,
            Self::Large => 18.0,
            Self::ExtraLarge => 20.0,
        };
        desktop + if touch { 2.0 } else { 0.0 }
    }
}

/// An accessible progress ring for known values or indeterminate work.
pub struct CircularProgress<'a> {
    theme: &'a Theme,
    value: Option<f32>,
    diameter: Option<f32>,
    label: &'a str,
    size: CircularSize,
    touch: bool,
    motion: Motion,
}
impl<'a> CircularProgress<'a> {
    pub fn new(theme: &'a Theme) -> Self {
        Self {
            theme,
            value: None,
            diameter: None,
            label: "Progress",
            size: CircularSize::Medium,
            touch: false,
            motion: Motion::Enabled,
        }
    }
    /// Draw a ring filled in proportion to a finite value in 0..=1.
    pub fn determinate(theme: &'a Theme, value: f32) -> Result<Self, &'static str> {
        if !value.is_finite() || !(0.0..=1.0).contains(&value) {
            return Err("progress must be between zero and one");
        }
        Ok(Self {
            value: Some(value),
            ..Self::new(theme)
        })
    }
    /// Override the size token with a diameter in logical points.
    pub fn diameter(mut self, points: f32) -> Result<Self, &'static str> {
        if !points.is_finite() || !(8.0..=256.0).contains(&points) {
            return Err("diameter must be between 8 and 256 points");
        }
        self.diameter = Some(points);
        Ok(self)
    }
    pub fn label(mut self, label: &'a str) -> Self {
        self.label = label;
        self
    }
    pub fn size(mut self, size: CircularSize, touch: bool) -> Self {
        self.size = size;
        self.touch = touch;
        self
    }
    pub fn motion(mut self, motion: Motion) -> Self {
        self.motion = motion;
        self
    }
    pub fn show(self, ui: &mut Ui) -> Response {
        use std::f32::consts::{FRAC_PI_2, TAU};
        let size = self
            .diameter
            .unwrap_or_else(|| self.size.points(self.touch));
        let (rect, response) = ui.allocate_exact_size(Vec2::splat(size), Sense::hover());
        response.widget_info(|| {
            let mut info =
                WidgetInfo::labeled(WidgetType::ProgressIndicator, ui.is_enabled(), self.label);
            info.value = self.value.map(|value| f64::from((value * 100.0).round()));
            info
        });
        if !ui.is_rect_visible(rect) {
            return response;
        }
        let thickness = (size * 0.10).clamp(1.5, 9.0);
        let radius = (size - thickness) / 2.0;
        let center = rect.center();
        let painter = ui.painter();
        painter.circle_stroke(center, radius, Stroke::new(thickness, self.theme.secondary));
        let (start, sweep) = if let Some(value) = self.value {
            (-FRAC_PI_2, value * TAU)
        } else if self.motion == Motion::Enabled
            && ui.style().animation_time > 0.0
            && self.theme.motion.animations_enabled(ui.ctx())
        {
            ui.ctx()
                .request_repaint_after(std::time::Duration::from_millis(16));
            let phase = ui.input(|i| i.time.rem_euclid(1.25) / 1.25) as f32;
            let wave = 0.5 - 0.5 * (phase * TAU).cos();
            (phase * TAU - FRAC_PI_2, (0.20 + 0.53 * wave) * TAU)
        } else {
            (-FRAC_PI_2, TAU * 0.65)
        };
        let stroke = Stroke::new(thickness, self.theme.primary);
        if sweep >= TAU {
            painter.circle_stroke(center, radius, stroke);
        } else if sweep > 0.0 {
            let segments = (sweep * (radius / 4.0).max(5.0)).ceil() as usize;
            let point = |t: f32| center + Vec2::angled(start + sweep * t) * radius;
            let points = (0..=segments)
                .map(|i| point(i as f32 / segments as f32))
                .collect();
            painter.add(Shape::line(points, stroke));
            painter.circle_filled(point(0.0), thickness / 2.0, stroke.color);
            painter.circle_filled(point(1.0), thickness / 2.0, stroke.color);
        }
        response
    }
}
