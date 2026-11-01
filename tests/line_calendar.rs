use egui_kittest::{
    Harness,
    kittest::{NodeT, Queryable},
};
use temivu::{Date, LineCalendar, LineCalendarOrientation, LineCalendarState, Theme};

struct State {
    calendar: LineCalendarState,
    activated: Option<Date>,
    month_changed: Option<Date>,
    changed: bool,
}

fn date(year: i32, month: u8, day: u8) -> Date {
    Date::new(year, month, day).expect("test date is valid")
}

#[derive(Clone, Copy)]
struct Options {
    anchor: Date,
    selected: Option<Date>,
    today: Option<Date>,
    min: Date,
    max: Date,
    days: usize,
    orientation: LineCalendarOrientation,
    enabled: bool,
}

impl Default for Options {
    fn default() -> Self {
        Self {
            anchor: date(2024, 7, 10),
            selected: None,
            today: Some(date(2024, 7, 15)),
            min: date(2024, 1, 1),
            max: date(2024, 12, 31),
            days: 7,
            orientation: LineCalendarOrientation::Horizontal,
            enabled: true,
        }
    }
}

fn build(options: Options) -> Harness<'static, State> {
    Harness::builder()
        .with_size(egui::vec2(460.0, 200.0))
        .build_ui_state(
            move |ui, state: &mut State| {
                let theme = Theme::light();
                let mut calendar = LineCalendar::new(&mut state.calendar, &theme)
                    .days(options.days)
                    .orientation(options.orientation)
                    .bounds(options.min, options.max)
                    .enabled(options.enabled);
                if let Some(today) = options.today {
                    calendar = calendar.today(today);
                }
                let output = calendar.show(ui);
                state.activated = output.activated.or(state.activated);
                state.month_changed = output.month_changed.or(state.month_changed);
                state.changed |= output.changed;
            },
            State {
                calendar: LineCalendarState::new(options.anchor, options.selected),
                activated: None,
                month_changed: None,
                changed: false,
            },
        )
}

fn focus(harness: &mut Harness<'static, State>) {
    harness
        .get_by_role_and_label(egui::accesskit::Role::List, "Date strip")
        .focus();
    harness.run();
}

#[test]
fn renders_a_window_around_the_anchor_with_accessible_labels() {
    let mut harness = build(Options::default());
    harness.run();

    harness.get_by_role_and_label(egui::accesskit::Role::List, "Date strip");
    harness.get_by_role_and_label(egui::accesskit::Role::ListItem, "July 10, 2024");
    harness.get_by_role_and_label(egui::accesskit::Role::ListItem, "July 16, 2024");
    harness.get_by_role_and_label(egui::accesskit::Role::ListItem, "July 15, 2024, Today");
    assert!(
        harness
            .query_by_role_and_label(egui::accesskit::Role::ListItem, "July 17, 2024")
            .is_none(),
        "the strip renders exactly its configured window"
    );
    // Selected state is exposed even before any interaction.
    let today =
        harness.get_by_role_and_label(egui::accesskit::Role::ListItem, "July 15, 2024, Today");
    assert_eq!(today.accesskit_node().is_selected(), Some(false));
}

#[test]
fn clicking_a_day_selects_it_and_reports_activation() {
    let mut harness = build(Options::default());
    harness.run();

    harness
        .get_by_role_and_label(egui::accesskit::Role::ListItem, "July 12, 2024")
        .click();
    harness.run();

    assert_eq!(harness.state().calendar.selected(), Some(date(2024, 7, 12)));
    assert_eq!(harness.state().activated, Some(date(2024, 7, 12)));
    let node = harness.get_by_role_and_label(egui::accesskit::Role::ListItem, "July 12, 2024");
    assert_eq!(node.accesskit_node().is_selected(), Some(true));
    assert!(harness.state().changed);
}

#[test]
fn keyboard_arrows_move_focus_and_enter_selects() {
    let mut harness = build(Options::default());
    harness.run();
    focus(&mut harness);
    assert_eq!(harness.state().calendar.focused(), Some(date(2024, 7, 15)));

    harness.key_press(egui::Key::ArrowRight);
    harness.run();
    assert_eq!(harness.state().calendar.focused(), Some(date(2024, 7, 16)));

    harness.key_press(egui::Key::ArrowLeft);
    harness.run();
    assert_eq!(harness.state().calendar.focused(), Some(date(2024, 7, 15)));

    harness.key_press(egui::Key::Home);
    harness.run();
    assert_eq!(harness.state().calendar.focused(), Some(date(2024, 7, 10)));

    harness.key_press(egui::Key::End);
    harness.run();
    assert_eq!(harness.state().calendar.focused(), Some(date(2024, 7, 16)));

    harness.key_press(egui::Key::Enter);
    harness.run();
    assert_eq!(harness.state().calendar.selected(), Some(date(2024, 7, 16)));
    assert_eq!(harness.state().activated, Some(date(2024, 7, 16)));
}

