#![cfg(feature = "demo")]

#[allow(dead_code)]
#[path = "../examples/showcase.rs"]
mod showcase;

use eframe::App;
use egui_kittest::{
    Harness,
    kittest::{NodeT, Queryable},
};

fn harness() -> Harness<'static, showcase::Showcase> {
    let mut harness = Harness::builder()
        .with_size(egui::vec2(900.0, 2600.0))
        .build_ui_state(
            |ui, app: &mut showcase::Showcase| {
                let mut frame = eframe::Frame::_new_kittest();
                app.ui(ui, &mut frame);
            },
            showcase::Showcase::default(),
        );
    harness.run_steps(3);
    harness
}

fn navigate(harness: &mut Harness<'static, showcase::Showcase>, page: &str) {
    harness.get_by_label(page).click();
    harness.run_steps(3);
}

#[test]
fn navigation_remains_usable_in_a_narrow_viewport() {
    let mut harness = Harness::builder()
        .with_size(egui::vec2(390.0, 844.0))
        .build_ui_state(
            |ui, app: &mut showcase::Showcase| {
                let mut frame = eframe::Frame::_new_kittest();
                app.ui(ui, &mut frame);
            },
            showcase::Showcase::default(),
        );
    harness.run_steps(3);
    let controls = harness.get_by_label("Controls");
    let rect = controls.rect();
    assert!(rect.left() >= 0.0 && rect.right() <= 390.0);
    controls.click();
    harness.run_steps(3);
    harness.get_by_label("Controls and inputs");
    for label in ["Name", "Email"] {
        let field = harness.get_by_role_and_label(egui::accesskit::Role::TextInput, label);
        assert!(
            field.rect().right() <= 390.0,
            "{label} extends outside the narrow viewport: {:?}",
            field.rect()
        );
    }
    let navigation = ["Overview", "Controls", "Overlays", "Data", "Localization"]
        .map(|label| harness.get_by_label(label).rect());
    for rect in navigation {
        assert!(rect.right() <= 390.0);
        assert_eq!(
            rect.center().y,
            navigation[0].center().y,
            "compact navigation must fit on one row"
        );
    }
}

#[test]
fn phone_safe_area_keeps_both_floating_menus_and_dock_reachable() {
    let mut harness = Harness::builder()
        .with_size(egui::vec2(320.0, 640.0))
        .build_ui_state(
            |ui, app: &mut showcase::Showcase| {
                app.ui(ui, &mut eframe::Frame::_new_kittest());
            },
            showcase::Showcase::default(),
        );
    harness.input_mut().safe_area_insets = Some(egui::SafeAreaInsets(egui::epaint::MarginF32 {
        left: 8.0,
        right: 8.0,
        top: 28.0,
        bottom: 24.0,
    }));
    harness.run_steps(4);
    let first_heading = harness.get_by_label("TEMIVU / COMPONENT STUDIO").rect();
    let header_button = harness.get_by_label("Menu").rect();
    assert!(first_heading.top() >= header_button.bottom() + 8.0);
    for label in ["Menu", "Quick menu"] {
        let trigger = harness.get_by_label(label);
        assert!(trigger.rect().top() >= 36.0);
        trigger.click();
        harness.run_steps(4);
        let row = harness.get_by_role_and_label(egui::accesskit::Role::MenuItem, "Localization");
        assert!(row.rect().left() >= 8.0 && row.rect().right() <= 312.0);
        assert!(row.rect().top() >= 28.0 && row.rect().bottom() <= 616.0);
        harness.key_press(egui::Key::Escape);
        harness.run_steps(4);
    }
    let dock = harness.get_by_label("Localization");
    assert!(dock.rect().bottom() <= 600.0);
}

