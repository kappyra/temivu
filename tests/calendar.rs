use egui_kittest::{
    Harness,
    kittest::{NodeT, Queryable},
};
use temivu::{Calendar, CalendarLabels, CalendarState, CalendarStyle, Date, Dialog, Theme};

#[test]
fn calendar_supports_custom_selection_color_and_touch_size() {
    let selected = date(2024, 7, 15);
    let mut harness = Harness::builder()
        .with_size(egui::vec2(500.0, 600.0))
        .build_ui_state(
            |ui, state: &mut CalendarState| {
                let theme = Theme::mint_light().touch();
                let mut style = CalendarStyle::from_theme(&theme);
                style.selected_fill = egui::Color32::from_rgb(30, 60, 120);
                style.radius = 12;
                Calendar::new(state, &theme).style(style).show(ui);
            },
            CalendarState::new(selected, Some(selected)),
        );
    harness.run();
    let cell =
        harness.get_by_role_and_label(egui::accesskit::Role::GridCell, "Monday, July 15, 2024");
    assert!(cell.rect().width() >= 44.0);
    assert!(harness.output().shapes.iter().any(|shape| matches!(&shape.shape, egui::Shape::Rect(rect) if rect.fill == egui::Color32::from_rgb(30, 60, 120))));
}

fn date(year: i32, month: u8, day: u8) -> Date {
    Date::new(year, month, day).expect("test date is valid")
}

fn build(state: CalendarState, today: Date) -> Harness<'static, CalendarState> {
    Harness::builder()
        .with_size(egui::vec2(360.0, 480.0))
        .build_ui_state(
            move |ui, state: &mut CalendarState| {
                let theme = Theme::light();
                Calendar::new(state, &theme)
                    .year_navigation(true)
                    .today(today)
                    .show(ui);
            },
            state,
        )
}

#[test]
fn renders_month_heading_weekdays_and_accessible_day_cells() {
    let today = date(2024, 7, 15);
    let mut harness = build(CalendarState::new(today, None), today);
    harness.run();

    let heading = harness.get_by_role_and_label(egui::accesskit::Role::Heading, "July 2024");
    assert_eq!(heading.accesskit_node().level(), Some(2));
    harness.get_by_role_and_label(egui::accesskit::Role::Grid, "Calendar");

    let today_cell = harness.get_by_role_and_label(
        egui::accesskit::Role::GridCell,
        "Monday, July 15, 2024, Today",
    );
    assert_eq!(today_cell.accesskit_node().is_selected(), Some(false));
    assert!(!today_cell.accesskit_node().is_disabled());
    // July 2024 starts on a Monday, so the 15th is row 2, column 0.
    assert_eq!(today_cell.accesskit_node().data().row_index(), Some(2));
    assert_eq!(today_cell.accesskit_node().data().column_index(), Some(0));

    // Weekday column headers are exposed in Monday-first order.
    assert_eq!(
        harness
            .get_all_by_role_and_label(egui::accesskit::Role::ColumnHeader, "S")
            .count(),
        2,
        "Saturday and Sunday both use the narrow name S"
    );
    harness.get_by_role_and_label(egui::accesskit::Role::ColumnHeader, "M");

    // July 2024 starts on a Monday, so trailing adjacent-month days fill the
    // last row and are labelled with their real date.
    harness.get_by_role_and_label(egui::accesskit::Role::GridCell, "Monday, July 1, 2024");
    harness.get_by_role_and_label(egui::accesskit::Role::GridCell, "Thursday, August 1, 2024");
}

