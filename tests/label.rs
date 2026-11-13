use egui_kittest::{Harness, kittest::Queryable};
use temivu::{FieldLabel, LabelLayout, Theme};

#[test]
fn label_field_accepts_input_and_error_can_change() {
    let mut harness = Harness::builder().build_ui_state(
        |ui, state: &mut (String, bool)| {
            let theme = Theme::light();
            let mut label = FieldLabel::new(&theme, LabelLayout::Vertical)
                .label("Name")
                .description("Enter a name");
            if state.1 {
                label = label.error("Required");
            }
            label.show(ui, |ui| ui.text_edit_singleline(&mut state.0));
        },
        (String::new(), true),
    );
    assert!(harness.get_by_label("Required").rect().is_positive());
    let input = harness.get_by_label("Name");
    input.click();
    input.type_text("Alice");
    harness.run();
    assert_eq!(harness.state().0, "Alice");
    harness.state_mut().1 = false;
    harness.run();
    assert!(harness.query_by_label("Required").is_none());
}

#[test]
fn horizontal_modes_preserve_order_and_child_only_works() {
    for layout in [
        LabelLayout::HorizontalLeading,
        LabelLayout::HorizontalTrailing,
    ] {
        let mut harness = Harness::builder().build_ui(|ui| {
            FieldLabel::new(&Theme::light(), layout)
                .label("Label")
                .description("Help")
                .error("Error")
                .show(ui, |ui| ui.button("Control"));
        });
        harness.run();
        let label = harness
            .get_by_role_and_label(egui::Role::Label, "Label")
            .rect();
        let button = harness
            .get_by_role_and_label(egui::Role::Button, "Label")
            .rect();
        assert!(harness.get_by_label("Help").rect().is_positive());
        assert!(harness.get_by_label("Error").rect().is_positive());
        if layout == LabelLayout::HorizontalLeading {
            assert!(label.center().x < button.center().x);
        } else {
            assert!(label.center().x > button.center().x);
        }
    }
    let harness = Harness::builder().build_ui(|ui| {
        FieldLabel::new(&Theme::light(), LabelLayout::Vertical).show(ui, |ui| ui.button("Only"));
    });
    assert!(harness.get_by_label("Only").rect().is_positive());
}

#[test]
fn help_and_errors_are_merged_into_control_semantics() {
    use egui_kittest::kittest::NodeT;
    for layout in [
        LabelLayout::Vertical,
        LabelLayout::HorizontalLeading,
        LabelLayout::HorizontalTrailing,
    ] {
        let mut harness = Harness::builder().build_ui_state(
            |ui, state: &mut (String, bool)| {
                let theme = Theme::light();
                let mut label = FieldLabel::new(&theme, layout)
                    .label("Name")
                    .description("Help");
                if state.1 {
                    label = label.error("Required");
                }
                label.show(ui, |ui| {
                    let response = ui.text_edit_singleline(&mut state.0);
                    ui.ctx().accesskit_node_builder(response.id, |node| {
                        node.set_description("Existing")
                    });
                    response
                });
            },
            (String::new(), true),
        );
        harness.run();
        let control = harness.get_by_role_and_label(egui::accesskit::Role::TextInput, "Name");
        let description = control.accesskit_node().description().unwrap();
        for text in ["Existing", "Help", "Required"] {
            assert!(description.contains(text), "{description}");
        }
        assert_eq!(
            control.accesskit_node().data().invalid(),
            Some(egui::accesskit::Invalid::True)
        );
        harness.state_mut().1 = false;
        harness.run();
        let control = harness.get_by_role_and_label(egui::accesskit::Role::TextInput, "Name");
        assert_eq!(control.accesskit_node().data().invalid(), None);
        let description = control.accesskit_node().description().unwrap();
        assert!(description.contains("Existing") && description.contains("Help"));
        assert!(!description.contains("Required"));
    }
}
