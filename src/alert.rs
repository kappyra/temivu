//! Experimental Forui alert port. Visual and live-region parity remain under review.
use crate::Theme;
use egui::{Color32, Frame, Margin, Response, RichText, Stroke, Ui, WidgetInfo, WidgetType};

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum AlertVariant {
    #[default]
    Primary,
    Destructive,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct AlertStyle {
    pub fill: Color32,
    pub border: Color32,
    pub title: Color32,
    pub subtitle: Color32,
    pub icon: Color32,
    pub padding: Margin,
}
impl AlertStyle {
    pub fn from_theme(theme: &Theme, variant: AlertVariant) -> Self {
        let (title, subtitle, icon) = match variant {
            AlertVariant::Primary => (theme.foreground, theme.muted_foreground, theme.foreground),
            AlertVariant::Destructive => (theme.destructive, theme.destructive, theme.destructive),
        };
        Self {
            fill: theme.card,
            border: theme.border,
            title,
            subtitle,
            icon,
            padding: Margin::symmetric(16, 12),
        }
    }
}

/// Text-based alert with a configurable live region, style, and text icon.
pub struct Alert<'a> {
    title: &'a str,
    subtitle: Option<&'a str>,
    icon: &'a str,
    style: AlertStyle,
    live: bool,
    theme: &'a Theme,
    touch: bool,
}
impl<'a> Alert<'a> {
    pub fn new(title: &'a str, theme: &'a Theme) -> Self {
        Self {
            title,
            subtitle: None,
            icon: "ⓘ",
            style: AlertStyle::from_theme(theme, AlertVariant::Primary),
            live: true,
            theme,
            touch: false,
        }
    }
    pub fn subtitle(mut self, subtitle: &'a str) -> Self {
        self.subtitle = Some(subtitle);
        self
    }
    pub fn icon(mut self, icon: &'a str) -> Self {
        self.icon = icon;
        self
    }
    pub fn style(mut self, style: AlertStyle) -> Self {
        self.style = style;
        self
    }
    pub fn live_region(mut self, live: bool) -> Self {
        self.live = live;
        self
    }
    /// Use Forui's larger touch typography and icon sizes.
    pub fn touch(mut self, touch: bool) -> Self {
        self.touch = touch;
        self
    }
    pub fn show(self, ui: &mut Ui) -> Response {
        let result = Frame::new()
            .fill(self.style.fill)
            .stroke(Stroke::new(1.0, self.style.border))
            .corner_radius(10)
            .inner_margin(self.style.padding)
            .show(ui, |ui| {
                // Forui `FAlertStyle`: icon is body.md sized (16 desktop / 18
                // touch); text starts 10 points after it, with 2 points between
                // the title and subtitle.
                let icon_size = if self.touch { 18.0 } else { 16.0 };
                let title_size = self.theme.typography.body.size;
                let subtitle_size = if self.touch {
                    self.theme.typography.caption.size
                } else {
                    self.theme.typography.body.size
                };
                ui.horizontal_top(|ui| {
                    ui.spacing_mut().item_spacing.x = 0.0;
                    ui.label(
                        RichText::new(self.icon)
                            .size(icon_size)
                            .color(self.style.icon),
                    );
                    ui.add_space(10.0);
                    ui.vertical(|ui| {
                        ui.spacing_mut().item_spacing.y = 2.0;
                        ui.label(
                            RichText::new(self.title)
                                .size(title_size)
                                .color(self.style.title),
                        );
                        if let Some(subtitle) = self.subtitle {
                            ui.label(
                                RichText::new(subtitle)
                                    .size(subtitle_size)
                                    .color(self.style.subtitle),
                            );
                        }
                    });
                });
            });
        let response = result.response;
        let label = match self.subtitle {
            Some(subtitle) => format!("{}: {subtitle}", self.title),
            None => self.title.to_owned(),
        };
        response.widget_info(|| WidgetInfo::labeled(WidgetType::Other, ui.is_enabled(), &label));
        ui.ctx().accesskit_node_builder(response.id, |node| {
            node.set_role(egui::accesskit::Role::Alert);
            node.set_live(if self.live {
                egui::accesskit::Live::Polite
            } else {
                egui::accesskit::Live::Off
            });
        });
        response
    }
}
