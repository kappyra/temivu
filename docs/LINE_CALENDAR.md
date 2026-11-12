# Line calendar (experimental)

A compact horizontal or vertical strip of consecutive days, ported from Forui's
`FLineCalendar`. It is caller-owned like the full `Calendar`: the first visible
day, the selection and the keyboard focus live in `LineCalendarState`, and the
library never reads the wall clock.

```rust
use temivu::{Date, LineCalendar, LineCalendarState, Theme};

fn example(ui: &mut egui::Ui, state: &mut LineCalendarState) {
    let theme = Theme::light();
    let today = Date::new(2024, 7, 15).expect("a valid date");
    let output = LineCalendar::new(state, &theme).today(today).show(ui);
    if let Some(date) = output.activated {
        let _selected = date;
    }
}
```

## Window, state and navigation

The strip renders a fixed window of `LineCalendar::days` consecutive days
starting at `LineCalendarState::anchor` (14 by default). Unlike upstream's
virtualised scroll view this keeps the widget count bounded and deterministic:
moving outside the window shifts the anchor by the minimum number of days.
`LineCalendarState::selected` holds the selected day, `focused` the keyboard
cursor, and `visible_month()` returns the anchor's month as its first day.
Navigation changes `visible_month`, and the response reports the new month once
as `month_changed` so a surrounding header can update. When the caller moves the
selection outside the window and the strip has no keyboard focus, the next frame
scrolls it into view.

`LineCalendar::bounds` enforces inclusive minimum and maximum dates,
`LineCalendar::selectable` disables individual days, `LineCalendar::today` marks
today with a dot and the ", Today" accessible suffix, `LineCalendar::toggleable`
makes pressing the selected day clear it, and `LineCalendar::orientation` and
`LineCalendar::item_size` control the layout. `radius` overrides the day-cell
corners, and `lighting` can override the inherited Vesper material or turn it
off for that strip. With keyboard focus (Tab), arrows
step by one day along the strip and by one week across it, Home/End jump to the
window edges, Page Up/Page Down change the month, and Enter/Space select the
focused day. Arrow keys are locked to the strip so focus cannot escape while
navigating.

## Accessibility

The strip is a focusable `List` whose accessible name defaults to "Date strip".
Each day is a `ListItem` with a full label ("July 15, 2024", plus ", Today" when
applicable) and a selected state; days outside the bounds or rejected by the
predicate are disabled. The focused day is the list's active descendant, so
screen readers follow arrow-key movement. `LineCalendarLabels` replaces the list
label, today suffix, per-day label builder and short weekday names.

## Known gaps

Free scrolling, scroll controllers, programmatic scroll animations, item
builders, RTL, custom styles, screen-reader testing and upstream visual parity
remain open. Because the window is fixed, very large `days` values render that
many items; choose a window that covers the container. See `COVERAGE.md`.