#[test]
fn showcase_pages_theme_and_controls_work() {
    let mut harness = harness();
    harness.get_by_label("Thoughtful components for beautiful, accessible interfaces.");
    let heading_color = |harness: &Harness<'_, showcase::Showcase>| {
        harness
            .output()
            .shapes
            .iter()
            .find_map(|shape| match &shape.shape {
                egui::Shape::Text(text) if text.galley.job.text == "Welcome to Temivu" => Some(
                    text.override_text_color
                        .unwrap_or(text.galley.job.sections[0].format.color),
                ),
                _ => None,
            })
    };
    assert_eq!(
        heading_color(&harness),
        Some(temivu::Theme::light().foreground)
    );
    harness.get_by_label("Dark theme").click();
    harness.run_steps(3);
    assert!(harness.ctx.style_of(harness.ctx.theme()).visuals.dark_mode);
    assert_eq!(
        heading_color(&harness),
        Some(temivu::Theme::dark().foreground)
    );
    harness.get_by_label("High contrast").click();
    harness.run_steps(3);
    assert_eq!(
        harness
            .get_by_label("High contrast")
            .accesskit_node()
            .toggled(),
        Some(egui::accesskit::Toggled::True)
    );
    harness.get_by_label("Reduced motion").click();
    harness.run_steps(3);
    assert_eq!(
        harness.ctx.style_of(harness.ctx.theme()).animation_time,
        0.0
    );

    navigate(&mut harness, "Controls");
    harness.get_by_label("Price range start");
    harness.get_by_label("Price range end");
    harness.get_by_label("Accept terms");
    assert_eq!(
        harness.get_by_label("Option A").accesskit_node().toggled(),
        Some(egui::accesskit::Toggled::True)
    );
    harness.get_by_label("Option B").click();
    harness.run_steps(3);
    assert_eq!(
        harness.get_by_label("Option B").accesskit_node().toggled(),
        Some(egui::accesskit::Toggled::True)
    );
    assert_eq!(
        harness.get_by_label("Option A").accesskit_node().toggled(),
        Some(egui::accesskit::Toggled::False)
    );
    harness.get_by_label("Privacy").click();
    harness.run_steps(3);
    harness.get_by_label("Privacy settings content");
    harness.get_by_label("Details").click();
    harness.run_steps(3);
    harness.get_by_label("Accordion content");

    harness
        .get_by_role_and_label(egui::accesskit::Role::ComboBox, "Color")
        .click();
    harness.run_steps(3);
    harness.get_by_label("Blue").click();
    harness.run_steps(3);
    assert_eq!(
        harness
            .get_by_role_and_label(egui::accesskit::Role::ComboBox, "Color")
            .accesskit_node()
            .value()
            .as_deref(),
        Some("Blue")
    );
    harness.get_by_label("Appearance");
    let date_field = harness.get_by_role_and_label(egui::accesskit::Role::TextInput, "Start date");
    date_field.click();
    harness.run_steps(3);
    harness
        .get_by_role_and_label(egui::accesskit::Role::TextInput, "Start date")
        .type_text("2026-12-01");
    harness.run_steps(3);
    assert_eq!(
        harness
            .get_by_role_and_label(egui::accesskit::Role::TextInput, "Start date")
            .accesskit_node()
            .value()
            .as_deref(),
        Some("2026-12-01")
    );
    harness.get_by_role_and_label(egui::accesskit::Role::Button, "Choose date");
    let time_field = harness.get_by_role_and_label(egui::accesskit::Role::TextInput, "Start time");
    time_field.click();
    harness.run_steps(3);
    harness
        .get_by_role_and_label(egui::accesskit::Role::TextInput, "Start time")
        .type_text("14:30");
    harness.run_steps(3);
    assert_eq!(
        harness
            .get_by_role_and_label(egui::accesskit::Role::TextInput, "Start time")
            .accesskit_node()
            .value()
            .as_deref(),
        Some("14:30")
    );
    let fruit = harness.get_by_role_and_label(egui::accesskit::Role::ComboBox, "Fruit");
    fruit.click();
    harness.run_steps(3);
    harness
        .get_by_role_and_label(egui::accesskit::Role::ComboBox, "Fruit")
        .type_text("Ban");
    harness.run_steps(3);
    harness.get_by_label("Banana").click();
    harness.run_steps(3);
    assert_eq!(
        harness
            .get_by_role_and_label(egui::accesskit::Role::ComboBox, "Fruit")
            .accesskit_node()
            .value()
            .as_deref(),
        Some("Banana")
    );
    harness
        .get_by_role_and_label(egui::accesskit::Role::ListBoxOption, "Large")
        .click();
    harness.run_steps(3);
    assert_eq!(
        harness
            .get_by_role_and_label(egui::accesskit::Role::ListBoxOption, "Large")
            .accesskit_node()
            .is_selected(),
        Some(true)
    );
    harness
        .get_by_role_and_label(egui::accesskit::Role::ComboBox, "Notification channels")
        .click();
    harness.run_steps(3);
    harness.get_by_label("Email digest").click();
    harness.run_steps(3);
    assert_eq!(
        harness
            .get_by_role_and_label(egui::accesskit::Role::ComboBox, "Notification channels")
            .accesskit_node()
            .value()
            .as_deref(),
        Some("Email digest")
    );
    harness.key_press(egui::Key::Escape);
    harness.run_steps(3);

    harness
        .get_by_role_and_label(egui::accesskit::Role::TextInput, "Verification code")
        .focus();
    harness.run_steps(3);
    harness
        .get_by_role_and_label(egui::accesskit::Role::TextInput, "Verification code")
        .type_text("123456");
    harness.run_steps(3);
    assert_eq!(
        harness
            .get_by_role_and_label(egui::accesskit::Role::TextInput, "Verification code")
            .accesskit_node()
            .value()
            .as_deref(),
        Some("123456")
    );
}

