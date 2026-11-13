use egui_kittest::{
    Harness,
    kittest::{NodeT, Queryable},
};
use temivu::{Resizable, ResizableAxis, Theme};

struct State {
    fraction: f32,
    extents: (f32, f32),
    changes: usize,
    /// The main-axis extent available to the resizable in the last frame.
    total: f32,
}

impl State {
    fn new() -> Self {
        Self {
            fraction: 0.5,
            extents: (0.0, 0.0),
            changes: 0,
            total: 0.0,
        }
    }
}

const DIVIDER: f32 = 10.0;

fn view(ui: &mut egui::Ui, state: &mut State) {
    let theme = Theme::light();
    let total = ui.available_width();
    let State {
        fraction,
        extents,
        changes,
        total: recorded_total,
    } = state;
    *recorded_total = total;
    let response = Resizable::new(
        egui::Id::unique("editor-split"),
        ResizableAxis::Horizontal,
        fraction,
        &theme,
    )
    .divider_extent(DIVIDER)
    .min_extents(80.0, 80.0)
    .show(
        ui,
        |ui| {
            ui.label("First pane");
        },
        |ui| {
            ui.label("Second pane");
        },
    );
    *extents = (response.first_extent, response.second_extent);
    *changes += usize::from(response.changed);
}

fn harness() -> Harness<'static, State> {
    Harness::builder()
        .with_size(egui::vec2(400.0, 300.0))
        .build_ui_state(view, State::new())
}

#[test]
fn pane_extents_are_deterministic_and_reported_to_accessibility() {
    let mut harness = harness();
    harness.run();
    let range = harness.state().total - DIVIDER;
    assert_eq!(harness.state().extents, (range / 2.0, range / 2.0));

    let expected = format!("{:.0}, {:.0}", range / 2.0, range / 2.0);
    let divider = harness.get_by_role_and_label(egui::accesskit::Role::Slider, "Resize panes");
    assert_eq!(
        divider.accesskit_node().value().as_deref(),
        Some(expected.as_str())
    );
    assert_eq!(
        divider.accesskit_node().numeric_value(),
        Some(f64::from(range / 2.0))
    );
    assert_eq!(divider.accesskit_node().min_numeric_value(), Some(80.0));
    assert_eq!(
        divider.accesskit_node().max_numeric_value(),
        Some(f64::from(range - 80.0))
    );

    harness.run();
    assert_eq!(
        harness.state().extents,
        (range / 2.0, range / 2.0),
        "idle frames must not change the split"
    );
    assert_eq!(harness.state().changes, 0);
}

#[test]
fn dragging_the_divider_resizes_both_panes_within_the_minimums() {
    let mut harness = harness();
    harness.run();
    let range = harness.state().total - DIVIDER;
    let start = range / 2.0;
    let center = harness
        .get_by_role_and_label(egui::accesskit::Role::Slider, "Resize panes")
        .rect()
        .center();

    harness.hover_at(center);
    harness.run();
    harness.drag_at(center);
    harness.run();
    harness.hover_at(center + egui::vec2(60.0, 0.0));
    harness.run();
    harness.drop_at(center + egui::vec2(60.0, 0.0));
    harness.run();
    assert!(
        (harness.state().extents.0 - (start + 60.0)).abs() < 2.0,
        "dragging 60 points right should widen the first pane, got {:?}",
        harness.state().extents
    );
    assert!(harness.state().changes > 0);

    // Dragging far past the right pane's minimum clamps instead of collapsing it.
    let center = harness
        .get_by_role_and_label(egui::accesskit::Role::Slider, "Resize panes")
        .rect()
        .center();
    harness.hover_at(center);
    harness.run();
    harness.drag_at(center);
    harness.run();
    harness.hover_at(center + egui::vec2(1000.0, 0.0));
    harness.run();
    harness.drop_at(center + egui::vec2(1000.0, 0.0));
    harness.run();
    assert_eq!(harness.state().extents, (range - 80.0, 80.0));
}

#[test]
fn arrow_keys_resize_the_focused_divider_and_keep_focus() {
    let mut harness = Harness::builder()
        .with_size(egui::vec2(400.0, 300.0))
        .build_ui_state(
            |ui, state: &mut State| {
                let theme = Theme::light();
                let total = ui.available_width();
                let State {
                    fraction,
                    extents,
                    changes,
                    total: recorded_total,
                } = state;
                *recorded_total = total;
                let response = Resizable::new(
                    egui::Id::unique("keyboard-split"),
                    ResizableAxis::Horizontal,
                    fraction,
                    &theme,
                )
                .divider_extent(DIVIDER)
                .min_extents(80.0, 80.0)
                .keyboard_step(10.0)
                .show(ui, |_| {}, |_| {});
                *extents = (response.first_extent, response.second_extent);
                *changes += usize::from(response.changed);
            },
            State::new(),
        );
    harness.run();
    let range = harness.state().total - DIVIDER;
    let start = range / 2.0;
    harness
        .get_by_role_and_label(egui::accesskit::Role::Slider, "Resize panes")
        .focus();
    harness.run();

    harness.key_press(egui::Key::ArrowRight);
    harness.run();
    assert_eq!(
        harness.state().extents,
        (start + 10.0, range - start - 10.0)
    );
    assert!(harness.state().changes > 0);
    assert!(
        harness
            .get_by_role_and_label(egui::accesskit::Role::Slider, "Resize panes")
            .is_focused(),
        "arrow keys must resize instead of moving focus away"
    );

    harness.key_press(egui::Key::ArrowLeft);
    harness.run();
    assert_eq!(harness.state().extents, (start, range - start));

    for _ in 0..40 {
        harness.key_press(egui::Key::ArrowRight);
        harness.run();
    }
    assert_eq!(harness.state().extents, (range - 80.0, 80.0));

    for _ in 0..40 {
        harness.key_press(egui::Key::ArrowLeft);
        harness.run();
    }
    assert_eq!(harness.state().extents, (80.0, range - 80.0));
}