#[test]
fn clicking_a_day_selects_it_and_updates_caller_state() {
    let today = date(2024, 7, 15);
    let mut harness = build(CalendarState::new(today, None), today);
    harness.run();
    assert_eq!(harness.state().selected(), None);

    harness
        .get_by_role_and_label(egui::accesskit::Role::GridCell, "Saturday, July 20, 2024")
        .click();
    harness.run();

    assert_eq!(harness.state().selected(), Some(date(2024, 7, 20)));
    let selected =
        harness.get_by_role_and_label(egui::accesskit::Role::GridCell, "Saturday, July 20, 2024");
    assert_eq!(selected.accesskit_node().is_selected(), Some(true));
    let other =
        harness.get_by_role_and_label(egui::accesskit::Role::GridCell, "Sunday, July 21, 2024");
    assert_eq!(other.accesskit_node().is_selected(), Some(false));
}

#[test]
fn clicking_an_out_of_bounds_or_disabled_day_does_not_select_it() {
    let today = date(2024, 7, 15);
    let disabled = |date: Date| date.day() != 15;
    let mut harness = Harness::builder()
        .with_size(egui::vec2(360.0, 480.0))
        .build_ui_state(
            move |ui, state: &mut CalendarState| {
                let theme = Theme::light();
                Calendar::new(state, &theme)
                    .today(today)
                    .bounds(date(2024, 7, 15), date(2024, 8, 31))
                    .selectable(&disabled)
                    .show(ui);
            },
            CalendarState::new(today, None),
        );
    harness.run();

    let disabled_cell =
        harness.get_by_role_and_label(egui::accesskit::Role::GridCell, "Monday, July 8, 2024");
    assert!(disabled_cell.accesskit_node().is_disabled());
    disabled_cell.click();
    harness.run();
    assert_eq!(harness.state().selected(), None);

    // July 15 is in bounds but the caller's predicate disables it.
    let today_cell = harness.get_by_role_and_label(
        egui::accesskit::Role::GridCell,
        "Monday, July 15, 2024, Today",
    );
    assert!(today_cell.accesskit_node().is_disabled());
    today_cell.click();
    harness.run();
    assert_eq!(harness.state().selected(), None);

    // July 20 satisfies both the bounds and the predicate.
    harness
        .get_by_role_and_label(egui::accesskit::Role::GridCell, "Saturday, July 20, 2024")
        .click();
    harness.run();
    assert_eq!(harness.state().selected(), Some(date(2024, 7, 20)));
}

#[test]
fn month_and_year_navigation_buttons_update_the_visible_month() {
    let today = date(2024, 7, 15);
    let mut harness = build(CalendarState::new(today, None), today);
    harness.run();

    harness
        .get_by_role_and_label(egui::accesskit::Role::Button, "Next month")
        .click();
    harness.run();
    assert_eq!(harness.state().visible_month(), date(2024, 8, 1));
    harness.get_by_role_and_label(egui::accesskit::Role::Heading, "August 2024");

    harness
        .get_by_role_and_label(egui::accesskit::Role::Button, "Previous month")
        .click();
    harness.run();
    assert_eq!(harness.state().visible_month(), date(2024, 7, 1));

    harness
        .get_by_role_and_label(egui::accesskit::Role::Button, "Next year")
        .click();
    harness.run();
    assert_eq!(harness.state().visible_month(), date(2025, 7, 1));
    harness.get_by_role_and_label(egui::accesskit::Role::Heading, "July 2025");

    harness
        .get_by_role_and_label(egui::accesskit::Role::Button, "Previous year")
        .click();
    harness.run();
    assert_eq!(harness.state().visible_month(), date(2024, 7, 1));
}

