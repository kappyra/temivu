use egui_kittest::{
    Harness,
    kittest::{NodeT, Queryable},
};
use temivu::{Alert, AlertStyle, AlertVariant, Theme};

#[test]
fn resolves_variant_colors_from_theme() {
    let theme = Theme::light();
    let normal = AlertStyle::from_theme(&theme, AlertVariant::Primary);
    assert_eq!(normal.fill, theme.card);
    assert_eq!(normal.subtitle, theme.muted_foreground);
    let destructive = AlertStyle::from_theme(&theme, AlertVariant::Destructive);
    assert_eq!(destructive.title, theme.destructive);
    assert_eq!(destructive.subtitle, theme.destructive);
    assert_eq!(destructive.icon, theme.destructive);
}

#[test]
fn live_region_can_be_enabled_and_disabled() {
    let mut harness = Harness::builder().build_ui_state(
        |ui, live: &mut bool| {
            let theme = Theme::light();
            let style = AlertStyle::from_theme(&theme, AlertVariant::Destructive);
            Alert::new("Warning", &theme)
                .subtitle("Network lost")
                .icon("!")
                .style(style)
                .live_region(*live)
                .show(ui);
        },
        true,
    );
    harness.run();
    let node = harness.get_by_label("Warning: Network lost");
    assert_eq!(node.accesskit_node().live(), egui::accesskit::Live::Polite);
    assert_eq!(node.accesskit_node().role(), egui::accesskit::Role::Alert);
    assert!(harness.get_by_label("Warning").rect().is_positive());
    *harness.state_mut() = false;
    harness.run();
    assert_eq!(
        harness
            .get_by_label("Warning: Network lost")
            .accesskit_node()
            .live(),
        egui::accesskit::Live::Off
    );
}

#[test]
fn alert_uses_forui_icon_and_title_sizes() {
    let theme = Theme::light();
    let title_size = theme.typography.body.size;
    let mut harness = Harness::builder().build_ui(move |ui| {
        Alert::new("Saved", &theme)
            .subtitle("Your changes are saved")
            .show(ui);
    });
    harness.run();
    let mut icon = false;
    let mut title = None;
    for shape in &harness.output().shapes {
        if let egui::Shape::Text(text) = &shape.shape {
            let size = text.galley.job.sections[0].format.font_id.size;
            if text.galley.job.text == "Saved" {
                title = Some(size);
            }
        }
        if matches!(&shape.shape, egui::Shape::Circle(circle) if circle.radius > 5.0 && circle.stroke.width > 0.0)
        {
            icon = true;
        }
    }
    assert!(icon, "the alert draws a vector information icon");
    assert_eq!(title.expect("alert title"), title_size);
}
