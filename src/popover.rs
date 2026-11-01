//! Experimental anchored popover. Nested overlays, motion and full parity remain open.
use egui::{Id, InnerResponse, Key, Popup, PopupCloseBehavior, Response, Ui};

/// A caller-controlled, non-modal anchored popover.
///
/// IDs must be globally unique and stable. Call `show` each frame, including
/// when closed. If `focus_on_open` is enabled, return an enabled focusable
/// response from the content callback so keyboard focus can enter the popup.
/// Nested overlays, external removal and screen-reader behavior remain unverified.
///
/// ```
/// # fn example(ui: &mut egui::Ui, open: &mut bool) {
/// use temivu::Popover;
/// let anchor = ui.button("Options");
/// if anchor.clicked() { *open = !*open; }
/// let _ = Popover::new(egui::Id::new("options-popover"), "Options", open)
///     .focus_on_open(true)
///     .show(&anchor, |ui| ui.button("Apply"));
/// # }
/// ```
pub struct Popover<'a> {
    id: Id,
    label: &'a str,
    open: &'a mut bool,
    focus_on_open: bool,
}

impl<'a> Popover<'a> {
    pub fn new(id: Id, label: &'a str, open: &'a mut bool) -> Self {
        Self {
            id,
            label,
            open,
            focus_on_open: false,
        }
    }
    /// Focus the returned content response when the popup opens with anchor focus.
    pub fn focus_on_open(mut self, focus: bool) -> Self {
        self.focus_on_open = focus;
        self
    }
    pub fn show(
        self,
        anchor: &Response,
        content: impl FnOnce(&mut Ui) -> Response,
    ) -> Option<InnerResponse<Response>> {
        let ctx = &anchor.ctx;
        let open_key = self.id.with("temivu-popover-open");
        let focus_key = self.id.with("temivu-popover-focus");
        let was_open = ctx.data(|data| data.get_temp::<bool>(open_key).unwrap_or(false));
        let previous_popup_focus = ctx.data(|data| data.get_temp::<Id>(focus_key));
        let focused_before = ctx.memory(|memory| memory.focused());
        let just_opened = *self.open && !was_open;
        let escape = ctx.input(|input| input.key_pressed(Key::Escape));
        let keyboard_activate =
            ctx.input(|input| input.key_pressed(Key::Enter) || input.key_pressed(Key::Space));
        let result = Popup::from_response(anchor)
            .id(self.id)
            .open_bool(self.open)
            .close_behavior(PopupCloseBehavior::CloseOnClickOutside)
            .show(content);
        let focused_in_popup = result.as_ref().and_then(|popup| {
            focused_before.filter(|id| {
                ctx.read_response(*id)
                    .is_some_and(|focused| focused.layer_id == popup.response.layer_id)
            })
        });
        ctx.data_mut(|data| {
            if let Some(id) = focused_in_popup {
                data.insert_temp(focus_key, id);
            } else if focused_before.is_some() {
                data.remove::<Id>(focus_key);
            }
            data.insert_temp(open_key, *self.open);
        });
        if just_opened
            && self.focus_on_open
            && anchor.has_focus()
            && let Some(popup) = &result
        {
            popup.inner.request_focus();
        }
        let popup_had_focus = focused_in_popup.is_some()
            || previous_popup_focus.is_some_and(|id| {
                focused_before == Some(id)
                    || (focused_before.is_none() && ctx.memory(|m| m.had_focus_last_frame(id)))
            });
        let focused_after = ctx.memory(|memory| memory.focused());
        let focus_not_redirected = focused_after.is_none() || focused_after == focused_before;
        if was_open
            && !*self.open
            && (escape || keyboard_activate)
            && popup_had_focus
            && focus_not_redirected
        {
            anchor.request_focus();
        }
        if let Some(popup) = &result {
            ctx.accesskit_node_builder(popup.response.id, |node| {
                node.set_role(egui::accesskit::Role::Group);
                node.set_label(self.label);
            });
        }
        result
    }
}
