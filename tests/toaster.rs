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
    state.push(ToastEntry::new(egui::Id::unique("first"), "First toast"));
    state.push(ToastEntry::new(egui::Id::unique("second"), "Second toast").description("Details"));
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
            egui::Id::unique(format!("queued-{index}")),
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
        ToastEntry::new(egui::Id::unique("instant"), "Instant")
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
    state.push(ToastEntry::new(egui::Id::unique("sticky"), "Sticky").timeout(None));
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
    state.push(ToastEntry::new(egui::Id::unique("same"), "First title"));
    state.push(ToastEntry::new(egui::Id::unique("same"), "Second title"));
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
        ToastEntry::new(egui::Id::unique("timed"), "Timed")
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
    state.push(ToastEntry::new(egui::Id::unique("first"), "First"));
    state.push(ToastEntry::new(egui::Id::unique("second"), "Second"));
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
    state.push(ToastEntry::new(
        egui::Id::unique("keyboard"),
        "Keyboard toast",
    ));
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
        state.push(ToastEntry::new(egui::Id::unique("anchored"), "Anchored").timeout(None));
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
        ToastEntry::new(egui::Id::unique("with-icon"), "Saved")
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

#[test]
fn dismissed_card_keeps_paint_until_the_queue_finishes_its_exit() {
    let id = egui::Id::unique("exit");
    let mut state = ToasterState::default();
    state.push(ToastEntry::new(id, "Saved").timeout(None));
    let mut harness = Harness::builder().build_ui_state(view, state);
    harness.run();
    harness
        .ctx
        .all_styles_mut(|style| style.animation_time = 0.2);
    harness.input_mut().time = Some(0.0);
    harness.step();

    harness.state_mut().dismiss(id);
    assert!(harness.state().is_empty());
    assert!(harness.state().needs_render());
    harness.input_mut().time = Some(0.02);
    harness.step();
    assert!(
        harness
            .query_by_role(egui::accesskit::Role::Alert)
            .is_none()
    );
    assert!(harness.state().needs_render());

    harness.input_mut().time = Some(1.0);
    harness.step();
    assert!(!harness.state().needs_render());
}

#[test]
fn notification_center_includes_pending_entries_and_clears_only_multiple() {
    let mut state = ToasterState::new(1);
    for index in 0..3 {
        state.push(
            ToastEntry::new(egui::Id::unique(index), format!("Update {index}")).timeout(None),
        );
    }
    let mut harness = Harness::builder()
        .with_size(egui::vec2(390.0, 480.0))
        .build_ui_state(
            |ui, (state, open): &mut (ToasterState, bool)| {
                let theme = Theme::light();
                ui.add_space(220.0);
                let trigger = ui.button("View all notifications");
                *open |= trigger.clicked();
                state.show_center(&trigger, open, &theme, Default::default());
                if !*open {
                    Toaster::new(state, &theme, "Dismiss").show(ui);
                }
            },
            (state, false),
        );

    harness.run();
    assert_eq!(
        harness
            .query_all_by_role(egui::accesskit::Role::Alert)
            .count(),
        1,
    );
    harness.get_by_label("View all notifications").click();
    harness.run();
    let center = harness.get_by_role_and_label(egui::accesskit::Role::Dialog, "Notifications");
    let panel = center.rect();
    assert!(panel.left() >= 0.0 && panel.right() <= 390.0);
    assert!(
        panel.height() < 360.0,
        "three short notices fit in a compact panel: {panel:?}"
    );
    let close = harness.get_by_label("Close notifications").rect();
    assert!((close.width() - close.height()).abs() <= 1.0);
    let title = harness
        .output()
        .shapes
        .iter()
        .find_map(|entry| match &entry.shape {
            egui::Shape::Text(text) if text.galley.job.text == "Notifications" => {
                Some(text.galley.rect.translate(text.pos.to_vec2()))
            }
            _ => None,
        })
        .expect("center title");
    assert!(
        (title.center().x - panel.center().x).abs() <= 1.0,
        "{title:?} vs {panel:?}"
    );
    assert!(
        (title.center().y - close.center().y).abs() <= 1.0,
        "{title:?} vs {close:?}"
    );
    let cross = harness
        .output()
        .shapes
        .iter()
        .filter(|entry| match &entry.shape {
            egui::Shape::LineSegment { points, .. } => {
                points.iter().all(|point| close.contains(*point))
                    && points[0].lerp(points[1], 0.5).distance(close.center()) <= 0.5
            }
            _ => false,
        })
        .count();
    assert_eq!(
        cross, 2,
        "both strokes should cross at the center of the button"
    );
    let clear = harness.get_by_label("Clear all").rect();
    assert!((clear.center().x - panel.center().x).abs() <= 1.0);
    let first = harness.get_by_label("Dismiss: Update 0").rect();
    let last = harness.get_by_label("Dismiss: Update 2").rect();
    assert!(first.height() >= 44.0 && last.bottom() < panel.bottom());
    assert!(
        last.top() - first.top() < 160.0,
        "rows should hug their content"
    );
    harness.get_by_label("Clear all");

    for index in 0..2 {
        harness
            .get_by_label(format!("Dismiss: Update {index}").as_str())
            .click();
        harness.run();
    }
    assert_eq!(harness.state().0.len(), 1);
    assert!(harness.query_by_label("Clear all").is_none());
    let single = harness
        .get_by_role_and_label(egui::accesskit::Role::Dialog, "Notifications")
        .rect();
    assert!(
        single.height() < 165.0,
        "one short notice should fit in a short panel: {single:?}"
    );

    harness
        .state_mut()
        .0
        .push(ToastEntry::new(egui::Id::unique(3), "New update"));
    harness.run_steps(3);
    harness.get_by_label("Clear all").click();
    harness.run();
    assert!(harness.state().0.is_empty());
    harness.get_by_label("No notifications");
    harness.key_press(egui::Key::Escape);
    harness.run();
    assert!(!harness.state().1);
}

