use egui_kittest::{Harness, kittest::Queryable};
use temivu::context_menu;

#[test]
fn keyboard_shortcut_opens_and_escape_dismisses_menu() {
    let mut harness = Harness::builder().build_ui(|ui| {
        let target = ui.button("Target");
        context_menu(&target, |ui| ui.button("Menu action"));
    });
    harness.run();
    harness.get_by_label("Target").focus();
    harness.run();
    harness.key_press_modifiers(egui::Modifiers::SHIFT, egui::Key::F10);
    harness.run();
    assert!(harness.query_by_label("Menu action").is_some());
    assert!(harness.get_by_label("Menu action").is_focused());
    harness.get_by_label("Menu action").focus();
    harness.run();
    harness.key_press(egui::Key::Escape);
    harness.run();
    assert!(harness.query_by_label("Menu action").is_none());
    assert!(harness.get_by_label("Target").is_focused());
}

#[test]
fn secondary_click_opens_menu() {
    let mut harness = Harness::builder().build_ui(|ui| {
        let target = ui.button("Target");
        context_menu(&target, |ui| ui.button("Menu action"));
    });
    harness.run();
    let pos = harness.get_by_label("Target").rect().center();
    harness.event(egui::Event::PointerMoved(pos));
    for pressed in [true, false] {
        harness.event(egui::Event::PointerButton {
            pos,
            button: egui::PointerButton::Secondary,
            pressed,
            modifiers: egui::Modifiers::NONE,
        });
        harness.run();
    }
    assert!(harness.query_by_label("Menu action").is_some());
}

#[test]
fn disabling_target_closes_its_open_menu() {
    let mut harness = Harness::builder().build_ui_state(
        |ui, enabled: &mut bool| {
            let target = ui.add_enabled(*enabled, egui::Button::new("Target"));
            context_menu(&target, |ui| ui.button("Menu action"));
        },
        true,
    );
    harness.run();
    harness.get_by_label("Target").focus();
    harness.run();
    harness.key_press_modifiers(egui::Modifiers::SHIFT, egui::Key::F10);
    harness.run();
    assert!(harness.query_by_label("Menu action").is_some());
    *harness.state_mut() = false;
    harness.run();
    assert!(harness.query_by_label("Menu action").is_none());
}

#[test]
fn keyboard_action_dismisses_and_returns_focus() {
    let mut harness = Harness::builder().build_ui_state(
        |ui, count: &mut u32| {
            let target = ui.button("Target");
            context_menu(&target, |ui| {
                let action = ui.button("Menu action");
                if action.clicked() {
                    *count += 1;
                    ui.close();
                }
                action
            });
        },
        0,
    );
    harness.run();
    harness.get_by_label("Target").focus();
    harness.run();
    harness.key_press_modifiers(egui::Modifiers::SHIFT, egui::Key::F10);
    harness.run();
    assert!(harness.get_by_label("Menu action").is_focused());
    harness.key_press(egui::Key::Enter);
    harness.run();
    assert_eq!(*harness.state(), 1);
    assert!(harness.query_by_label("Menu action").is_none());
    assert!(harness.get_by_label("Target").is_focused());
}

#[test]
fn second_keyboard_action_returns_focus_to_anchor() {
    let mut harness = Harness::builder().build_ui_state(
        |ui, count: &mut u32| {
            let anchor = ui.button("Target");
            context_menu(&anchor, |ui| {
                let first = ui.button("First");
                if ui.button("Second").clicked() {
                    *count += 1;
                    ui.close();
                }
                first
            });
        },
        0,
    );
    harness.run();
    harness.get_by_label("Target").focus();
    harness.run();
    harness.key_press_modifiers(egui::Modifiers::SHIFT, egui::Key::F10);
    harness.run();
    assert!(harness.get_by_label("First").is_focused());
    harness.key_press(egui::Key::Tab);
    harness.run();
    assert!(harness.get_by_label("Second").is_focused());
    harness.key_press(egui::Key::Enter);
    harness.run();
    assert_eq!(*harness.state(), 1);
    assert!(harness.query_by_label("Second").is_none());
    assert!(harness.get_by_label("Target").is_focused());
}

#[test]
fn escape_does_not_steal_focus_from_another_control() {
    let mut harness = Harness::builder().build_ui(|ui| {
        let anchor = ui.button("Target");
        context_menu(&anchor, |ui| ui.button("Action"));
        let _ = ui.button("Other");
    });
    harness.run();
    harness.get_by_label("Target").focus();
    harness.run();
    harness.key_press_modifiers(egui::Modifiers::SHIFT, egui::Key::F10);
    harness.run();
    harness.get_by_label("Other").focus();
    harness.run();
    assert!(harness.get_by_label("Other").is_focused());
    harness.key_press(egui::Key::Escape);
    harness.run();
    assert!(!harness.get_by_label("Target").is_focused());
}

#[test]
fn outside_pointer_dismissal_keeps_new_focus() {
    let mut harness = Harness::builder().build_ui(|ui| {
        let anchor = ui.button("Target");
        context_menu(&anchor, |ui| ui.button("Action"));
        let _ = ui.button("Other");
    });
    harness.run();
    harness.get_by_label("Target").focus();
    harness.run();
    harness.key_press_modifiers(egui::Modifiers::SHIFT, egui::Key::F10);
    harness.run();
    harness.get_by_label("Other").click();
    harness.run();
    assert!(!harness.get_by_label("Target").is_focused());
}

#[test]
fn egui_escape_also_surrenders_focus_without_a_menu() {
    let mut harness = Harness::builder().build_ui(|ui| {
        let _ = ui.button("Other");
    });
    harness.run();
    harness.get_by_label("Other").focus();
    harness.run();
    assert!(harness.get_by_label("Other").is_focused());
    harness.key_press(egui::Key::Escape);
    harness.run();
    assert!(!harness.get_by_label("Other").is_focused());
}

#[test]
fn action_can_redirect_focus_without_anchor_overriding_it() {
    let destination = egui::Id::new("destination");
    let mut harness = Harness::builder().build_ui(move |ui| {
        let anchor = ui.button("Target");
        context_menu(&anchor, |ui| {
            let action = ui.button("Move focus");
            if action.clicked() {
                ui.memory_mut(|memory| memory.request_focus(destination));
                ui.close();
            }
            action
        });
        let (rect, _) = ui.allocate_exact_size(egui::vec2(90.0, 32.0), egui::Sense::hover());
        let other = ui.interact(rect, destination, egui::Sense::click());
        other.widget_info(|| {
            egui::WidgetInfo::labeled(egui::WidgetType::Button, true, "Destination")
        });
    });
    harness.run();
    harness.get_by_label("Target").focus();
    harness.run();
    harness.key_press_modifiers(egui::Modifiers::SHIFT, egui::Key::F10);
    harness.run();
    assert!(harness.get_by_label("Move focus").is_focused());
    harness.key_press(egui::Key::Enter);
    harness.run();
    assert!(harness.get_by_label("Destination").is_focused());
}
