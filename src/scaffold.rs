//! Preliminary scaffold layout; safe areas, keyboard insets and RTL remain open.
use crate::Theme;
use egui::{CentralPanel, Frame, Id, Margin, Panel, Ui};

/// Arranges a sidebar beside header, content and footer panels.
/// Use at the application root before adding other top-level panels. The ID
/// must be stable and unique among scaffolds in the same context.
pub struct Scaffold<'a> {
    id: Id,
    theme: &'a Theme,
    sidebar_width: Option<f32>,
    header: bool,
    footer: bool,
    content_padding: bool,
}

impl<'a> Scaffold<'a> {
    pub fn new(id: Id, theme: &'a Theme) -> Self {
        Self {
            id,
            theme,
            sidebar_width: None,
            header: false,
            footer: false,
            content_padding: true,
        }
    }
    /// Enables a sidebar with a preferred width in logical points. The actual
    /// width is capped at 40% of the available width to retain content space.
    pub fn sidebar(mut self, width: f32) -> Result<Self, &'static str> {
        if !width.is_finite() || width <= 0.0 {
            return Err("sidebar width must be finite and positive");
        }
        self.sidebar_width = Some(width);
        Ok(self)
    }
    pub fn header(mut self, enabled: bool) -> Self {
        self.header = enabled;
        self
    }
    pub fn footer(mut self, enabled: bool) -> Self {
        self.footer = enabled;
        self
    }
    pub fn content_padding(mut self, enabled: bool) -> Self {
        self.content_padding = enabled;
        self
    }

    /// Only enabled region callbacks are invoked; the content callback is always invoked.
    pub fn show<R>(
        self,
        ui: &mut Ui,
        sidebar: impl FnOnce(&mut Ui),
        header: impl FnOnce(&mut Ui),
        footer: impl FnOnce(&mut Ui),
        content: impl FnOnce(&mut Ui) -> R,
    ) -> R {
        if let Some(preferred) = self.sidebar_width {
            // Retain room for content on small windows instead of extending beyond the viewport.
            let width = preferred.min((ui.available_width() * 0.4).max(0.0));
            Panel::left(self.id.with("sidebar"))
                .resizable(false)
                .exact_size(width)
                .show_separator_line(false)
                .frame(Frame::NONE.fill(self.theme.card))
                .show(ui, sidebar);
        }
        if self.header {
            Panel::top(self.id.with("header"))
                .resizable(false)
                .show_separator_line(false)
                .frame(Frame::NONE.fill(self.theme.background))
                .show(ui, header);
        }
        if self.footer {
            Panel::bottom(self.id.with("footer"))
                .resizable(false)
                .show_separator_line(false)
                .frame(Frame::NONE.fill(self.theme.background))
                .show(ui, footer);
        }
        // Forui `FStyle.pagePadding`: 12 horizontal, 8 vertical.
        let margin = if self.content_padding {
            Margin::symmetric(12, 8)
        } else {
            Margin::ZERO
        };
        CentralPanel::default()
            .frame(Frame::NONE.fill(self.theme.background).inner_margin(margin))
            .show(ui, content)
            .inner
    }
}