#[test]
fn paging_reports_the_new_visible_month() {
    let mut harness = build(Options {
        anchor: date(2024, 7, 27),
        today: Some(date(2024, 7, 29)),
        ..Options::default()
    });
    harness.run();
    focus(&mut harness);
    assert_eq!(harness.state().calendar.focused(), Some(date(2024, 7, 29)));

    harness.key_press(egui::Key::PageDown);
    harness.run();
    assert_eq!(harness.state().calendar.focused(), Some(date(2024, 8, 29)));
    assert_eq!(harness.state().month_changed, Some(date(2024, 8, 1)));
    assert_eq!(harness.state().calendar.visible_month(), date(2024, 8, 1));
    harness.get_by_role_and_label(egui::accesskit::Role::ListItem, "August 29, 2024");

    harness.key_press(egui::Key::PageUp);
    harness.run();
    assert_eq!(harness.state().calendar.focused(), Some(date(2024, 7, 29)));
}

#[test]
fn bounds_and_predicate_disable_days_and_keyboard_skips_them() {
    let disabled = |day: Date| day.day() != 14;
    let mut harness = Harness::builder()
        .with_size(egui::vec2(460.0, 200.0))
        .build_ui_state(
            move |ui, state: &mut State| {
                let theme = Theme::light();
                let output = LineCalendar::new(&mut state.calendar, &theme)
                    .days(7)
                    .bounds(date(2024, 7, 12), date(2024, 7, 18))
                    .today(date(2024, 7, 13))
                    .selectable(&disabled)
                    .show(ui);
                state.activated = output.activated.or(state.activated);
                state.changed |= output.changed;
            },
            State {
                calendar: LineCalendarState::new(date(2024, 7, 10), None),
                activated: None,
                month_changed: None,
                changed: false,
            },
        );
    harness.run();

    // The window is clamped into the bounds, so earlier days are not rendered.
    assert_eq!(harness.state().calendar.anchor(), date(2024, 7, 12));
    let out_of_bounds =
        harness.get_by_role_and_label(egui::accesskit::Role::ListItem, "July 14, 2024");
    assert!(out_of_bounds.accesskit_node().is_disabled());
    out_of_bounds.click();
    harness.run();
    assert_eq!(harness.state().calendar.selected(), None);

    focus(&mut harness);
    assert_eq!(harness.state().calendar.focused(), Some(date(2024, 7, 13)));
    // The next day is disabled by the predicate, so focus skips to the 15th.
    harness.key_press(egui::Key::ArrowRight);
    harness.run();
    assert_eq!(harness.state().calendar.focused(), Some(date(2024, 7, 15)));
}

#[test]
fn vertical_orientation_navigates_with_vertical_arrows() {
    let mut harness = build(Options {
        orientation: LineCalendarOrientation::Vertical,
        ..Options::default()
    });
    harness.run();
    focus(&mut harness);

    harness.key_press(egui::Key::ArrowDown);
    harness.run();
    assert_eq!(harness.state().calendar.focused(), Some(date(2024, 7, 16)));

    harness.key_press(egui::Key::ArrowUp);
    harness.run();
    assert_eq!(harness.state().calendar.focused(), Some(date(2024, 7, 15)));
}

#[test]
fn a_far_selection_moves_the_window_on_show() {
    let mut harness = build(Options {
        anchor: date(2024, 7, 1),
        selected: Some(date(2024, 7, 30)),
        ..Options::default()
    });
    harness.run();
    assert_eq!(harness.state().calendar.anchor(), date(2024, 7, 24));
    harness.get_by_role_and_label(egui::accesskit::Role::ListItem, "July 30, 2024");
}

#[test]
fn a_disabled_strip_ignores_input() {
    let mut harness = build(Options {
        enabled: false,
        ..Options::default()
    });
    harness.run();
    let day = harness.get_by_role_and_label(egui::accesskit::Role::ListItem, "July 12, 2024");
    assert!(day.accesskit_node().is_disabled());
    day.click();
    harness.run();
    assert_eq!(harness.state().calendar.selected(), None);

    harness
        .get_by_role_and_label(egui::accesskit::Role::List, "Date strip")
        .focus();
    harness.run();
    harness.key_press(egui::Key::ArrowRight);
    harness.run();
    assert_eq!(harness.state().calendar.focused(), None);
}
