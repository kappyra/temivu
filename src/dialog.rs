//! Modal dialog port; full route and focus parity remain open. A themed dialog
//! draws Forui's card, border and md radius over the theme barrier color.
use crate::Theme;
use egui::{Context, Frame, Id, Stroke};

#[derive(Clone, Default)]
struct PreviousFocus(Option<Id>);

fn restore_focus(ctx: &Context, key: Id) {
    if let Some(PreviousFocus(Some(id))) =
        ctx.data_mut(|data| data.remove_temp::<PreviousFocus>(key))
    {
        ctx.memory_mut(|memory| memory.request_focus(id));
    }
}

/// A caller-owned modal. The ID must remain unique and stable while open.
/// Opening multiple modals in the same frame has undefined focus ordering;
/// open at most one new modal per frame.
pub struct Dialog<'a> {
    id: Id,
    label: &'a str,
    barrier_dismissible: bool,
    theme: Option<&'a Theme>,
}

impl<'a> Dialog<'a> {
    pub fn new(id: Id, label: &'a str) -> Self {
        Self {
            id,
            label,
            barrier_dismissible: true,
            theme: None,
        }
    }

    pub fn barrier_dismissible(mut self, dismissible: bool) -> Self {
        self.barrier_dismissible = dismissible;
        self
    }

    /// Draw Forui's dialog surface: card fill, 1 point border, md radius and the
    /// theme's barrier color. Content typography and padding remain caller-owned.
    pub fn theme(mut self, theme: &'a Theme) -> Self {
        self.theme = Some(theme);
        self
    }

    /// Returns the content result while open; updates `open` on dismissal.
    /// Call every frame, including after closing, so focus can be restored.
    pub fn show<R>(
        self,
        ctx: &Context,
        open: &mut bool,
        content: impl FnOnce(&mut egui::Ui) -> R,
    ) -> Option<R> {
        let focus_key = self.id.with("temivu-dialog-previous-focus");
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
        let area =
            egui::Modal::default_area(self.id).sense(egui::Sense::focusable_noninteractive());
        let mut modal = egui::Modal::new(self.id).area(area);
        if let Some(theme) = self.theme {
            modal = modal.backdrop_color(theme.barrier).frame(
                Frame::new()
                    .fill(theme.card)
                    .stroke(Stroke::new(1.0, theme.border))
                    .corner_radius(10),
            );
        }
        let response = modal.show(ctx, content);
        ctx.accesskit_node_builder(response.response.id, |node| {
            node.set_role(egui::accesskit::Role::Dialog);
            node.set_label(self.label);
            node.set_modal();
        });
        if newly_opened {
            response.response.request_focus();
        }
        // Escape applies only to the topmost modal and must not preempt popups.
        let escape = response.is_top_modal
            && !response.any_popup_open
            && ctx.input_mut(|input| input.consume_key(egui::Modifiers::NONE, egui::Key::Escape));
        if response.response.should_close()
            || escape
            || (self.barrier_dismissible && response.backdrop_response.clicked())
        {
            *open = false;
            // The modal still owns focus in this frame. Restore after it disappears.
            ctx.request_repaint();
        }
        Some(response.inner)
    }
}
