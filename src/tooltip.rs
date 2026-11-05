//! Experimental text tooltip attached to an existing egui response.
use egui::{Id, Key, Modifiers, Response};

fn describe(response: &Response, text: &str) {
    response.ctx.accesskit_node_builder(response.id, |node| {
        let combined = match node.description() {
            Some(previous) if previous != text => format!("{previous}\n{text}"),
            _ => text.to_owned(),
        };
        node.set_description(combined);
    });
}

/// Attach a description and an anchored tooltip to an existing widget response.
/// Text appears on hover or keyboard focus; Escape suppresses it until both
/// hover and focus leave. The underlying widget retains its own semantics.
pub fn tooltip(response: Response, text: &str) -> Response {
    let context = response.ctx.clone();
    describe(&response, text);
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

/// Show caller-supplied link information above an existing widget, or below
/// when there is no room. The preview does not load the destination.
pub fn link_preview(response: Response, destination: &str, theme: &crate::Theme) -> Response {
    let ctx = &response.ctx;
    let id = response.id.with("link-preview");
    let active = response.enabled() && (response.hovered() || response.has_focus());
    let mut dismissed = active && ctx.data(|data| data.get_temp::<bool>(id).unwrap_or(false));
    if active
        && !ctx.any_popup_open()
        && !dismissed
        && ctx.memory(|memory| memory.top_modal_layer().is_none())
        && ctx.input_mut(|input| input.consume_key(Modifiers::NONE, Key::Escape))
    {
        dismissed = true;
    }
    ctx.data_mut(|data| data.insert_temp(id, dismissed));
    describe(&response, destination);
    let amount = crate::motion::spring_f32(
        ctx,
        id.with("rise"),
        0.0,
        if active && !dismissed { 1.0 } else { 0.0 },
        theme,
    )
    .clamp(0.0, 1.0);
    if amount > 0.001 {
        let screen = ctx.content_rect().shrink(5.0);
        let painter = ctx
            .layer_painter(egui::LayerId::new(egui::Order::Tooltip, id))
            .with_clip_rect(screen);
        let mut job = egui::text::LayoutJob::simple(
            destination.to_owned(),
            egui::FontId::proportional(theme.typography.body.size),
            theme.foreground.gamma_multiply(amount),
            (screen.width() - 24.0).max(1.0),
        );
        job.wrap.max_rows = 1;
        job.wrap.break_anywhere = true;
        let text = painter.layout_job(job);
        let size = text.size() + egui::vec2(24.0, 14.0);
        let anchor = ctx
            .layer_transform_to_global(response.layer_id)
            .map_or(response.rect, |transform| transform * response.rect);
        let x = (anchor.center().x - size.x * 0.5)
            .clamp(screen.left(), (screen.right() - size.x).max(screen.left()));
        let above = anchor.top() - size.y - 9.0;
        let y = if above >= screen.top() {
            above + 8.0 * (1.0 - amount)
        } else {
            anchor.bottom() + 9.0 - 8.0 * (1.0 - amount)
        };
        let y = y.clamp(screen.top(), (screen.bottom() - size.y).max(screen.top()));
        let rect = egui::Rect::from_min_size(egui::pos2(x, y), size);
        painter.rect(
            rect,
            (size.y * 0.5) as u8,
            theme.card.gamma_multiply(amount),
            egui::Stroke::new(1.0, theme.border.gamma_multiply(amount)),
            egui::StrokeKind::Inside,
        );
        painter.galley(
            rect.min + egui::vec2(12.0, 7.0),
            text,
            theme.foreground.gamma_multiply(amount),
        );
    }
    response
}
