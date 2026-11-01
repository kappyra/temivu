use egui_kittest::{
    Harness,
    kittest::{NodeT, Queryable},
};
use temivu::{Breadcrumb, BreadcrumbItem, Theme};

#[test]
fn links_report_activated_index_and_current_item_is_selected() {
    let mut harness = Harness::builder().build_ui_state(
        |ui, activated: &mut Option<usize>| {
            let items = [
                BreadcrumbItem::link("Home"),
                BreadcrumbItem::link("Library"),
                BreadcrumbItem::current("Current"),
            ];
            if let Some(index) = Breadcrumb::new(&items, &Theme::light()).show(ui) {
                *activated = Some(index);
            }
        },
        None,
    );
    harness.run();
    assert_eq!(
        harness.get_by_label("Home").accesskit_node().role(),
        egui::accesskit::Role::Link
    );
    assert_eq!(
        harness
            .get_by_label("Current")
            .accesskit_node()
            .is_selected(),
        Some(true)
    );
    harness.get_by_label("Library").click();
    harness.run();
    assert_eq!(*harness.state(), Some(1));
}

#[test]
fn empty_and_noninteractive_breadcrumbs_never_report_activation() {
    let mut harness = Harness::builder().build_ui_state(
        |ui, activated: &mut Option<usize>| {
            let _ = Breadcrumb::new(&[], &Theme::dark()).show(ui);
            let items = [BreadcrumbItem::current("Current")];
            *activated = Breadcrumb::new(&items, &Theme::dark()).show(ui);
        },
        None,
    );
    harness.get_by_label("Current").click();
    harness.run();
    assert_eq!(*harness.state(), None);
}

#[test]
fn keyboard_activation_reports_the_correct_crumb() {
    let mut harness = Harness::builder().build_ui_state(
        |ui, activated: &mut Option<usize>| {
            let items = [
                BreadcrumbItem::link("Home"),
                BreadcrumbItem::current("Current"),
            ];
            if let Some(index) = Breadcrumb::new(&items, &Theme::light()).show(ui) {
                *activated = Some(index);
            }
        },
        None,
    );
    harness.get_by_label("Home").focus();
    harness.run();
    harness.key_press(egui::Key::Enter);
    harness.run();
    assert_eq!(*harness.state(), Some(0));
}