#[test]
fn stacked_preview_actions_are_under_the_cards_and_only_for_multiple_notices() {
    let mut state = ToasterState::default();
    state.push(ToastEntry::new(egui::Id::unique(0), "First notice").timeout(None));
    let mut harness = Harness::builder()
        .with_size(egui::vec2(390.0, 844.0))
        .build_ui_state(
            |ui, (state, open): &mut (ToasterState, bool)| {
                let theme = Theme::light();
                Toaster::new(state, &theme, "Dismiss").show_with_center(
                    ui,
                    open,
                    Default::default(),
                );
            },
            (state, false),
        );
    harness.run();
    assert!(harness.query_by_label("Clear all").is_none());
    assert!(harness.query_by_label("View all notifications").is_none());

    harness
        .state_mut()
        .0
        .push(ToastEntry::new(egui::Id::unique(1), "Second notice").timeout(None));
    harness.run();
    let first = harness
        .get_by_role_and_label(egui::accesskit::Role::Alert, "First notice")
        .rect();
    let second = harness
        .get_by_role_and_label(egui::accesskit::Role::Alert, "Second notice")
        .rect();
    let view = harness.get_by_label("View all notifications");
    assert!((0.0..=12.0).contains(&(second.top() - first.bottom())));
    assert!((first.center().x - second.center().x).abs() <= 1.0);
    assert!(view.rect().top() > second.bottom());
    assert!(harness.query_by_label("+1").is_none());

    for index in 2..5 {
        harness.state_mut().0.push(
            ToastEntry::new(egui::Id::unique(index), format!("Notice {index}")).timeout(None),
        );
    }
    harness.run();
    harness.get_by_label("+2");
    assert!(harness.query_by_label("Dismiss: Notice 4").is_none());
    harness.get_by_label("Dismiss: First notice").click();
    harness.run();
    harness.get_by_label("+1");
    harness.get_by_label("View all notifications").click();
    harness.run();
    harness.get_by_role_and_label(egui::accesskit::Role::Dialog, "Notifications");
    harness.get_by_label("Dismiss: Notice 4");
    harness.get_by_label("Close notifications").click();
    harness.run();
    harness.get_by_label("Clear all").click();
    harness.run();
    assert!(harness.state().0.is_empty());
    assert!(harness.query_by_label("Clear all").is_none());
}
