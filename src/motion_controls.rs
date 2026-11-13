//! Compact, caller-controlled widgets with spring feedback.
use crate::{
    Theme,
    liquid::{self, DropletPart, LiquidStyle},
    motion,
};
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
            let extent = 4.0 * motion::press_scale(stretch);
            line((-extent, -extent), (extent, extent));
            line((-extent, extent), (extent, -extent));
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

/// The Temivu press phase for an arbitrary enabled egui control response.
pub fn press_amount(response: &Response, theme: &Theme) -> f32 {
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
    response
        .widget_info(|| egui::WidgetInfo::labeled(egui::Role::Button, response.enabled(), label));
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
    let side = height(theme);
    let available = ui.available_width().max(1.0);
    let collapsed_width = side.min(available);
    let id = ui.next_auto_id().with("quantity");
    let style = LiquidStyle::droplet();
    let expanded = *value > min || min == max;
    let phase = liquid::droplet_group_progress(ui.ctx(), id, expanded, style, theme).min(1.0);
    let count_width = label_width(ui, theme, &[&min.to_string(), &max.to_string()]) + 12.0;
    let full_width = (2.0 * side + count_width).min(available);
    let width = egui::lerp(collapsed_width..=full_width, phase);
    let button_width = egui::lerp(collapsed_width..=side.min(full_width * 0.40), phase);
    let (rect, _) = ui.allocate_exact_size(vec2(width, side), Sense::hover());
    let increase_rect = rect.with_min_x(rect.right() - button_width);
    let corners = (side * 0.5) as u8;
    let frame = egui::Frame::new()
        .fill(theme.muted)
        .stroke(Stroke::new(1.0, theme.border))
        .corner_radius(corners);
    if phase > 0.001 && phase < 0.999 && width > button_width + 1.0 {
        // The count and decrement button grow as a lobe from the plus button.
        let height = side * egui::lerp(0.40..=1.0, phase);
        let lobe = Rect::from_center_size(
            egui::pos2(
                (rect.left() + increase_rect.center().x) * 0.5,
                rect.center().y,
            ),
            vec2(increase_rect.center().x - rect.left(), height),
        );
        let parts = [
            DropletPart {
                rect: lobe,
                corners: ((height * 0.5) as u8).into(),
            },
            DropletPart {
                rect: increase_rect,
                corners: corners.into(),
            },
            DropletPart {
                rect: Rect::from_center_size(rect.center(), vec2(width, side * phase.powi(3))),
                corners: corners.into(),
            },
        ];
        let smoothing = style.blend_radius * (1.0 - phase) * phase * 3.0;
        let envelope = rect
            .with_max_x(rect.left() + full_width)
            .expand(style.blend_radius * 0.5 + 2.0);
        liquid::paint_joined_parts(ui.painter(), parts, envelope, frame, smoothing);
    } else {
        ui.painter().rect(
            rect,
            corners,
            frame.fill,
            frame.stroke,
            egui::StrokeKind::Inside,
        );
    }

    let button = |ui: &mut Ui, rect: Rect, id: Id, label: &str, glyph, enabled: bool| {
        let mut builder = egui::UiBuilder::new().id_salt(id).max_rect(rect);
        if !enabled {
            builder = builder.disabled();
        }
        let response = ui.new_child(builder).interact(rect, id, Sense::click());
        response.widget_info(|| {
            egui::WidgetInfo::labeled(egui::Role::Button, response.enabled(), label)
        });
        let press = press_amount(&response, theme);
        let hover = motion::animate_bool(ui.ctx(), id.with("hover"), response.hovered(), theme);
        let visual = rect.scale_from_center(motion::press_scale(press));
        let painter = ui.painter().with_clip_rect(rect.expand(2.0));
        let fill = theme.secondary.gamma_multiply(hover);
        painter.add(motion::press_shape(
            egui::epaint::RectShape::filled(visual, corners, fill),
            press,
        ));
        if response.has_focus() {
            painter.rect_stroke(
                visual.shrink(1.0),
                corners,
                Stroke::new(2.0, theme.foreground),
                egui::StrokeKind::Inside,
            );
        }
        let ink = theme
            .foreground
            .gamma_multiply(if response.enabled() { 1.0 } else { 0.46 });
        icon(&painter, visual.center(), glyph, 0.0, ink);
        response
    };

    let leading_width = width - button_width;
    let decrease_id = id.with("decrease");
    let restore_focus = !expanded && ui.memory(|memory| memory.has_focus(decrease_id));
    let decrease_width = side.min(leading_width - count_width.min(leading_width * 0.45));
    let decrease_rect = rect.with_max_x(rect.left() + decrease_width);
    if expanded && decrease_width >= button_width * 0.55 {
        let decrease = button(
            ui,
            decrease_rect,
            decrease_id,
            decrease_label,
            Icon::Minus,
            *value > min,
        );
        if decrease.clicked() {
            *value = value.saturating_sub(1);
            if *value == min && decrease.has_focus() {
                ui.memory_mut(|memory| memory.request_focus(id.with("increase")));
            }
        }
    } else {
        // Keep subsequent widgets' automatic IDs stable as the minus folds away.
        ui.skip_ahead_auto_ids(1);
        if decrease_width > 2.0 {
            icon(
                &ui.painter().with_clip_rect(decrease_rect),
                decrease_rect.center(),
                Icon::Minus,
                0.0,
                theme.foreground.gamma_multiply(phase),
            );
        }
    }

    let increase = button(
        ui,
        increase_rect,
        id.with("increase"),
        increase_label,
        Icon::Plus,
        *value < max,
    );
    if restore_focus {
        increase.request_focus();
    }
    if increase.clicked() {
        *value = value.saturating_add(1);
    }
    let label = value.to_string();
    ui.ctx()
        .accesskit_node_builder(increase.id, |node| node.set_description(label.as_str()));
    let count = rect
        .with_min_x(decrease_rect.right())
        .with_max_x(increase_rect.left());
    if count.width() > 2.0 {
        let previous = ui.ctx().data(|data| data.get_temp::<i32>(id.with("value")));
        let direction = if previous.is_some_and(|old| old > *value) {
            -1.0
        } else {
            1.0
        };
        label_reel(
            ui,
            theme,
            id.with("count"),
            count,
            &label,
            theme.foreground.gamma_multiply(phase),
            direction,
        );
    }
    ui.ctx()
        .data_mut(|data| data.insert_temp(id.with("value"), *value));
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
    response
        .widget_info(|| egui::WidgetInfo::labeled(egui::Role::Button, response.enabled(), label));
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
        egui::WidgetInfo::selected(egui::Role::Button, response.enabled(), *selected, label)
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
            egui::WidgetInfo::labeled(egui::Role::Button, response.enabled(), labels[index])
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

// The two circular ends share straight tangents. The trailing end narrows
// during travel and returns to the same capsule at either endpoint.
fn search_surface(painter: &egui::Painter, rect: Rect, phase: f32, fill: Color32, stroke: Stroke) {
    use egui::epaint::{PathShape, PathStroke};

    let radius = rect.size().min_elem() * 0.5;
    let taper = 4.0 * phase * (1.0 - phase);
    let leading = rect.left_center() + vec2(radius, 0.0);
    let trailing_radius = radius * (1.0 - 0.24 * taper);
    let trailing = rect.right_center() - vec2(trailing_radius, 0.0);
    let distance = trailing.x - leading.x;
    let delta = radius - trailing_radius;
    if delta < 0.1 || distance <= delta + 0.1 {
        painter.rect(
            rect,
            radius.round() as u8,
            fill,
            stroke,
            egui::StrokeKind::Inside,
        );
        return;
    }

    let tangent = (delta / distance).acos();
    let mut points = Vec::with_capacity(26);
    for (center, radius, from, to) in [
        (trailing, trailing_radius, -tangent, tangent),
        (leading, radius, tangent, std::f32::consts::TAU - tangent),
    ] {
        points.extend(
            (0..=12).map(|step| {
                center + Vec2::angled(egui::lerp(from..=to, step as f32 / 12.0)) * radius
            }),
        );
    }
    painter.add(PathShape::convex_polygon(
        points,
        fill,
        PathStroke::from(stroke).with_kind(egui::StrokeKind::Inside),
    ));
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
    let side = height(theme);
    let max_width = ui.available_width().clamp(1.0, 320.0);
    let start_width = side.min(max_width);
    let opening = *open;
    let resize = liquid::DropletResize::new(
        ui.ctx(),
        id.with("size"),
        vec2(start_width, side),
        vec2(max_width, side),
        opening,
        theme,
    );
    let width = resize.size.x.clamp(start_width, max_width);
    let expanded = resize.phase;
    let (rect, _) = ui.allocate_exact_size(vec2(width, side), Sense::hover());
    let trigger_rect = rect.with_max_x(rect.left() + start_width);
    let reveal_start = (start_width + 38.0).min(max_width * 0.9).max(start_width);
    let show_field = *open && width >= reveal_start;
    let focused = ui.memory(|memory| memory.focused());
    let press = if !show_field {
        let trigger = ui.interact(trigger_rect, trigger_id, Sense::click());
        trigger.widget_info(|| {
            egui::WidgetInfo::labeled(egui::Role::Button, trigger.enabled(), label)
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
        press_amount(&trigger, theme)
    } else {
        motion::press(ui.ctx(), trigger_id, false, false, theme)
    };
    let drawn = rect.scale_from_center(1.0 - 0.05 * press);
    let focused_field = *open && (focused == Some(input_id) || focused == Some(close_id));
    let focus = motion::animate_bool(ui.ctx(), id.with("focus"), focused_field, theme);
    let border = Stroke::new(
        1.0 + 0.5 * focus,
        mix(theme.border, theme.foreground, focus),
    );
    search_surface(
        ui.painter(),
        drawn,
        if opening { expanded } else { 0.0 },
        mix(theme.muted, theme.card, expanded),
        border,
    );

    let leading = egui::lerp(start_width..=start_width.min(max_width * 0.36), expanded);
    let icon_center = drawn.left_center() + vec2(leading * 0.5, 0.0);
    icon(
        &ui.painter().with_clip_rect(drawn),
        icon_center,
        Icon::Search,
        0.0,
        theme.foreground,
    );

    let close_width = (side - 6.0).min(width * 0.34);
    let close_rect = rect.with_min_x(rect.right() - close_width);
    let field_rect = Rect::from_min_max(
        rect.min + vec2(leading, 3.0),
        close_rect.left_bottom() - vec2(2.0_f32.min(width * 0.05), 3.0),
    );
    let reveal = if max_width <= start_width {
        expanded
    } else {
        crate::Easing::EaseInOut.apply((expanded - 0.06) / 0.55)
    };
    let mut close_press = 0.0;
    let mut close_hover = false;
    let mut close_focused = false;
    let response = if show_field && field_rect.is_positive() {
        // Overlay the allocated capsule without advancing the parent cursor again.
        let mut input_ui = ui.new_child(
            egui::UiBuilder::new()
                .scope_id(id.with("field"))
                .max_rect(field_rect)
                .layout(egui::Layout::centered_and_justified(
                    egui::Direction::TopDown,
                )),
        );
        input_ui.set_clip_rect(ui.clip_rect().intersect(field_rect));
        input_ui.multiply_opacity(reveal);
        let field = input_ui.add(
            egui::TextEdit::singleline(query)
                .id(input_id)
                .hint_text(label)
                .margin(egui::Margin::ZERO)
                .align(egui::Align2::LEFT_CENTER)
                .desired_width(field_rect.width())
                .frame(egui::Frame::NONE),
        );
        let close = ui.interact(close_rect, close_id, Sense::click());
        ui.ctx()
            .accesskit_node_builder(field.id, |node| node.set_label(label));
        close.widget_info(|| {
            egui::WidgetInfo::labeled(egui::Role::Button, close.enabled(), close_label)
        });
        if pending_focus || focused == Some(trigger_id) {
            field.request_focus();
            pending_focus = false;
        }
        close_press = press_amount(&close, theme);
        close_hover = close.hovered();
        close_focused = close.has_focus();
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
        Some(field)
    } else {
        // Match the input child UI's ID allocation while the field is hidden.
        ui.skip_ahead_auto_ids(1);
        if !*open && focused == Some(trigger_id) {
            ui.painter().circle_stroke(
                icon_center,
                (start_width.min(side) * 0.5 - 1.0).max(0.0),
                Stroke::new(
                    1.5,
                    theme
                        .foreground
                        .gamma_multiply(crate::Easing::EaseInOut.apply(1.0 - expanded / 0.24)),
                ),
            );
        }
        let preview_opacity = if opening {
            reveal
        } else {
            crate::Easing::EaseInOut.apply((expanded - 0.50) / 0.50)
        };
        if field_rect.is_positive() && preview_opacity > 0.0 {
            let (preview, ink) = if query.is_empty() {
                (label, theme.muted_foreground)
            } else {
                (query.as_str(), theme.foreground)
            };
            ui.painter().with_clip_rect(field_rect).text(
                field_rect.left_center(),
                Align2::LEFT_CENTER,
                preview,
                font(theme),
                ink.gamma_multiply(preview_opacity),
            );
        }
        None
    };
    let hover = motion::animate_bool(ui.ctx(), close_id.with("hover"), close_hover, theme);
    let close_clip = close_rect.shrink2(vec2(3.0, 8.0)).intersect(drawn);
    let close_opacity = if opening {
        reveal
    } else {
        crate::Easing::EaseInOut.apply((expanded - 0.72) / 0.28)
    };
    if close_opacity > 0.0 && close_clip.width() >= 10.0 {
        let ink =
            mix(theme.muted_foreground, theme.foreground, hover).gamma_multiply(close_opacity);
        let painter = ui.painter().with_clip_rect(close_clip);
        let center = close_rect.center() + vec2(2.0 * (1.0 - reveal), 0.0);
        icon(&painter, center, Icon::Close, close_press, ink);
        if close_focused {
            painter.circle_stroke(center, 9.0, Stroke::new(1.5, ink));
        }
    }
    ui.ctx()
        .data_mut(|data| data.insert_temp(id, pending_focus));
    response
}
