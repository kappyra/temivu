//! Experimental Forui divider port. Golden-image parity is not yet verified.
use crate::Theme;
use egui::{Color32, Margin, Rect, Response, Sense, Ui, Vec2};

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum DividerAxis {
    #[default]
    Horizontal,
    Vertical,
}

/// Divider paint and margins. A zero, infinite, or NaN thickness is rejected.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct DividerStyle {
    pub color: Color32,
    pub padding: Margin,
    width: f32,
}

impl DividerStyle {
    pub fn new(theme: &Theme, axis: DividerAxis) -> Self {
        Self {
            color: theme.secondary,
            padding: match axis {
                DividerAxis::Horizontal => Margin::symmetric(0, 16),
                DividerAxis::Vertical => Margin::symmetric(16, 0),
            },
            width: 1.0,
        }
    }

    /// Set a positive, finite line thickness; invalid values leave the style unchanged.
    pub fn set_width(&mut self, width: f32) -> Result<(), &'static str> {
        if !width.is_finite() || width <= 0.0 {
            return Err("divider width must be positive and finite");
        }
        self.width = width;
        Ok(())
    }

    pub fn width(&self) -> f32 {
        self.width
    }
}

/// A decorative, non-interactive line with optional explicit length.
/// Set a length for vertical dividers when their parent has an unbounded height.
pub struct Divider {
    axis: DividerAxis,
    style: DividerStyle,
    length: Option<f32>,
}

impl Divider {
    pub fn new(theme: &Theme, axis: DividerAxis) -> Self {
        Self {
            axis,
            style: DividerStyle::new(theme, axis),
            length: None,
        }
    }

    pub fn style(mut self, style: DividerStyle) -> Self {
        self.style = style;
        self
    }

    /// Returns an error for a negative or non-finite length.
    pub fn set_length(&mut self, length: f32) -> Result<(), &'static str> {
        if !length.is_finite() || length < 0.0 {
            return Err("divider length must be non-negative and finite");
        }
        self.length = Some(length);
        Ok(())
    }

    pub fn show(self, ui: &mut Ui) -> Response {
        let available = match self.axis {
            DividerAxis::Horizontal => ui.available_width(),
            DividerAxis::Vertical => ui.available_height(),
        };
        // Avoid handing egui an unbounded rectangle if the layout offers infinity.
        let padding = self.style.padding;
        let along_padding = match self.axis {
            DividerAxis::Horizontal => f32::from(padding.left) + f32::from(padding.right),
            DividerAxis::Vertical => f32::from(padding.top) + f32::from(padding.bottom),
        };
        let length = self.length.unwrap_or(if available.is_finite() {
            (available - along_padding).max(0.0)
        } else {
            0.0
        });
        let (left, right, top, bottom) = (
            f32::from(padding.left),
            f32::from(padding.right),
            f32::from(padding.top),
            f32::from(padding.bottom),
        );
        let line_size = match self.axis {
            DividerAxis::Horizontal => Vec2::new(length, self.style.width),
            DividerAxis::Vertical => Vec2::new(self.style.width, length),
        };
        let size = Vec2::new(
            (line_size.x + left + right).max(0.0),
            (line_size.y + top + bottom).max(0.0),
        );
        let (rect, response) = ui.allocate_exact_size(size, Sense::hover());
        let line_rect = Rect::from_min_size(rect.min + Vec2::new(left, top), line_size);
        ui.painter().rect_filled(line_rect, 0, self.style.color);
        response
    }
}
