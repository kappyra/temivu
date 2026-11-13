//! Preliminary port of Forui's line calendar: a compact strip of consecutive
//! days with caller-owned selection, an anchor (visible date), bounds and a
//! disabled-date predicate.
//!
//! The strip renders a fixed window of days around its anchor instead of a
//! free-scrolling virtual list, so it never allocates the whole 1900-2100 range
//! and stays deterministic in headless tests. Free scrolling, the scroll
//! controller, custom item builders and upstream visual parity remain open. See
//! `docs/LINE_CALENDAR.md`.
//!
//! ```
//! # fn example(ui: &mut egui::Ui) {
//! use temivu::{Date, LineCalendar, LineCalendarState, Theme};
//!
//! let theme = Theme::light();
//! let today = Date::new(2024, 7, 15).expect("a valid date");
//! let mut state = LineCalendarState::new(today.add_days(-3).expect("in range"), None);
//! let output = LineCalendar::new(&mut state, &theme).today(today).show(ui);
//! if let Some(date) = output.activated {
//!     let _selected = date;
//! }
//! # }
//! ```

use crate::calendar::{Calendar, Date, Weekday, month_name};
use crate::{Theme, Vesper};
use egui::{Id, Rect, Response, Sense, Stroke, StrokeKind, Ui, Vec2};

const DEFAULT_DAYS: usize = 14;
const DEFAULT_ITEM_WIDTH: f32 = 44.0;
const DEFAULT_ITEM_HEIGHT: f32 = 52.0;
const ITEM_SPACING: f32 = 6.0;

/// The direction a line calendar's strip runs in.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum LineCalendarOrientation {
    /// Days run left to right.
    #[default]
    Horizontal,
    /// Days run top to bottom.
    Vertical,
}

/// Caller-owned line-calendar state: the first visible day, the selected day
/// and the keyboard-focused day.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct LineCalendarState {
    anchor: Date,
    selected: Option<Date>,
    focused: Option<Date>,
}

impl LineCalendarState {
    /// Show a window starting at `anchor` with `selected` selected.
    pub fn new(anchor: Date, selected: Option<Date>) -> Self {
        Self {
            anchor,
            selected,
            focused: None,
        }
    }

    /// The first visible day.
    pub fn anchor(&self) -> Date {
        self.anchor
    }

    /// The selected day, if any.
    pub fn selected(&self) -> Option<Date> {
        self.selected
    }

    /// The keyboard-focused day, if any.
    pub fn focused(&self) -> Option<Date> {
        self.focused
    }

    /// The month containing the first visible day, as its first day.
    pub fn visible_month(&self) -> Date {
        self.anchor.first_of_month()
    }

    /// Move the window to start at `anchor`; returns whether it changed.
    pub fn set_anchor(&mut self, anchor: Date) -> bool {
        let changed = self.anchor != anchor;
        self.anchor = anchor;
        changed
    }

    /// Replace the selected day; returns whether it changed.
    pub fn set_selected(&mut self, selected: Option<Date>) -> bool {
        let changed = self.selected != selected;
        self.selected = selected;
        changed
    }

    /// Replace the keyboard-focused day; returns whether it changed.
    pub fn set_focused(&mut self, focused: Option<Date>) -> bool {
        let changed = self.focused != focused;
        self.focused = focused;
        changed
    }
}

/// English defaults for the strip's accessible labels.
#[derive(Clone, Copy)]
pub struct LineCalendarLabels<'a> {
    /// Accessible name of the strip.
    pub list: &'a str,
    /// Text appended to a day's label when it is today.
    pub today: &'a str,
    /// Builds a day's accessible label, for example `"July 15, 2024"`.
    pub day_label: &'a dyn Fn(Date) -> String,
    /// Short weekday names indexed Monday through Sunday.
    pub weekday_names: [&'a str; 7],
}

impl<'a> Default for LineCalendarLabels<'a> {
    fn default() -> Self {
        LineCalendarLabels::english()
    }
}

