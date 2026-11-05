//! Compact, caller-controlled widgets with spring feedback.
use crate::{Theme, motion};
use egui::{Align2, Color32, FontId, Id, Key, Pos2, Rect, Response, Sense, Stroke, Ui, Vec2, vec2};
use std::ops::RangeInclusive;

/// The three presentation modes offered by [`layout_toggle`].
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub enum ViewMode {
    #[default]
    Column,
    Grid,
    List,
}

impl ViewMode {
    fn index(self) -> usize {
        self as usize
    }
}

/// Direction reported by [`direction_pad`].
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum Direction {
    Up,
    Left,
    Right,
    Down,
}

#[derive(Clone, Copy)]
enum Icon {
    Plus,
    Minus,
    Check,
    Search,
    Close,
    Arrow(Direction),
    Layout(ViewMode),
}

fn mix(from: Color32, to: Color32, amount: f32) -> Color32 {
    Color32::from(egui::Rgba::from(from) * (1.0 - amount) + egui::Rgba::from(to) * amount)
}

fn height(theme: &Theme) -> f32 {
    if theme.touch { 48.0 } else { 42.0 }
}

fn font(theme: &Theme) -> FontId {
    FontId::proportional(theme.typography.body.size)
}

fn icon(painter: &egui::Painter, center: Pos2, icon: Icon, stretch: f32, color: Color32) {
    let stroke = Stroke::new(1.6, color);
    let line = |a: (f32, f32), b: (f32, f32)| {
        painter.line_segment([center + vec2(a.0, a.1), center + vec2(b.0, b.1)], stroke);
    };
    match icon {
        Icon::Plus | Icon::Minus => {
            line((-5.0, 0.0), (5.0, 0.0));
            if matches!(icon, Icon::Plus) {
                line((0.0, -5.0), (0.0, 5.0));
            }
        }
        Icon::Check => {
            line((-5.0, 0.0), (-1.5, 3.5));
            line((-1.5, 3.5), (5.5, -4.0));
        }
        Icon::Search => {
            painter.circle_stroke(center - Vec2::splat(1.5), 4.5, stroke);
            line((2.0, 2.0), (6.0, 6.0));
        }
        Icon::Close => {
            line((-4.0, -4.0), (4.0, 4.0));
            line((-4.0, 4.0), (4.0, -4.0));
        }
        Icon::Arrow(direction) => {
            let along = match direction {
                Direction::Up => -Vec2::Y,
                Direction::Left => -Vec2::X,
                Direction::Right => Vec2::X,
                Direction::Down => Vec2::Y,
            };
            let across = along.rot90();
            let tip = center + along * (4.5 + 2.0 * stretch);
            painter.line_segment([center - along * (5.0 + 2.0 * stretch), tip], stroke);
            painter.line_segment([tip - along * 4.0 + across * 4.0, tip], stroke);
            painter.line_segment([tip - along * 4.0 - across * 4.0, tip], stroke);
        }
        Icon::Layout(mode) => match mode {
            ViewMode::List => {
                for y in [-4.0, 0.0, 4.0] {
                    line((-5.5, y), (5.5, y));
                }
            }
            ViewMode::Column | ViewMode::Grid => {
                let cell = |x, y, h| {
                    painter.rect_filled(
                        Rect::from_min_size(center + vec2(x, y), vec2(4.0, h)),
                        1,
                        color,
                    );
                };
                if mode == ViewMode::Column {
                    cell(-5.0, -5.0, 10.0);
                    cell(1.0, -5.0, 10.0);
                } else {
                    for x in [-5.0, 1.0] {
                        cell(x, -5.0, 4.0);
                        cell(x, 1.0, 4.0);
                    }
                }
            }
        },
    }
}

pub(crate) fn press_amount(response: &Response, theme: &Theme) -> f32 {
    let pressed = response.enabled()
        && (response.is_pointer_button_down_on()
            || (response.has_focus()
                && response
                    .ctx
                    .input(|input| input.key_down(Key::Space) || input.key_down(Key::Enter))));
    motion::press(
        &response.ctx,
        response.id,
        pressed,
        response.clicked(),
        theme,
    )
}

