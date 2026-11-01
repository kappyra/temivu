//! Experimental text tooltip attached to an existing egui response.
use egui::{Id, Key, Modifiers, Response};

/// Attach a description and an anchored tooltip to an existing widget response.
/// Text appears on hover or keyboard focus; Escape suppresses it until both
/// hover and focus leave. The underlying widget retains its own semantics.
pub fn tooltip(response: Response, text: &str) -> Response {
    let context = response.ctx.clone();
    context.accesskit_node_builder(response.id, |node| {
        let combined = match node.description() {
            Some(previous) if previous != text => format!("{previous}\n{text}"),
            _ => text.to_owned(),
        };
        node.set_description(combined);
    });
    let dismissed_id: Id = response.id.with("forui-tooltip-dismissed");
    let active = response.hovered() || response.has_focus();
    let mut dismissed = context.data(|data| data.get_temp::<bool>(dismissed_id).unwrap_or(false));
    if !active {
        dismissed = false;
    }
    let visible =
        response.is_tooltip_open() || (response.enabled() && response.has_focus() && !dismissed);
    // Escape belongs to the topmost popup or modal first; the focused widget
    // behind a dialog must not swallow it.
    let overlay_open =
        context.any_popup_open() || context.memory(|memory| memory.top_modal_layer().is_some());
    if visible
        && !overlay_open
        && context.input_mut(|input| input.consume_key(Modifiers::NONE, Key::Escape))
    {
        dismissed = true;
    }
    context.data_mut(|data| data.insert_temp(dismissed_id, dismissed));
    if !dismissed {
        // Forui's tooltip is an inverted surface: `foreground` fill, `background`
        // text, sm (8) radius and 12/6 padding. Egui's current visuals carry
        // those two colors once `Theme::apply` has run.
        let tooltip = if response.enabled() && response.has_focus() {
            egui::Tooltip::for_widget(&response)
        } else if response.enabled() {
            egui::Tooltip::for_enabled(&response)
        } else {
            egui::Tooltip::for_disabled(&response)
        };
        let style = context.global_style();
        let fill = style
            .visuals
            .override_text_color
            .unwrap_or_else(|| style.visuals.text_color());
        let text_color = style.visuals.panel_fill;
        let body_size = style
            .text_styles
            .get(&egui::TextStyle::Body)
            .map_or(14.0, |font| font.size);
        let mut tooltip = tooltip.gap(7.0);
        tooltip.popup = tooltip.popup.frame(
            egui::Frame::new()
                .fill(fill)
                .corner_radius(8)
                .inner_margin(egui::Margin::symmetric(12, 6)),
        );
        tooltip.show(|ui| {
            ui.label(
                egui::RichText::new(text)
                    .size((body_size - 2.0).max(9.0))
                    .color(text_color),
            );
        });
    }
    response
}
