//! A first, intentionally incomplete port of Forui's button.
use crate::{Sizes, Theme};
use egui::{Color32, Response, RichText, Stroke, Ui, Vec2};

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum ButtonVariant {
    #[default]
    Primary,
    Secondary,
    Destructive,
    Outline,
    Ghost,
}
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum ButtonSize {
    ExtraSmall,
    Small,
    #[default]
    Medium,
    Large,
}
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum InputMode {
    #[default]
    Desktop,
    Touch,
}

impl ButtonSize {
    /// Content padding, matching Forui's `FButtonStyle` contentPadding.
    ///
    /// Desktop: xs 8/6, sm 10/9, md 10/11, lg 10/13 (horizontal/vertical).
    /// Touch: xs 10/9, sm 12/12, md 12/14, lg 12/16.
    pub fn padding(self, mode: InputMode) -> Vec2 {
        match (mode, self) {
            (InputMode::Desktop, Self::ExtraSmall) => Vec2::new(8.0, 6.0),
            (InputMode::Desktop, Self::Small) => Vec2::new(10.0, 9.0),
            (InputMode::Desktop, Self::Medium) => Vec2::new(10.0, 11.0),
            (InputMode::Desktop, Self::Large) => Vec2::new(10.0, 13.0),
            (InputMode::Touch, Self::ExtraSmall) => Vec2::new(10.0, 9.0),
            (InputMode::Touch, Self::Small) => Vec2::new(12.0, 12.0),
            (InputMode::Touch, Self::Medium) => Vec2::new(12.0, 14.0),
            (InputMode::Touch, Self::Large) => Vec2::new(12.0, 16.0),
        }
    }

    /// Minimum button height in logical pixels, matching upstream size tokens.
    pub fn height(self, mode: InputMode) -> f32 {
        let field = match mode {
            InputMode::Desktop => Sizes::desktop().field,
            InputMode::Touch => Sizes::touch().field,
        };
        match self {
            Self::ExtraSmall => field.xs,
            Self::Small => field.sm,
            Self::Medium => field.md,
            Self::Large => field.lg,
        }
    }
}

/// A text button. Response exposes egui's click, focus and pointer state.
/// Complex content, selected state, long-press and full style parity remain open.
pub struct Button<'a> {
    text: &'a str,
    theme: &'a Theme,
    variant: ButtonVariant,
    size: ButtonSize,
    input: InputMode,
    enabled: bool,
    icon: Option<&'a str>,
}

impl<'a> Button<'a> {
    pub fn new(text: &'a str, theme: &'a Theme) -> Self {
        Self {
            text,
            theme,
            variant: ButtonVariant::Primary,
            size: ButtonSize::Medium,
            input: InputMode::Desktop,
            enabled: true,
            icon: None,
        }
    }
    pub fn variant(mut self, variant: ButtonVariant) -> Self {
        self.variant = variant;
        self
    }
    pub fn size(mut self, size: ButtonSize) -> Self {
        self.size = size;
        self
    }
    pub fn input_mode(mut self, input: InputMode) -> Self {
        self.input = input;
        self
    }
    pub fn enabled(mut self, enabled: bool) -> Self {
        self.enabled = enabled;
        self
    }
    /// An optional leading icon rendered as a text glyph. This approximates the
    /// upstream vector icon, and the glyph must exist in the active font (it is
    /// rendered as a missing-glyph box otherwise). The button's accessible name
    /// is overridden to the visible label text, so the glyph is not announced.
    pub fn icon(mut self, icon: &'a str) -> Self {
        self.icon = Some(icon);
        self
    }
    pub fn show(self, ui: &mut Ui) -> Response {
        let (fill, foreground, border) = match self.variant {
            ButtonVariant::Primary => (
                self.theme.primary,
                self.theme.primary_foreground,
                Color32::TRANSPARENT,
            ),
            ButtonVariant::Secondary => (
                self.theme.secondary,
                self.theme.secondary_foreground,
                Color32::TRANSPARENT,
            ),
            ButtonVariant::Destructive => {
                let [r, g, b, _] = self.theme.destructive.to_srgba_unmultiplied();
                let alpha = if self.theme.dark { 51 } else { 26 };
                (
                    Color32::from_rgba_unmultiplied(r, g, b, alpha),
                    self.theme.destructive,
                    Color32::TRANSPARENT,
                )
            }
            ButtonVariant::Outline => (
                self.theme.card,
                self.theme.secondary_foreground,
                self.theme.border,
            ),
            ButtonVariant::Ghost => (
                Color32::TRANSPARENT,
                self.theme.secondary_foreground,
                Color32::TRANSPARENT,
            ),
        };
        let height = self.size.height(self.input);
        let radius = if self.size == ButtonSize::ExtraSmall {
            8
        } else {
            10
        };
        let hover_fill = match self.variant {
            ButtonVariant::Primary | ButtonVariant::Secondary => self.theme.hover(fill),
            ButtonVariant::Destructive => {
                let [r, g, b, _] = self.theme.destructive.to_srgba_unmultiplied();
                Color32::from_rgba_unmultiplied(r, g, b, if self.theme.dark { 77 } else { 51 })
            }
            ButtonVariant::Outline | ButtonVariant::Ghost => self.theme.secondary,
        };
        let content = if let Some(icon) = self.icon {
            let mut job = egui::text::LayoutJob::default();
            let format = egui::TextFormat {
                font_id: egui::FontId::proportional(self.theme.typography.body.size),
                color: foreground,
                ..Default::default()
            };
            job.append(icon, 0.0, format.clone());
            job.append(" ", 0.0, format.clone());
            job.append(self.text, 0.0, format);
            egui::WidgetText::from(job)
        } else {
            egui::WidgetText::from(
                RichText::new(self.text)
                    .size(self.theme.typography.body.size)
                    .color(foreground),
            )
        };
        let button = egui::Button::new(content)
            .frame(true)
            .corner_radius(radius)
            .min_size(Vec2::splat(height));
        let response = ui
            .scope(|ui| {
                ui.spacing_mut().interact_size.y = height;
                ui.spacing_mut().button_padding = self.size.padding(self.input);
                let widgets = &mut ui.visuals_mut().widgets;
                widgets.inactive.weak_bg_fill = fill;
                widgets.hovered.weak_bg_fill = hover_fill;
                widgets.active.weak_bg_fill = hover_fill;
                widgets.noninteractive.weak_bg_fill = fill;
                for visual in [
                    &mut widgets.inactive,
                    &mut widgets.hovered,
                    &mut widgets.active,
                    &mut widgets.noninteractive,
                ] {
                    visual.bg_stroke = Stroke::new(1.0, border);
                }
                ui.add_enabled(self.enabled, button)
            })
            .inner;
        if self.icon.is_some() {
            // Keep the accessible name identical to the visible label text.
            ui.ctx().accesskit_node_builder(response.id, |node| {
                node.set_label(self.text);
            });
        }
        if response.has_focus() {
            ui.painter().rect_stroke(
                response.rect.expand(2.0),
                radius,
                Stroke::new(2.0, self.theme.foreground),
                egui::StrokeKind::Outside,
            );
        }
        response
    }
}
