//! Preliminary port of Forui's grid calendar: a single-month day grid with
//! month/year navigation, caller-owned state, date bounds, a disabled-date
//! predicate and keyboard grid navigation.
//!
//! Swipe paging, contiguous-range selection,
//! localization of date formats and upstream visual parity remain open. See
//! `docs/CALENDAR.md`.
//!
//! The calendar never reads the wall clock. Pass [`Calendar::today`] so tests
//! and applications decide what "today" means.
//!
//! ```
//! # fn example(ui: &mut egui::Ui) {
//! use temivu::{Calendar, CalendarState, Date, Theme};
//!
//! let theme = Theme::light();
//! let today = Date::new(2024, 7, 15).expect("a valid date");
//! let mut state = CalendarState::new(today, None);
//!
//! let output = Calendar::new(&mut state, &theme).today(today).show(ui);
//! if output.activated.is_some() {
//!     let _selected = state.selected();
//! }
//! # }
//! ```

use crate::Theme;
use crate::tokens::Sizes;
use egui::{Id, Rect, Response, Sense, Stroke, StrokeKind, Ui, Vec2};

const MIN_YEAR: i32 = 1;
const MAX_YEAR: i32 = 9999;
const DAYS_PER_WEEK: usize = 7;
const WEEKS_PER_VIEW: usize = 6;

/// A validated proleptic Gregorian date without a time or time zone.
///
/// Years are in `1..=9999`. Construction rejects invalid months, days and
/// years, so a `Date` value is always representable.
///
/// ```
/// use temivu::Date;
///
/// let date = Date::new(2024, 2, 29).expect("2024 is a leap year");
/// assert_eq!(date.weekday(), temivu::Weekday::Thursday);
/// assert_eq!(date.add_days(1).map(|next| next.day()), Some(1));
/// assert!(Date::new(2023, 2, 29).is_err());
/// ```
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Date {
    year: i32,
    month: u8,
    day: u8,
}

/// Why a date could not be constructed.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum DateError {
    /// The year is outside `1..=9999`.
    YearOutOfRange(i32),
    /// The month is outside `1..=12`.
    InvalidMonth(u8),
    /// The day is outside `1..=days_in_month(year, month)`.
    InvalidDay { year: i32, month: u8, day: u8 },
}

impl std::fmt::Display for DateError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::YearOutOfRange(year) => {
                write!(f, "year {year} is outside {MIN_YEAR}..={MAX_YEAR}")
            }
            Self::InvalidMonth(month) => write!(f, "month {month} is outside 1..=12"),
            Self::InvalidDay { year, month, day } => {
                write!(f, "day {day} is invalid for {year}-{month}")
            }
        }
    }
}

impl std::error::Error for DateError {}

impl Date {
    /// The earliest representable date.
    pub const MIN: Self = Self {
        year: MIN_YEAR,
        month: 1,
        day: 1,
    };

    /// The latest representable date.
    pub const MAX: Self = Self {
        year: MAX_YEAR,
        month: 12,
        day: 31,
    };

    /// Construct a date, validating the year, month and day.
    pub fn new(year: i32, month: u8, day: u8) -> Result<Self, DateError> {
        if !(MIN_YEAR..=MAX_YEAR).contains(&year) {
            return Err(DateError::YearOutOfRange(year));
        }
        let days = Self::days_in_month(year, month)?;
        if day == 0 || day > days {
            return Err(DateError::InvalidDay { year, month, day });
        }
        Ok(Self { year, month, day })
    }

    /// The date's year.
    pub fn year(self) -> i32 {
        self.year
    }

    /// The date's month, in `1..=12`.
    pub fn month(self) -> u8 {
        self.month
    }

    /// The date's day of month, in `1..=days_in_month(year, month)`.
    pub fn day(self) -> u8 {
        self.day
    }

    /// Whether `year` is a leap year in the proleptic Gregorian calendar.
    pub const fn is_leap_year(year: i32) -> bool {
        (year % 4 == 0 && year % 100 != 0) || year % 400 == 0
    }

    /// The number of days in a month, or an error for a month outside `1..=12`.
    pub fn days_in_month(year: i32, month: u8) -> Result<u8, DateError> {
        Ok(match month {
            1 | 3 | 5 | 7 | 8 | 10 | 12 => 31,
            4 | 6 | 9 | 11 => 30,
            2 if Self::is_leap_year(year) => 29,
            2 => 28,
            other => return Err(DateError::InvalidMonth(other)),
        })
    }

    /// The day of the week, with Monday as the start of the week.
    pub fn weekday(self) -> Weekday {
        let index = (self.days_since_epoch() + 3).rem_euclid(7) as usize;
        Weekday::from_monday_index(index).expect("remainder of seven is a weekday")
    }

    /// This date with the given day of the same month.
    pub fn with_day(self, day: u8) -> Result<Self, DateError> {
        Self::new(self.year, self.month, day)
    }

    /// The first day of the same month.
    pub fn first_of_month(self) -> Self {
        Self {
            year: self.year,
            month: self.month,
            day: 1,
        }
    }

    /// The last day of the same month.
    pub fn last_of_month(self) -> Self {
        let day =
            Self::days_in_month(self.year, self.month).expect("a Date always has a valid month");
        Self {
            year: self.year,
            month: self.month,
            day,
        }
    }

    /// This date shifted by `days`, or `None` when it leaves the supported range.
    pub fn add_days(self, days: i64) -> Option<Self> {
        let shifted = self.days_since_epoch().checked_add(days)?;
        if !(days_from_civil(MIN_YEAR, 1, 1)..=days_from_civil(MAX_YEAR, 12, 31)).contains(&shifted)
        {
            return None;
        }
        let (year, month, day) = civil_from_days(shifted);
        Some(Self { year, month, day })
    }

    /// The following day, or `None` after [`Date::MAX`].
    pub fn next_day(self) -> Option<Self> {
        self.add_days(1)
    }

    /// The previous day, or `None` before [`Date::MIN`].
    pub fn previous_day(self) -> Option<Self> {
        self.add_days(-1)
    }

    /// This date shifted by whole months, clamping the day to the target
    /// month's length (`2024-01-31 + 1 month` is `2024-02-29`).
    pub fn add_months(self, months: i64) -> Option<Self> {
        let total = i64::from(self.year)
            .checked_mul(12)?
            .checked_add(i64::from(self.month) - 1)?
            .checked_add(months)?;
        let year = total.div_euclid(12);
        if !(i64::from(MIN_YEAR)..=i64::from(MAX_YEAR)).contains(&year) {
            return None;
        }
        let year = i32::try_from(year).ok()?;
        let month = u8::try_from(total.rem_euclid(12) + 1).ok()?;
        let day = self.day.min(Self::days_in_month(year, month).ok()?);
        Some(Self { year, month, day })
    }

    /// Whole days from `self` to `other`; negative when `other` is earlier.
    pub fn days_until(self, other: Self) -> i64 {
        other.days_since_epoch() - self.days_since_epoch()
    }

    /// Clamp this date into the inclusive `min..=max` range.
    ///
    /// When `min` is after `max` the range is empty and `min` is returned.
    pub fn clamp(self, min: Self, max: Self) -> Self {
        if min > max || self < min {
            min
        } else if self > max {
            max
        } else {
            self
        }
    }

    /// The English name of the date's month.
    pub fn month_name(self) -> &'static str {
        month_name(self.month)
    }

    /// The English name of the date's weekday.
    pub fn weekday_name(self) -> &'static str {
        self.weekday().name()
    }

    fn days_since_epoch(self) -> i64 {
        days_from_civil(self.year, self.month, self.day)
    }
}

impl std::fmt::Display for Date {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:04}-{:02}-{:02}", self.year, self.month, self.day)
    }
}

/// A day of the week, with Monday as index zero.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum Weekday {
    Monday,
    Tuesday,
    Wednesday,
    Thursday,
    Friday,
    Saturday,
    Sunday,
}

impl Weekday {
    /// Every weekday in Monday-through-Sunday order.
    pub const ALL: [Self; DAYS_PER_WEEK] = [
        Self::Monday,
        Self::Tuesday,
        Self::Wednesday,
        Self::Thursday,
        Self::Friday,
        Self::Saturday,
        Self::Sunday,
    ];

    /// The weekday for a Monday-based index, or `None` outside `0..=6`.
    pub fn from_monday_index(index: usize) -> Option<Self> {
        Self::ALL.get(index).copied()
    }

    /// This weekday's Monday-based index.
    pub fn index(self) -> usize {
        match self {
            Self::Monday => 0,
            Self::Tuesday => 1,
            Self::Wednesday => 2,
            Self::Thursday => 3,
            Self::Friday => 4,
            Self::Saturday => 5,
            Self::Sunday => 6,
        }
    }

