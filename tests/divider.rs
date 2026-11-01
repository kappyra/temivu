use egui_kittest::{Harness, kittest::Queryable};
use temivu::{Divider, DividerAxis, DividerStyle, Theme};

#[test]
fn invalid_dimensions_are_rejected_without_mutating_style() {
    let theme = Theme::light();
    let mut style = DividerStyle::new(&theme, DividerAxis::Horizontal);
    assert_eq!(style.color, theme.secondary);
    for value in [0.0, -1.0, f32::NAN, f32::INFINITY] {
        assert!(style.set_width(value).is_err());
        assert_eq!(style.width(), 1.0);
    }
    style.set_width(2.0).unwrap();
    let mut divider = Divider::new(&theme, DividerAxis::Horizontal);
    assert!(divider.set_length(f32::NAN).is_err());
    assert!(divider.set_length(-1.0).is_err());
    assert!(divider.set_length(100.0).is_ok());
}

#[test]
fn draws_both_axes_with_correct_margins_and_theme_color() {
    let mut harness = Harness::builder().build_ui_state(
        |ui, rects: &mut [egui::Rect; 2]| {
            let theme = Theme::light();
            ui.label("Before divider");
            let mut horizontal = Divider::new(&theme, DividerAxis::Horizontal);
            horizontal.set_length(100.0).unwrap();
            rects[0] = horizontal.show(ui).rect;
            let mut vertical = Divider::new(&theme, DividerAxis::Vertical)
                .style(DividerStyle::new(&theme, DividerAxis::Vertical));
            vertical.set_length(80.0).unwrap();
            rects[1] = vertical.show(ui).rect;
            ui.label("After divider");
        },
        [egui::Rect::NOTHING; 2],
    );
    harness.run();
    assert!(harness.get_by_label("Before divider").rect().is_positive());
    assert!(harness.get_by_label("After divider").rect().is_positive());
    assert_eq!(harness.state()[0].size(), egui::vec2(100.0, 33.0));
    assert_eq!(harness.state()[1].size(), egui::vec2(33.0, 80.0));
    let color = Theme::light().secondary;
    let rectangles: Vec<_> = harness
        .output()
        .shapes
        .iter()
        .filter_map(|shape| match &shape.shape {
            egui::Shape::Rect(rect) if rect.fill == color => Some(rect.rect.size()),
            _ => None,
        })
        .collect();
    assert!(rectangles.contains(&egui::vec2(100.0, 1.0)));
    assert!(rectangles.contains(&egui::vec2(1.0, 80.0)));
}

#[test]
fn implicit_divider_length_respects_padding_and_available_width() {
    let mut harness = Harness::builder().build_ui_state(
        |ui, widths: &mut [f32; 2]| {
            let theme = Theme::light();
            ui.set_width(100.0);
            let mut style = DividerStyle::new(&theme, DividerAxis::Horizontal);
            style.padding = egui::Margin::symmetric(10, 0);
            widths[0] = ui.available_width();
            widths[1] = Divider::new(&theme, DividerAxis::Horizontal)
                .style(style)
                .show(ui)
                .rect
                .width();
        },
        [0.0; 2],
    );
    harness.run();
    assert!((harness.state()[0] - harness.state()[1]).abs() < 0.01);
}