#[test]
fn showcase_overlays_open_and_close() {
    let mut harness = harness();
    navigate(&mut harness, "Overlays");
    harness.get_by_label("Click me").click();
    harness.run_steps(3);
    harness.get_by_label("Clicks: 1");
    harness.get_by_label("Click me").focus();
    harness.run_steps(3);
    harness.key_press_modifiers(egui::Modifiers::SHIFT, egui::Key::F10);
    harness.run_steps(3);
    harness.key_press(egui::Key::Enter);
    harness.run_steps(3);
    harness.get_by_label("Clicks: 0");

    harness.get_by_label("Open dialog").click();
    harness.run_steps(3);
    harness.get_by_role_and_label(egui::accesskit::Role::Dialog, "Confirm action");
    harness.key_press(egui::Key::Escape);
    harness.run_steps(3);
    harness.run_steps(3);
    assert!(
        harness
            .query_by_role(egui::accesskit::Role::Dialog)
            .is_none()
    );

    harness.get_by_label("Show popover").click();
    harness.run_steps(3);
    harness.get_by_label("Popover content");
    harness.key_press(egui::Key::Escape);
    harness.run_steps(3);
    assert!(harness.query_by_label("Popover content").is_none());

    harness.get_by_label("Show menu").click();
    harness.run_steps(3);
    harness.get_by_label("Rename").click();
    harness.run_steps(3);
    assert!(harness.query_by_label("Rename").is_none());

    harness.get_by_label("Open sheet").click();
    harness.run_steps(3);
    harness.get_by_role_and_label(egui::accesskit::Role::Dialog, "Filters");
    harness.get_by_label("Close").click();
    harness.run_steps(3);
    assert!(harness.query_by_label("Sheet content").is_none());

    harness.get_by_label("Show menu").click();
    harness.run_steps(3);
    harness.get_by_label("Pin to top").click();
    harness.run_steps(3);
    assert!(harness.query_by_label("Pin to top").is_none());
    harness.get_by_label("Show menu").click();
    harness.run_steps(3);
    assert_eq!(
        harness
            .get_by_role_and_label(egui::accesskit::Role::MenuItem, "Pin to top")
            .accesskit_node()
            .is_selected(),
        Some(true)
    );
    harness.key_press(egui::Key::Escape);
    harness.run_steps(3);

    harness.get_by_label("Queue notifications").click();
    harness.run_steps(3);
    harness.get_by_label("Queue notifications").click();
    harness.run_steps(3);
    assert_eq!(
        harness
            .query_all_by(|node: &egui_kittest::kittest::AccessKitNode<'_>| {
                node.role() == egui::accesskit::Role::Alert
                    && node
                        .label()
                        .is_some_and(|label| label.contains("Queued notification 1"))
            })
            .count(),
        1,
        "the first queued toast should be an alert"
    );
    assert_eq!(
        harness
            .query_all_by(|node: &egui_kittest::kittest::AccessKitNode<'_>| {
                node.role() == egui::accesskit::Role::Alert
                    && node
                        .label()
                        .is_some_and(|label| label.contains("Queued notification 2"))
            })
            .count(),
        1,
        "the second queued toast should still be visible"
    );
    harness
        .get_by_label("Dismiss: Queued notification 1")
        .click();
    harness.run_steps(3);
    assert_eq!(
        harness
            .query_all_by(|node: &egui_kittest::kittest::AccessKitNode<'_>| {
                node.role() == egui::accesskit::Role::Alert
                    && node
                        .label()
                        .is_some_and(|label| label.contains("Queued notification 2"))
            })
            .count(),
        1,
        "the second queued toast should still be visible"
    );
    assert_eq!(
        harness
            .query_all_by(|node: &egui_kittest::kittest::AccessKitNode<'_>| {
                node.role() == egui::accesskit::Role::Alert
                    && node
                        .label()
                        .is_some_and(|label| label.contains("Queued notification 1"))
            })
            .count(),
        0,
        "dismissing a queued toast should remove it from the overlay"
    );
    // Wait for the exit to reflow the anchored stack before clicking its next card.
    harness.run_steps(24);
    harness
        .get_by_label("Dismiss: Queued notification 2")
        .click();
    harness.run_steps(3);
    assert!(
        harness
            .query_by_label("Dismiss: Queued notification 2")
            .is_none(),
        "the remaining toast should still be dismissible after the stack reflows"
    );
    harness.get_by_label("Show notification").click();
    harness.run_steps(3);
    harness.get_by_role_and_label(
        egui::accesskit::Role::Alert,
        "Changes saved: Your preferences are up to date.",
    );
    harness.get_by_label("Dismiss").click();
    harness.run_steps(3);
    assert!(
        harness
            .query_by_role(egui::accesskit::Role::Alert)
            .is_none()
    );
}