#[test]
fn navigation_buttons_disable_at_the_date_bounds() {
    let today = date(2024, 7, 15);
    let bounds = (date(2024, 7, 1), date(2024, 8, 31));
    let mut harness = Harness::builder()
        .with_size(egui::vec2(360.0, 480.0))
        .build_ui_state(
            move |ui, state: &mut CalendarState| {
                let theme = Theme::light();
                Calendar::new(state, &theme)
                    .today(today)
                    .bounds(bounds.0, bounds.1)
                    .show(ui);
            },
            CalendarState::new(today, None),
        );
    harness.run();

    // The first allowed month cannot page backwards.
    let previous = harness.get_by_role_and_label(egui::accesskit::Role::Button, "Previous month");
    assert!(previous.accesskit_node().is_disabled());
    previous.click();
    harness.run();
    assert_eq!(harness.state().visible_month(), date(2024, 7, 1));

    harness
        .get_by_role_and_label(egui::accesskit::Role::Button, "Next month")
        .click();
    harness.run();
    assert_eq!(harness.state().visible_month(), date(2024, 8, 1));

    let next = harness.get_by_role_and_label(egui::accesskit::Role::Button, "Next month");
    assert!(next.accesskit_node().is_disabled());
    harness
        .get_by_role_and_label(egui::accesskit::Role::Button, "Previous month")
        .click();
    harness.run();
    assert_eq!(harness.state().visible_month(), date(2024, 7, 1));

    // An out-of-bounds initial month is clamped into the allowed range.
    let mut clamped = Harness::builder()
        .with_size(egui::vec2(360.0, 480.0))
        .build_ui_state(
            move |ui, state: &mut CalendarState| {
                let theme = Theme::light();
                Calendar::new(state, &theme)
                    .today(today)
                    .bounds(date(2024, 7, 1), date(2024, 8, 31))
                    .show(ui);
            },
            CalendarState::new(date(2024, 1, 15), None),
        );
    clamped.run();
    assert_eq!(clamped.state().visible_month(), date(2024, 7, 1));
    clamped.get_by_role_and_label(egui::accesskit::Role::Heading, "July 2024");
}

#[test]
fn keyboard_arrows_move_the_focused_day_across_months() {
    let today = date(2024, 7, 15);
    let mut harness = build(CalendarState::new(today, None), today);
    harness.run();
    harness
        .get_by_role_and_label(egui::accesskit::Role::Grid, "Calendar")
        .focus();
    harness.run();
    assert_eq!(harness.state().focused(), Some(date(2024, 7, 15)));

    harness.key_press(egui::Key::ArrowRight);
    harness.run();
    assert_eq!(harness.state().focused(), Some(date(2024, 7, 16)));

    harness.key_press(egui::Key::ArrowUp);
    harness.run();
    assert_eq!(harness.state().focused(), Some(date(2024, 7, 9)));

    harness.key_press(egui::Key::ArrowDown);
    harness.run();
    assert_eq!(harness.state().focused(), Some(date(2024, 7, 16)));

    // Moving back from the first of a month pages the visible month.
    harness.key_press(egui::Key::PageUp);
    harness.run();
    assert_eq!(harness.state().visible_month(), date(2024, 6, 1));
    assert_eq!(harness.state().focused(), Some(date(2024, 6, 16)));

    harness.key_press(egui::Key::PageDown);
    harness.run();
    assert_eq!(harness.state().visible_month(), date(2024, 7, 1));
    assert_eq!(harness.state().focused(), Some(date(2024, 7, 16)));

    harness.key_press_modifiers(egui::Modifiers::SHIFT, egui::Key::PageDown);
    harness.run();
    assert_eq!(harness.state().visible_month(), date(2025, 7, 1));
    harness.key_press_modifiers(egui::Modifiers::SHIFT, egui::Key::PageUp);
    harness.run();
    assert_eq!(harness.state().visible_month(), date(2024, 7, 1));
}

#[test]
fn home_and_end_move_focus_within_the_visible_week() {
    let today = date(2024, 7, 17);
    let mut harness = build(CalendarState::new(today, None), today);
    harness.run();
    harness
        .get_by_role_and_label(egui::accesskit::Role::Grid, "Calendar")
        .focus();
    harness.run();
    assert_eq!(harness.state().focused(), Some(date(2024, 7, 17)));

    harness.key_press(egui::Key::Home);
    harness.run();
    assert_eq!(harness.state().focused(), Some(date(2024, 7, 15)));

    harness.key_press(egui::Key::End);
    harness.run();
    assert_eq!(harness.state().focused(), Some(date(2024, 7, 21)));
}

