use egui_kittest::{
    Harness,
    kittest::{NodeT, Queryable},
};
use temivu::{AccordionItem, AccordionState, Theme};

#[test]
fn constraints_evict_oldest_and_preserve_minimum() {
    assert!(AccordionState::new(2, Some(1)).is_err());
    let mut state = AccordionState::new(1, Some(2)).unwrap();
    assert!(state.toggle(0));
    assert!(!state.toggle(0));
    assert!(state.toggle(1));
    assert!(state.toggle(2));
    assert_eq!(state.expanded(), &[1, 2]);
    assert!(state.toggle(1));
    assert!(!state.toggle(2));
    assert_eq!(state.expanded(), &[2]);
    let mut none = AccordionState::new(0, Some(0)).unwrap();
    assert!(!none.toggle(3));
}

#[test]
fn pointer_and_keyboard_toggle_content_and_expanded_semantics() {
    let mut harness = Harness::builder().build_ui_state(
        |ui, state: &mut AccordionState| {
            AccordionItem::new(state, 0, "Details", &Theme::light()).show(ui, |ui| {
                ui.label("Hidden content");
            });
        },
        AccordionState::new(0, None).unwrap(),
    );
    let node = harness.get_by_label("Details");
    assert_eq!(node.accesskit_node().role(), egui::accesskit::Role::Button);
    assert_eq!(node.accesskit_node().data().is_expanded(), Some(false));
    assert!(harness.query_by_label("Hidden content").is_none());
    node.click();
    harness.run();
    assert!(harness.state().is_expanded(0));
    assert_eq!(
        harness
            .get_by_label("Details")
            .accesskit_node()
            .data()
            .is_expanded(),
        Some(true)
    );
    assert!(harness.get_by_label("Hidden content").rect().is_positive());
    harness.get_by_label("Details").focus();
    harness.run();
    harness.key_press(egui::Key::Enter);
    harness.run();
    assert!(!harness.state().is_expanded(0));
    assert!(harness.query_by_label("Hidden content").is_none());
}

#[test]
fn item_focus_survives_other_expansion_and_reordering() {
    let mut harness = Harness::builder().build_ui_state(
        |ui, state: &mut (AccordionState, bool)| {
            let indices = if state.1 { [1, 0] } else { [0, 1] };
            for index in indices {
                let title = if index == 0 { "First" } else { "Second" };
                AccordionItem::new(&mut state.0, index, title, &Theme::light()).show(ui, |ui| {
                    ui.label("Content");
                });
            }
        },
        (AccordionState::new(0, None).unwrap(), false),
    );
    harness.get_by_label("Second").focus();
    harness.run();
    assert!(harness.state_mut().0.toggle(0));
    harness.run();
    assert!(harness.get_by_label("Second").accesskit_node().is_focused());
    harness.state_mut().1 = true;
    harness.run();
    assert!(harness.get_by_label("Second").accesskit_node().is_focused());
}

#[test]
fn separate_and_cloned_states_do_not_share_item_focus() {
    let state = AccordionState::new(0, None).unwrap();
    let mut harness = Harness::builder().build_ui_state(
        |ui, states: &mut [AccordionState; 2]| {
            for state in states {
                AccordionItem::new(state, 0, "Same title", &Theme::light()).show(ui, |_| {});
            }
        },
        [state.clone(), state],
    );
    harness.run();
    let nodes: Vec<_> = harness.get_all_by_label("Same title").collect();
    assert_eq!(nodes.len(), 2);
    nodes[0].focus();
    harness.run();
    harness.key_press(egui::Key::Enter);
    harness.run();
    assert!(harness.state()[0].is_expanded(0));
    assert!(!harness.state()[1].is_expanded(0));
}

#[test]
fn focused_child_survives_expansion_of_preceding_item() {
    let mut state = AccordionState::new(0, None).unwrap();
    assert!(state.toggle(1));
    let mut harness = Harness::builder().build_ui_state(
        |ui, state: &mut (AccordionState, String)| {
            AccordionItem::new(&mut state.0, 0, "First", &Theme::light()).show(ui, |ui| {
                ui.label("Earlier content");
            });
            AccordionItem::new(&mut state.0, 1, "Second", &Theme::light()).show(ui, |ui| {
                ui.text_edit_singleline(&mut state.1)
                    .accessible_name("Second item text");
            });
        },
        (state, String::new()),
    );
    harness
        .get_by_role(egui::accesskit::Role::TextInput)
        .focus();
    harness.run();
    assert!(harness.state_mut().0.toggle(0));
    harness.run();
    assert!(
        harness
            .get_by_role(egui::accesskit::Role::TextInput)
            .accesskit_node()
            .is_focused()
    );
}