    /// The English name, for example `"Monday"`.
    pub fn name(self) -> &'static str {
        match self {
            Self::Monday => "Monday",
            Self::Tuesday => "Tuesday",
            Self::Wednesday => "Wednesday",
            Self::Thursday => "Thursday",
            Self::Friday => "Friday",
            Self::Saturday => "Saturday",
            Self::Sunday => "Sunday",
        }
    }

    /// The English short name, for example `"Mon"`.
    pub fn short_name(self) -> &'static str {
        match self {
            Self::Monday => "Mon",
            Self::Tuesday => "Tue",
            Self::Wednesday => "Wed",
            Self::Thursday => "Thu",
            Self::Friday => "Fri",
            Self::Saturday => "Sat",
            Self::Sunday => "Sun",
        }
    }

    /// The English narrow name used in weekday headers, for example `"M"`.
    pub fn narrow_name(self) -> &'static str {
        match self {
            Self::Monday => "M",
            Self::Tuesday => "T",
            Self::Wednesday => "W",
            Self::Thursday => "T",
            Self::Friday => "F",
            Self::Saturday => "S",
            Self::Sunday => "S",
        }
    }
}

/// English month name for `month`, or `""` for an invalid month.
pub fn month_name(month: u8) -> &'static str {
    const NAMES: [&str; 12] = [
        "January",
        "February",
        "March",
        "April",
        "May",
        "June",
        "July",
        "August",
        "September",
        "October",
        "November",
        "December",
    ];
    usize::from(month)
        .checked_sub(1)
        .and_then(|index| NAMES.get(index))
        .copied()
        .unwrap_or("")
}

// Howard Hinnant's civil-date algorithms, which are exact for all Gregorian
// dates and avoid any dependency on a date crate.
const fn days_from_civil(year: i32, month: u8, day: u8) -> i64 {
    let year = if month <= 2 {
        year as i64 - 1
    } else {
        year as i64
    };
    let era = if year >= 0 { year } else { year - 399 } / 400;
    let year_of_era = year - era * 400; // [0, 399]
    let month_prime = ((month as i64) + 9) % 12; // March = 0
    let day_of_year = (153 * month_prime + 2) / 5 + day as i64 - 1; // [0, 365]
    let day_of_era = year_of_era * 365 + year_of_era / 4 - year_of_era / 100 + day_of_year;
    era * 146_097 + day_of_era - 719_468
}

const fn civil_from_days(days: i64) -> (i32, u8, u8) {
    let days = days + 719_468;
    let era = if days >= 0 { days } else { days - 146_096 } / 146_097;
    let day_of_era = days - era * 146_097; // [0, 146096]
    let year_of_era =
        (day_of_era - day_of_era / 1460 + day_of_era / 36_524 - day_of_era / 146_096) / 365;
    let year = year_of_era + era * 400;
    let day_of_year = day_of_era - (365 * year_of_era + year_of_era / 4 - year_of_era / 100);
    let month_prime = (5 * day_of_year + 2) / 153;
    let day = day_of_year - (153 * month_prime + 2) / 5 + 1;
    let month = if month_prime < 10 {
        month_prime + 3
    } else {
        month_prime - 9
    };
    let year = year + if month <= 2 { 1 } else { 0 };
    (year as i32, month as u8, day as u8)
}

/// Caller-owned calendar state: the visible month, the selected date and the
/// keyboard-focused date.
///
/// The visible month is always normalized to the first day of the month.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct CalendarState {
    visible_month: Date,
    selected: Option<Date>,
    focused: Option<Date>,
}

impl CalendarState {
    /// Create state showing `visible_month`'s month with `selected` selected.
    pub fn new(visible_month: Date, selected: Option<Date>) -> Self {
        Self {
            visible_month: visible_month.first_of_month(),
            selected,
            focused: None,
        }
    }

    /// The month currently shown, as its first day.
    pub fn visible_month(&self) -> Date {
        self.visible_month
    }

    /// The selected date, if any.
    pub fn selected(&self) -> Option<Date> {
        self.selected
    }

    /// The date that has keyboard focus, if any.
    pub fn focused(&self) -> Option<Date> {
        self.focused
    }

    /// Show the month containing `month`; returns whether the month changed.
    pub fn set_visible_month(&mut self, month: Date) -> bool {
        let month = month.first_of_month();
        let changed = self.visible_month != month;
        self.visible_month = month;
        changed
    }

    /// Replace the selected date; returns whether it changed.
    pub fn set_selected(&mut self, selected: Option<Date>) -> bool {
        let changed = self.selected != selected;
        self.selected = selected;
        changed
    }

    /// Replace the keyboard-focused date; returns whether it changed.
    pub fn set_focused(&mut self, focused: Option<Date>) -> bool {
        let changed = self.focused != focused;
        self.focused = focused;
        changed
    }
}

/// English defaults for the calendar's accessible and visible labels.
///
/// Month and weekday names, the month heading and the per-day accessible label
/// are English-only in this preliminary port; override the function pointers to
/// localize them. `first_day_of_week` is configured on [`Calendar`].
#[derive(Clone, Copy)]
pub struct CalendarLabels<'a> {
    /// Accessible name of the previous-month button.
    pub previous_month: &'a str,
    /// Accessible name of the next-month button.
    pub next_month: &'a str,
    /// Accessible name of the previous-year button.
    pub previous_year: &'a str,
    /// Accessible name of the next-year button.
    pub next_year: &'a str,
    /// Accessible name of the day grid.
    pub grid: &'a str,
    /// Text appended to a day's accessible label when it is today, e.g. `"Today"`.
    pub today: &'a str,
    /// Builds the month heading, for example `"July 2024"`.
    pub month_heading: &'a dyn Fn(i32, u8) -> String,
    /// Builds a day's accessible label, for example `"Monday, July 15, 2024"`.
    pub day_label: &'a dyn Fn(Date) -> String,
    /// Narrow column headers indexed Monday through Sunday.
    pub weekday_headers: [&'a str; DAYS_PER_WEEK],
}

impl<'a> Default for CalendarLabels<'a> {
    fn default() -> Self {
        CalendarLabels::english()
    }
}

impl CalendarLabels<'static> {
    /// The English label set used when the caller supplies none.
    pub fn english() -> Self {
        Self {
            previous_month: "Previous month",
            next_month: "Next month",
            previous_year: "Previous year",
            next_year: "Next year",
            grid: "Calendar",
            today: "Today",
            month_heading: &english_month_heading,
            day_label: &english_day_label,
            weekday_headers: Weekday::ALL.map(Weekday::narrow_name),
        }
    }
}

/// English defaults for the month/year picker that the calendar header opens.
///
/// Override these to localize the picker controls, month names, year heading
/// and grid labels; pass them with [`Calendar::picker_labels`].
#[derive(Clone, Copy)]
pub struct CalendarPickerLabels<'a> {
    /// Accessible name of the button that opens the month picker.
    pub choose_month: &'a str,
    /// Accessible name of the button that opens the year picker.
    pub choose_year: &'a str,
    /// Accessible name of the previous year button in the month picker.
    pub previous_year: &'a str,
    /// Accessible name of the next year button in the month picker.
    pub next_year: &'a str,
    /// Accessible name of the previous-years page button in the year picker.
    pub previous_years: &'a str,
    /// Accessible name of the next-years page button in the year picker.
    pub next_years: &'a str,
    /// Accessible name of the month grid.
    pub months_grid: &'a str,
    /// Accessible name of the year grid.
    pub years_grid: &'a str,
    /// January through December names.
    pub month_names: [&'a str; 12],
    /// Formats a single year, for example `"2024"`.
    pub year: &'a dyn Fn(i32) -> String,
    /// Formats the inclusive year-page range, for example `"2017 – 2028"`.
    pub years_heading: &'a dyn Fn(i32, i32) -> String,
}

impl<'a> Default for CalendarPickerLabels<'a> {
    fn default() -> Self {
        let english = english_picker_labels();
        Self {
            choose_month: english.choose_month,
            choose_year: english.choose_year,
            previous_year: english.previous_year,
            next_year: english.next_year,
            previous_years: english.previous_years,
            next_years: english.next_years,
            months_grid: english.months_grid,
            years_grid: english.years_grid,
            month_names: english.month_names,
            year: english.year,
            years_heading: english.years_heading,
        }
    }
}

fn english_picker_labels() -> CalendarPickerLabels<'static> {
    CalendarPickerLabels {
        choose_month: "Choose month",
        choose_year: "Choose year",
        previous_year: "Previous year",
        next_year: "Next year",
        previous_years: "Previous years",
        next_years: "Next years",
        months_grid: "Months",
        years_grid: "Years",
        month_names: [
            "January",
            "February",
            "March",
            "April",
            "May",
            "June",
            "July",
            "August",
            "September",
            "October",
            "November",
            "December",
        ],
        year: &english_year,
        years_heading: &english_years_heading,
    }
}

impl CalendarPickerLabels<'static> {
    /// The English picker labels used when the caller supplies none.
    pub fn english() -> Self {
        english_picker_labels()
    }
}

fn english_year(year: i32) -> String {
    format!("{year}")
}

fn english_years_heading(first: i32, last: i32) -> String {
    format!("{first} – {last}")
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
enum PickerMode {
    #[default]
    Days,
    Months,
    Years,
}

fn english_month_heading(year: i32, month: u8) -> String {
    format!("{} {year}", month_name(month))
}

fn english_day_label(date: Date) -> String {
    format!(
        "{}, {} {}, {}",
        date.weekday().name(),
        date.month_name(),
        date.day(),
        date.year()
    )
}

/// Builder options shared by the calendar's pure navigation helpers.
struct Config<'a> {
    sizes: Sizes,
    min_date: Date,
    max_date: Date,
    selectable: Option<&'a dyn Fn(Date) -> bool>,
    first_day_of_week: Weekday,
    today: Option<Date>,
    toggleable: bool,
    enabled: bool,
    labels: CalendarLabels<'a>,
    picker: CalendarPickerLabels<'a>,
}