#[test]
fn enter_and_space_select_the_focused_day() {
    let today = date(2024, 7, 15);
    let mut harness = build(CalendarState::new(today, None), today);
    harness.run();
    harness
        .get_by_role_and_label(egui::accesskit::Role::Grid, "Calendar")
        .focus();
    harness.run();

    harness.key_press(egui::Key::ArrowRight);
    harness.run();
    harness.key_press(egui::Key::Enter);
    harness.run();
    assert_eq!(harness.state().selected(), Some(date(2024, 7, 16)));

    harness.key_press(egui::Key::ArrowRight);
    harness.run();
    harness.key_press(egui::Key::Space);
    harness.run();
    assert_eq!(harness.state().selected(), Some(date(2024, 7, 17)));
}

#[test]
fn toggleable_selection_clears_on_a_second_click() {
    let today = date(2024, 7, 15);
    let mut harness = Harness::builder()
        .with_size(egui::vec2(360.0, 480.0))
        .build_ui_state(
            move |ui, state: &mut CalendarState| {
                let theme = Theme::light();
                Calendar::new(state, &theme)
                    .today(today)
                    .toggleable(true)
                    .show(ui);
            },
            CalendarState::new(today, None),
        );
    harness.run();
    for _ in 0..2 {
        harness
            .get_by_role_and_label(egui::accesskit::Role::GridCell, "Saturday, July 20, 2024")
            .click();
        harness.run();
    }
    assert_eq!(harness.state().selected(), None);
}

#[test]
fn first_day_of_week_rotates_the_columns() {
    let today = date(2024, 7, 15);
    let mut harness = Harness::builder()
        .with_size(egui::vec2(360.0, 480.0))
        .build_ui_state(
            move |ui, state: &mut CalendarState| {
                let theme = Theme::light();
                Calendar::new(state, &theme)
                    .today(today)
                    .first_day_of_week(temivu::Weekday::Sunday)
                    .show(ui);
            },
            CalendarState::new(today, None),
        );
    harness.run();
    let cell = harness.get_by_role_and_label(
        egui::accesskit::Role::GridCell,
        "Monday, July 15, 2024, Today",
    );
    assert_eq!(cell.accesskit_node().data().column_index(), Some(1));
    assert_eq!(cell.accesskit_node().data().row_index(), Some(2));
}

#[test]
fn caller_labels_override_the_english_defaults() {
    let today = date(2024, 7, 15);
    let labels = CalendarLabels {
        next_month: "Naechster Monat",
        day_label: &|date: Date| date.to_string(),
        ..CalendarLabels::english()
    };
    let mut harness = Harness::builder()
        .with_size(egui::vec2(360.0, 480.0))
        .build_ui_state(
            move |ui, state: &mut CalendarState| {
                let theme = Theme::light();
                Calendar::new(state, &theme)
                    .today(today)
                    .labels(labels)
                    .show(ui);
            },
            CalendarState::new(today, None),
        );
    harness.run();
    harness.get_by_role_and_label(egui::accesskit::Role::Button, "Naechster Monat");
    harness.get_by_role_and_label(egui::accesskit::Role::GridCell, "2024-07-15, Today");
    harness.get_by_role_and_label(egui::accesskit::Role::GridCell, "2024-07-16");
    // The default English label is replaced, not duplicated.
    assert!(
        harness
            .query_by_role_and_label(
                egui::accesskit::Role::GridCell,
                "Monday, July 15, 2024, Today"
            )
            .is_none()
    );
}