pub(crate) fn paint_pill(
    ui: &Ui,
    theme: &Theme,
    response: &Response,
    press: f32,
    active: f32,
) -> (Rect, Color32) {
    let hover = motion::animate_bool(
        ui.ctx(),
        response.id.with("hover"),
        response.hovered(),
        theme,
    );
    let rect = response.rect;
    let radius = (rect.height() * 0.5).round() as u8;
    let surface = rect.scale_from_center(motion::press_scale(press));
    motion::remember_anchor(response, surface);
    let fill = mix(
        mix(theme.muted, theme.secondary, hover),
        theme.primary,
        active,
    );
    let ink = mix(theme.foreground, theme.primary_foreground, active);
    let opacity = if response.enabled() { 1.0 } else { 0.46 };
    ui.painter().add(motion::press_shape(
        egui::epaint::RectShape::filled(surface, radius, fill.gamma_multiply(opacity)),
        press,
    ));
    if response.has_focus() {
        ui.painter().add(motion::press_shape(
            egui::epaint::RectShape::stroke(
                surface.expand(1.0),
                radius,
                Stroke::new(2.0, theme.foreground),
                egui::StrokeKind::Outside,
            ),
            press,
        ));
    }
    (surface, ink.gamma_multiply(opacity))
}

pub(crate) fn button_response(ui: &mut Ui, size: Vec2) -> Response {
    let (rect, allocation) = ui.allocate_exact_size(size, Sense::hover());
    ui.interact(rect, allocation.id.with("button"), Sense::click())
}

fn icon_button(ui: &mut Ui, theme: &Theme, label: &str, glyph: Icon, enabled: bool) -> Response {
    let side = height(theme).min(ui.available_width().max(1.0));
    let response = ui
        .add_enabled_ui(enabled, |ui| button_response(ui, Vec2::splat(side)))
        .inner;
    response.widget_info(|| {
        egui::WidgetInfo::labeled(egui::WidgetType::Button, response.enabled(), label)
    });
    let press = press_amount(&response, theme);
    let (rect, ink) = paint_pill(ui, theme, &response, press, 0.0);
    let stretch = motion::spring_bool(
        ui.ctx(),
        response.id.with("stretch"),
        response.hovered() || response.has_focus(),
        theme,
    );
    icon(
        ui.painter(),
        rect.center(),
        glyph,
        (stretch + press).clamp(0.0, 1.4),
        ink,
    );
    response
}

fn label_width(ui: &Ui, theme: &Theme, labels: &[&str]) -> f32 {
    labels
        .iter()
        .map(|label| {
            ui.painter()
                .layout_no_wrap((*label).to_owned(), font(theme), theme.foreground)
                .size()
                .x
        })
        .fold(0.0, f32::max)
}

// Slide between discrete labels without animating the underlying value.
fn label_reel(
    ui: &Ui,
    theme: &Theme,
    id: Id,
    rect: Rect,
    label: &str,
    color: Color32,
    direction: f32,
) {
    #[derive(Clone)]
    struct Reel {
        current: String,
        previous: String,
        direction: f32,
    }
    let mut reel = ui
        .ctx()
        .data(|data| data.get_temp::<Reel>(id))
        .unwrap_or_else(|| Reel {
            current: label.to_owned(),
            previous: label.to_owned(),
            direction,
        });
    if reel.current != label {
        reel.previous = std::mem::replace(&mut reel.current, label.to_owned());
        reel.direction = direction;
        motion::reset(ui.ctx(), id.with("slide"));
    }
    let initial = if reel.previous == reel.current {
        1.0
    } else {
        0.0
    };
    let t = motion::spring_f32(ui.ctx(), id.with("slide"), initial, 1.0, theme).clamp(0.0, 1.0);
    let painter = ui.painter().with_clip_rect(rect);
    let distance = rect.height() * 0.43 * reel.direction;
    if t < 1.0 {
        painter.text(
            rect.center() - vec2(0.0, distance * t),
            Align2::CENTER_CENTER,
            &reel.previous,
            font(theme),
            color.gamma_multiply(1.0 - t),
        );
    } else {
        reel.previous.clone_from(&reel.current);
    }
    painter.text(
        rect.center() + vec2(0.0, distance * (1.0 - t)),
        Align2::CENTER_CENTER,
        label,
        font(theme),
        color.gamma_multiply(t),
    );
    ui.ctx().data_mut(|data| data.insert_temp(id, reel));
}

