use egui_kittest::{
    Harness,
    kittest::{NodeT, Queryable},
};
use temivu::{
    Date, DateField, DateFieldLabels, DateFieldState, DateFormatter, DateParseError, DateParser,
    Messages, Theme, format_iso_date, parse_iso_date,
};

struct State {
    field: DateFieldState,
    changed: bool,
    error: Option<String>,
    open: bool,
}

#[derive(Clone, Copy)]
struct Options {
    min: Option<Date>,
    max: Option<Date>,
    today: Option<Date>,
    initial_month: Option<Date>,
    messages: Messages,
    enabled: bool,
    parser: &'static DateParser<'static>,
    formatter: &'static DateFormatter<'static>,
}

impl Options {
    fn english() -> Self {
        Self {
            min: None,
            max: None,
            today: None,
            initial_month: None,
            messages: Messages::for_locale("en"),
            enabled: true,
            parser: &parse_iso_date,
            formatter: &format_iso_date,
        }
    }
}

fn date(year: i32, month: u8, day: u8) -> Date {
    Date::new(year, month, day).expect("test date is valid")
}

fn build(options: Options) -> Harness<'static, State> {
    Harness::builder()
        .with_size(egui::vec2(460.0, 420.0))
        .build_ui_state(
            move |ui, state: &mut State| {
                let theme = Theme::light();
                let mut field = DateField::new(&mut state.field, &theme, "Due date")
                    .messages(options.messages)
                    .parser(options.parser)
                    .formatter(options.formatter)
                    .enabled(options.enabled);
                if let (Some(min), Some(max)) = (options.min, options.max) {
                    field = field.bounds(min, max);
                }
                if let Some(today) = options.today {
                    field = field.today(today);
                }
                if let Some(initial_month) = options.initial_month {
                    field = field.initial_month(initial_month);
                }
                let output = field.show(ui);
                state.changed |= output.changed;
                state.error = output.error;
                state.open = output.open;
            },
            State {
                field: DateFieldState::new(),
                changed: false,
                error: None,
                open: false,
            },
        )
}

fn text_edit<'a>(harness: &'a Harness<'static, State>) -> egui_kittest::Node<'a> {
    harness.get_by_role_and_label(egui::accesskit::Role::TextInput, "Due date")
}

#[test]
fn typing_an_iso_date_updates_the_caller_value() {
    let mut harness = build(Options::english());
    harness.run();
    text_edit(&harness).click();
    harness.run();

    text_edit(&harness).type_text("2024-07-15");
    harness.run();

    assert_eq!(harness.state().field.text(), "2024-07-15");
    assert_eq!(harness.state().field.value(), Some(date(2024, 7, 15)));
    assert_eq!(harness.state().error, None);
    assert!(harness.state().changed);
    assert_eq!(text_edit(&harness).value().as_deref(), Some("2024-07-15"));
}

#[test]
fn invalid_text_shows_a_localized_error_and_clears_the_value() {
    let mut harness = build(Options {
        messages: Messages::for_locale("de"),
        ..Options::english()
    });
    harness.run();
    text_edit(&harness).click();
    harness.run();
    text_edit(&harness).type_text("2023-02-29");
    harness.run();

    assert_eq!(harness.state().field.value(), None);
    assert_eq!(harness.state().error.as_deref(), Some("Ungültiges Datum."));
    let node = text_edit(&harness);
    assert_eq!(
        node.accesskit_node().data().invalid(),
        Some(egui::accesskit::Invalid::True)
    );
    assert_eq!(
        node.accesskit_node().description().as_deref(),
        Some("Ungültiges Datum.")
    );
}

#[test]
fn bounds_reject_dates_outside_the_range() {
    let mut harness = build(Options {
        min: Some(date(2024, 7, 1)),
        max: Some(date(2024, 7, 31)),
        today: Some(date(2024, 7, 15)),
        initial_month: Some(date(2024, 7, 15)),
        ..Options::english()
    });
    harness.run();
    text_edit(&harness).click();
    harness.run();
    text_edit(&harness).type_text("2024-08-01");
    harness.run();

    assert_eq!(harness.state().field.value(), None);
    assert_eq!(
        harness.state().error.as_deref(),
        Some("Date is outside the allowed range.")
    );

    harness.state_mut().field.set_text("2024-07-31");
    harness.run();
    assert_eq!(harness.state().error, None);
    assert_eq!(harness.state().field.value(), Some(date(2024, 7, 31)));
}

