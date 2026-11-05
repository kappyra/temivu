# Calendar (experimental)

`Calendar` uses the supplied theme's sizes, including the touch variant.
`CalendarStyle::from_theme(&theme)` and `Calendar::style(...)` customize day
selection, hover and focus colors and corner radius. Month/year picker styling
is separate and is not yet covered by these overrides.

A single-month calendar day grid ported from Forui's grid calendar. Visible
month, selection and keyboard focus are caller-owned; the library never reads
the wall clock.

```rust
use temivu::{Calendar, CalendarState, Date, Theme};

fn example(ui: &mut egui::Ui, state: &mut CalendarState) {
    let theme = Theme::light();
    let today = Date::new(2024, 7, 15).expect("a valid date");
    let output = Calendar::new(state, &theme)
        .today(today)
        .bounds(
            Date::new(2024, 1, 1).expect("a valid date"),
            Date::new(2024, 12, 31).expect("a valid date"),
        )
        .show(ui);
    if let Some(date) = output.activated {
        let _selected = date;
    }
}
```

## State and date arithmetic

`CalendarState` stores the visible month (normalized to the first day), the
selected `Option<Date>` and the keyboard-focused `Option<Date>`. `Date` is a
validated proleptic Gregorian date with no time zone and no dependency on a date
crate: `Date::new` rejects invalid months, days and years, and pure helpers
cover leap years, month lengths, weekday computation, day/month shifting with
end-of-month clamping, ordering and `clamp`. Supply `today` explicitly;
`CalendarState::new` and `Calendar` accept it from the caller.

## Navigation, bounds and disabled dates

The header exposes previous/next month buttons with accessible names. Enable
`year_navigation(true)` to add direct year-jump buttons, or use the month/year
picker from the compact default header. `Calendar::bounds` sets inclusive minimum and maximum dates,
clamping an out-of-range visible month and disabling navigation (and day cells)
outside the range. `Calendar::selectable` adds a caller predicate that disables
individual dates. `Calendar::toggleable` makes re-selecting the selected date
clear it. `Calendar::first_day_of_week` and `Calendar::sizes` (cell side from
`Sizes::calendar`, shrunk to fit seven columns) adapt the grid.

Keyboard grid navigation focuses the whole grid: arrows move a day (skipping
disabled dates and paging across months), Page Up / Page Down change the month,
Shift+Page Up / Page Down change the year, Home / End move to the first / last
selectable day of the visible week, and Enter / Space select the focused day.
The focused day becomes the grid's AccessKit active descendant.

## Accessibility

The month heading is a level-2 heading, the grid is a `Grid` node with one
`GridCell` child per rendered day (including adjacent-month days) exposing
selected and disabled state plus row/column indices, and weekday columns are
`ColumnHeader` nodes. Day labels default to English such as
`"Monday, July 15, 2024"` with `", Today"` appended for today. Month headings,
day labels, weekday headers and button names can be replaced through
`CalendarLabels`; localization of date formats is not implemented.

## Month/year picker

The header exposes a small chooser button (accessible name
`CalendarPickerLabels::choose_month`) that opens a 3x4 month sub-grid. Months
outside the date bounds are disabled, the visible month is selected, and
choosing a month returns to the day grid. The month view has its own year
navigation plus a "choose year" control that opens a 12-year page sub-grid with
previous/next page buttons; choosing a year returns to the month view (clamped
to the date bounds), and Escape steps back one view at a time
(year -> month -> days). Grid roles/labels and the selected state are exposed.

Picker text defaults to English and can be localized with
`CalendarPickerLabels` via `Calendar::picker_labels`; it covers the chooser and
navigation accessible names, the month names, the year formatter and the
year-range heading.

## Known gaps

Swipe paging, contiguous range and multi-date selection, arrow-key navigation
inside the picker sub-grids (they use normal button focus/Tab/Enter), localized
date formats beyond the picker labels, RTL layout, animation, hover/pressed
parity, screen-reader testing and upstream visual parity remain open.

The picker consumes Escape before a hosting dialog can see it (so stepping back
a view never also closes the dialog), enters the year page containing the
visible year and re-anchors if the caller changes the visible year, resets its
ephemeral mode when the calendar is re-added after a gap of passes, focuses the
selected month/year on entry, and exposes the selected month/year through the
AccessKit selected state (in addition to egui's button toggle state). See `COVERAGE.md`.

## Floating date field surface

Date fields open a padded, rounded calendar surface with a preferred 280-point desktop content width (308 with touch sizing), capped to the viewport. The calendar caps its own width and uses compact painted chevrons. Days use circular selection by default; `CalendarStyle::radius` remains configurable. Existing date arithmetic, month/year pickers, bounds and keyboard semantics are preserved.
