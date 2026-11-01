use egui_kittest::{Harness, kittest::Queryable};
use temivu::{Header, InputMode, Theme};

#[test]
fn header_action_is_interactive_and_has_minimum_touch_height() {
    let mut harness = Harness::builder()
        .with_size(egui::vec2(260.0, 140.0))
        .build_ui_state(
            |ui, state: &mut (bool, f32)| {
                let output = Header::new(
                    "A long navigation title that must be truncated",
                    &Theme::light(),
                )
                .input_mode(InputMode::Touch)
                .show(ui, |ui| ui.button("Settings").clicked());
                state.0 |= output.inner;
                state.1 = output.response.rect.height();
            },
            (false, 0.0),
        );
    harness.get_by_role_and_label(
        egui::accesskit::Role::Heading,
        "A long navigation title that must be truncated",
    );
    harness.get_by_label("Settings").click();
    harness.run();
    assert!(harness.state().0);
    assert!(harness.state().1 >= 62.0);
}

#[test]
fn header_title_uses_forui_display_xl2_size() {
    for (theme, touch, expected) in [
        (Theme::light(), false, 22.0),
        (Theme::light().touch(), true, 30.0),
    ] {
        let mut harness = Harness::builder().build_ui(move |ui| {
            Header::new("Page title", &theme)
                .input_mode(if touch {
                    temivu::InputMode::Touch
                } else {
                    temivu::InputMode::Desktop
                })
                .show(ui, |_| {});
        });
        harness.run();
        let size = harness
            .output()
            .shapes
            .iter()
            .find_map(|shape| match &shape.shape {
                egui::Shape::Text(text) if text.galley.job.text.contains("Page title") => {
                    Some(text.galley.job.sections[0].format.font_id.size)
                }
                _ => None,
            })
            .expect("header title text");
        assert_eq!(size, expected);
    }
}
