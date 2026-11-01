//! Preliminary Forui card port. Rounded clipping and superellipse parity are not implemented.
use crate::Theme;
use egui::{Color32, Frame, InnerResponse, Margin, Stroke, Ui};

/// Explicit card styling. The padding is a convenience for content builders:
/// as in Forui, the card itself does not apply it implicitly.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct CardStyle {
    pub fill: Color32,
    pub border: Stroke,
    pub corner_radius: u8,
    pub padding: Margin,
    pub title_color: Color32,
    pub subtitle_color: Color32,
}

impl CardStyle {
    pub fn from_theme(theme: &Theme) -> Self {
        Self {
            fill: theme.card,
            border: Stroke::new(1.0, theme.border),
            corner_radius: 14,
            padding: Margin::same(16),
            title_color: theme.foreground,
            subtitle_color: theme.muted_foreground,
        }
    }
}

/// Decorates arbitrary immediate-mode content without imposing padding.
pub struct Card {
    style: CardStyle,
}

impl Card {
    pub fn new(theme: &Theme) -> Self {
        Self {
            style: CardStyle::from_theme(theme),
        }
    }

    pub fn style(mut self, style: CardStyle) -> Self {
        self.style = style;
        self
    }

    /// Build the content once per frame; the style's padding is available to the builder.
    pub fn show<R>(
        self,
        ui: &mut Ui,
        content: impl FnOnce(&mut Ui, CardStyle) -> R,
    ) -> InnerResponse<R> {
        Frame::new()
            .fill(self.style.fill)
            .stroke(self.style.border)
            .corner_radius(self.style.corner_radius)
            .show(ui, |ui| content(ui, self.style))
    }
}
