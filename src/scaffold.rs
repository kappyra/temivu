//! Preliminary scaffold layout; safe areas, keyboard insets and RTL remain open.
use crate::{SurfaceShape, Theme};
use egui::{Align2, Area, CentralPanel, Frame, Id, Margin, Panel, Stroke, Ui, vec2};

/// Arranges a sidebar beside header, content and footer panels.
/// Use at the application root before adding other top-level panels. The ID
/// must be stable and unique among scaffolds in the same context.
pub struct Scaffold<'a> {
    id: Id,
    theme: &'a Theme,
    sidebar_width: Option<f32>,
    header: bool,
    floating_header: bool,
    header_frame: Option<Frame>,
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
            floating_header: false,
            header_frame: None,
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
    /// Place the header above the page so that content can scroll under it.
    /// Leave top clearance inside scrollable content for its initial position.
    pub fn floating_header(mut self, enabled: bool) -> Self {
        self.floating_header = enabled;
        self.header |= enabled;
        self
    }
    /// Customize the floating header's material and insets.
    pub fn floating_header_frame(mut self, frame: Frame) -> Self {
        self.header_frame = Some(frame);
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
        let floating_header = if self.header && !self.floating_header {
            Panel::top(self.id.with("header"))
                .resizable(false)
                .show_separator_line(false)
                .frame(Frame::NONE.fill(self.theme.background))
                .show(ui, header);
            None
        } else {
            self.header.then_some(header)
        };
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
        let result = CentralPanel::default()
            .frame(Frame::NONE.fill(self.theme.background).inner_margin(margin))
            .show(ui, content)
            .inner;
        if let Some(header) = floating_header {
            let ctx = ui.ctx();
            let viewport = ctx.content_rect();
            let inset = (viewport.width() * 0.04).clamp(8.0, 20.0);
            let gap = if viewport.width() < 600.0 { 12.0 } else { 10.0 };
            let frame = self.header_frame.unwrap_or_else(|| {
                Frame::new()
                    .fill(self.theme.card)
                    .stroke(Stroke::new(1.0, self.theme.border))
                    .inner_margin(Margin::symmetric(6, 3))
                    .corner_radius(SurfaceShape::Control.radius(vec2(300.0, 56.0)))
            });
            let width =
                (viewport.width() - 2.0 * inset - frame.total_margin().sum().x).clamp(1.0, 1040.0);
            Area::new(self.id.with("floating-header"))
                .anchor(Align2::CENTER_TOP, vec2(0.0, gap))
                .fade_in(false)
                .show(ctx, |ui| {
                    let frame = crate::glass::floating_frame(ctx, self.theme, frame);
                    let profile = crate::Glass::context(ctx).map(|_| crate::GlassProfile::Floating);
                    crate::glass::show_frame(ui, frame, profile, |ui| {
                        ui.set_width(width);
                        header(ui);
                    });
                });
        }
        result
    }
}