impl Config<'_> {
    fn day_selectable(&self, date: Date) -> bool {
        self.enabled
            && date >= self.min_date
            && date <= self.max_date
            && self.selectable.is_none_or(|selectable| selectable(date))
    }

    fn first_selectable_in_month(&self, month: Date) -> Option<Date> {
        let days = Date::days_in_month(month.year(), month.month()).ok()?;
        (1..=days)
            .map(|day| Date::new(month.year(), month.month(), day))
            .find_map(|date| date.ok().filter(|date| self.day_selectable(*date)))
    }

    fn month_in_bounds(&self, month: Date) -> bool {
        let month = month.first_of_month();
        month >= self.min_date.first_of_month() && month <= self.max_date.first_of_month()
    }

    fn shifted_month(&self, month: Date, delta: i64) -> Option<Date> {
        month
            .add_months(delta)
            .filter(|target| self.month_in_bounds(*target))
    }

    // Port of Forui's day-picker `focusable` preference: keep the day of month
    // when it is selectable, otherwise use the first selectable day.
    fn focus_candidate(&self, month: Date, preferred_day: u8) -> Option<Date> {
        let last = Date::days_in_month(month.year(), month.month()).ok()?;
        let day = preferred_day.clamp(1, last);
        let preferred = Date::new(month.year(), month.month(), day).ok();
        preferred
            .filter(|date| self.day_selectable(*date))
            .or_else(|| self.first_selectable_in_month(month))
    }

    fn default_focus_date(&self, month: Date) -> Option<Date> {
        let preferred = self
            .today
            .filter(|today| today.first_of_month() == month)
            .unwrap_or(month);
        self.focus_candidate(month, preferred.day())
    }

    /// Move focus by `step` days, skipping unselectable dates and paging the
    /// visible month with the focused date. Searches at most two years.
    fn move_focus(&self, state: &mut CalendarState, step: i64) {
        let start = match state.focused() {
            Some(focused) => focused,
            None => {
                if let Some(date) = self.default_focus_date(state.visible_month()) {
                    state.set_focused(Some(date));
                }
                return;
            }
        };
        let limit = if step.abs() <= 1 { 366 * 2 } else { 60 };
        for amount in 1..=limit {
            let Some(candidate) = start.add_days(step * amount) else {
                return;
            };
            if candidate < self.min_date || candidate > self.max_date {
                return;
            }
            if self.day_selectable(candidate) {
                state.set_focused(Some(candidate));
                state.set_visible_month(candidate);
                return;
            }
        }
    }

    /// Move focus to the first (`end == false`) or last selectable day of the
    /// focused week, clamped to the visible month like upstream.
    fn move_focus_to_week_edge(&self, state: &mut CalendarState, end: bool) {
        let Some(focused) = state.focused() else {
            return;
        };
        let offset = (focused.weekday().index() + DAYS_PER_WEEK - self.first_day_of_week.index())
            % DAYS_PER_WEEK;
        let month = state.visible_month();
        let first = month.first_of_month().max(self.min_date);
        let last = month.last_of_month().min(self.max_date);
        let week_start = focused
            .add_days(-(offset as i64))
            .unwrap_or(first)
            .max(first);
        let week_end = focused
            .add_days((DAYS_PER_WEEK - 1 - offset) as i64)
            .unwrap_or(last)
            .min(last);
        let candidate = if end {
            (0..=week_end.days_until(week_start).abs())
                .filter_map(|days| week_end.add_days(-days))
                .find(|date| self.day_selectable(*date))
        } else {
            (0..=week_start.days_until(week_end))
                .filter_map(|days| week_start.add_days(days))
                .find(|date| self.day_selectable(*date))
        };
        if candidate.is_some() {
            state.set_focused(candidate);
        }
    }

    /// Page the visible month by `delta` months, keeping the focused day when
    /// possible.
    fn page_month(&self, state: &mut CalendarState, delta: i64) {
        let Some(target) = self.shifted_month(state.visible_month(), delta) else {
            return;
        };
        let preferred = state.focused().map_or(1, Date::day);
        state.set_visible_month(target);
        state.set_focused(self.focus_candidate(target, preferred));
    }

    /// Select the focused date; returns it when selection is possible.
    fn select_focused(&self, state: &mut CalendarState) -> Option<Date> {
        let focused = state.focused()?;
        if !self.day_selectable(focused) {
            return None;
        }
        if self.toggleable && state.selected() == Some(focused) {
            state.set_selected(None);
        } else {
            state.set_selected(Some(focused));
        }
        Some(focused)
    }
}

/// What [`Calendar::show`] reports for the current frame.
pub struct CalendarResponse {
    /// The day grid's response; it owns keyboard focus for grid navigation.
    pub response: Response,
    /// The day selected this frame by click, Enter or Space.
    pub activated: Option<Date>,
    /// Whether the visible month, the selection or the focused date changed.
    pub changed: bool,
}

/// Colors and corner radius for day cells.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct CalendarStyle {
    pub selected_fill: egui::Color32,
    pub selected_text: egui::Color32,
    pub hover_fill: egui::Color32,
    pub focus: egui::Color32,
    pub radius: u8,
}

impl CalendarStyle {
    pub fn from_theme(theme: &Theme) -> Self {
        Self {
            selected_fill: theme.primary,
            selected_text: theme.primary_foreground,
            hover_fill: theme.secondary,
            focus: theme.foreground,
            radius: 255,
        }
    }
}

/// A single-month calendar grid bound to caller-owned [`CalendarState`].
pub struct Calendar<'a> {
    state: &'a mut CalendarState,
    theme: &'a Theme,
    config: Config<'a>,
    id: Option<Id>,
    style: CalendarStyle,
    year_navigation: bool,
    picker_back_label: &'a str,
}

impl<'a> Calendar<'a> {
    /// The default inclusive date bounds: 1900-01-01 through 2100-12-31.
    pub fn default_min_date() -> Date {
        Date::new(1900, 1, 1).expect("the default minimum date is valid")
    }

    /// The default upper inclusive date bound.
    pub fn default_max_date() -> Date {
        Date::new(2100, 12, 31).expect("the default maximum date is valid")
    }

    /// Create a calendar for `state` using `theme`'s colors.
    pub fn new(state: &'a mut CalendarState, theme: &'a Theme) -> Self {
        Self {
            state,
            theme,
            config: Config {
                sizes: theme.sizes,
                min_date: Self::default_min_date(),
                max_date: Self::default_max_date(),
                selectable: None,
                first_day_of_week: Weekday::Monday,
                today: None,
                toggleable: false,
                enabled: true,
                labels: CalendarLabels::english(),
                picker: CalendarPickerLabels::english(),
            },
            id: None,
            style: CalendarStyle::from_theme(theme),
            year_navigation: false,
            picker_back_label: "Back",
        }
    }

    /// Add direct previous/next year buttons. The compact default uses the
    /// month/year picker for year changes, keeping the day header uncluttered.
    pub fn year_navigation(mut self, visible: bool) -> Self {
        self.year_navigation = visible;
        self
    }

    /// Set the accessible label for returning from a month or year picker.
    pub fn picker_back_label(mut self, label: &'a str) -> Self {
        self.picker_back_label = label;
        self
    }

    /// Set the inclusive selectable date bounds. The argument order does not
    /// matter: the earlier date is the minimum.
    pub fn bounds(mut self, min: Date, max: Date) -> Self {
        self.config.min_date = min.min(max);
        self.config.max_date = max.max(min);
        self
    }

    /// Set a caller-supplied predicate that disables individual dates.
    pub fn selectable(mut self, selectable: &'a dyn Fn(Date) -> bool) -> Self {
        self.config.selectable = Some(selectable);
        self
    }

    /// Set the caller-supplied "today" used for the today highlight and as the
    /// initial keyboard focus. Library code never reads the wall clock.
    pub fn today(mut self, today: Date) -> Self {
        self.config.today = Some(today);
        self
    }

    /// Set the first day of the week. Defaults to Monday.
    pub fn first_day_of_week(mut self, weekday: Weekday) -> Self {
        self.config.first_day_of_week = weekday;
        self
    }

    /// Re-select the selected date to clear the selection when it is pressed
    /// again. Defaults to `false`.
    pub fn toggleable(mut self, toggleable: bool) -> Self {
        self.config.toggleable = toggleable;
        self
    }

    /// Disable the whole calendar.
    pub fn enabled(mut self, enabled: bool) -> Self {
        self.config.enabled = enabled;
        self
    }

    /// Set responsive size references. Day cells prefer at least 40 points,
    /// shrinking when the available width cannot fit seven columns.
    pub fn sizes(mut self, sizes: Sizes) -> Self {
        self.config.sizes = sizes;
        self
    }

    /// Override day-cell colors and corner radius.
    pub fn style(mut self, style: CalendarStyle) -> Self {
        self.style = style;
        self
    }

