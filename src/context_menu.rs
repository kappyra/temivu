//! Preliminary context menu. Touch gestures, nested menus and full parity remain open.
use egui::{InnerResponse, Key, Modifiers, Popup, Response, Ui};

/// Attach a context menu to a focusable, clickable response.
///
/// Secondary click opens at the pointer; Shift+F10 opens below the focused
/// response. `ui.close()` from a menu item dismisses it. The response must
/// have a stable ID and sense clicks. Return the first enabled menu item
/// from `contents`; it receives focus when the menu is opened by keyboard. Nested menu and touch behavior are not
/// yet supported or verified.
pub fn context_menu(
    response: &Response,
    contents: impl FnOnce(&mut Ui) -> Response,
) -> Option<InnerResponse<Response>> {
    let ctx = &response.ctx;
    let focused_before = ctx.memory(|memory| memory.focused());
    let popup_focus_key = response.id.with("temivu-context-menu-focused-item");
    let previous_popup_focus = ctx.data(|data| data.get_temp::<egui::Id>(popup_focus_key));
    let keyboard_activate =
        ctx.input(|input| input.key_pressed(Key::Enter) || input.key_pressed(Key::Space));
    let anchor_id = response.id.with("temivu-context-menu-keyboard-anchor");
    let keyboard = response.enabled()
        && response.has_focus()
        && ctx.input_mut(|input| input.consume_key(Modifiers::SHIFT, Key::F10));
    if keyboard || response.secondary_clicked() {
        ctx.data_mut(|data| data.insert_temp(anchor_id, keyboard));
    }
    let keyboard_anchor = ctx.data(|data| data.get_temp::<bool>(anchor_id).unwrap_or(false));
    let was_open = Popup::is_id_open(ctx, Popup::default_response_id(response));
    let escape = ctx.input(|input| input.key_pressed(Key::Escape));
    let mut popup = Popup::context_menu(response);
    if keyboard_anchor {
        popup = popup.anchor(response);
    }
    if !response.enabled() {
        popup = popup.open_memory(Some(false.into()));
    } else if keyboard {
        popup = popup.open_memory(Some(true.into()));
    }
    let result = popup.show(contents);
    let focused_in_popup = result.as_ref().and_then(|popup| {
        focused_before.filter(|id| {
            ctx.read_response(*id)
                .is_some_and(|focused| focused.layer_id == popup.response.layer_id)
        })
    });
    ctx.data_mut(|data| {
        if let Some(id) = focused_in_popup {
            data.insert_temp(popup_focus_key, id);
        } else if focused_before.is_some() {
            data.remove::<egui::Id>(popup_focus_key);
        }
    });
    if keyboard && let Some(content) = &result {
        content.inner.request_focus();
    }
    let popup_had_focus = focused_in_popup.is_some()
        || previous_popup_focus.is_some_and(|id| {
            focused_before == Some(id)
                || (focused_before.is_none() && ctx.memory(|m| m.had_focus_last_frame(id)))
        });
    let closed = !Popup::is_id_open(ctx, Popup::default_response_id(response));
    let focused_after = ctx.memory(|memory| memory.focused());
    let focus_not_redirected = focused_after.is_none() || focused_after == focused_before;
    if response.enabled()
        && was_open
        && popup_had_focus
        && closed
        && (escape || keyboard_activate)
        && focus_not_redirected
    {
        response.request_focus();
    }
    result
}
