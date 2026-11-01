use egui_kittest::{
    Harness,
    kittest::{NodeT, Queryable},
};
use temivu::{Item, ItemVariant, Theme};

#[test]
fn item_reports_selection_description_and_keyboard_activation() {
    let mut harness = Harness::builder().build_ui_state(
        |ui, selected: &mut bool| {
            if Item::new("Account", &Theme::light())
                .description("Manage profile")
                .selected(*selected)
                .show(ui)
                .clicked()
            {
                *selected = !*selected;
            }
        },
        false,
    );
    harness.run();
    let node = harness.get_by_label("Account");
    assert_eq!(
        node.accesskit_node().description().as_deref(),
        Some("Manage profile")
    );
    assert_eq!(node.accesskit_node().is_selected(), Some(false));
    assert_eq!(node.accesskit_node().toggled(), None);
    node.focus();
    harness.run();
    harness.key_press(egui::Key::Enter);
    harness.run();
    assert!(*harness.state());
    assert_eq!(
        harness
            .get_by_label("Account")
            .accesskit_node()
            .is_selected(),
        Some(true)
    );
    assert_eq!(
        harness.get_by_label("Account").accesskit_node().toggled(),
        None
    );
}

#[test]
fn disabled_destructive_item_ignores_stale_accessibility_click() {
    let mut harness = Harness::builder().build_ui_state(
        |ui, clicked: &mut bool| {
            if Item::new("Delete", &Theme::light())
                .variant(ItemVariant::Destructive)
                .enabled(false)
                .show(ui)
                .clicked()
            {
                *clicked = true;
            }
        },
        false,
    );
    harness.run();
    let node = harness.get_by_label("Delete");
    assert!(node.accesskit_node().is_disabled());
    let (target_node, target_tree) = node.accesskit_node().locate();
    harness.event(egui::Event::AccessKitActionRequest(
        egui::accesskit::ActionRequest {
            action: egui::accesskit::Action::Click,
            target_node,
            target_tree,
            data: None,
        },
    ));
    harness.run();
    assert!(!*harness.state());
}

#[test]
fn long_item_wraps_within_narrow_viewport() {
    let mut harness = Harness::builder()
        .with_size(egui::vec2(200.0, 300.0))
        .build_ui(|ui| {
            Item::new("A long account settings title", &Theme::light())
                .description("Manage your profile and preferences for this device")
                .show(ui);
        });
    harness.run();
    assert!(
        harness
            .get_by_label("A long account settings title")
            .rect()
            .right()
            <= 200.0
    );
}

#[test]
fn selected_item_uses_forui_secondary_fill_and_keeps_text_readable() {
    for theme in [Theme::light(), Theme::dark()] {
        // Active rows blend the upstream text colors just enough to reach AA.
        for color in [theme.foreground, theme.muted_foreground, theme.destructive] {
            assert!(
                Theme::contrast_ratio(
                    theme.readable_on(color, theme.secondary, 4.5),
                    theme.secondary
                ) >= 4.5
            );
        }
        let selected_fill = theme.secondary;
        let mut harness = Harness::builder().build_ui(move |ui| {
            Item::new("Selected", &theme).selected(true).show(ui);
        });
        harness.run();
        let container = harness
            .output()
            .shapes
            .iter()
            .find_map(|shape| match &shape.shape {
                egui::Shape::Rect(rect)
                    if rect.fill == selected_fill
                        && rect.corner_radius.nw == 10
                        && rect.stroke.width == 0.0 =>
                {
                    Some(rect.rect)
                }
                _ => None,
            })
            .expect("selected item uses the upstream secondary fill and md radius");
        // Forui item: 4 point outer margin and the 30 point desktop item height.
        let row = harness.get_by_label("Selected").rect();
        assert_eq!(container.left(), row.left() + 4.0);
        assert_eq!(container.right(), row.right() - 4.0);
        assert!(
            container.height() >= 29.0,
            "item height: {}",
            container.height()
        );
    }
}

#[test]
fn item_hover_uses_forui_secondary_fill() {
    let theme = Theme::light();
    let hovered_fill = theme.secondary;
    let mut harness = Harness::builder().build_ui(move |ui| {
        Item::new("Hover me", &theme).show(ui);
    });
    harness.run();
    assert!(!harness.output().shapes.iter().any(|shape| matches!(
        &shape.shape,
        egui::Shape::Rect(rect) if rect.fill == hovered_fill && rect.corner_radius.nw == 10
    )));
    let rect = harness.get_by_label("Hover me").rect();
    harness.hover_at(rect.center());
    harness.run();
    assert!(harness.output().shapes.iter().any(|shape| matches!(
        &shape.shape,
        egui::Shape::Rect(rect) if rect.fill == hovered_fill && rect.corner_radius.nw == 10
    )));
}

#[test]
fn destructive_item_keeps_forui_destructive_text_color() {
    let theme = Theme::light();
    let destructive = theme.destructive;
    let mut harness = Harness::builder().build_ui(move |ui| {
        Item::new("Delete", &theme)
            .variant(ItemVariant::Destructive)
            .show(ui);
    });
    harness.run();
    assert!(harness.output().shapes.iter().any(|shape| matches!(
        &shape.shape,
        egui::Shape::Text(text) if text.fallback_color == destructive
    )));
}

#[test]
fn own_disabled_item_is_dimmed_once() {
    let theme = Theme::light();
    let dimmed = theme.disable(theme.foreground);
    let mut harness = Harness::builder().build_ui(move |ui| {
        Item::new("Own disabled", &theme).enabled(false).show(ui);
    });
    harness.run();
    assert!(
        harness.output().shapes.iter().any(|shape| matches!(
            &shape.shape,
            egui::Shape::Text(text) if text.fallback_color == dimmed
        )),
        "an item disabled by its own flag should use the Forui dim color"
    );
}

#[test]
fn item_nested_in_a_disabled_ui_is_not_dimmed_twice() {
    let theme = Theme::light();
    let mut harness = Harness::builder().build_ui(move |ui| {
        ui.add_enabled_ui(false, |ui| {
            Item::new("Parent disabled", &theme).show(ui);
        });
    });
    harness.run();
    // egui already fades shapes painted inside a disabled Ui to about half
    // opacity. The item must pass its undimmed colors through so the result is
    // one 50% fade, not two (which would leave roughly 25% alpha).
    let faded_alpha = harness
        .output()
        .shapes
        .iter()
        .filter_map(|shape| match &shape.shape {
            egui::Shape::Text(text) => Some(text.fallback_color.to_srgba_unmultiplied()[3]),
            _ => None,
        })
        .max()
        .expect("the item paints text");
    assert!(
        faded_alpha > 96 && faded_alpha < 192,
        "expected a single egui fade (about 128 alpha), got {faded_alpha}"
    );
}
