//! Experimental port of Forui's badge. Upstream style deltas and visual parity
//! are not yet implemented; see `docs/COVERAGE.md` before relying on this API.

use crate::Theme;
use egui::{Color32, Frame, InnerResponse, Label, Margin, Response, RichText, Stroke, Ui};

/// The four upstream badge color variants.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum BadgeVariant {
    #[default]
    Primary,
    Secondary,
    Outline,
    Destructive,
}

/// An explicit badge style. Customizing these values does not mutate the theme.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct BadgeStyle {
    pub fill: Color32,
    pub foreground: Color32,
    pub stroke: Stroke,
    pub padding: Margin,
}

impl BadgeStyle {
    /// Resolve the default colors and desktop or touch padding from a theme.
    pub fn from_theme(theme: &Theme, variant: BadgeVariant, touch: bool) -> Self {
        let (fill, foreground, border) = match variant {
            BadgeVariant::Primary => (
                theme.primary,
                theme.primary_foreground,
                Color32::TRANSPARENT,
            ),
            BadgeVariant::Secondary => (
                theme.secondary,
                theme.secondary_foreground,
                Color32::TRANSPARENT,
            ),
            BadgeVariant::Outline => (theme.card, theme.foreground, theme.border),
            BadgeVariant::Destructive => {
                // Forui replaces alpha rather than multiplying the original alpha.
                let [red, green, blue, _] = theme.destructive.to_srgba_unmultiplied();
                let alpha = if theme.dark { 51 } else { 26 };
                (
                    Color32::from_rgba_unmultiplied(red, green, blue, alpha),
                    theme.destructive,
                    Color32::TRANSPARENT,
                )
            }
        };
        Self {
            fill,
            foreground,
            stroke: Stroke::new(
                if variant == BadgeVariant::Outline {
                    1.0
                } else {
                    0.0
                },
                border,
            ),
            padding: Margin::symmetric(if touch { 12 } else { 10 }, 6),
        }
    }

    fn frame(self) -> Frame {
        Frame::new()
            .fill(self.fill)
            .stroke(self.stroke)
            .corner_radius(255)
    }
}

/// Non-interactive, text-based badge. The returned response describes the outer frame.
pub struct Badge<'a> {
    text: &'a str,
    style: BadgeStyle,
}

impl<'a> Badge<'a> {
    pub fn new(text: &'a str, theme: &Theme) -> Self {
        Self {
            text,
            style: BadgeStyle::from_theme(theme, BadgeVariant::Primary, false),
        }
    }

    /// Replace the resolved style; use [`BadgeStyle::from_theme`] to select a variant.
    pub fn style(mut self, style: BadgeStyle) -> Self {
        self.style = style;
        self
    }

    pub fn show(self, ui: &mut Ui) -> Response {
        self.style
            .frame()
            .inner_margin(self.style.padding)
            .show(ui, |ui| {
                ui.add(Label::new(
                    RichText::new(self.text)
                        .size(12.0)
                        .color(self.style.foreground),
                ))
            })
            .response
    }
}

/// Render custom badge contents without implicit padding, like Forui's raw builder.
pub fn show_raw<R>(
    ui: &mut Ui,
    style: BadgeStyle,
    contents: impl FnOnce(&mut Ui, BadgeStyle) -> R,
) -> InnerResponse<R> {
    style.frame().show(ui, |ui| contents(ui, style))
}