/// Adjust an integer in a nonempty inclusive range. At its minimum, the
/// stepper folds down to the increment button. Returns whether the value changed.
pub fn quantity_stepper(
    ui: &mut Ui,
    theme: &Theme,
    value: &mut i32,
    range: RangeInclusive<i32>,
    decrease_label: &str,
    increase_label: &str,
) -> bool {
    let (min, max) = range.into_inner();
    if min > max {
        return false;
    }
    let before = *value;
    *value = (*value).clamp(min, max);
    let id = ui.next_auto_id().with("quantity");
    let side = height(theme);
    let progress =
        motion::spring_bool(ui.ctx(), id, *value > min || min == max, theme).clamp(0.0, 1.0);
    let widest = label_width(
        ui,
        theme,
        &[&min.to_string(), &max.to_string(), &value.to_string()],
    ) + 12.0;
    ui.horizontal(|ui| {
        ui.spacing_mut().item_spacing.x = 4.0;
        if progress > 0.015 {
            let width = ((side + widest + 4.0) * progress)
                .min((ui.available_width() - side - 4.0).max(0.0));
            if width > 0.0 {
                let (rect, _) = ui.allocate_exact_size(vec2(width, side), Sense::hover());
                {
                    let mut ui = ui.new_child(egui::UiBuilder::new().id(id).max_rect(rect));
                    ui.set_clip_rect(ui.clip_rect().intersect(rect));
                    ui.set_opacity(progress);
                    if width >= side {
                        if icon_button(&mut ui, theme, decrease_label, Icon::Minus, *value > min)
                            .clicked()
                        {
                            *value = value.saturating_sub(1);
                        }
                        let count = Rect::from_min_max(rect.min + vec2(side, 0.0), rect.max);
                        if count.is_positive() {
                            let previous =
                                ui.ctx().data(|data| data.get_temp::<i32>(id.with("value")));
                            let direction = if previous.is_some_and(|old| old > *value) {
                                -1.0
                            } else {
                                1.0
                            };
                            label_reel(
                                &ui,
                                theme,
                                id.with("count"),
                                count,
                                &value.to_string(),
                                theme.foreground,
                                direction,
                            );
                            ui.ctx()
                                .data_mut(|data| data.insert_temp(id.with("value"), *value));
                        }
                    }
                }
            }
        }
        let increase = ui
            .push_id(id.with("increase"), |ui| {
                icon_button(ui, theme, increase_label, Icon::Plus, *value < max)
            })
            .inner;
        ui.ctx()
            .accesskit_node_builder(increase.id, |node| node.set_description(value.to_string()));
        if increase.clicked() {
            *value = value.saturating_add(1);
        }
    });
    if *value != before {
        ui.ctx().request_repaint();
    }
    *value != before
}

/// Cycle through column, grid and list layouts, animating the icon and label.
pub fn layout_toggle(
    ui: &mut Ui,
    theme: &Theme,
    mode: &mut ViewMode,
    labels: [&str; 3],
) -> Response {
    const MODES: [ViewMode; 3] = [ViewMode::Column, ViewMode::Grid, ViewMode::List];
    let h = height(theme);
    let width = (label_width(ui, theme, &labels) + 52.0).min(ui.available_width().max(1.0));
    let mut response = button_response(ui, vec2(width, h));
    let rect = response.rect;
    if response.clicked() {
        *mode = MODES[(mode.index() + 1) % MODES.len()];
        response.mark_changed();
    }
    let label = labels[mode.index()];
    response.widget_info(|| {
        egui::WidgetInfo::labeled(egui::WidgetType::Button, response.enabled(), label)
    });
    let press = press_amount(&response, theme);
    let (_, ink) = paint_pill(ui, theme, &response, press, 0.0);
    let center = egui::pos2(rect.left() + 22.0, rect.center().y);
    let painter = ui.painter().with_clip_rect(rect);
    for candidate in MODES {
        let amount = motion::spring_bool(
            ui.ctx(),
            response.id.with(candidate.index()),
            candidate == *mode,
            theme,
        )
        .clamp(0.0, 1.0);
        if amount > 0.0 {
            icon(
                &painter,
                center + vec2(0.0, 8.0 * (1.0 - amount)),
                Icon::Layout(candidate),
                0.0,
                ink.gamma_multiply(amount),
            );
        }
    }
    let text = Rect::from_min_max(rect.min + vec2(38.0, 0.0), rect.max - vec2(10.0, 0.0));
    label_reel(ui, theme, response.id.with("label"), text, label, ink, 1.0);
    response
}