#[test]
fn vertical_axis_uses_vertical_arrows() {
    let mut harness = Harness::builder()
        .with_size(egui::vec2(300.0, 400.0))
        .build_ui_state(
            |ui, state: &mut State| {
                let theme = Theme::light();
                let total = ui.available_height();
                let State {
                    fraction,
                    extents,
                    changes,
                    total: recorded_total,
                } = state;
                *recorded_total = total;
                let response = Resizable::new(
                    egui::Id::unique("vertical-split"),
                    ResizableAxis::Vertical,
                    fraction,
                    &theme,
                )
                .divider_extent(DIVIDER)
                .keyboard_step(20.0)
                .show(ui, |_| {}, |_| {});
                *extents = (response.first_extent, response.second_extent);
                *changes += usize::from(response.changed);
            },
            State::new(),
        );
    harness.run();
    let range = harness.state().total - DIVIDER;
    assert_eq!(harness.state().extents, (range / 2.0, range / 2.0));
    harness
        .get_by_role_and_label(egui::accesskit::Role::Slider, "Resize panes")
        .focus();
    harness.run();

    harness.key_press(egui::Key::ArrowDown);
    harness.run();
    assert_eq!(
        harness.state().extents,
        (range / 2.0 + 20.0, range / 2.0 - 20.0)
    );
    harness.key_press(egui::Key::ArrowUp);
    harness.run();
    assert_eq!(harness.state().extents, (range / 2.0, range / 2.0));
}

#[test]
fn minimums_clamp_deterministically_in_a_tiny_window() {
    let mut harness = Harness::builder()
        .with_size(egui::vec2(100.0, 80.0))
        .build_ui_state(view, State::new());
    harness.run();
    let range = harness.state().total - DIVIDER;
    assert!(range > 0.0);
    // The requested minimums do not fit; both are scaled proportionally and the
    // split still fills the window instead of collapsing a pane.
    assert_eq!(harness.state().extents, (range / 2.0, range / 2.0));
    assert_eq!(harness.state().extents.0 + harness.state().extents.1, range);
}

#[test]
fn split_scales_with_the_window_and_keeps_the_fraction() {
    let mut harness = harness();
    harness.run();
    let range = harness.state().total - DIVIDER;
    assert_eq!(harness.state().extents, (range / 2.0, range / 2.0));

    harness.set_size(egui::vec2(600.0, 300.0));
    harness.run();
    let range = harness.state().total - DIVIDER;
    assert_eq!(harness.state().extents, (range / 2.0, range / 2.0));
    assert_eq!(harness.state().fraction, 0.5);
}

#[test]
fn invalid_keyboard_steps_are_ignored() {
    let mut harness = Harness::builder()
        .with_size(egui::vec2(400.0, 300.0))
        .build_ui_state(
            |ui, state: &mut State| {
                let theme = Theme::light();
                let total = ui.available_width();
                let State {
                    fraction,
                    extents,
                    changes,
                    total: recorded_total,
                } = state;
                *recorded_total = total;
                let response = Resizable::new(
                    egui::Id::unique("invalid-step-split"),
                    ResizableAxis::Horizontal,
                    fraction,
                    &theme,
                )
                .divider_extent(DIVIDER)
                .min_extents(80.0, 80.0)
                .keyboard_step(f32::NAN)
                .keyboard_step(-4.0)
                .show(ui, |_| {}, |_| {});
                *extents = (response.first_extent, response.second_extent);
                *changes += usize::from(response.changed);
            },
            State::new(),
        );
    harness.run();
    let range = harness.state().total - DIVIDER;
    let start = range / 2.0;
    harness
        .get_by_role_and_label(egui::accesskit::Role::Slider, "Resize panes")
        .focus();
    harness.run();
    harness.key_press(egui::Key::ArrowRight);
    harness.run();
    let (first, second) = harness.state().extents;
    assert!(first.is_finite() && second.is_finite());
    assert!(
        first > start && second < range - start,
        "default step should apply"
    );
}