#[test]
fn keyboard_focus_moves_the_active_descendant() {
    let today = date(2024, 7, 15);
    let mut harness = build(CalendarState::new(today, None), today);
    harness.run();
    harness
        .get_by_role_and_label(egui::accesskit::Role::Grid, "Calendar")
        .focus();
    harness.run();
    assert!(
        harness
            .get_by_role_and_label(
                egui::accesskit::Role::GridCell,
                "Monday, July 15, 2024, Today"
            )
            .is_focused(),
        "the focused day should be the grid's active descendant"
    );

    harness.key_press(egui::Key::ArrowRight);
    harness.run();
    assert!(
        harness
            .get_by_role_and_label(egui::accesskit::Role::GridCell, "Tuesday, July 16, 2024")
            .is_focused()
    );
}

#[test]
fn clicking_an_adjacent_month_day_pages_the_grid() {
    let today = date(2024, 7, 15);
    let mut harness = build(CalendarState::new(today, None), today);
    harness.run();

    harness
        .get_by_role_and_label(egui::accesskit::Role::GridCell, "Thursday, August 1, 2024")
        .click();
    harness.run();

    assert_eq!(harness.state().selected(), Some(date(2024, 8, 1)));
    assert_eq!(harness.state().visible_month(), date(2024, 8, 1));
    harness.get_by_role_and_label(egui::accesskit::Role::Heading, "August 2024");
}

#[test]
fn a_disabled_calendar_exposes_disabled_state_and_ignores_input() {
    let today = date(2024, 7, 15);
    let mut harness = Harness::builder()
        .with_size(egui::vec2(360.0, 480.0))
        .build_ui_state(
            move |ui, state: &mut CalendarState| {
                let theme = Theme::light();
                Calendar::new(state, &theme)
                    .today(today)
                    .enabled(false)
                    .show(ui);
            },
            CalendarState::new(today, None),
        );
    harness.run();

    assert!(
        harness
            .get_by_role_and_label(egui::accesskit::Role::Button, "Next month")
            .accesskit_node()
            .is_disabled()
    );
    let cell =
        harness.get_by_role_and_label(egui::accesskit::Role::GridCell, "Saturday, July 20, 2024");
    assert!(cell.accesskit_node().is_disabled());
    cell.click();
    harness.run();
    assert_eq!(harness.state().selected(), None);
    assert_eq!(harness.state().visible_month(), date(2024, 7, 1));
}

#[test]
fn month_picker_selects_a_month_and_returns_to_the_day_grid() {
    let today = date(2024, 7, 15);
    let mut harness = build(CalendarState::new(today, None), today);
    harness.run();
    harness.get_by_label("Choose month").click();
    harness.run();
    harness.get_by_role_and_label(egui::accesskit::Role::Grid, "Months");
    harness.get_by_label("March").click();
    harness.run();
    harness.get_by_role_and_label(egui::accesskit::Role::Heading, "March 2024");
    assert_eq!(harness.state().visible_month(), date(2024, 3, 1));
    assert!(
        harness
            .query_by_role_and_label(egui::accesskit::Role::Grid, "Months")
            .is_none(),
        "selecting a month should return to the day grid"
    );
}

#[test]
fn year_picker_pages_and_returns_to_the_month_grid() {
    let today = date(2024, 7, 15);
    let mut harness = build(CalendarState::new(today, None), today);
    harness.run();
    harness.get_by_label("Choose month").click();
    harness.run();
    harness.get_by_label("Choose year").click();
    harness.run();
    harness.get_by_role_and_label(egui::accesskit::Role::Grid, "Years");
    // The default 12-year page for 2024 is 2017..=2028.
    for year in ["2017", "2028"] {
        harness.get_by_label(year);
    }
    harness.get_by_label("2025").click();
    harness.run();
    harness.get_by_role_and_label(egui::accesskit::Role::Grid, "Months");
    harness.get_by_label("February").click();
    harness.run();
    harness.get_by_role_and_label(egui::accesskit::Role::Heading, "February 2025");
    assert_eq!(harness.state().visible_month(), date(2025, 2, 1));
}

