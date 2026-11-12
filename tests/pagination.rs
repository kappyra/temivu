use egui_kittest::{
    Harness,
    kittest::{NodeT, Queryable},
};
use temivu::{PageItem, PaginationState, Theme, pagination};

#[test]
fn compact_controls_are_centered_and_hold_their_positions_as_digits_change() {
    let mut harness = Harness::builder()
        .with_size(egui::vec2(390.0, 240.0))
        .build_ui_state(
            |ui, state: &mut PaginationState| {
                temivu::pagination_compact(
                    ui,
                    state,
                    &Theme::light().touch(),
                    "Previous",
                    "Next",
                    |page, total| format!("Page {page} of {total}"),
                );
            },
            PaginationState::new(12, 8, 0, false).unwrap(),
        );
    harness.run();
    let previous = harness.get_by_label("Previous").rect();
    let next = harness.get_by_label("Next").rect();
    let caption = harness.get_by_label("Page 9 of 12").rect();
    assert_eq!(previous.size(), next.size());
    assert_eq!(previous.center().y, next.center().y);
    assert!((previous.center().lerp(next.center(), 0.5).x - caption.center().x).abs() <= 1.0);
    assert!(previous.height() >= 44.0);
    harness.get_by_label("Next").click();
    harness.run();
    harness.get_by_label("Page 10 of 12");
    assert_eq!(harness.get_by_label("Previous").rect(), previous);
    assert_eq!(harness.get_by_label("Next").rect(), next);
}

#[test]
fn controller_enforces_limits_and_clamps_when_page_count_changes() {
    assert!(PaginationState::new(0, 0, 1, true).is_err());
    assert!(PaginationState::new(3, 3, 1, true).is_err());
    assert!(PaginationState::new(usize::MAX, 0, usize::MAX, true).is_err());
    let mut state = PaginationState::new(10, 9, 1, true).unwrap();
    state.next();
    assert_eq!(state.page(), 9);
    state.set_pages(3).unwrap();
    assert_eq!(state.page(), 2);
    assert!(state.set_page(3).is_err());
    assert!(state.set_pages(0).is_err());
    assert_eq!(state.pages(), 3);
    state.previous();
    assert_eq!(state.page(), 1);
}

#[test]
fn page_window_includes_edges_and_ellipsis() {
    let state = PaginationState::new(20, 10, 1, true).unwrap();
    assert_eq!(
        state.items(),
        vec![
            PageItem::Page(0),
            PageItem::Ellipsis,
            PageItem::Page(9),
            PageItem::Page(10),
            PageItem::Page(11),
            PageItem::Ellipsis,
            PageItem::Page(19)
        ]
    );
    let small = PaginationState::new(3, 1, 1, true).unwrap();
    assert_eq!(
        small.items(),
        vec![PageItem::Page(0), PageItem::Page(1), PageItem::Page(2)]
    );
}

#[test]
fn page_click_and_previous_next_change_selection() {
    let mut harness = Harness::builder().build_ui_state(
        |ui, state: &mut PaginationState| {
            pagination(ui, state, &Theme::light(), "Previous", "Next");
        },
        PaginationState::new(5, 0, 1, true).unwrap(),
    );
    harness.run();
    assert!(
        harness
            .get_by_label("Previous")
            .accesskit_node()
            .is_disabled()
    );
    harness.get_by_label("3").click();
    harness.run();
    assert_eq!(harness.state().page(), 2);
    assert_eq!(
        harness.get_by_label("3").accesskit_node().is_selected(),
        Some(true)
    );
    harness.get_by_label("Next").click();
    harness.run();
    assert_eq!(harness.state().page(), 3);
}

#[test]
fn disabled_pagination_ignores_stale_accessibility_clicks() {
    let mut harness = Harness::builder().build_ui_state(
        |ui, state: &mut PaginationState| {
            ui.disable();
            pagination(ui, state, &Theme::light(), "Previous", "Next");
        },
        PaginationState::new(3, 0, 1, true).unwrap(),
    );
    harness.run();
    let node = harness.get_by_label("2");
    let (target_node, target_tree) = node.accesskit_node().locate();
    harness.event(egui::Event::AccessKitActionRequest(
        egui::accesskit::ActionRequest {
            action: egui::accesskit::Action::Click,
            target_node,
            target_tree,
            data: None,
        },
    ));
    harness.run();
    assert_eq!(harness.state().page(), 0);
}

