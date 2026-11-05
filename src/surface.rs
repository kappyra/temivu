//! Shared floating-surface paint; interaction remains owned by egui.
use crate::Theme;

pub(crate) fn popup_frame(theme: &Theme, padding: i8) -> egui::Frame {
    egui::Frame::new()
        .fill(theme.card)
        .stroke(egui::Stroke::new(1.0, theme.border))
        .corner_radius(32)
        .inner_margin(padding)
        .shadow(egui::epaint::Shadow {
            offset: [0, 6],
            blur: 20,
            spread: 0,
            color: egui::Color32::from_black_alpha(if theme.dark { 70 } else { 22 }),
        })
}

pub(crate) fn chevron(
    ui: &egui::Ui,
    rect: egui::Rect,
    visuals: &egui::style::WidgetVisuals,
    open: f32,
) {
    let direction = 1.0 - 2.0 * open.clamp(0.0, 1.0);
    let center = rect.center();
    ui.painter().add(egui::Shape::line(
        vec![
            center + egui::vec2(-4.0, -2.0 * direction),
            center + egui::vec2(0.0, 2.0 * direction),
            center + egui::vec2(4.0, -2.0 * direction),
        ],
        egui::Stroke::new(1.5, visuals.fg_stroke.color),
    ));
}

/// Derive semantic paint colors without changing the application's font/style state.
pub(crate) fn context_theme(ctx: &egui::Context) -> Theme {
    let style = ctx.global_style();
    let mut theme = if style.visuals.dark_mode {
        Theme::dark()
    } else {
        Theme::light()
    };
    theme.card = style.visuals.window_fill();
    theme.border = style.visuals.window_stroke().color;
    theme.foreground = style.visuals.text_color();
    theme.motion.reduced_motion = style.animation_time <= 0.0;
    theme
}