#[test]
fn showcase_data_and_localization_work() {
    let mut harness = harness();
    navigate(&mut harness, "Data");
    let item = harness.get_by_label("Project brief");
    assert_eq!(item.accesskit_node().is_selected(), Some(false));
    item.click();
    harness.run_steps(3);
    assert_eq!(
        harness
            .get_by_label("Project brief")
            .accesskit_node()
            .is_selected(),
        Some(true)
    );
    harness.get_by_label("Bluetooth").click();
    harness.run_steps(3);
    assert_eq!(
        harness
            .get_by_label("Bluetooth")
            .accesskit_node()
            .is_selected(),
        Some(true)
    );
    harness.get_by_label("Page 2 of 12");
    harness.get_by_label("Next").click();
    harness.run_steps(3);
    harness.get_by_label("Page 3 of 12");

    harness.get_by_label("FOLDERS");
    harness.get_by_label("FOLDER CONTENTS");
    harness.get_by_label("November 2026");
    harness.get_by_label_contains("November 1, 2026").click();
    harness.run_steps(3);
    assert_eq!(
        harness
            .get_by_label_contains("November 1, 2026")
            .accesskit_node()
            .is_selected(),
        Some(true)
    );

    harness.get_by_role_and_label(egui::accesskit::Role::Tree, "Demo navigation");
    harness
        .get_by_role_and_label(egui::accesskit::Role::TreeItem, "Inbox")
        .click();
    harness.run_steps(3);
    assert_eq!(
        harness
            .get_by_role_and_label(egui::accesskit::Role::TreeItem, "Inbox")
            .accesskit_node()
            .is_selected(),
        Some(true)
    );
    harness.get_by_role_and_label(egui::accesskit::Role::List, "Radios");
    harness
        .get_by_role_and_label(egui::accesskit::Role::ListItem, "Airplane mode")
        .click();
    harness.run_steps(3);
    assert_eq!(
        harness
            .get_by_role_and_label(egui::accesskit::Role::ListItem, "Airplane mode")
            .accesskit_node()
            .is_selected(),
        Some(true)
    );
    // The page contains a standalone time picker and a combined date-time
    // picker whose embedded time wheels also expose a "Time" group, so assert
    // the combined group and the first hour wheel rather than a unique "Time".
    harness.get_by_role_and_label(egui::accesskit::Role::Group, "Date and time");
    harness.get_by_role_and_label(egui::accesskit::Role::List, "Date strip");
    harness
        .get_by_role_and_label(egui::accesskit::Role::ListItem, "November 14, 2026")
        .click();
    harness.run_steps(3);
    assert_eq!(
        harness
            .get_by_role_and_label(egui::accesskit::Role::ListItem, "November 14, 2026")
            .accesskit_node()
            .is_selected(),
        Some(true)
    );
}