    /// Replace the English label defaults.
    pub fn labels(mut self, labels: CalendarLabels<'a>) -> Self {
        self.config.labels = labels;
        self
    }

    /// Replace the English month/year picker label defaults.
    pub fn picker_labels(mut self, picker: CalendarPickerLabels<'a>) -> Self {
        self.config.picker = picker;
        self
    }

    /// Give the calendar a stable, globally unique ID across layout changes.
    pub fn id(mut self, id: Id) -> Self {
        self.id = Some(id);
        self
    }

    /// Draw the calendar and return its response.
    pub fn show(self, ui: &mut Ui) -> CalendarResponse {
        let id = self
            .id
            .unwrap_or_else(|| ui.next_auto_id().with("temivu-calendar"));
        ui.scope_builder(egui::UiBuilder::new().scope_id(id), |ui| {
            ui.set_width(
                ui.available_width()
                    .min(self.config.sizes.calendar.max(40.0) * 7.0),
            );
            ui.spacing_mut().item_spacing.x = 4.0;
            ui.spacing_mut().button_padding = Vec2::splat(4.0);
            self.show_inner(ui, id)
        })
        .inner
    }

    fn show_inner(self, ui: &mut Ui, id: Id) -> CalendarResponse {
        let Self {
            state,
            theme,
            config,
            style,
            year_navigation,
            picker_back_label,
            ..
        } = self;

        let mut changed = false;
        let mut activated: Option<Date> = None;

        // Keep the visible month inside the bounds and the focused date inside
        // the visible month, so a caller or a resize can never strand them.
        let min_month = config.min_date.first_of_month();
        let max_month = config.max_date.first_of_month();
        if state.visible_month() < min_month {
            changed |= state.set_visible_month(min_month);
        } else if state.visible_month() > max_month {
            changed |= state.set_visible_month(max_month);
        }
        if let Some(focused) = state.focused()
            && focused.first_of_month() != state.visible_month()
        {
            let target = config.focus_candidate(state.visible_month(), focused.day());
            changed |= state.set_focused(target);
        }

        let labels = config.labels;
        let picker = config.picker;
        let grid_id = id.with("grid");
        let mode_key = id.with("temivu-calendar-picker-mode");
        let page_key = id.with("temivu-calendar-picker-page");
        let page_anchor_key = id.with("temivu-calendar-picker-page-anchor");
        let mode_pass_key = id.with("temivu-calendar-picker-pass");
        let focus_pending_key = id.with("temivu-calendar-picker-focus-pending");
        let restore_days_key = id.with("temivu-calendar-picker-restore-days");
        let mut mode = ui
            .ctx()
            .data(|data| data.get_temp::<PickerMode>(mode_key).unwrap_or_default());
        // A calendar that is removed and re-added (or whose id is reused) should
        // not silently reopen in the picker: a gap of more than a couple of
        // passes resets the ephemeral picker state.
        let current_pass = ui.ctx().cumulative_pass_nr();
        let last_pass = ui
            .ctx()
            .data(|data| data.get_temp::<u64>(mode_pass_key).unwrap_or(current_pass));
        if mode != PickerMode::Days && current_pass.saturating_sub(last_pass) > 2 {
            mode = PickerMode::Days;
            ui.ctx().data_mut(|data| {
                data.remove::<i32>(page_key);
                data.remove::<i32>(page_anchor_key);
            });
            ui.ctx().request_repaint();
        }
        ui.ctx()
            .data_mut(|data| data.insert_temp(mode_pass_key, current_pass));
        if mode != PickerMode::Days
            && !ui.ctx().any_popup_open()
            && ui.input_mut(|input| input.consume_key(egui::Modifiers::NONE, egui::Key::Escape))
        {
            mode = match mode {
                PickerMode::Years => PickerMode::Months,
                _ => PickerMode::Days,
            };
            ui.ctx().request_repaint();
        }
        let mut open_picker = false;
        if mode == PickerMode::Days {
            let heading_text =
                (labels.month_heading)(state.visible_month().year(), state.visible_month().month());
            let mut month_delta = 0i64;
            ui.horizontal(|ui| {
                let year_navigation = year_navigation && ui.available_width() >= 280.0;
                let buttons = if year_navigation { 5.0 } else { 3.0 };
                let heading_width =
                    (ui.available_width() - buttons * (navigation_size(theme) + 4.0)).max(1.0);
                let heading = ui.add_sized(
                    [heading_width, navigation_size(theme)],
                    egui::Label::new(
                        egui::RichText::new(&heading_text)
                            .size(theme.typography.heading.size)
                            .strong(),
                    )
                    .truncate()
                    .sense(if config.enabled {
                        Sense::click()
                    } else {
                        Sense::hover()
                    }),
                );
                ui.ctx().accesskit_node_builder(heading.id, |node| {
                    // Label-type widgets carry their text in the value slot, so
                    // promote the text to the heading's label as well.
                    node.set_role(egui::accesskit::Role::Heading);
                    node.set_level(2);
                    node.set_label(heading_text.clone());
                });
                open_picker =
                    navigation_button(ui, theme, "v", picker.choose_month, config.enabled)
                        || (config.enabled && heading.clicked());
                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    let visible = state.visible_month();
                    let can_next_year =
                        config.enabled && config.shifted_month(visible, 12).is_some();
                    let can_next_month =
                        config.enabled && config.shifted_month(visible, 1).is_some();
                    let can_previous_month =
                        config.enabled && config.shifted_month(visible, -1).is_some();
                    let can_previous_year =
                        config.enabled && config.shifted_month(visible, -12).is_some();
                    if year_navigation
                        && navigation_button(ui, theme, ">>", labels.next_year, can_next_year)
                    {
                        month_delta += 12;
                    }
                    if navigation_button(ui, theme, ">", labels.next_month, can_next_month) {
                        month_delta += 1;
                    }
                    if navigation_button(ui, theme, "<", labels.previous_month, can_previous_month)
                    {
                        month_delta -= 1;
                    }
                    if year_navigation
                        && navigation_button(
                            ui,
                            theme,
                            "<<",
                            labels.previous_year,
                            can_previous_year,
                        )
                    {
                        month_delta -= 12;
                    }
                });
            });
            if month_delta != 0
                && let Some(target) = config.shifted_month(state.visible_month(), month_delta)
            {
                changed |= state.set_visible_month(target);
                let preferred = state.focused().map_or(1, Date::day);
                changed |= state.set_focused(config.focus_candidate(target, preferred));
            }
        }
        if open_picker {
            mode = PickerMode::Months;
            ui.ctx()
                .data_mut(|data| data.insert_temp(focus_pending_key, true));
            ui.ctx().request_repaint();
        }
        ui.ctx().data_mut(|data| data.insert_temp(mode_key, mode));
        if mode != PickerMode::Days {
            let (picker_response, picker_changed) = show_calendar_picker(
                ui,
                id,
                theme,
                &config,
                state,
                mode,
                mode_key,
                page_key,
                page_anchor_key,
                focus_pending_key,
                restore_days_key,
                picker_back_label,
            );
            changed |= picker_changed;
            let mut response = picker_response;
            if changed {
                response.mark_changed();
            }
            return CalendarResponse {
                response,
                activated: None,
                changed,
            };
        }
        if ui
            .ctx()
            .data(|data| data.get_temp::<bool>(restore_days_key).unwrap_or(false))
        {
            ui.ctx()
                .data_mut(|data| data.insert_temp(restore_days_key, false));
            ui.memory_mut(|memory| memory.request_focus(grid_id));
        }

        // Keyboard navigation. The grid's focusable response is allocated below
        // but keeps a stable explicit ID, so the previous frame's focus can be
        // read here to settle the visible month before layout.
        let grid_focused = ui.memory(|memory| memory.has_focus(grid_id));
        if grid_focused {
            // Keep egui's spatial focus navigation from stealing the arrow keys
            // that drive the day grid.
            ui.memory_mut(|memory| {
                memory.set_focus_lock_filter(
                    grid_id,
                    egui::EventFilter {
                        horizontal_arrows: true,
                        vertical_arrows: true,
                        ..Default::default()
                    },
                );
            });
            if state.focused().is_none() {
                let target = config.default_focus_date(state.visible_month());
                changed |= state.set_focused(target);
            } else if state
                .focused()
                .is_some_and(|focused| focused.first_of_month() != state.visible_month())
            {
                let preferred = state.focused().map_or(1, Date::day);
                let target = config.focus_candidate(state.visible_month(), preferred);
                changed |= state.set_focused(target);
            }
            let (
                arrow_left,
                arrow_right,
                arrow_up,
                arrow_down,
                home,
                end,
                page_up,
                page_down,
                accept,
                shift,
            ) = ui.input(|input| {
                (
                    input.key_pressed(egui::Key::ArrowLeft),
                    input.key_pressed(egui::Key::ArrowRight),
                    input.key_pressed(egui::Key::ArrowUp),
                    input.key_pressed(egui::Key::ArrowDown),
                    input.key_pressed(egui::Key::Home),
                    input.key_pressed(egui::Key::End),
                    input.key_pressed(egui::Key::PageUp),
                    input.key_pressed(egui::Key::PageDown),
                    input.key_pressed(egui::Key::Enter) || input.key_pressed(egui::Key::Space),
                    input.modifiers.shift,
                )
            });
            let before = (state.visible_month(), state.focused(), state.selected());
            if arrow_left {
                config.move_focus(state, -1);
            } else if arrow_right {
                config.move_focus(state, 1);
            } else if arrow_up {
                config.move_focus(state, -7);
            } else if arrow_down {
                config.move_focus(state, 7);
            } else if home {
                config.move_focus_to_week_edge(state, false);
            } else if end {
                config.move_focus_to_week_edge(state, true);
            } else if page_up {
                config.page_month(state, if shift { -12 } else { -1 });
            } else if page_down {
                config.page_month(state, if shift { 12 } else { 1 });
            } else if accept && let Some(date) = config.select_focused(state) {
                activated = Some(date);
            }
            if before != (state.visible_month(), state.focused(), state.selected()) {
                changed = true;
                if activated.is_some() || state.selected() != before.2 {
                    // Selection feedback is painted next frame; request one so
                    // the change is visible even without further input.
                    ui.ctx().request_repaint();
                }
                if state.visible_month() != before.0 || state.focused() != before.1 {
                    ui.ctx().request_repaint();
                }
            }
        }

        // Grid geometry. The cell shrinks to fit the available width so the
        // grid stays usable in narrow containers.
        let available = ui.available_width();
        let cell = config
            .sizes
            .calendar
            .max(40.0)
            .min((available / DAYS_PER_WEEK as f32).max(1.0))
            .max(1.0);
        let header_height = (cell * 0.75).max(14.0);
        let spacing = 2.0;
        let visible = state.visible_month();
        let days = Date::days_in_month(visible.year(), visible.month())
            .expect("the visible month is valid");
        let offset = (visible.weekday().index() + DAYS_PER_WEEK - config.first_day_of_week.index())
            % DAYS_PER_WEEK;
        let rows = (offset + usize::from(days)).div_ceil(DAYS_PER_WEEK);
        let grid_width = cell * DAYS_PER_WEEK as f32;
        let grid_height = header_height
            + 4.0
            + WEEKS_PER_VIEW as f32 * cell
            + (WEEKS_PER_VIEW - 1) as f32 * spacing;
        let (grid_rect, _reserve) =
            ui.allocate_exact_size(Vec2::new(grid_width, grid_height), Sense::hover());
        let grid_sense = if config.enabled {
            Sense::click()
        } else {
            Sense::hover()
        };
        let mut response = ui.interact(grid_rect, grid_id, grid_sense);
        let focus_visible = grid_focused || response.has_focus();
        ui.ctx().accesskit_node_builder(grid_id, |node| {
            node.set_role(egui::accesskit::Role::Grid);
            node.set_label(labels.grid);
        });

        // The grid may gain keyboard focus this frame (for example from an
        // AccessKit focus request or Tab), which is only visible after
        // registration. Seed the focused day here so the ring and active
        // descendant render immediately, lock the arrow keys for the next
        // frame and request a follow-up frame so the lock takes effect before
        // the next key press.
        if response.has_focus() {
            ui.memory_mut(|memory| {
                memory.set_focus_lock_filter(
                    grid_id,
                    egui::EventFilter {
                        horizontal_arrows: true,
                        vertical_arrows: true,
                        ..Default::default()
                    },
                );
            });
            if response.gained_focus() {
                ui.ctx().request_repaint();
            }
            match state.focused() {
                None => {
                    changed |= state.set_focused(config.default_focus_date(state.visible_month()));
                }
                Some(focused) if focused.first_of_month() != state.visible_month() => {
                    let preferred = focused.day();
                    changed |=
                        state.set_focused(config.focus_candidate(state.visible_month(), preferred));
                }
                Some(_) => {}
            }
        }

        let weekday_height = header_height;
        let grid_start = visible.first_of_month().add_days(-(offset as i64));
        let focus_cell = state.focused();

        let month = visible.year() as f32 * 12.0 + f32::from(visible.month());
        let animated_month =
            crate::motion::spring_f32(ui.ctx(), id.with("month-slide"), month, month, theme);
        let travel = (animated_month - month).clamp(-1.0, 1.0);
        let mut clicked_date = None;
        ui.scope_builder(
            egui::UiBuilder::new()
                .scope_id(id.with("cells"))
                .accessibility_parent(grid_id),
            |ui| {
                let painter = ui.painter();
                // Weekday column headers.
                for column in 0..DAYS_PER_WEEK {
                    let weekday = Weekday::from_monday_index(
                        (config.first_day_of_week.index() + column) % DAYS_PER_WEEK,
                    )
                    .expect("a column is a weekday");
                    let rect = Rect::from_min_size(
                        egui::pos2(grid_rect.min.x + column as f32 * cell, grid_rect.min.y),
                        Vec2::new(cell, weekday_height),
                    );
                    let header_id = id.with(("weekday", column));
                    ui.interact(rect, header_id, Sense::hover());
                    ui.ctx().accesskit_node_builder(header_id, |node| {
                        node.set_role(egui::accesskit::Role::ColumnHeader);
                        node.set_label(labels.weekday_headers[weekday.index()]);
                        node.set_bounds(accesskit_rect(rect));
                    });
                    painter.text(
                        rect.center(),
                        egui::Align2::CENTER_CENTER,
                        labels.weekday_headers[weekday.index()],
                        egui::FontId::proportional(
                            theme
                                .typography
                                .caption
                                .size
                                .min((cell * 0.34).clamp(9.0, 14.0)),
                        ),
                        theme.muted_foreground,
                    );
                }

                let mut cells = painter.with_clip_rect(grid_rect.intersect(ui.clip_rect()));
                cells.multiply_opacity(1.0 - travel.abs() * 0.22);
                for row in 0..rows {
                    for column in 0..DAYS_PER_WEEK {
                        let index = row * DAYS_PER_WEEK + column;
                        let Some(date) = grid_start.and_then(|start| start.add_days(index as i64))
                        else {
                            continue;
                        };
                        let rect = Rect::from_min_size(
                            egui::pos2(
                                grid_rect.min.x + column as f32 * cell,
                                grid_rect.min.y
                                    + weekday_height
                                    + 4.0
                                    + row as f32 * (cell + spacing),
                            ),
                            Vec2::splat(cell),
                        );
                        let day_id = id.with(("day", date.year(), date.month(), date.day()));
                        let cell_response = ui.interact(rect, day_id, Sense::CLICK);
                        let selectable = config.day_selectable(date);
                        let selected = state.selected() == Some(date);
                        let today = config.today == Some(date);
                        let adjacent =
                            date.month() != visible.month() || date.year() != visible.year();
                        let focused = focus_cell == Some(date);
                        let selected_amount = crate::motion::spring_bool(
                            ui.ctx(),
                            day_id.with("selected-motion"),
                            selected,
                            theme,
                        );
                        ui.ctx().accesskit_node_builder(day_id, |node| {
                            node.set_role(egui::accesskit::Role::GridCell);
                            node.set_label(day_accessible_label(&labels, date, today));
                            node.set_bounds(accesskit_rect(rect));
                            node.set_row_index(row);
                            node.set_column_index(column);
                            node.set_selected(selected);
                            if selectable {
                                node.add_action(egui::accesskit::Action::Click);
                            } else {
                                node.clear_actions();
                                node.set_disabled();
                            }
                        });
                        let text = date.day().to_string();
                        paint_day_cell(
                            &cells,
                            rect,
                            theme,
                            style,
                            DayCell {
                                text: &text,
                                selected,
                                today,
                                adjacent,
                                disabled: !selectable,
                                hovered: cell_response.hovered(),
                                focused: focused && focus_visible,
                                press: if selectable {
                                    crate::motion_controls::press_amount(&cell_response, theme)
                                } else {
                                    0.0
                                },
                            },
                            (cell * 0.44).clamp(9.0, 18.0),
                            selected_amount,
                        );
                        if selectable && cell_response.hovered() {
                            ui.output_mut(|output| {
                                output.cursor_icon = egui::CursorIcon::PointingHand;
                            });
                        }
                        if selectable && cell_response.clicked() {
                            clicked_date = Some(date);
                        }
                    }
                }

                if let (Some(focused), Some(_)) = (focus_cell, grid_start) {
                    let focused_id =
                        id.with(("day", focused.year(), focused.month(), focused.day()));
                    ui.ctx().accesskit_node_builder(grid_id, |node| {
                        node.set_active_descendant(focused_id.accesskit_id());
                    });
                }
            },
        );

        if let Some(today) = config.today
            && (config.min_date..=config.max_date).contains(&today)
            && crate::Button::new(labels.today, theme)
                .variant(crate::ButtonVariant::Ghost)
                .size(crate::ButtonSize::Small)
                .enabled(config.enabled)
                .show(ui)
                .clicked()
        {
            let month = today.first_of_month();
            changed |= state.set_visible_month(month);
            changed |= state.set_focused(config.focus_candidate(month, today.day()));
            ui.memory_mut(|memory| memory.request_focus(grid_id));
            ui.ctx().request_repaint();
        }

        if let Some(date) = clicked_date {
            activated = Some(date);
            changed |= state.set_focused(Some(date));
            // Clicking a trailing or leading adjacent-month day pages the grid
            // to that month, like upstream.
            if date.first_of_month() != state.visible_month() {
                changed |= state.set_visible_month(date);
            }
            if config.toggleable && state.selected() == Some(date) {
                changed |= state.set_selected(None);
            } else {
                changed |= state.set_selected(Some(date));
            }
            response.request_focus();
            ui.ctx().request_repaint();
        }

        if changed {
            response.mark_changed();
        }
        CalendarResponse {
            response,
            activated,
            changed,
        }
    }
}

