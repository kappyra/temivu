use egui_kittest::{
    Harness,
    kittest::{NodeT, Queryable},
};
use temivu::{Theme, ToastEntry, Toaster, ToasterState};

fn view(ui: &mut egui::Ui, state: &mut ToasterState) {
    let theme = Theme::light();
    Toaster::new(state, &theme, "Dismiss").show(ui);
}

#[test]
fn queued_toasts_render_and_can_be_dismissed() {
    let mut state = ToasterState::new(3);
    state.push(ToastEntry::new(egui::Id::new("first"), "First toast"));
    state.push(ToastEntry::new(egui::Id::new("second"), "Second toast").description("Details"));
    let mut harness = Harness::builder().build_ui_state(view, state);
    harness.run();
    assert_eq!(
        harness
            .query_all_by_role(egui::accesskit::Role::Alert)
            .count(),
        2,
        "both queued toasts should be rendered"
    );
    harness.get_by_label("Dismiss: First toast").click();
    harness.run();
    assert_eq!(harness.state().len(), 1);
    assert_eq!(
        harness
            .query_all_by_role(egui::accesskit::Role::Alert)
            .count(),
        1
    );
}

#[test]
fn the_limit_queues_remaining_toasts() {
    let mut state = ToasterState::new(2);
    for index in 0..3 {
        state.push(ToastEntry::new(
            egui::Id::new(format!("queued-{index}")),
            format!("Toast {index}"),
        ));
    }
    let mut harness = Harness::builder().build_ui_state(view, state);
    harness.run();
    assert_eq!(
        harness
            .query_all_by_role(egui::accesskit::Role::Alert)
            .count(),
        2
    );
    harness.get_by_label("Dismiss: Toast 0").click();
    harness.run();
    assert_eq!(
        harness
            .query_all_by_role(egui::accesskit::Role::Alert)
            .count(),
        2,
        "the queued toast should take the dismissed slot"
    );
    assert_eq!(harness.state().len(), 2);
}

#[test]
fn zero_timeout_toasts_are_pruned_immediately() {
    let mut state = ToasterState::new(3);
    state.push(
        ToastEntry::new(egui::Id::new("instant"), "Instant")
            .timeout(Some(std::time::Duration::ZERO)),
    );
    let mut harness = Harness::builder().build_ui_state(view, state);
    harness.run();
    assert_eq!(harness.state().len(), 0);
    assert_eq!(
        harness
            .query_all_by_role(egui::accesskit::Role::Alert)
            .count(),
        0
    );
}

#[test]
fn persistent_toasts_stay_until_dismissed() {
    let mut state = ToasterState::new(3);
    state.push(ToastEntry::new(egui::Id::new("sticky"), "Sticky").timeout(None));
    let mut harness = Harness::builder().build_ui_state(view, state);
    harness.run_steps(5);
    assert_eq!(
        harness
            .query_all_by_role(egui::accesskit::Role::Alert)
            .count(),
        1,
        "a persistent toast should not expire"
    );
}

#[test]
fn pushing_the_same_id_replaces_the_entry() {
    let mut state = ToasterState::new(3);
    state.push(ToastEntry::new(egui::Id::new("same"), "First title"));
    state.push(ToastEntry::new(egui::Id::new("same"), "Second title"));
    let mut harness = Harness::builder().build_ui_state(view, state);
    harness.run();
    assert_eq!(harness.state().len(), 1);
    harness.get_by_role_and_label(egui::accesskit::Role::Alert, "Second title");
}

#[test]
fn limit_is_clamped_to_supported_bounds() {
    assert_eq!(ToasterState::new(0).limit(), 1);
    assert_eq!(ToasterState::new(100).limit(), 8);
    assert_eq!(ToasterState::default().limit(), 3);
}

#[test]
fn timed_toasts_expire_through_the_simulated_clock() {
    let mut state = ToasterState::new(3);
    state.push(
        ToastEntry::new(egui::Id::new("timed"), "Timed")
            .timeout(Some(std::time::Duration::from_secs(5))),
    );
    let mut harness = Harness::builder().build_ui_state(view, state);
    harness.run();
    assert_eq!(
        harness.state().len(),
        1,
        "the timed toast should start visible"
    );
    let now = harness.ctx.input(|input| input.time);
    harness.input_mut().time = Some(now + 6.0);
    harness.step();
    assert_eq!(harness.state().len(), 0, "the timed toast should expire");
}

#[test]
fn stacked_toasts_have_unique_dismiss_accessible_names() {
    let mut state = ToasterState::new(3);
    state.push(ToastEntry::new(egui::Id::new("first"), "First"));
    state.push(ToastEntry::new(egui::Id::new("second"), "Second"));
    let mut harness = Harness::builder().build_ui_state(view, state);
    harness.run();
    harness.get_by_label("Dismiss: First");
    harness.get_by_label("Dismiss: Second");
    assert!(
        harness.query_all_by_label("Dismiss").count() == 0,
        "toaster dismiss buttons should not share the plain dismiss label"
    );
}

#[test]
fn dismissing_with_the_keyboard_removes_the_focused_toast() {
    let mut state = ToasterState::new(3);
    state.push(ToastEntry::new(egui::Id::new("keyboard"), "Keyboard toast"));
    let mut harness = Harness::builder().build_ui_state(view, state);
    harness.run();
    harness.get_by_label("Dismiss: Keyboard toast").focus();
    harness.run();
    harness.key_press(egui::Key::Enter);
    harness.run();
    assert_eq!(harness.state().len(), 0);
    assert!(
        harness
            .query_by_role(egui::accesskit::Role::Alert)
            .is_none()
    );
}

#[test]
fn custom_anchors_keep_the_overlay_on_screen() {
    use egui::Align2;

    for (anchor, expect_right, expect_bottom) in [
        (Align2::RIGHT_TOP, true, false),
        (Align2::LEFT_BOTTOM, false, true),
        (Align2::CENTER_BOTTOM, false, true),
    ] {
        let mut state = ToasterState::new(3);
        state.push(ToastEntry::new(egui::Id::new("anchored"), "Anchored").timeout(None));
        let mut harness = Harness::builder()
            .with_size(egui::vec2(480.0, 360.0))
            .build_ui_state(
                move |ui, state: &mut ToasterState| {
                    let theme = Theme::light();
                    Toaster::new(state, &theme, "Dismiss")
                        .anchor(anchor)
                        .show(ui);
                },
                state,
            );
        harness.run();
        let rect = harness
            .get_by_role_and_label(egui::accesskit::Role::Alert, "Anchored")
            .rect();
        assert!(rect.min.x >= 0.0 && rect.min.y >= 0.0);
        assert!(rect.max.x <= 480.0 && rect.max.y <= 360.0);
        if expect_right {
            assert!(rect.max.x > 240.0);
        } else {
            assert!(rect.min.x < 240.0);
        }
        assert_eq!(rect.max.y > 180.0, expect_bottom);
    }
}

#[test]
fn toast_entries_can_render_a_decorative_icon() {
    let mut state = ToasterState::new(3);
    state.push(
        ToastEntry::new(egui::Id::new("with-icon"), "Saved")
            .icon("!")
            .timeout(None),
    );
    let mut harness = Harness::builder().build_ui_state(view, state);
    harness.run();
    harness.get_by_role_and_label(egui::accesskit::Role::Alert, "Saved");
    assert!(
        harness.get_by_label("!").accesskit_node().is_hidden(),
        "the approximate icon glyph should be hidden from assistive technology"
    );
}