#[test]
fn localization_preview_renders_scripts_and_pages_at_phone_width() {
    let mut harness = Harness::builder()
        .with_size(egui::vec2(390.0, 844.0))
        .build_ui_state(
            |ui, app: &mut showcase::Showcase| {
                let mut frame = eframe::Frame::_new_kittest();
                app.ui(ui, &mut frame);
            },
            showcase::Showcase::default(),
        );
    harness.run_steps(3);
    navigate(&mut harness, "Localization");
    for label in [
        "English (en)",
        "Deutsch (de)",
        "العربية (ar)",
        "日本語 (ja)",
        "中文 (zh)",
        "Français (fr)",
    ] {
        assert!(harness.get_by_label(label).rect().right() <= 390.0);
    }
    harness.get_by_label("Deutsch (de)").click();
    harness.run_steps(3);
    harness.get_by_label("Wird geladen");
    harness.get_by_label("Zurück");
    harness.get_by_label("Weiter").click();
    harness.run_steps(3);
    harness.get_by_label("Seite 2");
    harness.get_by_label("日本語 (ja)").click();
    harness.run_steps(3);
    harness.get_by_label("読み込み中");
    for text in ["العربية", "日本語", "中文", "読み込み中"] {
        assert!(
            harness
                .ctx
                .fonts_mut(|fonts| fonts.has_glyphs(&egui::FontId::proportional(16.0), text))
        );
    }
}

#[test]
fn a_non_animated_page_stops_requesting_repaints() {
    let mut harness = harness();
    navigate(&mut harness, "Controls");
    // Allow the floating navigation's spring to settle after changing tabs.
    harness.run_steps(8);
    assert!(
        !harness.ctx.has_requested_repaint(),
        "the controls page should not keep requesting repaints when idle: {:?}",
        harness.ctx.repaint_causes()
    );
}

#[test]
fn upload_preview_runs_to_completion_and_resets() {
    let mut harness = harness();
    harness.get_by_label("Start upload").click();
    harness.step();
    let started = harness.ctx.input(|input| input.time);
    harness.input_mut().time = Some(started + 1.5);
    harness.step();
    let midway = harness
        .get_by_label("Upload progress")
        .accesskit_node()
        .numeric_value()
        .unwrap();
    assert!((45.0..55.0).contains(&midway));
    harness.input_mut().time = Some(started + 3.1);
    harness.run_steps(3);
    assert_eq!(
        harness
            .get_by_label("Upload progress")
            .accesskit_node()
            .numeric_value(),
        Some(100.0)
    );
    harness.get_by_label("Reset").click();
    harness.run_steps(3);
    assert_eq!(
        harness
            .get_by_label("Upload progress")
            .accesskit_node()
            .numeric_value(),
        Some(0.0)
    );
}