impl LineCalendarLabels<'static> {
    /// The English label set used when the caller supplies none.
    pub fn english() -> Self {
        Self {
            list: "Date strip",
            today: "Today",
            day_label: &english_day_label,
            weekday_names: Weekday::ALL.map(Weekday::short_name),
        }
    }
}

fn english_day_label(date: Date) -> String {
    format!(
        "{} {}, {}",
        month_name(date.month()),
        date.day(),
        date.year()
    )
}

/// What [`LineCalendar::show`] reports for the current frame.
pub struct LineCalendarResponse {
    /// The focusable strip response; it owns keyboard focus.
    pub response: Response,
    /// The day selected this frame by click, Enter or Space.
    pub activated: Option<Date>,
    /// The new visible month (as its first day) when the window moved into a
    /// different month this frame.
    pub month_changed: Option<Date>,
    /// Whether the anchor, focus or selection changed.
    pub changed: bool,
}

/// Builder options shared by the strip's navigation helpers.
struct Config<'a> {
    min_date: Date,
    max_date: Date,
    selectable: Option<&'a dyn Fn(Date) -> bool>,
    span: usize,
    enabled: bool,
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

    /// Step from `from` by whole days until a selectable day is found.
    fn step_selectable(&self, from: Date, step: i64, limit: i64) -> Option<Date> {
        let mut candidate = from;
        for _ in 0..limit {
            candidate = candidate.add_days(step)?;
            if candidate < self.min_date || candidate > self.max_date {
                return None;
            }
            if self.day_selectable(candidate) {
                return Some(candidate);
            }
        }
        None
    }

    /// The date one month away, resolved to a selectable day in that month.
    fn page_month(&self, from: Date, delta: i64) -> Option<Date> {
        let month = from.add_months(delta)?;
        if month < self.min_date.first_of_month() || month > self.max_date.first_of_month() {
            return None;
        }
        if self.day_selectable(month) {
            return Some(month);
        }
        self.first_selectable_in_month(month)
    }
}

/// A horizontal or vertical strip of consecutive days.
pub struct LineCalendar<'a> {
    state: &'a mut LineCalendarState,
    theme: &'a Theme,
    config: Config<'a>,
    today: Option<Date>,
    days: usize,
    orientation: LineCalendarOrientation,
    item_size: Option<Vec2>,
    labels: LineCalendarLabels<'a>,
    toggleable: bool,
    id: Option<Id>,
    radius: u8,
    lighting: Option<Option<Vesper>>,
}

impl<'a> LineCalendar<'a> {
    /// Create a strip for `state` using `theme`'s colors.
    pub fn new(state: &'a mut LineCalendarState, theme: &'a Theme) -> Self {
        Self {
            state,
            theme,
            config: Config {
                min_date: Calendar::default_min_date(),
                max_date: Calendar::default_max_date(),
                selectable: None,
                span: DEFAULT_DAYS,
                enabled: true,
            },
            today: None,
            days: DEFAULT_DAYS,
            orientation: LineCalendarOrientation::Horizontal,
            item_size: None,
            labels: LineCalendarLabels::english(),
            toggleable: false,
            id: None,
            radius: 12,
            lighting: None,
        }
    }

    /// Set the inclusive selectable date bounds. The argument order does not
    /// matter: the earlier date is the minimum.
    pub fn bounds(mut self, min: Date, max: Date) -> Self {
        self.config.min_date = min.min(max);
        self.config.max_date = max.max(min);
        self
    }

    /// Set a caller-supplied predicate that disables individual days.
    pub fn selectable(mut self, selectable: &'a dyn Fn(Date) -> bool) -> Self {
        self.config.selectable = Some(selectable);
        self
    }

    /// Set the caller-supplied "today" used for the today indicator and the
    /// initial keyboard focus. Library code never reads the wall clock.
    pub fn today(mut self, today: Date) -> Self {
        self.today = Some(today);
        self
    }