#[derive(Clone, Copy)]
struct DayCell<'a> {
    text: &'a str,
    selected: bool,
    today: bool,
    adjacent: bool,
    disabled: bool,
    hovered: bool,
    focused: bool,
    press: f32,
}

fn paint_day_cell(
    painter: &egui::Painter,
    rect: Rect,
    theme: &Theme,
    style: CalendarStyle,
    cell: DayCell<'_>,
    font_size: f32,
    selected_amount: f32,
) {
    let rect = rect.scale_from_center(crate::motion::press_scale(cell.press));
    let radius = style.radius;
    let surface = rect.shrink(1.0);
    let paint_fill = |bounds, color| {
        painter.add(crate::motion::press_shape(
            egui::epaint::RectShape::filled(bounds, radius, color),
            cell.press,
        ));
    };
    let (fill, text_color) = if cell.selected {
        (style.selected_fill, style.selected_text)
    } else if cell.disabled {
        (egui::Color32::TRANSPARENT, theme.muted_foreground)
    } else if cell.hovered {
        (style.hover_fill, theme.secondary_foreground)
    } else if cell.adjacent {
        (egui::Color32::TRANSPARENT, theme.muted_foreground)
    } else {
        (egui::Color32::TRANSPARENT, theme.foreground)
    };
    if !cell.selected && selected_amount > 0.0 {
        let fading = style
            .selected_fill
            .gamma_multiply(selected_amount.min(1.0) * 0.12);
        paint_fill(surface, fading);
    }
    if fill != egui::Color32::TRANSPARENT {
        let bounds = if cell.selected {
            surface.scale_from_center(0.84 + 0.16 * selected_amount.clamp(0.0, 1.0))
        } else {
            surface
        };
        paint_fill(bounds, fill);
    }
    if cell.focused {
        painter.add(crate::motion::press_shape(
            egui::epaint::RectShape::stroke(
                surface,
                radius,
                Stroke::new(2.0, style.focus),
                StrokeKind::Inside,
            ),
            cell.press,
        ));
    }
    let text_rect = painter.text(
        rect.center(),
        egui::Align2::CENTER_CENTER,
        cell.text,
        egui::FontId::proportional(font_size),
        text_color,
    );
    if cell.today {
        let y = text_rect.max.y + 1.0;
        painter.line_segment(
            [
                egui::pos2(text_rect.min.x, y),
                egui::pos2(text_rect.max.x, y),
            ],
            Stroke::new(1.0, text_color),
        );
    }
}

