use egui_kittest::{
    Harness,
    kittest::{NodeT, Queryable},
};
use temivu::{Tabs, Theme};

#[test]
fn rejects_empty_and_out_of_bounds() {
    let theme = Theme::light();
    let mut index = 0;
    assert!(Tabs::new(&mut index, &[], &theme).is_err());
    index = 1;
    assert!(Tabs::new(&mut index, &["Only"], &theme).is_err());
}

#[test]
fn click_selects_content_and_reports_tab_role() {
    let mut harness = Harness::builder().build_ui_state(
        |ui, selected: &mut usize| {
            Tabs::new(selected, &["Account", "Settings"], &Theme::light())
                .unwrap()
                .show(ui, |ui, index| {
                    ui.label(if index == 0 {
                        "Account content"
                    } else {
                        "Settings content"
                    });
                });
        },
        0,
    );
    assert!(harness.get_by_label("Account content").rect().is_positive());
    let node = harness.get_by_label("Account");
    assert_eq!(node.accesskit_node().role(), egui::accesskit::Role::Tab);
    assert_eq!(node.accesskit_node().is_selected(), Some(true));
    harness.get_by_label("Settings").click();
    harness.run();
    assert_eq!(*harness.state(), 1);
    assert!(
        harness
            .get_by_label("Settings content")
            .rect()
            .is_positive()
    );
    assert!(harness.query_by_label("Account content").is_none());
    assert_eq!(
        harness
            .get_by_label("Settings")
            .accesskit_node()
            .is_selected(),
        Some(true)
    );
}

#[test]
fn arrow_keys_select_next_tab() {
    let mut harness = Harness::builder().build_ui_state(
        |ui, selected: &mut usize| {
            Tabs::new(selected, &["First", "Second"], &Theme::light())
                .unwrap()
                .show(ui, |_, _| ());
        },
        0,
    );
    harness.get_by_label("First").focus();
    harness.run();
    harness.key_press(egui::Key::ArrowRight);
    harness.run();
    assert_eq!(*harness.state(), 1);
}

#[test]
fn arrow_navigation_wraps_without_focusing_adjacent_controls() {
    let mut harness = Harness::builder().build_ui_state(
        |ui, selected| {
            let _ = ui.button("Before");
            Tabs::new(selected, &["First", "Second", "Third"], &Theme::light())
                .unwrap()
                .show(ui, |_, _| ());
            let _ = ui.button("After");
        },
        0usize,
    );
    harness.get_by_label("First").focus();
    harness.run();
    harness.key_press(egui::Key::ArrowLeft);
    harness.run();
    assert_eq!(*harness.state(), 2);
    assert!(harness.get_by_label("Third").accesskit_node().is_focused());
    harness.key_press(egui::Key::ArrowRight);
    harness.run();
    assert_eq!(*harness.state(), 0);
    assert!(harness.get_by_label("First").accesskit_node().is_focused());
}

#[test]
fn tabs_container_uses_forui_muted_surface_border_and_radius() {
    let theme = Theme::light();
    let fill = theme.muted;
    let border = theme.muted;
    let mut selected = 0;
    let mut harness = Harness::builder().build_ui(move |ui| {
        Tabs::new(&mut selected, &["One", "Two"], &theme)
            .unwrap()
            .show(ui, |ui, _| {
                ui.label("content");
            });
    });
    harness.run();
    let container = harness
        .output()
        .shapes
        .iter()
        .find_map(|shape| match &shape.shape {
            egui::Shape::Rect(rect)
                if rect.fill == fill
                    && rect.stroke.color == border
                    && rect.stroke.width == 1.0
                    && rect.corner_radius.nw == 10 =>
            {
                Some(rect.rect)
            }
            _ => None,
        })
        .expect("tabs container");
    // 36 point tab minimum height plus 4 points of padding on each side.
    assert!(
        container.height() >= 44.0,
        "tabs height: {}",
        container.height()
    );
}