#[test]
#[ignore = "manual CPU timing probe; not a wall-clock test gate"]
fn theme_switch_timing_probe() {
    let mut harness = harness();
    let mut times = Vec::new();
    for index in 0..30 {
        harness.get_by_label("Dark theme").click();
        let start = std::time::Instant::now();
        harness.run_steps(3);
        if index >= 4 {
            times.push(start.elapsed().as_secs_f64() * 1_000.0);
        }
    }
    times.sort_by(f64::total_cmp);
    eprintln!(
        "theme switch (3 headless frames): median {:.3} ms, max {:.3} ms",
        times[times.len() / 2],
        times.last().unwrap()
    );
}

#[test]
fn showcase_uses_droplet_for_the_surface_preview() {
    use temivu::liquid::{LiquidStyle, SurfaceTransition};
    let mut harness = harness();
    navigate(&mut harness, "Controls");
    assert_eq!(
        LiquidStyle::context_default(&harness.ctx).transition,
        SurfaceTransition::Droplet
    );
    harness
        .get_by_role_and_label(
            egui::accesskit::Role::Button,
            "Preview Droplet from any point",
        )
        .click();
    harness.run_steps(3);
    harness.get_by_label("Expand and retract from the same control.");
    harness
        .get_by_role_and_label(egui::accesskit::Role::Button, "Done")
        .click();
    harness.run_steps(3);
    assert!(
        harness
            .query_by_label("Expand and retract from the same control.")
            .is_none()
    );
}

#[test]
fn motion_gallery_exposes_examples_and_loading_on_a_narrow_page() {
    let mut harness = Harness::builder()
        .with_size(egui::vec2(390.0, 3000.0))
        .build_ui_state(
            |ui, app: &mut showcase::Showcase| {
                app.ui(ui, &mut eframe::Frame::_new_kittest());
            },
            showcase::Showcase::default(),
        );
    harness.run_steps(3);
    navigate(&mut harness, "Controls");
    harness.get_by_label("Motion examples").click();
    harness.run_steps(3);
    harness.get_by_label("Motion gallery");
    for label in [
        "Media",
        "Surfaces",
        "Records",
        "Foundations",
        "Circular loading",
        "Segments",
        "Quantity",
        "Layout",
        "Follow",
        "Pagination",
        "History",
        "Switch",
        "Action menu",
        "Category",
        "Link preview",
        "Search",
        "Direction pad",
    ] {
        assert!(
            harness
                .get_all_by_label(label)
                .any(|node| node.rect().right() <= 390.0),
            "{label}"
        );
    }
    harness.get_by_label("Increase quantity").click();
    harness.run_steps(5);
    assert!(harness.get_by_label("Decrease quantity").rect().right() <= 390.0);
}

#[test]
fn motion_lab_switches_presets_and_exposes_custom_parameters() {
    let mut harness = harness();
    harness.get_by_label("Widgets").click();
    harness.run_steps(3);
    harness
        .get_by_role_and_label(egui::accesskit::Role::TextInput, "Find a widget")
        .focus();
    harness.run_steps(2);
    harness.event(egui::Event::Text("motion lab".to_owned()));
    harness.run_steps(3);
    harness.get_by_label("Smooth").click();
    harness.run_steps(3);
    harness.get_by_label("Critical  ·  ζ 1.00");
    harness.get_by_label("Custom spring").click();
    harness.run_steps(3);
    harness.get_by_role_and_label(egui::accesskit::Role::Slider, "Mass");
    harness.get_by_role_and_label(egui::accesskit::Role::Slider, "Stiffness");
    harness.get_by_role_and_label(egui::accesskit::Role::Slider, "Damping");
}

