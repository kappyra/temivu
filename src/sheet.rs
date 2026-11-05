//! Modal bottom sheet with a spring slide.
use crate::{Button, ButtonVariant, Dialog, Theme};
use egui::{Context, Id, Ui, Vec2};

/// A caller-owned modal sheet anchored to the bottom of the window.
///
/// The sheet blocks pointer and keyboard access to the rest of the UI, moves
/// focus into the sheet when it opens and restores the previous focus after a
/// normal dismissal. `open` is updated by Escape, the dismiss button, a
/// backdrop click when enabled, and [`Ui::close`] calls inside the content.
/// The ID must remain unique and stable while open; call `show` every frame,
/// including while closed, so focus can be restored.
///
/// ```
/// # fn example(ui: &mut egui::Ui, theme: &temivu::Theme, open: &mut bool) {
/// use temivu::Sheet;
/// Sheet::new(egui::Id::new("filters-sheet"), "Filters", theme).show(
///     ui.ctx(),
///     open,
///     |ui| {
///         ui.label("Sheet content");
///     },
/// );
/// # }
/// ```
pub struct Sheet<'a> {
    id: Id,
    title: &'a str,
    theme: &'a Theme,
    dismiss_button: bool,
    barrier_dismissible: bool,
    max_width: f32,
}

impl<'a> Sheet<'a> {
    pub fn new(id: Id, title: &'a str, theme: &'a Theme) -> Self {
        Self {
            id,
            title,
            theme,
            dismiss_button: true,
            barrier_dismissible: true,
            max_width: 560.0,
        }
    }

    /// Show the header's dismiss button. Defaults to `true`.
    pub fn dismiss_button(mut self, dismiss_button: bool) -> Self {
        self.dismiss_button = dismiss_button;
        self
    }

    /// Whether a backdrop click dismisses the sheet. Defaults to `true`.
    pub fn barrier_dismissible(mut self, dismissible: bool) -> Self {
        self.barrier_dismissible = dismissible;
        self
    }

    /// The sheet's maximum total width, including its padding. Defaults to
    /// 560 points and is clamped to the window on small screens.
    pub fn max_width(mut self, max_width: f32) -> Self {
        self.max_width = max_width;
        self
    }

    /// Returns the content's result while open; updates `open` on dismissal.
    pub fn show<R>(
        self,
        ctx: &Context,
        open: &mut bool,
        content: impl FnOnce(&mut Ui) -> R,
    ) -> Option<R> {
        const MARGIN: f32 = 8.0;
        const PADDING: f32 = 22.0;
        const STROKE: f32 = 1.0;
        let available = ctx.content_rect().width();
        let sheet_width = (available - 2.0 * MARGIN)
            .max(0.0)
            .min(self.max_width.max(0.0));
        let content_width = (sheet_width - 2.0 * PADDING - 2.0 * STROKE).max(0.0);

        let frame = egui::Frame::new()
            .fill(self.theme.card)
            .stroke(egui::Stroke::new(STROKE, self.theme.border))
            .corner_radius(egui::CornerRadius {
                nw: 28,
                ne: 28,
                sw: 0,
                se: 0,
            })
            .inner_margin(egui::Margin::symmetric(PADDING as i8, PADDING as i8));
        let area =
            egui::Modal::default_area(self.id).anchor(egui::Align2::CENTER_BOTTOM, Vec2::ZERO);
        Dialog::new(self.id, self.title)
            .theme(self.theme)
            .barrier_dismissible(self.barrier_dismissible)
            .frame(frame)
            .area(area)
            .slide_from_bottom()
            .show(ctx, open, |ui| {
                ui.set_width(content_width);
                let handle = egui::Rect::from_center_size(
                    egui::pos2(ui.max_rect().center().x, ui.cursor().min.y + 2.0),
                    egui::vec2(34.0, 4.0),
                );
                ui.painter().rect_filled(
                    handle,
                    2,
                    self.theme.muted_foreground.gamma_multiply(0.45),
                );
                ui.add_space(10.0);
                egui::Sides::new().show(
                    ui,
                    |ui| {
                        ui.label(egui::RichText::new(self.title).size(20.0).strong());
                    },
                    |ui| {
                        if self.dismiss_button
                            && Button::new("Close", self.theme)
                                .variant(ButtonVariant::Ghost)
                                .show(ui)
                                .clicked()
                        {
                            ui.close();
                        }
                    },
                );
                ui.add_space(12.0);
                content(ui)
            })
    }
}