/// A state button for actions such as Follow/Following or Save/Saved.
pub fn toggle_button(
    ui: &mut Ui,
    theme: &Theme,
    selected: &mut bool,
    labels: [&str; 2],
) -> Response {
    let width = (label_width(ui, theme, &labels) + 55.0).min(ui.available_width().max(1.0));
    let mut response = button_response(ui, vec2(width, height(theme)));
    let rect = response.rect;
    if response.clicked() {
        *selected = !*selected;
        response.mark_changed();
    }
    let label = labels[usize::from(*selected)];
    response.widget_info(|| {
        egui::WidgetInfo::selected(
            egui::WidgetType::Button,
            response.enabled(),
            *selected,
            label,
        )
    });
    ui.ctx().accesskit_node_builder(response.id, |node| {
        node.set_toggled(if *selected {
            egui::accesskit::Toggled::True
        } else {
            egui::accesskit::Toggled::False
        });
    });
    let press = press_amount(&response, theme);
    let active = motion::spring_bool(ui.ctx(), response.id.with("selected"), *selected, theme)
        .clamp(0.0, 1.0);
    let (_, ink) = paint_pill(ui, theme, &response, press, active);
    let center = egui::pos2(rect.left() + 21.0, rect.center().y);
    let painter = ui.painter().with_clip_rect(rect);
    icon(
        &painter,
        center - vec2(0.0, 4.0 * active),
        Icon::Plus,
        0.0,
        ink.gamma_multiply(1.0 - active),
    );
    icon(
        &painter,
        center + vec2(0.0, 4.0 * (1.0 - active)),
        Icon::Check,
        0.0,
        ink.gamma_multiply(active),
    );
    let text = Rect::from_min_max(rect.min + vec2(37.0, 0.0), rect.max - vec2(10.0, 0.0));
    label_reel(
        ui,
        theme,
        response.id.with("label"),
        text,
        label,
        ink,
        if *selected { 1.0 } else { -1.0 },
    );
    response
}

/// Two arrow buttons with stretch feedback. Labels are used by assistive technology.
pub fn navigation_arrows(
    ui: &mut Ui,
    theme: &Theme,
    previous: (&str, bool),
    next: (&str, bool),
) -> (Response, Response) {
    ui.horizontal(|ui| {
        (
            icon_button(
                ui,
                theme,
                previous.0,
                Icon::Arrow(Direction::Left),
                previous.1,
            ),
            icon_button(ui, theme, next.0, Icon::Arrow(Direction::Right), next.1),
        )
    })
    .inner
}

// A rounded, convex petal pointing toward the center of the pad.
fn paint_petal(ui: &Ui, theme: &Theme, response: &Response, direction: Direction, press: f32) {
    let inward = match direction {
        Direction::Up => Vec2::Y,
        Direction::Left => Vec2::X,
        Direction::Right => -Vec2::X,
        Direction::Down => -Vec2::Y,
    };
    let across = inward.rot90();
    let center = response.rect.center() - inward * (2.0 * press);
    let side = response.rect.width();
    let point = |x: f32, y: f32| {
        center
            + across * (x * side * (1.0 + 0.08 * press))
            + inward * (y * side * (1.0 - 0.10 * press))
    };
    let outline = [
        point(-0.48, -0.49),
        point(0.48, -0.49),
        point(0.48, 0.19),
        point(0.0, 0.68),
        point(-0.48, 0.19),
    ];
    let mut rounded = Vec::with_capacity(15);
    for (index, &corner) in outline.iter().enumerate() {
        let before = outline[(index + outline.len() - 1) % outline.len()];
        let after = outline[(index + 1) % outline.len()];
        let start = corner.lerp(before, 0.22);
        let end = corner.lerp(after, 0.22);
        rounded.extend([
            start,
            start.lerp(corner, 0.5).lerp(corner.lerp(end, 0.5), 0.5),
            end,
        ]);
    }
    let hover = motion::animate_bool(
        ui.ctx(),
        response.id.with("hover"),
        response.hovered(),
        theme,
    );
    let mut fill = mix(theme.primary, theme.hover(theme.primary), hover);
    if !response.enabled() {
        fill = fill.gamma_multiply(0.46);
    }
    let outline = if response.has_focus() {
        Stroke::new(2.0, theme.foreground)
    } else {
        Stroke::NONE
    };
    ui.painter()
        .add(egui::Shape::convex_polygon(rounded, fill, outline));
    let tip = center - inward * (3.0 + press);
    for side in [-1.0, 1.0] {
        ui.painter().line_segment(
            [tip + inward * 4.0 + across * (side * 4.0), tip],
            Stroke::new(1.9, theme.primary_foreground),
        );
    }
}

