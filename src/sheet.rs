//! Modal bottom sheet with a spring slide.
use crate::{Button, ButtonVariant, Dialog, InputMode, Theme};
use egui::{Context, Id, Ui, Vec2};

/// A caller-owned modal sheet anchored to the bottom of the window.
///
/// The sheet blocks pointer and keyboard access to the rest of the UI, moves
/// focus into the sheet when it opens and restores the previous focus after a
/// normal dismissal. Escape, the dismiss button, a downward drag on the
/// handle, an enabled backdrop click, and [`Ui::close`] all update `open`.
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
    description: Option<&'a str>,
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
            description: None,
            dismiss_button: true,
            barrier_dismissible: true,
            max_width: 560.0,
        }
    }

    /// Optional supporting text above the scrollable content.
    pub fn description(mut self, description: &'a str) -> Self {
        self.description = Some(description);
        self
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
        let viewport = ctx.content_rect();
        let compact = self.theme.touch || viewport.width() < 600.0;
        let inset = if compact { 8.0 } else { 24.0 };
        let padding = if compact { 16 } else { 24 };
        let target_width = (viewport.width() - 2.0 * inset)
            .max(0.0)
            .min(self.max_width.max(0.0));
        let width = crate::motion::spring_size(
            ctx,
            self.id.with("sheet-size"),
            egui::vec2(target_width, 0.0),
            self.theme,
        )
        .x
        .min((viewport.width() - 2.0 * inset).max(0.0));
        let content_width = (width - 2.0 * f32::from(padding) - 2.0).max(0.0);

        let mut frame = crate::surface::popup_frame(self.theme, padding)
            .corner_radius(28)
            .inner_margin(egui::Margin {
                top: 8,
                ..egui::Margin::same(padding)
            });
        if compact {
            frame.corner_radius.sw = 0;
            frame.corner_radius.se = 0;
        }
        let bottom = if compact { 0.0 } else { inset };
        let area = egui::Modal::default_area(self.id)
            .anchor(egui::Align2::CENTER_BOTTOM, Vec2::new(0.0, -bottom));
        Dialog::new(self.id, self.title)
            .theme(self.theme)
            .barrier_dismissible(self.barrier_dismissible)
            .frame(frame)
            .area(area)
            .slide_from_bottom()
            .show(ctx, open, |ui| {
                ui.set_width(content_width);
                ui.spacing_mut().item_spacing.y = 0.0;
                let (grip, gesture) = ui.allocate_exact_size(
                    egui::vec2(content_width, if compact { 28.0 } else { 20.0 }),
                    egui::Sense::drag(),
                );
                let gesture = gesture.on_hover_cursor(egui::CursorIcon::Grab);
                let previous = drag_offset(ctx, self.id);
                let spring = self.id.with("sheet-grip-return");
                if let Some(delta) = gesture.total_drag_delta() {
                    crate::motion::reset(ctx, spring);
                    set_drag_offset(ctx, self.id, delta.y.clamp(0.0, viewport.height() * 0.75));
                } else if gesture.drag_stopped() && previous >= 64.0 {
                    ui.close();
                } else if previous > 0.0 {
                    let offset = crate::motion::spring_f32_with_damping(
                        ctx, spring, previous, 0.0, 0.86, self.theme,
                    );
                    set_drag_offset(ctx, self.id, offset.max(0.0));
                }
                ui.painter().rect_filled(
                    egui::Rect::from_center_size(grip.center(), egui::vec2(34.0, 4.0)),
                    2,
                    self.theme.muted_foreground.gamma_multiply(0.48),
                );
                ui.add_space(4.0);
                ui.with_layout(egui::Layout::right_to_left(egui::Align::TOP), |ui| {
                    if self.dismiss_button {
                        let input = if compact {
                            InputMode::Touch
                        } else {
                            InputMode::Desktop
                        };
                        if Button::new("Close", self.theme)
                            .input_mode(input)
                            .variant(ButtonVariant::Secondary)
                            .show(ui)
                            .clicked()
                        {
                            ui.close();
                        }
                    }
                    ui.vertical(|ui| {
                        ui.set_width(ui.available_width());
                        ui.spacing_mut().item_spacing.y = 4.0;
                        ui.label(
                            egui::RichText::new(self.title)
                                .size(self.theme.typography.body.size + 5.0)
                                .strong()
                                .color(self.theme.foreground),
                        );
                    });
                });
                ui.add_space(12.0);
                let chrome = ui.min_rect().height() + f32::from(padding) + bottom + 22.0;
                let max_body = (viewport.height() * 0.9 - chrome).max(0.0);
                egui::ScrollArea::vertical()
                    .max_height(max_body)
                    .min_scrolled_height(0.0)
                    .auto_shrink([false, true])
                    .show(ui, |ui| {
                        ui.spacing_mut().item_spacing.y = 10.0;
                        if let Some(description) = self.description {
                            ui.label(
                                egui::RichText::new(description).color(self.theme.muted_foreground),
                            );
                        }
                        content(ui)
                    })
                    .inner
            })
    }
}

pub(crate) fn drag_offset(ctx: &Context, id: Id) -> f32 {
    ctx.data(|data| data.get_temp::<f32>(id.with("sheet-drag")))
        .unwrap_or(0.0)
}

pub(crate) fn set_drag_offset(ctx: &Context, id: Id, offset: f32) {
    ctx.data_mut(|data| {
        let key = id.with("sheet-drag");
        if offset == 0.0 {
            data.remove::<f32>(key);
        } else {
            data.insert_temp(key, offset);
        }
    });
}
