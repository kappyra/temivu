use egui_kittest::{Harness, kittest::Queryable};
use temivu::Popover;

fn view(ui: &mut egui::Ui, state: &mut (bool, usize)) {
    let anchor = ui.button("Options");
    if anchor.clicked() {
        state.0 = !state.0;
    }
    Popover::new(egui::Id::new("popover"), "Options panel", &mut state.0)
        .focus_on_open(true)
        .show(&anchor, |ui| {
            let action = ui.button("Apply action");
            if action.clicked() {
                state.1 += 1;
                ui.close();
            }
            action
        });
    let _ = ui.button("Other");
}

#[test]
fn keyboard_open_escape_closes_and_restores_anchor_focus() {
    let mut harness = Harness::builder().build_ui_state(view, (false, 0));
    harness.run();
    harness.get_by_label("Options").focus();
    harness.run();
    harness.key_press(egui::Key::Enter);
    harness.run();
    assert!(harness.state().0);
    assert!(harness.get_by_label("Apply action").is_focused());
    harness.get_by_role_and_label(egui::accesskit::Role::Group, "Options panel");
    harness.key_press(egui::Key::Escape);
    harness.run();
    assert!(!harness.state().0);
    assert!(harness.query_by_label("Apply action").is_none());
    assert!(harness.get_by_label("Options").is_focused());
}

#[test]
fn keyboard_action_closes_and_returns_focus() {
    let mut harness = Harness::builder().build_ui_state(view, (false, 0));
    harness.run();
    harness.get_by_label("Options").focus();
    harness.run();
    harness.key_press(egui::Key::Enter);
    harness.run();
    assert!(harness.get_by_label("Apply action").is_focused());
    harness.key_press(egui::Key::Enter);
    harness.run();
    assert_eq!(*harness.state(), (false, 1));
    assert!(harness.get_by_label("Options").is_focused());
}

#[test]
fn outside_click_dismisses_without_refocusing_anchor() {
    let mut harness = Harness::builder().build_ui_state(view, (false, 0));
    harness.run();
    harness.get_by_label("Options").click();
    harness.run();
    assert!(harness.state().0);
    // The new detached surface overlaps the lower edge of this tiny fixture's
    // button. Click its visible top-left corner, which is genuinely outside.
    let outside = harness.get_by_label("Other").rect().left_top() + egui::vec2(2.0, 2.0);
    harness.hover_at(outside);
    harness.step();
    harness.drag_at(outside);
    harness.step();
    harness.drop_at(outside);
    harness.run();
    assert!(!harness.state().0);
    assert!(!harness.get_by_label("Options").is_focused());
}

#[test]
fn escape_after_external_focus_does_not_focus_anchor() {
    let mut harness = Harness::builder().build_ui_state(view, (false, 0));
    harness.run();
    harness.get_by_label("Options").focus();
    harness.run();
    harness.key_press(egui::Key::Enter);
    harness.run();
    harness.get_by_label("Other").focus();
    harness.run();
    harness.key_press(egui::Key::Escape);
    harness.run();
    assert!(!harness.state().0);
    assert!(!harness.get_by_label("Options").is_focused());
}

#[test]
fn stale_action_after_programmatic_close_does_not_activate() {
    use egui_kittest::kittest::NodeT;
    let mut harness = Harness::builder().build_ui_state(view, (false, 0));
    harness.run();
    harness.get_by_label("Options").click();
    harness.run();
    let (target_node, target_tree) = harness
        .get_by_label("Apply action")
        .accesskit_node()
        .locate();
    harness.state_mut().0 = false;
    harness.run();
    harness.event(egui::Event::AccessKitActionRequest(
        egui::accesskit::ActionRequest {
            action: egui::accesskit::Action::Click,
            target_node,
            target_tree,
            data: None,
        },
    ));
    harness.run();
    assert_eq!(*harness.state(), (false, 0));
}

#[test]
fn programmatic_open_does_not_steal_unrelated_focus_by_default() {
    let mut harness = Harness::builder().build_ui_state(
        |ui, open: &mut bool| {
            let anchor = ui.button("Options");
            Popover::new(egui::Id::new("popover"), "Options panel", open)
                .show(&anchor, |ui| ui.button("Apply action"));
            let _ = ui.button("Other");
        },
        false,
    );
    harness.run();
    harness.get_by_label("Other").focus();
    harness.run();
    *harness.state_mut() = true;
    harness.run();
    assert!(harness.get_by_label("Other").is_focused());
    harness.get_by_label("Apply action");
}

#[test]
fn action_can_redirect_focus_without_anchor_overriding_it() {
    let destination = egui::Id::new("destination");
    let mut harness = Harness::builder().build_ui_state(
        move |ui, open: &mut bool| {
            let anchor = ui.button("Options");
            if anchor.clicked() {
                *open = !*open;
            }
            Popover::new(egui::Id::new("popover"), "Options panel", open)
                .focus_on_open(true)
                .show(&anchor, |ui| {
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
        },
        false,
    );
    harness.run();
    harness.get_by_label("Options").focus();
    harness.run();
    harness.key_press(egui::Key::Enter);
    harness.run();
    assert!(harness.get_by_label("Move focus").is_focused());
    harness.key_press(egui::Key::Enter);
    harness.run();
    assert!(harness.get_by_label("Destination").is_focused());
}

#[test]
fn stable_id_keeps_popup_focus_after_prior_content_changes() {
    let mut harness = Harness::builder().build_ui_state(
        |ui, state: &mut (bool, bool)| {
            if state.1 {
                let _ = ui.button("Inserted");
            }
            let anchor = ui.button("Options");
            Popover::new(egui::Id::new("popover"), "Options panel", &mut state.0)
                .show(&anchor, |ui| ui.button("Apply action"));
        },
        (true, false),
    );
    harness.run();
    harness.get_by_label("Apply action").focus();
    harness.run();
    assert!(harness.get_by_label("Apply action").is_focused());
    harness.state_mut().1 = true;
    harness.run();
    assert!(harness.get_by_label("Apply action").is_focused());
}