/// A four-way directional control. The labels are ordered up, left, right, down.
pub fn direction_pad(ui: &mut Ui, theme: &Theme, labels: [&str; 4]) -> Option<Direction> {
    let side = height(theme).min((ui.available_width() / 3.0 - 4.0).max(1.0));
    let gap = 3.0;
    let (outer, _) = ui.allocate_exact_size(Vec2::splat(side * 3.0 + gap * 2.0), Sense::hover());
    let id = ui.next_auto_id();
    let mut clicked = None;
    let mut focused = None;
    let mut ids = [id; 4];
    for (index, (direction, x, y)) in [
        (Direction::Up, 1.0, 0.0),
        (Direction::Left, 0.0, 1.0),
        (Direction::Right, 2.0, 1.0),
        (Direction::Down, 1.0, 2.0),
    ]
    .into_iter()
    .enumerate()
    {
        let rect = Rect::from_min_size(outer.min + vec2(x, y) * (side + gap), Vec2::splat(side));
        let response = ui.interact(rect, id.with(index), Sense::click());
        ids[index] = response.id;
        response.widget_info(|| {
            egui::WidgetInfo::labeled(egui::WidgetType::Button, response.enabled(), labels[index])
        });
        let press = press_amount(&response, theme);
        paint_petal(ui, theme, &response, direction, press);
        if response.clicked() {
            clicked = Some(direction);
        }
        if response.enabled() && response.has_focus() {
            focused = Some(response.id);
        }
    }
    if let Some(id) = focused {
        ui.memory_mut(|memory| {
            memory.set_focus_lock_filter(
                id,
                egui::EventFilter {
                    horizontal_arrows: true,
                    vertical_arrows: true,
                    ..Default::default()
                },
            )
        });
        for (index, (key, direction)) in [
            (Key::ArrowUp, Direction::Up),
            (Key::ArrowLeft, Direction::Left),
            (Key::ArrowRight, Direction::Right),
            (Key::ArrowDown, Direction::Down),
        ]
        .into_iter()
        .enumerate()
        {
            if ui.input_mut(|input| input.consume_key(egui::Modifiers::NONE, key)) {
                ui.memory_mut(|memory| memory.request_focus(ids[index]));
                return Some(direction);
            }
        }
    }
    clicked
}