fn day_accessible_label(labels: &CalendarLabels<'_>, date: Date, today: bool) -> String {
    let label = (labels.day_label)(date);
    if today {
        format!("{label}, {}", labels.today)
    } else {
        label
    }
}

fn accesskit_rect(rect: Rect) -> egui::accesskit::Rect {
    egui::accesskit::Rect {
        x0: f64::from(rect.min.x),
        y0: f64::from(rect.min.y),
        x1: f64::from(rect.max.x),
        y1: f64::from(rect.max.y),
    }
}

fn picker_page_start(year: i32) -> i32 {
    ((year - 1).div_euclid(12)) * 12 + 1
}

/// Render the month or year sub-grid and update the caller-owned state.
#[allow(clippy::too_many_arguments)]
fn show_calendar_picker(
    ui: &mut Ui,
    id: Id,
    theme: &Theme,
    config: &Config<'_>,
    state: &mut CalendarState,
    mode: PickerMode,
    mode_key: Id,
    page_key: Id,
    page_anchor_key: Id,
    focus_pending_key: Id,
    restore_days_key: Id,
    back_label: &str,
) -> (Response, bool) {
    let picker = config.picker;
    let mut changed = false;
    let mut next_mode = mode;
    let mut focus_pending = ui
        .ctx()
        .data(|data| data.get_temp::<bool>(focus_pending_key).unwrap_or(false));
    let cell_height = (config.sizes.calendar * 0.75).max(24.0);
    let response = ui
        .vertical(|ui| match mode {
            PickerMode::Months => {
                let year = state.visible_month().year();
                let mut year_delta = 0i64;
                let mut open_years = false;
                ui.horizontal(|ui| {
                    if navigation_button(ui, theme, "<", back_label, config.enabled) {
                        next_mode = PickerMode::Days;
                        ui.ctx()
                            .data_mut(|data| data.insert_temp(restore_days_key, true));
                        ui.ctx().request_repaint();
                    }
                    let heading_text = (picker.year)(year);
                    let heading = ui.label(egui::RichText::new(&heading_text).size(16.0).strong());
                    ui.ctx().accesskit_node_builder(heading.id, |node| {
                        node.set_role(egui::accesskit::Role::Heading);
                        node.set_level(2);
                        node.set_label(heading_text.clone());
                    });
                    if navigation_button(ui, theme, "v", picker.choose_year, config.enabled) {
                        let visible_year = state.visible_month().year();
                        ui.ctx().data_mut(|data| {
                            data.insert_temp(page_key, picker_page_start(visible_year));
                            data.insert_temp(page_anchor_key, visible_year);
                        });
                        open_years = true;
                    }
                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                        let visible = state.visible_month();
                        if navigation_button(
                            ui,
                            theme,
                            ">>",
                            picker.next_year,
                            config.enabled && config.shifted_month(visible, 12).is_some(),
                        ) {
                            year_delta = 12;
                        }
                        if navigation_button(
                            ui,
                            theme,
                            "<<",
                            picker.previous_year,
                            config.enabled && config.shifted_month(visible, -12).is_some(),
                        ) {
                            year_delta = -12;
                        }
                    });
                });
                if open_years {
                    next_mode = PickerMode::Years;
                    focus_pending = true;
                }
                if year_delta != 0
                    && let Some(target) = config.shifted_month(state.visible_month(), year_delta)
                {
                    changed |= state.set_visible_month(target);
                    let preferred = state.focused().map_or(1, Date::day);
                    changed |= state.set_focused(config.focus_candidate(target, preferred));
                }
                let grid_id = id.with("picker-months");
                // Upstream's month picker is a 3x4 grid, which also keeps the
                // full month names inside their cells.
                let width = ((ui.available_width() - 2.0 * 2.0) / 3.0).max(28.0);
                let mut clicked = None;
                let mut focus_candidate = None;
                let mut focus_selected = None;
                ui.scope_builder(egui::UiBuilder::new().scope_id(grid_id), |ui| {
                    ui.spacing_mut().item_spacing = Vec2::splat(2.0);
                    for row in 0..4 {
                        ui.horizontal(|ui| {
                            for column in 0..3 {
                                let month = (row * 3 + column + 1) as u8;
                                let first = Date::new(year, month, 1).expect("a valid month");
                                let enabled = config.enabled && config.month_in_bounds(first);
                                let selected = state.visible_month() == first;
                                let color = if selected {
                                    theme.primary_foreground
                                } else if enabled {
                                    theme.foreground
                                } else {
                                    theme.muted_foreground
                                };
                                let text =
                                    egui::RichText::new(picker.month_names[usize::from(month - 1)])
                                        .color(color);
                                let button = picker_cell(
                                    ui,
                                    theme,
                                    text,
                                    selected,
                                    enabled,
                                    Vec2::new(width, cell_height),
                                );
                                if enabled {
                                    focus_candidate.get_or_insert(button.id);
                                    if selected {
                                        focus_selected = Some(button.id);
                                    }
                                }
                                ui.ctx().accesskit_node_builder(button.id, |node| {
                                    if selected {
                                        node.set_selected(true);
                                    } else {
                                        node.clear_selected();
                                    }
                                });
                                if enabled && button.clicked() {
                                    clicked = Some(first);
                                }
                            }
                        });
                    }
                });
                if focus_pending {
                    if let Some(target) = focus_selected.or(focus_candidate) {
                        ui.memory_mut(|memory| memory.request_focus(target));
                    }
                    focus_pending = false;
                }
                ui.ctx().accesskit_node_builder(grid_id, |node| {
                    node.set_role(egui::accesskit::Role::Grid);
                    node.set_label(picker.months_grid);
                });
                if let Some(first) = clicked {
                    changed |= state.set_visible_month(first);
                    let preferred = state.focused().map_or(1, Date::day);
                    changed |= state.set_focused(config.focus_candidate(first, preferred));
                    next_mode = PickerMode::Days;
                    ui.ctx()
                        .data_mut(|data| data.insert_temp(restore_days_key, true));
                    ui.ctx().request_repaint();
                }
            }
            PickerMode::Years => {
                let visible_year = state.visible_month().year();
                let mut page = ui.ctx().data(|data| {
                    data.get_temp::<i32>(page_key)
                        .unwrap_or_else(|| picker_page_start(visible_year))
                });
                let anchor = ui.ctx().data(|data| {
                    data.get_temp::<i32>(page_anchor_key)
                        .unwrap_or(visible_year)
                });
                if anchor != visible_year {
                    page = picker_page_start(visible_year);
                }
                let mut page_delta = 0i32;
                ui.horizontal(|ui| {
                    if navigation_button(ui, theme, "<", back_label, config.enabled) {
                        next_mode = PickerMode::Months;
                        focus_pending = true;
                        ui.ctx().request_repaint();
                    }
                    let heading_text = (picker.years_heading)(page, page + 11);
                    let heading = ui.label(egui::RichText::new(&heading_text).size(16.0).strong());
                    ui.ctx().accesskit_node_builder(heading.id, |node| {
                        node.set_role(egui::accesskit::Role::Heading);
                        node.set_level(2);
                        node.set_label(heading_text.clone());
                    });
                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                        if navigation_button(
                            ui,
                            theme,
                            ">>",
                            picker.next_years,
                            config.enabled && page + 12 <= config.max_date.year(),
                        ) {
                            page_delta = 12;
                        }
                        if navigation_button(
                            ui,
                            theme,
                            "<<",
                            picker.previous_years,
                            config.enabled && page > config.min_date.year(),
                        ) {
                            page_delta = -12;
                        }
                    });
                });
                page += page_delta;
                ui.ctx().data_mut(|data| {
                    data.insert_temp(page_key, page);
                    data.insert_temp(page_anchor_key, visible_year);
                });
                let grid_id = id.with("picker-years");
                let width = ((ui.available_width() - 3.0 * 2.0) / 4.0).max(28.0);
                let mut clicked = None;
                let mut focus_candidate = None;
                let mut focus_selected = None;
                ui.scope_builder(egui::UiBuilder::new().scope_id(grid_id), |ui| {
                    ui.spacing_mut().item_spacing = Vec2::splat(2.0);
                    for row in 0..3 {
                        ui.horizontal(|ui| {
                            for column in 0..4 {
                                let year = page + (row * 4 + column);
                                let enabled = config.enabled
                                    && year >= config.min_date.year()
                                    && year <= config.max_date.year();
                                let selected = year == visible_year;
                                let color = if selected {
                                    theme.primary_foreground
                                } else if enabled {
                                    theme.foreground
                                } else {
                                    theme.muted_foreground
                                };
                                let text = egui::RichText::new((picker.year)(year)).color(color);
                                let button = picker_cell(
                                    ui,
                                    theme,
                                    text,
                                    selected,
                                    enabled,
                                    Vec2::new(width, cell_height),
                                );
                                if enabled {
                                    focus_candidate.get_or_insert(button.id);
                                    if selected {
                                        focus_selected = Some(button.id);
                                    }
                                }
                                ui.ctx().accesskit_node_builder(button.id, |node| {
                                    if selected {
                                        node.set_selected(true);
                                    } else {
                                        node.clear_selected();
                                    }
                                });
                                if enabled && button.clicked() {
                                    clicked = Some(year);
                                }
                            }
                        });
                    }
                });
                if focus_pending {
                    if let Some(target) = focus_selected.or(focus_candidate) {
                        ui.memory_mut(|memory| memory.request_focus(target));
                    }
                    focus_pending = false;
                }
                ui.ctx().accesskit_node_builder(grid_id, |node| {
                    node.set_role(egui::accesskit::Role::Grid);
                    node.set_label(picker.years_grid);
                });
                if let Some(year) = clicked {
                    let month = state.visible_month().month();
                    let target = Date::new(year, month, 1)
                        .expect("a valid month")
                        .max(config.min_date.first_of_month())
                        .min(config.max_date.first_of_month());
                    changed |= state.set_visible_month(target);
                    let preferred = state.focused().map_or(1, Date::day);
                    changed |= state.set_focused(config.focus_candidate(target, preferred));
                    next_mode = PickerMode::Months;
                    ui.ctx().request_repaint();
                }
            }
            PickerMode::Days => {}
        })
        .response;
    if mode == PickerMode::Years && next_mode == PickerMode::Months {
        focus_pending = true;
    }
    ui.ctx()
        .data_mut(|data| data.insert_temp(focus_pending_key, focus_pending));
    ui.ctx()
        .data_mut(|data| data.insert_temp(mode_key, next_mode));
    (response, changed)
}

