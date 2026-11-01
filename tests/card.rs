use egui_kittest::{Harness, kittest::Queryable};
use temivu::{Card, CardStyle, Theme};

#[test]
fn contents_are_accessible_and_interactive() {
    let mut harness = Harness::builder().build_ui_state(
        |ui, clicked| {
            Card::new(&Theme::light()).show(ui, |ui, style| {
                assert_eq!(style.padding, egui::Margin::same(16));
                if ui.button("Card action").clicked() {
                    *clicked = true;
                }
            });
        },
        false,
    );
    harness.get_by_label("Card action").click();
    harness.run();
    assert!(*harness.state());
}

#[test]
fn custom_style_is_painted_and_children_are_not_padded() {
    let mut harness = Harness::builder().build_ui_state(
        |ui, rects: &mut [egui::Rect; 2]| {
            let theme = Theme::light();
            let mut style = CardStyle::from_theme(&theme);
            style.fill = egui::Color32::from_rgb(19, 42, 73);
            let response = Card::new(&theme).style(style).show(ui, |ui, _| {
                ui.allocate_exact_size(egui::vec2(50.0, 40.0), egui::Sense::hover())
                    .0
            });
            rects[0] = response.response.rect;
            rects[1] = response.inner;
        },
        [egui::Rect::NOTHING; 2],
    );
    harness.run();
    assert_eq!(
        harness.state()[0].size(),
        harness.state()[1].size() + egui::vec2(2.0, 2.0)
    );
    assert!(harness.output().shapes.iter().any(|shape| matches!(
        &shape.shape, egui::Shape::Rect(rect) if rect.fill == egui::Color32::from_rgb(19, 42, 73)
    )));
}