/// An inline search field that grows from its button. Closing preserves the query.
/// The caller controls the open state and can use the returned field response for submission.
pub fn search_box(
    ui: &mut Ui,
    theme: &Theme,
    open: &mut bool,
    query: &mut String,
    label: &str,
    close_label: &str,
) -> Option<Response> {
    let id = ui.next_auto_id().with("search");
    let trigger_id = id.with("trigger");
    let input_id = id.with("input");
    let close_id = id.with("close");
    let mut pending_focus = ui
        .ctx()
        .data(|data| data.get_temp::<bool>(id).unwrap_or(false));
    let expanded = motion::spring_bool(ui.ctx(), id, *open, theme).clamp(0.0, 1.0);
    let side = height(theme);
    let max_width = ui.available_width().clamp(1.0, 320.0);
    let start_width = side.min(max_width);
    let width = egui::lerp(start_width..=max_width, expanded);
    let (rect, _) = ui.allocate_exact_size(vec2(width, side), Sense::hover());
    let trigger_rect = Rect::from_min_size(rect.min, vec2(start_width, side));
    let compact = max_width < 54.0;
    let reveal_start = (start_width + 38.0).min(max_width * 0.9).max(54.0);
    let ready = width >= reveal_start;
    let show_field = *open && (ready || compact);
    let focused = ui.memory(|memory| memory.focused());
    let mut press = 0.0;
    if !show_field {
        let trigger = ui.interact(trigger_rect, trigger_id, Sense::click());
        trigger.widget_info(|| {
            egui::WidgetInfo::labeled(egui::WidgetType::Button, trigger.enabled(), label)
        });
        if !*open && (pending_focus || focused == Some(input_id) || focused == Some(close_id)) {
            trigger.request_focus();
            pending_focus = false;
        }
        if trigger.clicked() && !*open {
            *open = true;
            pending_focus = true;
            ui.ctx().request_repaint();
        }
        press = press_amount(&trigger, theme);
    }
    let drawn = rect.scale_from_center(1.0 - 0.05 * press);
    let radius = (side * 0.5) as u8;
    ui.painter().rect(
        drawn,
        radius,
        mix(theme.muted, theme.card, expanded),
        Stroke::new(1.0, theme.border),
        egui::StrokeKind::Inside,
    );
    let response = if show_field {
        let (field, close) = if compact {
            let field = ui.add(
                egui::TextEdit::singleline(query)
                    .id(input_id)
                    .hint_text(label)
                    .desired_width(max_width),
            );
            let close = ui.interact(trigger_rect, close_id, Sense::click());
            (field, close)
        } else {
            let close_side = (side - 6.0).min(width * 0.38);
            let close_rect = Rect::from_center_size(
                egui::pos2(rect.right() - close_side * 0.5 - 3.0, rect.center().y),
                Vec2::splat(close_side),
            );
            let field_rect = Rect::from_min_max(
                rect.min + vec2(12.0, 3.0),
                close_rect.left_bottom() - vec2(4.0, 0.0),
            );
            // Overlay the allocated capsule without advancing the parent cursor again.
            let mut input_ui = ui.new_child(
                egui::UiBuilder::new()
                    .id(id.with("field"))
                    .max_rect(field_rect)
                    .layout(egui::Layout::centered_and_justified(
                        egui::Direction::TopDown,
                    )),
            );
            input_ui.set_clip_rect(ui.clip_rect().intersect(field_rect));
            input_ui.multiply_opacity(
                ((width - reveal_start) / 32.0)
                    .clamp(0.0, 1.0)
                    .max(if expanded >= 0.99 { 1.0 } else { 0.0 }),
            );
            let field = input_ui.add(
                egui::TextEdit::singleline(query)
                    .id(input_id)
                    .hint_text(label)
                    .margin(egui::Margin::ZERO)
                    .vertical_align(egui::Align::Center)
                    .desired_width(field_rect.width())
                    .frame(egui::Frame::NONE),
            );
            let close = ui.interact(close_rect, close_id, Sense::click());
            (field, close)
        };
        ui.ctx()
            .accesskit_node_builder(field.id, |node| node.set_label(label));
        close.widget_info(|| {
            egui::WidgetInfo::labeled(egui::WidgetType::Button, close.enabled(), close_label)
        });
        if pending_focus || focused == Some(trigger_id) {
            field.request_focus();
            pending_focus = false;
        }
        let close_press = press_amount(&close, theme);
        let (visual, ink) = paint_pill(ui, theme, &close, close_press, 0.0);
        icon(ui.painter(), visual.center(), Icon::Close, 0.0, ink);
        let escape = (field.has_focus() || field.lost_focus() || close.has_focus())
            && !ui.ctx().any_popup_open()
            && ui.input_mut(|input| input.consume_key(egui::Modifiers::NONE, Key::Escape));
        if close.clicked() || escape {
            *open = false;
            pending_focus = true;
            field.surrender_focus();
            close.surrender_focus();
            ui.ctx().request_repaint();
        }
        if field.has_focus() {
            ui.painter().rect_stroke(
                drawn,
                radius,
                Stroke::new(1.5, theme.foreground),
                egui::StrokeKind::Inside,
            );
        }
        Some(field)
    } else {
        icon(
            ui.painter(),
            trigger_rect.center(),
            Icon::Search,
            0.0,
            theme.foreground,
        );
        if ui.memory(|memory| memory.has_focus(trigger_id)) {
            ui.painter().rect_stroke(
                trigger_rect.expand(1.0),
                radius,
                Stroke::new(2.0, theme.foreground),
                egui::StrokeKind::Outside,
            );
        }
        let preview = if query.is_empty() {
            label
        } else {
            query.as_str()
        };
        let clip = rect
            .with_min_x(trigger_rect.right())
            .shrink2(vec2(4.0, 0.0));
        if clip.is_positive() {
            ui.painter().with_clip_rect(clip).text(
                clip.left_center(),
                Align2::LEFT_CENTER,
                preview,
                font(theme),
                theme.muted_foreground.gamma_multiply(expanded),
            );
        }
        None
    };
    ui.ctx()
        .data_mut(|data| data.insert_temp(id, pending_focus));
    response
}
