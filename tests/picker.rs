use egui_kittest::{
    Harness,
    kittest::{NodeT, Queryable},
};
use temivu::{Picker, PickerItem, Theme};

const SMALL: &str = "small";
const MEDIUM: &str = "medium";
const LARGE: &str = "large";

#[derive(Default)]
struct PickerState {
    selected: Option<egui::Id>,
    changed: bool,
    reported: Option<egui::Id>,
}

#[test]
fn pointer_click_selects_row_and_exposes_accessible_selection() {
    let mut harness = Harness::builder().build_ui_state(
        |ui, state: &mut PickerState| {
            let items = [
                PickerItem::new(egui::Id::unique(SMALL), "Small"),
                PickerItem::new(egui::Id::unique(MEDIUM), "Medium"),
                PickerItem::new(egui::Id::unique(LARGE), "Large"),
            ];
            let output = Picker::new(&mut state.selected, &items, &Theme::light())
                .label("Size")
                .id(egui::Id::unique("size-picker"))
                .show(ui);
            state.changed |= output.changed;
            state.reported = output.selected;
        },
        PickerState {
            selected: Some(egui::Id::unique(MEDIUM)),
            ..Default::default()
        },
    );
    harness.run();

    let wheel = harness.get_by_role_and_label(egui::accesskit::Role::ListBox, "Size");
    assert_eq!(wheel.accesskit_node().value().as_deref(), Some("Medium"));
    assert!(
        harness
            .get_by_role_and_label(egui::accesskit::Role::ListBoxOption, "Large")
            .rect()
            .is_positive()
    );

    harness
        .get_by_role_and_label(egui::accesskit::Role::ListBoxOption, "Large")
        .click();
    harness.run();

    assert_eq!(harness.state().selected, Some(egui::Id::unique(LARGE)));
    assert_eq!(harness.state().reported, Some(egui::Id::unique(LARGE)));
    assert!(
        harness.state().changed,
        "the response should report the change"
    );
    let wheel = harness.get_by_role_and_label(egui::accesskit::Role::ListBox, "Size");
    assert_eq!(wheel.accesskit_node().value().as_deref(), Some("Large"));
    assert!(wheel.is_focused(), "clicking a row should focus the wheel");
    assert_eq!(
        harness
            .get_by_role_and_label(egui::accesskit::Role::ListBoxOption, "Large")
            .accesskit_node()
            .data()
            .is_selected(),
        Some(true)
    );
}

#[test]
fn keyboard_navigation_skips_disabled_rows_and_clamps() {
    let mut harness = Harness::builder().build_ui_state(
        |ui, state: &mut PickerState| {
            let items = [
                PickerItem::new(egui::Id::unique("first"), "First"),
                PickerItem::new(egui::Id::unique("skipped"), "Skipped").enabled(false),
                PickerItem::new(egui::Id::unique("third"), "Third"),
            ];
            let output = Picker::new(&mut state.selected, &items, &Theme::light())
                .label("Level")
                .show(ui);
            state.changed |= output.changed;
            state.reported = output.selected;
        },
        PickerState::default(),
    );
    harness.run();
    assert!(
        harness
            .get_by_role_and_label(egui::accesskit::Role::ListBoxOption, "Skipped")
            .accesskit_node()
            .is_disabled()
    );
    harness
        .get_by_role_and_label(egui::accesskit::Role::ListBox, "Level")
        .focus();
    harness.run();

    harness.key_press(egui::Key::ArrowDown);
    harness.run();
    assert_eq!(harness.state().selected, Some(egui::Id::unique("first")));

    harness.key_press(egui::Key::ArrowDown);
    harness.run();
    assert_eq!(
        harness.state().selected,
        Some(egui::Id::unique("third")),
        "disabled rows should be skipped"
    );

    harness.key_press(egui::Key::ArrowDown);
    harness.run();
    assert_eq!(
        harness.state().selected,
        Some(egui::Id::unique("third")),
        "non-looping navigation should clamp"
    );

    harness.key_press(egui::Key::ArrowUp);
    harness.run();
    assert_eq!(harness.state().selected, Some(egui::Id::unique("first")));
    let wheel = harness.get_by_role_and_label(egui::accesskit::Role::ListBox, "Level");
    assert_eq!(wheel.accesskit_node().value().as_deref(), Some("First"));
    assert!(wheel.is_focused(), "keyboard navigation keeps wheel focus");
}