#[test]
fn escape_steps_back_through_the_picker_views() {
    let today = date(2024, 7, 15);
    let mut harness = build(CalendarState::new(today, None), today);
    harness.run();
    harness.get_by_label("Choose month").click();
    harness.run();
    harness.get_by_label("Choose year").click();
    harness.run();
    harness.get_by_role_and_label(egui::accesskit::Role::Grid, "Years");
    harness.key_press(egui::Key::Escape);
    harness.run();
    harness.get_by_role_and_label(egui::accesskit::Role::Grid, "Months");
    harness.key_press(egui::Key::Escape);
    harness.run();
    harness.get_by_role_and_label(egui::accesskit::Role::Heading, "July 2024");
    harness.get_by_role_and_label(egui::accesskit::Role::Grid, "Calendar");
}

#[test]
fn month_picker_disables_months_outside_the_date_bounds() {
    let today = date(2024, 6, 15);
    let mut harness = Harness::builder().build_ui_state(
        |ui, state: &mut CalendarState| {
            let theme = Theme::light();
            Calendar::new(state, &theme)
                .bounds(date(2024, 3, 10), date(2024, 10, 20))
                .today(date(2024, 6, 15))
                .show(ui);
        },
        CalendarState::new(today, None),
    );
    harness.run();
    harness.get_by_label("Choose month").click();
    harness.run();
    assert!(
        harness
            .get_by_label("January")
            .accesskit_node()
            .is_disabled()
    );
    assert!(!harness.get_by_label("May").accesskit_node().is_disabled());
    harness.get_by_label("January").click();
    harness.run();
    assert_eq!(
        harness.state().visible_month().month(),
        6,
        "clicking a disabled month must not change the visible month"
    );
    harness.get_by_role_and_label(egui::accesskit::Role::Grid, "Months");
}

#[test]
fn picker_focuses_the_selected_month_and_exposes_selected_state() {
    let today = date(2026, 11, 15);
    let mut harness = Harness::builder().build_ui_state(
        |ui, state: &mut CalendarState| {
            let theme = Theme::light();
            Calendar::new(state, &theme)
                .year_navigation(true)
                .today(today)
                .show(ui);
        },
        CalendarState::new(today, None),
    );
    harness.run();
    harness.get_by_label("Choose month").click();
    harness.run();
    let november = harness.get_by_label("November");
    assert_eq!(
        november.accesskit_node().is_selected(),
        Some(true),
        "the visible month should be exposed as selected"
    );
    assert!(
        november.is_focused(),
        "the picker should focus the selected month, not the first month"
    );
    harness.get_by_label("November").click();
    harness.run();
    assert_eq!(harness.state().visible_month(), date(2026, 11, 1));
}

#[test]
fn opening_the_year_view_shows_the_page_with_the_visible_year() {
    let today = date(2026, 11, 15);
    let mut harness = Harness::builder().build_ui_state(
        |ui, state: &mut CalendarState| {
            let theme = Theme::light();
            Calendar::new(state, &theme)
                .year_navigation(true)
                .today(today)
                .show(ui);
        },
        CalendarState::new(today, None),
    );
    harness.run();
    for _ in 0..4 {
        harness.get_by_label("Next year").click();
        harness.run();
    }
    assert_eq!(harness.state().visible_month().year(), 2030);
    harness.get_by_label("Choose month").click();
    harness.run();
    harness.get_by_label("Choose year").click();
    harness.run();
    harness.get_by_label("2029");
    let selected = harness.get_by_label("2030");
    eprintln!(
        "role={:?} selected={:?} toggled={:?} value={:?} label={:?}",
        selected.accesskit_node().role(),
        selected.accesskit_node().is_selected(),
        selected.accesskit_node().toggled(),
        selected.accesskit_node().value(),
        selected.accesskit_node().label()
    );
    assert_eq!(selected.accesskit_node().is_selected(), Some(true));
}

