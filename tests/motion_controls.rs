use egui_kittest::{
    Harness,
    kittest::{NodeT, Queryable},
};
use temivu::{
    Direction, Theme, ViewMode, direction_pad, layout_toggle, quantity_stepper, search_box,
    toggle_button,
};

#[test]
fn stepper_reaches_both_bounds_and_folds_at_zero() {
    let mut harness = Harness::builder().build_ui_state(
        |ui, count| {
            quantity_stepper(ui, &Theme::light(), count, 0..=1, "Less", "More");
        },
        0,
    );
    harness.run_steps(5);
    assert!(harness.query_by_label("Less").is_none());
    harness.get_by_label("More").click();
    harness.run_steps(5);
    assert_eq!(*harness.state(), 1);
    assert!(harness.get_by_label("More").accesskit_node().is_disabled());
    harness.get_by_label("Less").click();
    harness.run_steps(5);
    assert_eq!(*harness.state(), 0);
    assert!(harness.query_by_label("Less").is_none());
}

#[test]
fn search_expands_focuses_and_closes_without_losing_query() {
    let mut harness = Harness::builder()
        .with_size(egui::vec2(240.0, 160.0))
        .build_ui_state(
            |ui, state: &mut (bool, String)| {
                search_box(
                    ui,
                    &Theme::light(),
                    &mut state.0,
                    &mut state.1,
                    "Find",
                    "Close",
                );
            },
            (false, String::new()),
        );
    harness.run_steps(3);
    let trigger = harness.get_by_role_and_label(egui::accesskit::Role::Button, "Find");
    trigger.focus();
    harness.run();
    harness.key_press(egui::Key::Enter);
    harness.run_steps(5);
    let field = harness.get_by_role_and_label(egui::accesskit::Role::TextInput, "Find");
    assert!(field.accesskit_node().is_focused());
    assert!(field.rect().right() < 240.0);
    harness.event(egui::Event::Text("calendar".to_owned()));
    harness.run();
    harness.key_press(egui::Key::Escape);
    harness.run_steps(5);
    assert_eq!(harness.state(), &(false, "calendar".to_owned()));
    assert!(
        harness
            .get_by_role_and_label(egui::accesskit::Role::Button, "Find")
            .accesskit_node()
            .is_focused()
    );
}

#[test]
fn state_controls_report_changes_and_direction_pad_accepts_arrow_keys() {
    let mut harness = Harness::builder().build_ui_state(
        |ui, state: &mut (ViewMode, bool, Option<Direction>)| {
            let theme = Theme::light();
            layout_toggle(ui, &theme, &mut state.0, ["Columns", "Grid", "List"]);
            toggle_button(ui, &theme, &mut state.1, ["Follow", "Following"]);
            if let Some(direction) = direction_pad(ui, &theme, ["Up", "Left", "Right", "Down"]) {
                state.2 = Some(direction);
            }
        },
        (ViewMode::Column, false, None),
    );
    harness.run();
    harness.get_by_label("Columns").click();
    harness.run();
    harness.get_by_label("Follow").click();
    harness.run();
    assert_eq!(
        harness.get_by_label("Following").accesskit_node().toggled(),
        Some(egui::accesskit::Toggled::True)
    );
    harness.get_by_label("Up").focus();
    harness.run();
    harness.key_press(egui::Key::ArrowLeft);
    harness.run();
    assert_eq!(
        harness.state(),
        &(ViewMode::Grid, true, Some(Direction::Left))
    );
    assert!(harness.get_by_label("Left").accesskit_node().is_focused());
}

#[test]
fn quantity_joins_and_folds_using_the_droplet_contour() {
    let mut harness = Harness::builder().build_ui_state(
        |ui, count| {
            quantity_stepper(ui, &Theme::light(), count, 0..=9, "Less", "More");
        },
        0,
    );
    harness
        .ctx
        .all_styles_mut(|style| style.animation_time = 0.2);
    let mut step = |count, time| {
        *harness.state_mut() = count;
        harness.input_mut().time = Some(time);
        harness.step();
        let mesh = harness.output().shapes.iter().any(|entry| {
            matches!(&entry.shape, egui::Shape::Mesh(mesh) if mesh.is_valid() && !mesh.indices.is_empty())
        });
        (
            mesh,
            harness.get_by_label("More").rect().center().x,
            harness.query_by_label("Less").is_some(),
        )
    };

    let (mesh, collapsed, less) = step(0, 0.0);
    assert!(!mesh && !less);
    step(1, 0.02);
    let (mesh, joining, _) = step(1, 0.12);
    assert!(mesh && joining > collapsed);
    let (mesh, settled, less) = step(1, 1.0);
    assert!(!mesh && less && settled > joining);
    step(0, 1.02);
    let (mesh, closing, less) = step(0, 1.12);
    assert!(mesh && !less && closing < settled && closing > collapsed);
    let (mesh, restored, _) = step(0, 2.0);
    assert!(!mesh && (restored - collapsed).abs() < 0.1);
}

#[test]
fn search_capsule_morphs_in_both_directions() {
    let theme = Theme::light();
    let mut harness = Harness::builder()
        .with_size(egui::vec2(160.0, 120.0))
        .build_ui_state(
            |ui, state: &mut (bool, String)| {
                search_box(ui, &theme, &mut state.0, &mut state.1, "Find", "Close");
            },
            (false, "filters".to_owned()),
        );
    harness
        .ctx
        .all_styles_mut(|style| style.animation_time = 0.2);
    let mut step = |open, time| {
        harness.state_mut().0 = open;
        harness.input_mut().time = Some(time);
        harness.step();
        harness
            .output()
            .shapes
            .iter()
            .find_map(|entry| match &entry.shape {
                egui::Shape::Rect(rect)
                    if rect.stroke.color == theme.border && rect.corner_radius.nw == 21 =>
                {
                    Some((rect.rect, false))
                }
                egui::Shape::Path(path) if path.points.len() == 26 && path.closed => {
                    Some((egui::Rect::from_points(&path.points), true))
                }
                _ => None,
            })
            .expect("search capsule")
    };
    let (closed, _) = step(false, 0.0);
    step(true, 0.02);
    let (opening, tapered) = step(true, 0.05);
    assert!(tapered);
    let (full, _) = step(true, 1.0);
    assert!(closed.width() < opening.width() && opening.width() < full.width());
    assert_eq!(closed.height(), full.height());
    assert!(full.right() <= 160.0);
    step(false, 1.02);
    let (closing, tapered) = step(false, 1.05);
    assert!(!tapered);
    assert_eq!(closing.height(), full.height());
    assert!(closed.width() < closing.width() && closing.width() < full.width());
    assert_eq!(step(false, 2.0), (closed, false));
    assert_eq!(harness.state().1, "filters");
}