#[test]
fn wheel_scroll_selects_the_nearest_row_and_stops_repainting() {
    let mut harness = Harness::builder()
        .with_size(egui::vec2(320.0, 420.0))
        .build_ui_state(
            |ui, state: &mut PickerState| {
                let items: [PickerItem<'_>; 10] = std::array::from_fn(|index| {
                    PickerItem::new(egui::Id::unique(("number", index)), NUMBER_LABELS[index])
                });
                let output = Picker::new(&mut state.selected, &items, &Theme::light())
                    .label("Number")
                    .show(ui);
                state.changed |= output.changed;
                state.reported = output.selected;
            },
            PickerState {
                selected: Some(egui::Id::unique(("number", 0))),
                ..Default::default()
            },
        );
    harness.run();
    harness
        .get_by_role_and_label(egui::accesskit::Role::ListBox, "Number")
        .hover();
    harness.run();

    for _ in 0..3 {
        harness.input_mut().events.push(egui::Event::MouseWheel {
            source: egui::MouseWheelSource::Wheel,
            unit: egui::MouseWheelUnit::Point,
            delta: egui::vec2(0.0, -40.0),
            phase: egui::TouchPhase::Move,
            modifiers: egui::Modifiers::NONE,
        });
        harness.step();
    }
    harness.run_steps(30);

    assert!(harness.state().changed, "scrolling should select a new row");
    assert_ne!(
        harness.state().selected,
        Some(egui::Id::unique(("number", 0)))
    );
    let selected = harness
        .state()
        .selected
        .expect("a row should stay selected");
    let wheel = harness.get_by_role_and_label(egui::accesskit::Role::ListBox, "Number");
    assert!(wheel.accesskit_node().value().is_some());

    // A settled wheel must not keep the UI awake.
    harness.step();
    assert!(
        !harness.ctx.has_requested_repaint(),
        "an idle picker must not request repaints"
    );
    assert_eq!(harness.state().selected, Some(selected));
}

#[test]
fn disabled_picker_ignores_pointer_and_keyboard_input() {
    let mut harness = Harness::builder().build_ui_state(
        |ui, state: &mut PickerState| {
            let items = [
                PickerItem::new(egui::Id::unique("one"), "One"),
                PickerItem::new(egui::Id::unique("two"), "Two"),
            ];
            let output = Picker::new(&mut state.selected, &items, &Theme::light())
                .label("Disabled wheel")
                .enabled(false)
                .show(ui);
            state.changed |= output.changed;
        },
        PickerState {
            selected: Some(egui::Id::unique("one")),
            ..Default::default()
        },
    );
    harness.run();
    assert!(
        harness
            .get_by_role_and_label(egui::accesskit::Role::ListBox, "Disabled wheel")
            .accesskit_node()
            .is_disabled()
    );
    harness
        .get_by_role_and_label(egui::accesskit::Role::ListBoxOption, "Two")
        .click();
    harness.run();
    harness
        .get_by_role_and_label(egui::accesskit::Role::ListBox, "Disabled wheel")
        .focus();
    harness.run();
    harness.key_press(egui::Key::ArrowDown);
    harness.run();
    assert_eq!(harness.state().selected, Some(egui::Id::unique("one")));
    assert!(!harness.state().changed);
}

const NUMBER_LABELS: [&str; 10] = ["0", "1", "2", "3", "4", "5", "6", "7", "8", "9"];