#[test]
fn circular_loading_demo_fills_to_completion() {
    let mut harness = harness();
    navigate(&mut harness, "Controls");
    harness.get_by_label("Motion examples").click();
    harness.run_steps(3);
    assert_eq!(
        harness
            .get_by_label("Circular progress")
            .accesskit_node()
            .numeric_value(),
        None
    );
    harness.get_by_label("Start loading").click();
    harness.run_steps(14);
    assert_eq!(
        harness
            .get_by_label("Circular progress")
            .accesskit_node()
            .numeric_value(),
        Some(100.0)
    );
    assert!(
        !harness
            .get_by_label("Start loading")
            .accesskit_node()
            .is_disabled()
    );
}

#[test]
fn widgets_open_from_the_header_and_filter_by_name() {
    let mut harness = harness();
    harness.get_by_label("Widgets").click();
    harness.run_steps(3);
    harness.get_by_label("Motion gallery");
    harness.get_by_label("15 of 15 examples");
    harness
        .get_by_role_and_label(egui::accesskit::Role::TextInput, "Find a widget")
        .focus();
    harness.run_steps(2);
    harness.event(egui::Event::Text("direction".to_owned()));
    harness.run_steps(3);
    harness.get_by_label("1 of 15 examples");
    harness.get_by_label("Direction pad");
    assert!(harness.query_by_label("Quantity").is_none());
}

#[test]
fn catalog_searches_across_media_and_the_reused_sections() {
    let mut harness = harness();
    harness.get_by_label("Widgets").click();
    harness.run_steps(3);
    harness.get_by_label("Media").click();
    harness.run_steps(3);
    harness.get_by_label("Video player");
    harness
        .get_by_role_and_label(egui::accesskit::Role::TextInput, "Find a widget")
        .focus();
    harness.run_steps(2);
    harness.event(egui::Event::Text("resizable".to_owned()));
    harness.run_steps(3);
    harness.get_by_label("Navigation sidebar");
    assert!(harness.query_by_label("Video player").is_none());
}

#[test]
fn video_source_can_be_opened_from_the_keyboard() {
    let mut app = harness();
    app.get_by_label("Widgets").click();
    app.run_steps(3);
    app.get_by_label("Media").click();
    app.run_steps(3);
    app.get_by_label("Choose video…");
    app.get_by_label("Open path");
    app.get_by_role_and_label(egui::accesskit::Role::TextInput, "Local video path")
        .focus();
    app.run_steps(2);
    app.event(egui::Event::Text(" \".\" ".to_owned()));
    app.run_steps(2);
    app.key_press(egui::Key::Enter);
    app.run_steps(3);
    app.get_by_label("Choose a video file, not a folder.");
}

#[test]
fn widgets_opens_and_returns_without_a_page_morph() {
    let mut app = harness();
    let title = |app: &Harness<'_, showcase::Showcase>, label: &str| {
        app.output()
            .shapes
            .iter()
            .find_map(|entry| match &entry.shape {
                egui::Shape::Text(text)
                    if text.galley.text() == label
                        && text.opacity_factor == 1.0
                        && text.visual_bounding_rect().intersects(entry.clip_rect) =>
                {
                    Some(text.pos)
                }
                _ => None,
            })
    };
    let overview = title(&app, "Welcome to Temivu").expect("initial title");
    app.get_by_label("Widgets").click();
    app.input_mut().time = Some(10.0);
    app.step();
    app.input_mut().time = Some(10.02);
    app.step();
    let gallery = title(&app, "Motion gallery").expect("gallery is immediately visible");
    assert!(app.query_by_label("Welcome to Temivu").is_none());

    app.input_mut().time = Some(11.0);
    app.step();
    assert_eq!(title(&app, "Motion gallery"), Some(gallery));

    app.get_by_label("Back").click();
    app.input_mut().time = Some(11.03);
    app.step();
    app.input_mut().time = Some(11.05);
    app.step();
    assert_eq!(title(&app, "Welcome to Temivu"), Some(overview));
    assert!(app.query_by_label("Motion gallery").is_none());
}

