#![cfg(target_os = "android")]

#[path = "../showcase.rs"]
mod showcase;

use android_activity::AndroidApp;

// NativeActivity's Rust entry point is looked up by its exported symbol.
#[unsafe(no_mangle)]
fn android_main(app: AndroidApp) {
    let options = eframe::NativeOptions {
        android_app: Some(app.clone()),
        ..Default::default()
    };
    if let Err(error) = eframe::run_native(
        "Temivu Showcase",
        options,
        Box::new(move |_| Ok(Box::new(showcase::Showcase::for_android(app)))),
    ) {
        eprintln!("Temivu Showcase: {error}");
    }
}

fn safe_area(
    app: &AndroidApp,
    ctx: &egui::Context,
    input: &egui::RawInput,
) -> Option<egui::SafeAreaInsets> {
    let screen = input.screen_rect?;
    let scale = input.viewport().native_pixels_per_point.unwrap_or(1.0) * ctx.zoom_factor();
    let content = app.content_rect();
    let content = egui::Rect::from_min_max(
        egui::pos2(content.left as f32, content.top as f32) / scale,
        egui::pos2(content.right as f32, content.bottom as f32) / scale,
    );
    let reported = content.is_positive()
        && content.right() <= screen.width() + 2.0
        && content.bottom() <= screen.height() + 2.0;
    let mut insets = egui::epaint::MarginF32::ZERO;
    if reported {
        insets.left = content.left().max(0.0);
        insets.top = content.top().max(0.0);
        insets.right = (screen.width() - content.right()).max(0.0);
        insets.bottom = (screen.height() - content.bottom()).max(0.0);
    }
    // Some Android windows report their entire surface even when status icons
    // are drawn over it. Leave room at the top in this case.
    if insets.top < 1.0 {
        insets.top = 24.0_f32.min(screen.height() * 0.09);
    }
    Some(egui::SafeAreaInsets(insets))
}
