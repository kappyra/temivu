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
fn showcase_pages_theme_and_controls_work() {
    let mut harness = harness();
    harness.get_by_label("An experimental Forui-inspired component library for egui.");
    harness.get_by_label("Dark theme").click();
    harness.run_steps(3);
    assert!(harness.ctx.style_of(harness.ctx.theme()).visuals.dark_mode);
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
    if harness
        .query_by_label("Dismiss: Queued notification 2")
        .is_some()
    {
        harness
            .get_by_label("Dismiss: Queued notification 2")
            .click();
    }
    harness.run_steps(3);
    harness.get_by_label("Show notification").click();
    harness.run_steps(3);
    harness.get_by_role_and_label(
        egui::accesskit::Role::Alert,
        "Notification: This notification closes when dismissed.",
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
    let item = harness.get_by_label("Example item");
    assert_eq!(item.accesskit_node().is_selected(), Some(false));
    item.click();
    harness.run_steps(3);
    assert_eq!(
        harness
            .get_by_label("Example item")
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
    assert_eq!(
        harness.get_by_label("2").accesskit_node().is_selected(),
        Some(true)
    );
    harness.get_by_label("Next").click();
    harness.run_steps(3);
    assert_eq!(
        harness.get_by_label("3").accesskit_node().is_selected(),
        Some(true)
    );

    harness.get_by_label("Left pane");
    harness.get_by_label("Right pane");
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

    navigate(&mut harness, "Localization");
    harness.get_by_label("Deutsch (de)").click();
    harness.run_steps(3);
    harness.get_by_label("progressSemanticsLabel = Wird geladen");
    harness.get_by_label("Zurück");
    harness.get_by_label("Weiter").click();
    harness.run_steps(3);
    assert_eq!(
        harness
            .get_by_label("Seite 4")
            .accesskit_node()
            .is_selected(),
        Some(true)
    );
}

#[test]
fn a_non_animated_page_stops_requesting_repaints() {
    let mut harness = harness();
    navigate(&mut harness, "Controls");
    // The overview contains an indeterminate circular progress indicator that
    // asks for another frame; the controls page must settle instead.
    harness.step();
    assert!(
        !harness.ctx.has_requested_repaint(),
        "the controls page should not keep requesting repaints when idle"
    );
}