fn picker_cell(
    ui: &mut Ui,
    theme: &Theme,
    text: egui::RichText,
    selected: bool,
    enabled: bool,
    size: Vec2,
) -> Response {
    crate::button::press_native(ui, theme, |ui| {
        let mut button = egui::Button::selectable(selected, text)
            .corner_radius(255)
            .frame_when_inactive(selected)
            .truncate()
            .min_size(size);
        if selected {
            button = button.fill(theme.primary);
        }
        ui.add_enabled_ui(enabled, |ui| ui.add_sized(size, button))
            .inner
    })
}

fn navigation_size(theme: &Theme) -> f32 {
    if theme.touch { 40.0 } else { 32.0 }
}

fn navigation_button(
    ui: &mut Ui,
    theme: &Theme,
    visible: &str,
    accessible: &str,
    enabled: bool,
) -> bool {
    let response = crate::button::press_native(ui, theme, |ui| {
        let response = ui.add_enabled(
            enabled,
            egui::Button::new("")
                .frame_when_inactive(false)
                .corner_radius(255)
                .min_size(Vec2::splat(navigation_size(theme))),
        );
        let color = if enabled {
            theme.foreground
        } else {
            theme.muted_foreground
        };
        let stroke = Stroke::new(1.6, color);
        let center = response.rect.center();
        let painter = ui.painter();
        let chevron = |[start, point, end]: [egui::Pos2; 3]| {
            painter.line_segment([start, point], stroke);
            painter.line_segment([point, end], stroke);
        };
        if visible == "v" {
            chevron([
                center + egui::vec2(-4.0, -2.0),
                center + egui::vec2(0.0, 2.0),
                center + egui::vec2(4.0, -2.0),
            ]);
        } else {
            let direction = if visible.starts_with('<') { -1.0 } else { 1.0 };
            let count = visible.len().min(2);
            for index in 0..count {
                let offset = (index as f32 - (count - 1) as f32 / 2.0) * 5.0;
                let x = center.x + offset;
                chevron([
                    egui::pos2(x - direction * 2.0, center.y - 4.0),
                    egui::pos2(x + direction * 2.0, center.y),
                    egui::pos2(x - direction * 2.0, center.y + 4.0),
                ]);
            }
        }
        response
    });
    ui.ctx().accesskit_node_builder(response.id, |node| {
        node.set_label(accessible);
        node.clear_toggled();
    });
    response.clicked()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn date(year: i32, month: u8, day: u8) -> Date {
        Date::new(year, month, day).expect("test date is valid")
    }

    #[test]
    fn leap_years_follow_the_gregorian_rule() {
        for year in [1600, 2000, 2024, 2028, 2400] {
            assert!(Date::is_leap_year(year), "{year} should be a leap year");
        }
        for year in [1700, 1800, 1900, 2023, 2100, 2200] {
            assert!(
                !Date::is_leap_year(year),
                "{year} should not be a leap year"
            );
        }
    }

    #[test]
    fn month_lengths_are_exact() {
        let lengths = [31, 28, 31, 30, 31, 30, 31, 31, 30, 31, 30, 31];
        for (index, expected) in lengths.iter().enumerate() {
            let month = u8::try_from(index + 1).expect("month fits");
            assert_eq!(Date::days_in_month(2023, month), Ok(*expected));
        }
        assert_eq!(Date::days_in_month(2024, 2), Ok(29));
        assert_eq!(Date::days_in_month(1900, 2), Ok(28));
        assert_eq!(Date::days_in_month(2000, 2), Ok(29));
        assert_eq!(
            Date::days_in_month(2024, 0),
            Err(DateError::InvalidMonth(0))
        );
        assert_eq!(
            Date::days_in_month(2024, 13),
            Err(DateError::InvalidMonth(13))
        );
    }

    #[test]
    fn construction_rejects_invalid_dates() {
        assert!(Date::new(2024, 2, 29).is_ok());
        assert!(matches!(
            Date::new(2023, 2, 29),
            Err(DateError::InvalidDay {
                year: 2023,
                month: 2,
                day: 29
            })
        ));
        assert!(Date::new(2024, 4, 31).is_err());
        assert!(Date::new(2024, 1, 0).is_err());
        assert_eq!(Date::new(0, 1, 1), Err(DateError::YearOutOfRange(0)));
        assert_eq!(
            Date::new(10_000, 1, 1),
            Err(DateError::YearOutOfRange(10_000))
        );
        assert_eq!(Date::new(2024, 13, 1), Err(DateError::InvalidMonth(13)));
    }

    #[test]
    fn weekday_computation_matches_known_dates() {
        let cases = [
            ((1970, 1, 1), Weekday::Thursday),
            ((2000, 1, 1), Weekday::Saturday),
            ((2024, 2, 29), Weekday::Thursday),
            ((2024, 7, 15), Weekday::Monday),
            ((2024, 7, 21), Weekday::Sunday),
            ((2023, 12, 31), Weekday::Sunday),
            ((2100, 12, 31), Weekday::Friday),
            ((Date::MIN.year(), 1, 1), Weekday::Monday),
            ((Date::MAX.year(), 12, 31), Weekday::Friday),
        ];
        for ((year, month, day), weekday) in cases {
            assert_eq!(
                date(year, month, day).weekday(),
                weekday,
                "{year}-{month}-{day}"
            );
        }
        for (offset, weekday) in Weekday::ALL.iter().enumerate() {
            let day = u8::try_from(15 + offset).expect("day fits");
            assert_eq!(date(2024, 7, day).weekday(), *weekday);
        }
    }

    #[test]
    fn weekday_index_round_trips() {
        for weekday in Weekday::ALL {
            assert_eq!(Weekday::from_monday_index(weekday.index()), Some(weekday));
        }
        assert_eq!(Weekday::from_monday_index(7), None);
        assert_eq!(Weekday::Monday.index(), 0);
        assert_eq!(Weekday::Sunday.index(), 6);
    }

    #[test]
    fn add_days_crosses_month_and_year_boundaries() {
        assert_eq!(date(2024, 2, 28).add_days(1), Some(date(2024, 2, 29)));
        assert_eq!(date(2024, 2, 29).add_days(1), Some(date(2024, 3, 1)));
        assert_eq!(date(2023, 2, 28).add_days(1), Some(date(2023, 3, 1)));
        assert_eq!(date(2024, 12, 31).add_days(1), Some(date(2025, 1, 1)));
        assert_eq!(date(2024, 1, 1).add_days(-1), Some(date(2023, 12, 31)));
        assert_eq!(date(2024, 3, 1).add_days(-1), Some(date(2024, 2, 29)));
        assert_eq!(Date::MIN.add_days(-1), None);
        assert_eq!(Date::MAX.add_days(1), None);
        assert_eq!(Date::MAX.add_days(i64::MAX), None);
        assert_eq!(Date::MIN.add_days(i64::MIN), None);
        assert_eq!(date(2024, 1, 1).next_day(), Some(date(2024, 1, 2)));
        assert_eq!(date(2024, 1, 2).previous_day(), Some(date(2024, 1, 1)));
    }

    #[test]
    fn add_months_clamps_the_day() {
        assert_eq!(date(2024, 1, 31).add_months(1), Some(date(2024, 2, 29)));
        assert_eq!(date(2023, 1, 31).add_months(1), Some(date(2023, 2, 28)));
        assert_eq!(date(2024, 3, 31).add_months(-1), Some(date(2024, 2, 29)));
        assert_eq!(date(2024, 12, 15).add_months(1), Some(date(2025, 1, 15)));
        assert_eq!(date(2024, 1, 15).add_months(-1), Some(date(2023, 12, 15)));
        assert_eq!(date(2024, 1, 1).add_months(12), Some(date(2025, 1, 1)));
        assert_eq!(date(2024, 7, 15).add_months(0), Some(date(2024, 7, 15)));
        assert_eq!(
            date(2024, 1, 1).first_of_month().add_months(1),
            Some(date(2024, 2, 1))
        );
        assert_eq!(Date::MAX.add_months(1), None);
        assert_eq!(Date::MIN.add_months(-1), None);
        assert_eq!(Date::MAX.add_months(i64::MAX), None);
    }

    #[test]
    fn first_and_last_of_month_are_exact() {
        assert_eq!(date(2024, 2, 15).first_of_month(), date(2024, 2, 1));
        assert_eq!(date(2024, 2, 15).last_of_month(), date(2024, 2, 29));
        assert_eq!(date(2023, 2, 15).last_of_month(), date(2023, 2, 28));
        assert_eq!(date(2024, 12, 31).last_of_month(), date(2024, 12, 31));
    }

    #[test]
    fn days_until_counts_whole_days() {
        assert_eq!(date(2024, 1, 1).days_until(date(2024, 2, 1)), 31);
        assert_eq!(date(2024, 2, 1).days_until(date(2024, 1, 1)), -31);
        assert_eq!(date(2024, 1, 1).days_until(date(2024, 1, 1)), 0);
        assert_eq!(date(2023, 12, 31).days_until(date(2024, 1, 1)), 1);
    }

    #[test]
    fn ordering_is_lexicographic() {
        let mut dates = vec![
            date(2024, 7, 15),
            date(2023, 12, 31),
            date(2024, 1, 1),
            date(2024, 7, 14),
        ];
        dates.sort();
        assert_eq!(
            dates,
            vec![
                date(2023, 12, 31),
                date(2024, 1, 1),
                date(2024, 7, 14),
                date(2024, 7, 15)
            ]
        );
        assert!(date(2024, 7, 14) < date(2024, 7, 15));
    }

    #[test]
    fn clamp_respects_bounds() {
        let min = date(2024, 7, 10);
        let max = date(2024, 7, 20);
        assert_eq!(date(2024, 7, 1).clamp(min, max), min);
        assert_eq!(date(2024, 7, 25).clamp(min, max), max);
        assert_eq!(date(2024, 7, 15).clamp(min, max), date(2024, 7, 15));
        assert_eq!(date(2024, 7, 15).clamp(max, min), max);
    }

    #[test]
    fn with_day_validates_the_target_month() {
        assert_eq!(date(2024, 2, 1).with_day(29), Ok(date(2024, 2, 29)));
        assert!(date(2023, 2, 1).with_day(29).is_err());
        assert!(date(2024, 2, 1).with_day(0).is_err());
    }

    #[test]
    fn display_and_names_are_stable() {
        let value = date(2024, 7, 5);
        assert_eq!(value.to_string(), "2024-07-05");
        assert_eq!(value.month_name(), "July");
        assert_eq!(value.weekday_name(), "Friday");
        assert_eq!(month_name(1), "January");
        assert_eq!(month_name(12), "December");
        assert_eq!(month_name(0), "");
        assert_eq!(Weekday::Monday.short_name(), "Mon");
        assert_eq!(Weekday::Sunday.narrow_name(), "S");
    }

    #[test]
    fn state_normalizes_the_visible_month() {
        let mut state = CalendarState::new(date(2024, 7, 15), Some(date(2024, 7, 20)));
        assert_eq!(state.visible_month(), date(2024, 7, 1));
        assert_eq!(state.selected(), Some(date(2024, 7, 20)));
        assert_eq!(state.focused(), None);
        assert!(state.set_visible_month(date(2024, 8, 23)));
        assert_eq!(state.visible_month(), date(2024, 8, 1));
        assert!(!state.set_visible_month(date(2024, 8, 9)));
        assert!(state.set_selected(None));
        assert!(!state.set_selected(None));
        assert!(state.set_focused(Some(date(2024, 8, 3))));
        assert_eq!(state.focused(), Some(date(2024, 8, 3)));
    }

    #[test]
    fn config_bounds_and_focus_helpers() {
        let config = Config {
            sizes: Sizes::desktop(),
            min_date: date(2024, 7, 15),
            max_date: date(2024, 9, 10),
            selectable: None,
            first_day_of_week: Weekday::Monday,
            today: Some(date(2024, 8, 1)),
            toggleable: false,
            enabled: true,
            labels: CalendarLabels::english(),
            picker: CalendarPickerLabels::english(),
        };
        assert!(!config.day_selectable(date(2024, 7, 14)));
        assert!(config.day_selectable(date(2024, 7, 15)));
        assert!(!config.day_selectable(date(2024, 9, 11)));
        assert!(config.month_in_bounds(date(2024, 7, 1)));
        assert!(config.month_in_bounds(date(2024, 9, 1)));
        assert!(!config.month_in_bounds(date(2024, 6, 1)));
        assert_eq!(
            config.shifted_month(date(2024, 7, 1), 1),
            Some(date(2024, 8, 1))
        );
        assert_eq!(config.shifted_month(date(2024, 7, 1), -1), None);
        assert_eq!(
            config.focus_candidate(date(2024, 7, 1), 1),
            Some(date(2024, 7, 15))
        );
        assert_eq!(
            config.first_selectable_in_month(date(2024, 7, 1)),
            Some(date(2024, 7, 15))
        );
        assert_eq!(
            config.default_focus_date(date(2024, 8, 1)),
            Some(date(2024, 8, 1))
        );
    }

    #[test]
    fn config_enabled_and_predicate_gate_selectability() {
        let disabled = |_: Date| false;
        let mut config = Config {
            sizes: Sizes::desktop(),
            min_date: date(2024, 1, 1),
            max_date: date(2024, 12, 31),
            selectable: Some(&disabled),
            first_day_of_week: Weekday::Monday,
            today: None,
            toggleable: false,
            enabled: true,
            labels: CalendarLabels::english(),
            picker: CalendarPickerLabels::english(),
        };
        assert!(!config.day_selectable(date(2024, 7, 15)));
        config.selectable = None;
        config.enabled = false;
        assert!(!config.day_selectable(date(2024, 7, 15)));
        config.enabled = true;
        assert!(config.day_selectable(date(2024, 7, 15)));
    }
}