#[test]
fn page_windows_cover_edges_and_stay_bounded_for_large_counts() {
    use PageItem::{Ellipsis as Gap, Page};
    let state = |page| PaginationState::new(10, page, 1, true).unwrap().items();
    assert_eq!(
        state(0),
        vec![Page(0), Page(1), Page(2), Page(3), Page(4), Gap, Page(9)]
    );
    assert_eq!(
        state(4),
        vec![Page(0), Gap, Page(3), Page(4), Page(5), Gap, Page(9)]
    );
    assert_eq!(
        state(6),
        vec![Page(0), Gap, Page(5), Page(6), Page(7), Page(8), Page(9)]
    );
    let huge = PaginationState::new(usize::MAX, usize::MAX / 2, 1000, true).unwrap();
    assert!(huge.items().len() <= 4005);
}

#[test]
fn selected_page_has_contrasting_theme_colors() {
    for theme in [Theme::light(), Theme::dark()] {
        let expected_text = theme.primary_foreground;
        let expected_fill = theme.primary;
        let mut harness = Harness::builder().build_ui(move |ui| {
            theme.apply(ui.ctx());
            let mut state = PaginationState::new(3, 1, 1, true).unwrap();
            pagination(ui, &mut state, &theme, "Previous", "Next");
        });
        harness.run();
        let label = harness
            .output()
            .shapes
            .iter()
            .find_map(|shape| match &shape.shape {
                egui::Shape::Text(text) if text.galley.job.text == "2" => {
                    let color = text
                        .override_text_color
                        .unwrap_or(text.galley.job.sections[0].format.color);
                    Some(if color == egui::Color32::PLACEHOLDER {
                        text.fallback_color
                    } else {
                        color
                    })
                }
                _ => None,
            });
        assert_eq!(label, Some(expected_text));
        assert!(harness.output().shapes.iter().any(
            |shape| matches!(&shape.shape, egui::Shape::Rect(rect) if rect.fill == expected_fill)
        ));
    }
}

#[test]
fn page_focus_survives_window_shift_and_instances_do_not_share_ids() {
    let mut harness = Harness::builder()
        .with_size(egui::vec2(1000.0, 500.0))
        .build_ui_state(
            |ui, states: &mut [PaginationState; 2]| {
                temivu::pagination_with_id(
                    ui,
                    &mut states[0],
                    &Theme::light(),
                    "Previous A",
                    "Next A",
                    egui::Id::new("pagination-a"),
                );
                temivu::pagination_with_id(
                    ui,
                    &mut states[1],
                    &Theme::light(),
                    "Previous B",
                    "Next B",
                    egui::Id::new("pagination-b"),
                );
            },
            [
                PaginationState::new(20, 0, 1, true).unwrap(),
                PaginationState::new(20, 0, 1, true).unwrap(),
            ],
        );
    harness.run();
    let nodes: Vec<_> = harness.get_all_by_label("5").collect();
    assert_eq!(nodes.len(), 2);
    nodes[0].focus();
    harness.run();
    harness.key_press(egui::Key::Enter);
    harness.run();
    assert_eq!(harness.state()[0].page(), 4);
    assert_eq!(harness.state()[1].page(), 0);
    assert!(
        harness
            .get_all_by_label("5")
            .next()
            .unwrap()
            .accesskit_node()
            .is_focused()
    );
    harness.key_press(egui::Key::Enter);
    harness.run();
    assert_eq!(harness.state()[0].page(), 4);
    assert_eq!(harness.state()[1].page(), 0);
}