#[test]
fn showcase_glass_is_optional_and_yields_to_high_contrast() {
    let mut harness = harness();
    assert!(temivu::Glass::context(&harness.ctx).is_none());
    harness.get_by_label("Glass surfaces").click();
    harness.run_steps(3);
    assert!(temivu::Glass::context(&harness.ctx).is_some());
    harness.get_by_label("Glass tint");
    harness.get_by_label("Frost glassmorphism");
    harness.get_by_label("Prism optical glass");

    harness.get_by_label("Frost").click();
    harness.run_steps(3);
    assert!(
        harness
            .get_by_label("Frost")
            .accesskit_node()
            .is_selected()
            .unwrap_or(false)
    );
    assert_eq!(
        temivu::Glass::context(&harness.ctx),
        Some(temivu::Glass::new(0.28).mode(temivu::GlassMode::Frost))
    );

    harness.get_by_label("High contrast").click();
    harness.run_steps(3);
    assert!(temivu::Glass::context(&harness.ctx).is_none());
    assert!(harness.query_by_label("Glass studio").is_none());
    harness.get_by_label("High contrast").click();
    harness.run_steps(3);
    assert!(
        harness
            .get_by_label("Frost")
            .accesskit_node()
            .is_selected()
            .unwrap_or(false)
    );

    harness.get_by_label("Glass surfaces").click();
    harness.run_steps(3);
    assert!(temivu::Glass::context(&harness.ctx).is_none());
}

#[test]
fn touch_overscroll_resists_drag_and_returns_to_edge() {
    let mut harness = Harness::builder()
        .with_size(egui::vec2(390.0, 844.0))
        .build_ui_state(
            |ui, app: &mut showcase::Showcase| {
                app.ui(ui, &mut eframe::Frame::_new_kittest());
            },
            showcase::Showcase::default(),
        );
    harness.run_steps(3);

    let heading = |harness: &Harness<'_, showcase::Showcase>| {
        harness.get_by_label("Welcome to Temivu").rect().top()
    };
    let initial = heading(&harness);
    let start = egui::pos2(350.0, 90.0);
    let touch = |pos, phase| egui::Event::Touch {
        device_id: egui::TouchDeviceId(0),
        id: egui::TouchId(1),
        phase,
        pos,
        force: None,
    };
    let primary = |pos, pressed| egui::Event::PointerButton {
        pos,
        button: egui::PointerButton::Primary,
        pressed,
        modifiers: egui::Modifiers::NONE,
    };
    let mut time = 1.0;
    let mut step = |harness: &mut Harness<'_, showcase::Showcase>, events: Vec<_>| {
        time += 1.0 / 60.0;
        harness.input_mut().time = Some(time);
        harness.input_mut().events.extend(events);
        harness.step();
    };
    step(
        &mut harness,
        vec![
            egui::Event::PointerMoved(start),
            touch(start, egui::TouchPhase::Start),
        ],
    );
    step(&mut harness, vec![primary(start, true)]);
    let primed = start + egui::vec2(0.0, 60.0);
    step(
        &mut harness,
        vec![
            egui::Event::PointerMoved(primed),
            touch(primed, egui::TouchPhase::Move),
        ],
    );

    step(&mut harness, vec![]);
    let mut previous = heading(&harness);
    let mut last_delta = f32::INFINITY;
    for distance in [120.0, 180.0, 240.0] {
        let pos = start + egui::vec2(0.0, distance);
        step(
            &mut harness,
            vec![
                egui::Event::PointerMoved(pos),
                touch(pos, egui::TouchPhase::Move),
            ],
        );
        step(&mut harness, vec![]);
        let current = heading(&harness);
        let delta = current - previous;
        assert!(
            delta > 0.0 && delta < last_delta,
            "pull {distance}: moved {delta}"
        );
        previous = current;
        last_delta = delta;
    }

    let end = start + egui::vec2(0.0, 240.0);
    step(
        &mut harness,
        vec![
            touch(end, egui::TouchPhase::End),
            primary(end, false),
            egui::Event::PointerGone,
        ],
    );
    for _ in 0..90 {
        step(&mut harness, vec![]);
    }
    assert!((heading(&harness) - initial).abs() < 1.0);
}
