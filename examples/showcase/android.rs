#![cfg(target_os = "android")]

#[path = "../showcase.rs"]
mod showcase;

use android_activity::AndroidApp;

// NativeActivity's Rust entry point is looked up by its exported symbol.
#[unsafe(no_mangle)]
fn android_main(app: AndroidApp) {
    let options = eframe::NativeOptions {
        android_app: Some(app),
        ..Default::default()
    };
    if let Err(error) = eframe::run_native(
        "Temivu Showcase",
        options,
        Box::new(|_| Ok(Box::<showcase::Showcase>::default())),
    ) {
        eprintln!("Temivu Showcase: {error}");
    }
}
