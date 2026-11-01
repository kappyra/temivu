//! Preliminary modal bottom sheet. Drag gestures, entrance animation, sides
//! other than the bottom and full parity remain open.
use crate::Theme;
use egui::{Context, Id, Key, Modifiers, Ui, Vec2};

#[derive(Clone, Default)]
struct PreviousFocus(Option<Id>);

/// Restore the focus remembered before the sheet opened, once, after it closed.
fn restore_focus(ctx: &Context, key: Id) {
    if let Some(PreviousFocus(Some(id))) =
        ctx.data_mut(|data| data.remove_temp::<PreviousFocus>(key))
    {
        ctx.memory_mut(|memory| memory.request_focus(id));
    }
}

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
        let focus_key = self.id.with("temivu-sheet-previous-focus");
        if !*open {
            restore_focus(ctx, focus_key);
            return None;
        }
        let newly_opened = ctx
            .data(|data| data.get_temp::<PreviousFocus>(focus_key))
            .is_none();
        if newly_opened {
            let previous = ctx.memory(|memory| memory.focused());
            ctx.data_mut(|data| data.insert_temp(focus_key, PreviousFocus(previous)));
        }

        const MARGIN: f32 = 8.0;
        const PADDING: f32 = 16.0;
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
                nw: 12,
                ne: 12,
                sw: 0,
                se: 0,
            })
            .inner_margin(egui::Margin::symmetric(PADDING as i8, PADDING as i8));
        let area = egui::Modal::default_area(self.id)
            .sense(egui::Sense::focusable_noninteractive())
            .anchor(egui::Align2::CENTER_BOTTOM, Vec2::ZERO);

        let mut dismiss_clicked = false;
        let shown = egui::Modal::new(self.id)
            .area(area)
            .frame(frame)
            .backdrop_color(self.theme.barrier)
            .show(ctx, |ui| {
                ui.set_width(content_width);
                egui::Sides::new().show(
                    ui,
                    |ui| {
                        ui.label(
                            egui::RichText::new(self.title)
                                .size(18.0)
                                .strong()
                                .color(self.theme.foreground),
                        );
                    },
                    |ui| {
                        if self.dismiss_button && ui.button("Close").clicked() {
                            dismiss_clicked = true;
                        }
                    },
                );
                ui.add_space(8.0);
                content(ui)
            });
        let inner = shown.inner;

        ctx.accesskit_node_builder(shown.response.id, |node| {
            node.set_role(egui::accesskit::Role::Dialog);
            node.set_label(self.title);
            node.set_modal();
        });
        if newly_opened {
            shown.response.request_focus();
        }

        // Escape applies only to the topmost modal and must not preempt popups.
        let escape = shown.is_top_modal
            && !shown.any_popup_open
            && ctx.input_mut(|input| input.consume_key(Modifiers::NONE, Key::Escape));
        if dismiss_clicked
            || shown.response.should_close()
            || escape
            || (self.barrier_dismissible && shown.backdrop_response.clicked())
        {
            *open = false;
            // The sheet still owns focus in this frame. Restore after it disappears.
            ctx.request_repaint();
        }
        Some(inner)
    }
}