#[test]
fn calendar_button_opens_a_popover_and_selection_writes_the_text() {
    let mut harness = build(Options {
        today: Some(date(2024, 7, 15)),
        initial_month: Some(date(2024, 7, 15)),
        ..Options::english()
    });
    harness.run();

    harness
        .get_by_role_and_label(egui::accesskit::Role::Button, "Choose date")
        .click();
    harness.run();
    assert!(harness.state().open);
    harness.get_by_role_and_label(egui::accesskit::Role::Group, "Choose date");
    harness.get_by_role_and_label(egui::accesskit::Role::Grid, "Calendar");
    harness.get_by_role_and_label(
        egui::accesskit::Role::GridCell,
        "Monday, July 15, 2024, Today",
    );

    harness
        .get_by_role_and_label(egui::accesskit::Role::GridCell, "Saturday, July 20, 2024")
        .click();
    harness.run();

    assert!(!harness.state().open);
    assert_eq!(harness.state().field.value(), Some(date(2024, 7, 20)));
    assert_eq!(harness.state().field.text(), "2024-07-20");
    assert_eq!(text_edit(&harness).value().as_deref(), Some("2024-07-20"));
}

#[test]
fn escape_closes_the_popover() {
    let mut harness = build(Options {
        today: Some(date(2024, 7, 15)),
        ..Options::english()
    });
    harness.run();
    harness
        .get_by_role_and_label(egui::accesskit::Role::Button, "Choose date")
        .click();
    harness.run();
    assert!(harness.state().open);

    harness.key_press(egui::Key::Escape);
    harness.run();
    assert!(!harness.state().open);
    assert!(
        harness.query_by_role(egui::accesskit::Role::Grid).is_none(),
        "the calendar should be dismissed"
    );
}

#[test]
fn custom_parser_and_formatter_are_used() {
    fn parse_dotted(text: &str) -> Result<Date, DateParseError> {
        let mut parts = text.trim().split('.');
        let day = parts.next().ok_or(DateParseError::Format)?;
        let month = parts.next().ok_or(DateParseError::Format)?;
        let year = parts.next().ok_or(DateParseError::Format)?;
        if parts.next().is_some() {
            return Err(DateParseError::Format);
        }
        let day = day.parse::<u8>().map_err(|_| DateParseError::Format)?;
        let month = month.parse::<u8>().map_err(|_| DateParseError::Format)?;
        let year = year.parse::<i32>().map_err(|_| DateParseError::Format)?;
        Date::new(year, month, day).map_err(|_| DateParseError::InvalidDate)
    }
    fn format_dotted(date: Date) -> String {
        format!("{:02}.{:02}.{:04}", date.day(), date.month(), date.year())
    }

    let mut harness = build(Options {
        today: Some(date(2024, 7, 15)),
        initial_month: Some(date(2024, 7, 15)),
        parser: &parse_dotted,
        formatter: &format_dotted,
        ..Options::english()
    });
    harness.run();
    text_edit(&harness).click();
    harness.run();
    text_edit(&harness).type_text("05.07.2024");
    harness.run();
    assert_eq!(harness.state().field.value(), Some(date(2024, 7, 5)));

    harness
        .get_by_role_and_label(egui::accesskit::Role::Button, "Choose date")
        .click();
    harness.run();
    harness
        .get_by_role_and_label(egui::accesskit::Role::GridCell, "Saturday, July 20, 2024")
        .click();
    harness.run();
    assert_eq!(harness.state().field.text(), "20.07.2024");
    assert_eq!(harness.state().field.value(), Some(date(2024, 7, 20)));
}

#[test]
fn empty_text_is_valid_and_clears_the_value() {
    let mut harness = build(Options::english());
    harness.run();
    assert_eq!(harness.state().field.value(), None);
    assert_eq!(harness.state().error, None);
    let node = text_edit(&harness);
    assert_eq!(node.accesskit_node().data().invalid(), None);
    assert_eq!(node.value().as_deref(), Some(""));
}

#[test]
fn disabled_field_does_not_open_the_calendar() {
    let mut harness = build(Options {
        enabled: false,
        today: Some(date(2024, 7, 15)),
        ..Options::english()
    });
    harness.run();
    let button = harness.get_by_role_and_label(egui::accesskit::Role::Button, "Choose date");
    assert!(button.accesskit_node().is_disabled());
    button.click();
    harness.run();
    assert!(!harness.state().open);
    assert!(text_edit(&harness).accesskit_node().is_disabled());
}

#[test]
fn custom_labels_are_used() {
    let labels = DateFieldLabels {
        calendar_button: "Open calendar",
        out_of_range: "Outside range",
        ..DateFieldLabels::english()
    };
    let mut harness = Harness::builder()
        .with_size(egui::vec2(460.0, 200.0))
        .build_ui_state(
            move |ui, field: &mut DateFieldState| {
                let theme = Theme::light();
                DateField::new(field, &theme, "Due date")
                    .labels(labels)
                    .show(ui);
            },
            DateFieldState::new(),
        );
    harness.run();
    harness.get_by_role_and_label(egui::accesskit::Role::Button, "Open calendar");
}
