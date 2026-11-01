use egui_kittest::{Harness, kittest::Queryable};
use temivu::badge::show_raw;
use temivu::{Badge, BadgeStyle, BadgeVariant, Theme};

#[test]
fn badge_variants_resolve_from_custom_theme() {
    let mut theme = Theme::light();
    theme.card = egui::Color32::from_rgb(20, 30, 40);
    theme.foreground = egui::Color32::from_rgb(50, 60, 70);
    let outline = BadgeStyle::from_theme(&theme, BadgeVariant::Outline, false);
    assert_eq!(outline.fill, theme.card);
    assert_eq!(outline.foreground, theme.foreground);
    assert_eq!(outline.stroke.color, theme.border);
    assert_eq!(
        BadgeStyle::from_theme(&theme, BadgeVariant::Secondary, false).fill,
        theme.secondary
    );
    let destructive = BadgeStyle::from_theme(&theme, BadgeVariant::Destructive, false);
    assert_eq!(destructive.foreground, theme.destructive);
    assert_eq!(destructive.fill.a(), 26);
    theme.destructive = egui::Color32::from_rgba_unmultiplied(100, 50, 30, 128);
    let custom = BadgeStyle::from_theme(&theme, BadgeVariant::Destructive, false);
    assert_eq!(custom.fill.a(), 26);
}

#[test]
fn badges_expose_text_and_touch_padding_changes_layout() {
    let mut harness = Harness::builder().build_ui_state(
        |ui, widths: &mut [f32; 2]| {
            let theme = Theme::light();
            let desktop = Badge::new("Badge", &theme).show(ui);
            let touch = Badge::new("Badge", &theme)
                .style(BadgeStyle::from_theme(&theme, BadgeVariant::Primary, true))
                .show(ui);
            widths[0] = desktop.rect.width();
            widths[1] = touch.rect.width();
            show_raw(
                ui,
                BadgeStyle::from_theme(&theme, BadgeVariant::Outline, false),
                |ui, _| {
                    ui.label("Custom content");
                },
            );
        },
        [0.0f32; 2],
    );
    harness.run();
    let labels: Vec<_> = harness.get_all_by_label("Badge").collect();
    assert_eq!(labels.len(), 2);
    assert!(labels.iter().all(|node| node.rect().is_positive()));
    assert!(harness.get_by_label("Custom content").rect().is_positive());
    assert!((harness.state()[1] - harness.state()[0] - 4.0).abs() < 0.01);
}