#[test]
fn desktop_targets_are_large_enough_and_ellipsis_is_decorative() {
    let mut harness = Harness::builder()
        .with_size(egui::vec2(1000.0, 300.0))
        .build_ui(|ui| {
            let mut state = PaginationState::new(20, 10, 1, true).unwrap();
            pagination(ui, &mut state, &Theme::light(), "Previous", "Next");
        });
    harness.run();
    for label in ["Previous", "10", "11", "Next"] {
        let rect = harness.get_by_label(label).rect();
        assert!(
            rect.width() >= 32.0 && rect.height() >= 32.0,
            "{label}: {rect:?}"
        );
    }
    assert!(harness.query_by_label("…").is_none());
}

#[test]
fn default_pagination_keeps_focused_page_as_the_window_moves() {
    let mut harness = Harness::builder()
        .with_size(egui::vec2(800.0, 300.0))
        .build_ui_state(
            |ui, state: &mut PaginationState| {
                pagination(ui, state, &Theme::light(), "Previous", "Next")
            },
            PaginationState::new(20, 0, 1, true).unwrap(),
        );
    harness.get_by_label("5").focus();
    harness.run();
    harness.key_press(egui::Key::Enter);
    harness.run();
    assert_eq!(harness.state().page(), 4);
    assert!(harness.get_by_label("5").accesskit_node().is_focused());
}

#[test]
fn narrow_pagination_wraps_next_button_without_clipping() {
    let mut harness = Harness::builder()
        .with_size(egui::vec2(600.0, 480.0))
        .build_ui_state(
            |ui, state: &mut PaginationState| {
                pagination(ui, state, &Theme::light(), "Previous", "Next")
            },
            PaginationState::new(12, 4, 1, true).unwrap(),
        );
    harness.ctx.set_zoom_factor(1.5);
    harness.run();
    let next = harness.get_by_label("Next");
    assert!(next.rect().right() <= 400.0);
    assert!(harness.output().shapes.iter().any(|shape| matches!(
        &shape.shape,
        egui::Shape::Text(text) if text.galley.job.text == "Next" && text.galley.rows.len() == 1
    )));
    next.click();
    harness.run();
    assert_eq!(harness.state().page(), 5);
}

#[test]
fn long_translated_labels_and_page_controls_fit_a_narrow_viewport() {
    let previous = "Long translated previous label";
    let next = "Long translated next label";
    let mut harness = Harness::builder()
        .with_size(egui::vec2(200.0, 480.0))
        .build_ui_state(
            move |ui, state: &mut PaginationState| {
                pagination(ui, state, &Theme::light(), previous, next);
            },
            PaginationState::new(10, 3, 1, true).unwrap(),
        );
    harness.run();
    for label in [previous, "1", "3", "4", "5", "10", next] {
        let rect = harness.get_by_label(label).rect();
        assert!(
            rect.left() >= 0.0 && rect.right() <= 200.0,
            "{label}: {rect:?}"
        );
    }
    for shape in &harness.output().shapes {
        if let egui::Shape::Text(text) = &shape.shape {
            assert!(
                text.pos.x + text.galley.size().x <= 200.5,
                "text exceeds viewport: {:?}",
                text.galley.job.text
            );
        }
    }
    harness.get_by_label(next).click();
    harness.run();
    assert_eq!(harness.state().page(), 4);
}

#[test]
fn siblings_zero_with_edges_matches_upstream_page_window() {
    // Upstream's siblingRange/build produces [first, current, last] without an
    // ellipsis when siblings is zero and edges are shown; the port is an exact
    // translation, so this documents the shared quirk instead of "fixing" it.
    let state = PaginationState::new(5, 2, 0, true).expect("valid state");
    assert_eq!(
        state.items(),
        vec![PageItem::Page(0), PageItem::Page(1), PageItem::Page(2),]
    );
}

#[test]
fn touch_pagination_uses_forui_44_point_targets() {
    use temivu::{Theme, pagination};
    let theme = Theme::light().touch();
    let mut harness = Harness::builder().build_ui_state(
        |ui, state: &mut PaginationState| {
            pagination(ui, state, &theme, "Previous", "Next");
        },
        PaginationState::new(3, 0, 1, true).unwrap(),
    );
    harness.run();
    let page = harness.get_by_label("2").rect();
    assert!(page.width() >= 44.0 && page.height() >= 44.0, "{page:?}");
}