    /// Set how many consecutive days the strip renders. Defaults to 14.
    pub fn days(mut self, days: usize) -> Self {
        self.days = days.max(1);
        self
    }

    /// Lay the days out horizontally (default) or vertically.
    pub fn orientation(mut self, orientation: LineCalendarOrientation) -> Self {
        self.orientation = orientation;
        self
    }

    /// Override the default item size (`44 x 52` logical pixels). The strip may
    /// shrink items to fit the available cross-axis space.
    pub fn item_size(mut self, size: Vec2) -> Self {
        if size.is_finite() && size.min_elem() > 0.0 {
            self.item_size = Some(size);
        }
        self
    }

    /// Override the resting day-cell corner radius.
    pub fn radius(mut self, radius: u8) -> Self {
        self.radius = radius;
        self
    }

    /// Override inherited lighting. Pass `None` to disable it for this strip.
    pub fn lighting(mut self, lighting: Option<Vesper>) -> Self {
        self.lighting = Some(lighting);
        self
    }

    /// Re-select the selected day to clear it when pressed again.
    pub fn toggleable(mut self, toggleable: bool) -> Self {
        self.toggleable = toggleable;
        self
    }

    /// Replace the English label defaults.
    pub fn labels(mut self, labels: LineCalendarLabels<'a>) -> Self {
        self.labels = labels;
        self
    }

    /// Disable the strip.
    pub fn enabled(mut self, enabled: bool) -> Self {
        self.config.enabled = enabled;
        self
    }

    /// Give the strip a stable, globally unique ID across layout changes.
    pub fn id(mut self, id: Id) -> Self {
        self.id = Some(id);
        self
    }

    /// Draw the strip and return its response.
    pub fn show(self, ui: &mut Ui) -> LineCalendarResponse {
        let id = self
            .id
            .unwrap_or_else(|| ui.next_auto_id().with("temivu-line-calendar"));
        ui.scope_builder(egui::UiBuilder::new().scope_id(id), |ui| {
            self.show_inner(ui, id)
        })
        .inner
    }