#[test]
fn a_re_added_calendar_does_not_reopen_in_the_picker() {
    let today = date(2026, 11, 15);
    let mut harness = Harness::builder().build_ui_state(
        |ui, state: &mut (CalendarState, bool)| {
            if state.1 {
                let theme = Theme::light();
                Calendar::new(&mut state.0, &theme).today(today).show(ui);
            }
        },
        (CalendarState::new(today, None), true),
    );
    harness.run();
    harness.get_by_label("Choose month").click();
    harness.run();
    harness.get_by_role_and_label(egui::accesskit::Role::Grid, "Months");
    harness.state_mut().1 = false;
    harness.run_steps(4);
    harness.state_mut().1 = true;
    harness.run();
    harness.get_by_role_and_label(egui::accesskit::Role::Grid, "Calendar");
    assert!(
        harness
            .query_by_role_and_label(egui::accesskit::Role::Grid, "Months")
            .is_none(),
        "a re-added calendar should start in the day view"
    );
}

#[test]
fn picker_escape_does_not_also_close_a_hosting_dialog() {
    let today = date(2026, 11, 15);
    let mut harness = Harness::builder().build_ui_state(
        |ui, state: &mut (CalendarState, bool)| {
            let theme = Theme::light();
            Dialog::new(egui::Id::unique("calendar-host-dialog"), "Pick a date").show(
                ui.ctx(),
                &mut state.1,
                |ui| {
                    Calendar::new(&mut state.0, &theme).today(today).show(ui);
                },
            );
        },
        (CalendarState::new(today, None), true),
    );
    harness.run();
    harness.get_by_label("Choose month").click();
    harness.run();
    harness.get_by_role_and_label(egui::accesskit::Role::Grid, "Months");
    harness.key_press(egui::Key::Escape);
    harness.run();
    assert!(
        harness
            .query_by_role_and_label(egui::accesskit::Role::Dialog, "Pick a date")
            .is_some(),
        "the picker must consume Escape before the hosting dialog sees it"
    );
    harness.get_by_role_and_label(egui::accesskit::Role::Grid, "Calendar");
    harness.key_press(egui::Key::Escape);
    harness.run();
    assert!(
        harness
            .query_by_role_and_label(egui::accesskit::Role::Dialog, "Pick a date")
            .is_none(),
        "with the picker closed, Escape should close the dialog"
    );
}

#[test]
fn compact_calendar_keeps_its_grid_height_and_returns_to_today_without_selecting() {
    let today = date(2026, 2, 10);
    let selected = date(2026, 2, 8);
    let mut harness = Harness::builder()
        .with_size(egui::vec2(252.0, 460.0))
        .build_ui_state(
            move |ui, state: &mut CalendarState| {
                Calendar::new(state, &Theme::light().touch())
                    .today(today)
                    .year_navigation(true)
                    .show(ui);
            },
            CalendarState::new(today, Some(selected)),
        );
    harness.run();
    let grid = harness
        .get_by_role_and_label(egui::accesskit::Role::Grid, "Calendar")
        .rect();
    let next = harness.get_by_label("Next month");
    assert!(next.rect().right() <= 252.0);
    next.click();
    harness.run();
    assert_eq!(harness.state().visible_month(), date(2026, 3, 1));
    assert_eq!(
        harness
            .get_by_role_and_label(egui::accesskit::Role::Grid, "Calendar")
            .rect()
            .height(),
        grid.height(),
    );

    harness.get_by_label("Choose month").click();
    harness.run();
    harness.get_by_label("Choose year").click();
    harness.run();
    harness.get_by_label("Back").click();
    harness.run();
    harness.get_by_label("Months");
    harness.get_by_label("Back").click();
    harness.run();

    harness.get_by_label("Today").click();
    harness.run();
    assert_eq!(harness.state().visible_month(), today.first_of_month());
    assert_eq!(harness.state().selected(), Some(selected));
    assert_eq!(harness.state().focused(), Some(today));
}