    fn show_inner(self, ui: &mut Ui, id: Id) -> LineCalendarResponse {
        let Self {
            state,
            theme,
            mut config,
            today,
            days,
            orientation,
            item_size,
            labels,
            toggleable,
            radius,
            lighting,
            ..
        } = self;
        config.span = days.max(1);
        let mut changed = false;
        let mut activated: Option<Date> = None;

        // Keep the anchor inside the bounds and let a selection that the
        // caller moved pull the window when there is no keyboard focus.
        let clamped = window_start(
            state.anchor(),
            config.span,
            config.min_date,
            config.max_date,
        );
        changed |= state.set_anchor(clamped);
        if state.focused().is_none()
            && let Some(selected) = state.selected()
        {
            let (start, end) = window_bounds(
                state.anchor(),
                config.span,
                config.min_date,
                config.max_date,
            );
            if selected < start || selected > end {
                changed |= state.set_anchor(ensure_visible(
                    state.anchor(),
                    config.span,
                    selected,
                    config.min_date,
                    config.max_date,
                ));
            }
        }

        let month_before = state.visible_month();
        let strip_id = id.with("strip");
        let mut strip_focused = ui.memory(|memory| memory.has_focus(strip_id));
        if strip_focused {
            ui.memory_mut(|memory| {
                memory.set_focus_lock_filter(
                    strip_id,
                    egui::EventFilter {
                        horizontal_arrows: true,
                        vertical_arrows: true,
                        ..Default::default()
                    },
                );
            });
            if state.focused().is_none() {
                let (start, end) = window_bounds(
                    state.anchor(),
                    config.span,
                    config.min_date,
                    config.max_date,
                );
                let preferred = state
                    .selected()
                    .filter(|date| *date >= start && *date <= end)
                    .or_else(|| today.filter(|date| *date >= start && *date <= end))
                    .unwrap_or(start);
                let preferred = if config.day_selectable(preferred) {
                    Some(preferred)
                } else {
                    config.step_selectable(preferred, 1, 31).or(Some(start))
                };
                changed |= state.set_focused(preferred);
            }
            let before = (state.focused(), state.selected());
            let (along_back, along_forward, cross_back, cross_forward) = match orientation {
                LineCalendarOrientation::Horizontal => (
                    egui::Key::ArrowLeft,
                    egui::Key::ArrowRight,
                    egui::Key::ArrowUp,
                    egui::Key::ArrowDown,
                ),
                LineCalendarOrientation::Vertical => (
                    egui::Key::ArrowUp,
                    egui::Key::ArrowDown,
                    egui::Key::ArrowLeft,
                    egui::Key::ArrowRight,
                ),
            };
            let (
                back,
                forward,
                cross_back_pressed,
                cross_forward_pressed,
                home,
                end,
                page_up,
                page_down,
                accept,
            ) = ui.input(|input| {
                (
                    input.key_pressed(along_back),
                    input.key_pressed(along_forward),
                    input.key_pressed(cross_back),
                    input.key_pressed(cross_forward),
                    input.key_pressed(egui::Key::Home),
                    input.key_pressed(egui::Key::End),
                    input.key_pressed(egui::Key::PageUp),
                    input.key_pressed(egui::Key::PageDown),
                    input.key_pressed(egui::Key::Enter) || input.key_pressed(egui::Key::Space),
                )
            });
            if let Some(focused) = state.focused() {
                if back {
                    if let Some(date) = config.step_selectable(focused, -1, 366 * 2) {
                        changed |= state.set_focused(Some(date));
                    }
                } else if forward {
                    if let Some(date) = config.step_selectable(focused, 1, 366 * 2) {
                        changed |= state.set_focused(Some(date));
                    }
                } else if cross_back_pressed {
                    if let Some(date) = config.step_selectable(focused, -7, 60) {
                        changed |= state.set_focused(Some(date));
                    }
                } else if cross_forward_pressed {
                    if let Some(date) = config.step_selectable(focused, 7, 60) {
                        changed |= state.set_focused(Some(date));
                    }
                } else if home {
                    let (start, _) = window_bounds(
                        state.anchor(),
                        config.span,
                        config.min_date,
                        config.max_date,
                    );
                    if config.day_selectable(start) {
                        changed |= state.set_focused(Some(start));
                    } else if let Some(date) = config.step_selectable(start, 1, 31) {
                        changed |= state.set_focused(Some(date));
                    }
                } else if end {
                    let (_, end) = window_bounds(
                        state.anchor(),
                        config.span,
                        config.min_date,
                        config.max_date,
                    );
                    if config.day_selectable(end) {
                        changed |= state.set_focused(Some(end));
                    } else if let Some(date) = config.step_selectable(end, -1, 31) {
                        changed |= state.set_focused(Some(date));
                    }
                } else if page_up {
                    if let Some(date) = config.page_month(focused, -1) {
                        changed |= state.set_focused(Some(date));
                    }
                } else if page_down {
                    if let Some(date) = config.page_month(focused, 1) {
                        changed |= state.set_focused(Some(date));
                    }
                } else if accept && config.day_selectable(focused) {
                    if toggleable && state.selected() == Some(focused) {
                        changed |= state.set_selected(None);
                    } else {
                        changed |= state.set_selected(Some(focused));
                    }
                    activated = Some(focused);
                }
                if let Some(focused) = state.focused() {
                    changed |= state.set_anchor(ensure_visible(
                        state.anchor(),
                        config.span,
                        focused,
                        config.min_date,
                        config.max_date,
                    ));
                }
            }
            if (state.focused(), state.selected()) != before {
                changed = true;
                ui.ctx().request_repaint();
            }
        }

        // Layout. The strip renders a bounded window of days, so the number of
        // widgets stays constant regardless of the configured range.
        let (start, end) = window_bounds(
            state.anchor(),
            config.span,
            config.min_date,
            config.max_date,
        );
        let count = start.days_until(end).max(0) as usize + 1;
        let default_size = item_size.unwrap_or(Vec2::new(DEFAULT_ITEM_WIDTH, DEFAULT_ITEM_HEIGHT));
        let spacing = if orientation == LineCalendarOrientation::Horizontal {
            ITEM_SPACING.min(ui.available_width().max(1.0) / (2.0 * count as f32))
        } else {
            ITEM_SPACING
        };
        let (strip_origin, strip_size, item_width, item_height) = match orientation {
            LineCalendarOrientation::Horizontal => {
                let available = ui.available_width().max(1.0);
                let width = (available - spacing * (count.saturating_sub(1)) as f32) / count as f32;
                let width = default_size.x.min(width.max(0.0));
                let height = default_size.y;
                let size = Vec2::new(
                    width * count as f32 + spacing * (count.saturating_sub(1)) as f32,
                    height,
                );
                (ui.cursor().min, size, width, height)
            }
            LineCalendarOrientation::Vertical => {
                let width = ui.available_width().max(default_size.x);
                let available = ui.available_height();
                let height = if available.is_finite() && available > 0.0 {
                    ((available - spacing * (count.saturating_sub(1)) as f32) / count as f32)
                        .clamp(24.0, default_size.y)
                } else {
                    default_size.y
                };
                let size = Vec2::new(
                    width,
                    height * count as f32 + spacing * (count.saturating_sub(1)) as f32,
                );
                (ui.cursor().min, size, width, height)
            }
        };
        let (_, _reserve) = ui.allocate_exact_size(strip_size, Sense::hover());
        let strip_rect = Rect::from_min_size(strip_origin, strip_size);
        let strip_sense = if config.enabled {
            Sense::click()
        } else {
            Sense::hover()
        };
        let mut response = ui.interact(strip_rect, strip_id, strip_sense);
        strip_focused = strip_focused || response.has_focus();
        ui.ctx().accesskit_node_builder(strip_id, |node| {
            node.set_role(egui::accesskit::Role::List);
            node.set_label(labels.list);
        });
        if response.has_focus() {
            ui.memory_mut(|memory| {
                memory.set_focus_lock_filter(
                    strip_id,
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
            // The strip may gain keyboard focus after its layout: seed the
            // focused day so the focus ring renders and arrows have a start.
            if state.focused().is_none() {
                let preferred = state
                    .selected()
                    .filter(|date| *date >= start && *date <= end)
                    .or_else(|| today.filter(|date| *date >= start && *date <= end))
                    .unwrap_or(start);
                let preferred = if config.day_selectable(preferred) {
                    Some(preferred)
                } else {
                    config.step_selectable(preferred, 1, 31)
                };
                changed |= state.set_focused(preferred);
            }
        }

        let painter = ui.painter();
        let lighting = lighting.unwrap_or_else(|| Vesper::context(ui.ctx()));
        let mut clicked_date = None;
        for index in 0..count {
            let Some(date) = start.add_days(index as i64) else {
                continue;
            };
            let item_rect = match orientation {
                LineCalendarOrientation::Horizontal => Rect::from_min_size(
                    egui::pos2(
                        strip_rect.min.x + index as f32 * (item_width + spacing),
                        strip_rect.min.y,
                    ),
                    Vec2::new(item_width, item_height),
                ),
                LineCalendarOrientation::Vertical => Rect::from_min_size(
                    egui::pos2(
                        strip_rect.min.x,
                        strip_rect.min.y + index as f32 * (item_height + spacing),
                    ),
                    Vec2::new(item_width, item_height),
                ),
            };
            let item_id = id.with(("day", date.year(), date.month(), date.day()));
            let item_response = ui.interact(item_rect, item_id, Sense::CLICK);
            let selectable = config.day_selectable(date);
            let selected = state.selected() == Some(date);
            let is_today = today == Some(date);
            let focused = state.focused() == Some(date) && strip_focused;
            let label = day_accessible_label(&labels, date, is_today);
            ui.ctx().accesskit_node_builder(item_id, |node| {
                node.set_role(egui::accesskit::Role::ListItem);
                node.set_label(label.as_str());
                node.set_bounds(accesskit_rect(item_rect));
                node.set_selected(selected);
                if selectable {
                    node.add_action(egui::accesskit::Action::Click);
                } else {
                    node.clear_actions();
                    node.set_disabled();
                }
            });
            paint_item(
                painter,
                item_rect,
                theme,
                radius,
                &item_response,
                lighting,
                ItemVisual {
                    selected,
                    today: is_today,
                    disabled: !selectable,
                    hovered: item_response.hovered() && selectable,
                    focused,
                    weekday: labels.weekday_names[date.weekday().index()],
                    day: date.day(),
                },
            );
            if selectable && item_response.hovered() {
                ui.output_mut(|output| output.cursor_icon = egui::CursorIcon::PointingHand);
            }
            if selectable && item_response.clicked() {
                clicked_date = Some(date);
            }
        }

        if let Some(date) = clicked_date {
            activated = Some(date);
            changed |= state.set_focused(Some(date));
            if toggleable && state.selected() == Some(date) {
                changed |= state.set_selected(None);
            } else {
                changed |= state.set_selected(Some(date));
            }
            response.request_focus();
            ui.ctx().request_repaint();
        }

        if let Some(focused) = state.focused()
            && focused >= start
            && focused <= end
        {
            let focused_id = id.with(("day", focused.year(), focused.month(), focused.day()));
            ui.ctx().accesskit_node_builder(strip_id, |node| {
                node.set_active_descendant(focused_id.accesskit_id());
            });
        }

        let month_after = state.visible_month();
        let month_changed = (month_after != month_before).then_some(month_after);
        changed |= month_changed.is_some();
        if changed {
            response.mark_changed();
        }
        LineCalendarResponse {
            response,
            activated,
            month_changed,
            changed,
        }
    }
}

/// The first visible day for `anchor`, clamped so the full span fits the bounds.
fn window_start(anchor: Date, span: usize, min: Date, max: Date) -> Date {
    if span <= 1 {
        return anchor.clamp(min, max);
    }
    let max_start = max
        .add_days(-((span as i64) - 1))
        .map(|date| date.max(min))
        .unwrap_or(min);
    anchor.clamp(min, max_start)
}

/// The inclusive first and last visible day for `anchor`.
fn window_bounds(anchor: Date, span: usize, min: Date, max: Date) -> (Date, Date) {
    let start = window_start(anchor, span, min, max);
    let end = start
        .add_days((span.max(1) as i64) - 1)
        .map(|date| date.min(max))
        .unwrap_or(start);
    (start, end)
}

/// An anchor that keeps `target` inside the visible window.
fn ensure_visible(anchor: Date, span: usize, target: Date, min: Date, max: Date) -> Date {
    let (start, end) = window_bounds(anchor, span, min, max);
    if target < start {
        window_start(target, span, min, max)
    } else if target > end {
        let offset = (span.max(1) as i64) - 1;
        window_start(target.add_days(-offset).unwrap_or(min), span, min, max)
    } else {
        window_start(anchor, span, min, max)
    }
}

fn day_accessible_label(labels: &LineCalendarLabels<'_>, date: Date, today: bool) -> String {
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

struct ItemVisual<'a> {
    selected: bool,
    today: bool,
    disabled: bool,
    hovered: bool,
    focused: bool,
    weekday: &'a str,
    day: u8,
}

fn paint_item(
    painter: &egui::Painter,
    rect: Rect,
    theme: &Theme,
    radius: u8,
    response: &Response,
    lighting: Option<Vesper>,
    visual: ItemVisual<'_>,
) {
    if !painter.clip_rect().intersects(rect) {
        return;
    }
    let press = if visual.disabled {
        0.0
    } else {
        crate::motion_controls::press_amount(response, theme)
    };
    let rect = rect.scale_from_center(1.0 - 0.045 * press);
    let radius = egui::CornerRadius::same(radius);
    let (fill, text_color, weekday_color) = if visual.selected {
        (
            theme.primary,
            theme.primary_foreground,
            theme.primary_foreground,
        )
    } else if visual.disabled {
        (
            egui::Color32::TRANSPARENT,
            theme.muted_foreground,
            theme.muted_foreground,
        )
    } else if visual.hovered {
        (
            theme.secondary,
            theme.secondary_foreground,
            theme.secondary_foreground,
        )
    } else if visual.today {
        (
            theme.secondary,
            theme.secondary_foreground,
            theme.muted_foreground,
        )
    } else {
        (
            egui::Color32::TRANSPARENT,
            theme.foreground,
            theme.muted_foreground,
        )
    };
    let stroke = if visual.focused {
        Stroke::new(1.5, theme.readable_on(theme.foreground, fill, 4.5))
    } else {
        Stroke::NONE
    };
    painter.add(crate::motion::press_shape(
        egui::epaint::RectShape::new(rect, radius, fill, stroke, StrokeKind::Inside),
        press,
    ));
    if let Some(style) = lighting
        && !visual.disabled
        && let Some(light) = style.interactive_shape_on(
            painter,
            theme,
            response,
            rect,
            crate::motion::press_corners(rect, radius, press),
            fill,
        )
    {
        painter.add(light);
    }
    let body = theme.typography.body;
    let caption = theme.typography.caption;
    let text_color = if visual.disabled {
        theme.muted_foreground
    } else {
        text_color
    };
    painter.text(
        egui::pos2(rect.center().x, rect.center().y - caption.size * 0.78),
        egui::Align2::CENTER_CENTER,
        visual.weekday,
        egui::FontId::proportional(caption.size),
        if visual.selected || visual.hovered || visual.today {
            weekday_color
        } else {
            theme.muted_foreground
        },
    );
    painter.text(
        egui::pos2(rect.center().x, rect.center().y + body.size * 0.40),
        egui::Align2::CENTER_CENTER,
        visual.day,
        egui::FontId::proportional(body.size),
        text_color,
    );
    if visual.today {
        let dot = if visual.selected {
            theme.primary_foreground
        } else {
            theme.foreground
        };
        painter.circle_filled(egui::pos2(rect.center().x, rect.bottom() - 6.0), 1.5, dot);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn date(year: i32, month: u8, day: u8) -> Date {
        Date::new(year, month, day).expect("test date is valid")
    }

    #[test]
    fn window_clamps_anchor_to_the_bounds() {
        let min = date(2024, 7, 10);
        let max = date(2024, 7, 31);
        assert_eq!(
            window_start(date(2024, 7, 15), 7, min, max),
            date(2024, 7, 15)
        );
        assert_eq!(window_start(date(2024, 1, 1), 7, min, max), min);
        assert_eq!(
            window_start(date(2024, 12, 1), 7, min, max),
            date(2024, 7, 25)
        );
        let (start, end) = window_bounds(date(2024, 7, 15), 7, min, max);
        assert_eq!((start, end), (date(2024, 7, 15), date(2024, 7, 21)));
        let (start, end) = window_bounds(date(2024, 12, 1), 7, min, max);
        assert_eq!((start, end), (date(2024, 7, 25), date(2024, 7, 31)));
    }

    #[test]
    fn window_handles_ranges_smaller_than_the_span() {
        let min = date(2024, 7, 10);
        let max = date(2024, 7, 12);
        let (start, end) = window_bounds(date(2024, 7, 10), 14, min, max);
        assert_eq!((start, end), (min, max));
        assert_eq!(window_start(date(2024, 1, 1), 1, min, max), min);
    }

    #[test]
    fn ensure_visible_moves_the_anchor_by_the_minimum() {
        let min = date(2024, 1, 1);
        let max = date(2025, 12, 31);
        assert_eq!(
            ensure_visible(date(2024, 7, 15), 7, date(2024, 7, 17), min, max),
            date(2024, 7, 15)
        );
        assert_eq!(
            ensure_visible(date(2024, 7, 15), 7, date(2024, 7, 25), min, max),
            date(2024, 7, 19)
        );
        assert_eq!(
            ensure_visible(date(2024, 7, 15), 7, date(2024, 7, 1), min, max),
            date(2024, 7, 1)
        );
        assert_eq!(
            ensure_visible(date(2024, 7, 15), 7, date(2024, 7, 7), min, max),
            date(2024, 7, 7)
        );
    }

    #[test]
    fn selectable_stepping_skips_disabled_days_and_stops_at_bounds() {
        let disabled = |date: Date| date.day() != 16 && date.day() != 17;
        let config = Config {
            min_date: date(2024, 7, 1),
            max_date: date(2024, 7, 31),
            selectable: Some(&disabled),
            span: 7,
            enabled: true,
        };
        assert_eq!(
            config.step_selectable(date(2024, 7, 15), 1, 10),
            Some(date(2024, 7, 18))
        );
        assert_eq!(
            config.step_selectable(date(2024, 7, 18), -1, 10),
            Some(date(2024, 7, 15))
        );
        assert_eq!(config.step_selectable(date(2024, 7, 31), 1, 10), None);
        assert_eq!(config.step_selectable(date(2024, 7, 1), -1, 10), None);
        assert!(!config.day_selectable(date(2024, 7, 16)));
    }

    #[test]
    fn paging_clamps_to_bounds_and_skips_disabled_days() {
        let config = Config {
            min_date: date(2024, 7, 1),
            max_date: date(2024, 9, 30),
            selectable: None,
            span: 7,
            enabled: true,
        };
        assert_eq!(
            config.page_month(date(2024, 7, 31), 1),
            Some(date(2024, 8, 31))
        );
        assert_eq!(
            config.page_month(date(2024, 8, 31), -1),
            Some(date(2024, 7, 31))
        );
        assert_eq!(config.page_month(date(2024, 9, 15), 1), None);
        assert_eq!(config.page_month(date(2024, 7, 15), -1), None);

        let only_late = |date: Date| date.day() >= 20;
        let config = Config {
            min_date: date(2024, 7, 1),
            max_date: date(2024, 9, 30),
            selectable: Some(&only_late),
            span: 7,
            enabled: true,
        };
        assert_eq!(
            config.page_month(date(2024, 7, 15), 1),
            Some(date(2024, 8, 20))
        );
    }

    #[test]
    fn state_accessors_and_normalization() {
        let mut state = LineCalendarState::new(date(2024, 7, 15), Some(date(2024, 7, 18)));
        assert_eq!(state.anchor(), date(2024, 7, 15));
        assert_eq!(state.selected(), Some(date(2024, 7, 18)));
        assert_eq!(state.focused(), None);
        assert_eq!(state.visible_month(), date(2024, 7, 1));
        assert!(state.set_anchor(date(2024, 8, 1)));
        assert_eq!(state.visible_month(), date(2024, 8, 1));
        assert!(!state.set_anchor(date(2024, 8, 1)));
        assert!(state.set_selected(None));
        assert!(!state.set_selected(None));
        assert!(state.set_focused(Some(date(2024, 8, 2))));
        assert_eq!(state.focused(), Some(date(2024, 8, 2)));
    }

    #[test]
    fn labels_have_english_defaults() {
        assert_eq!(english_day_label(date(2024, 7, 15)), "July 15, 2024");
        let labels = LineCalendarLabels::english();
        assert_eq!(labels.list, "Date strip");
        assert_eq!(labels.today, "Today");
        assert_eq!(labels.weekday_names[Weekday::Monday.index()], "Mon");
        assert_eq!(labels.weekday_names[Weekday::Sunday.index()], "Sun");
    }
}
