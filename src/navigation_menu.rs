//! Responsive navigation surfaces using egui's modal/popover focus and dismissal.
use crate::{Button, ButtonVariant, Dialog, NavigationItem, Popover, SurfaceShape, Theme, Vesper};
use egui::{Id, Rect, Response, Sense, Stroke, Ui, Vec2, pos2, vec2};

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum MenuPresentation {
    /// A left drawer on narrow viewports, a floating island on wider viewports.
    #[default]
    Adaptive,
    FullScreen,
    /// A familiar left-edge navigation drawer with a sliding spring transition.
    Drawer,
    Floating,
}

/// A caller-controlled navigation menu. Call every frame for focus restoration.
/// Reuses the same destinations as bottom navigation; no platform-specific code.
pub struct NavigationMenu<'a> {
    id: Id,
    title: &'a str,
    open: &'a mut bool,
    theme: &'a Theme,
    presentation: MenuPresentation,
    width: f32,
    frame: Option<egui::Frame>,
    liquid: Option<crate::liquid::LiquidStyle>,
    lighting: Option<Option<Vesper>>,
}

impl<'a> NavigationMenu<'a> {
    pub fn new(id: Id, title: &'a str, open: &'a mut bool, theme: &'a Theme) -> Self {
        Self {
            id,
            title,
            open,
            theme,
            presentation: MenuPresentation::Adaptive,
            width: 320.0,
            frame: None,
            liquid: None,
            lighting: None,
        }
    }
    /// Override the context's popup/window animation preset.
    pub fn liquid(mut self, motion: crate::liquid::LiquidStyle) -> Self {
        self.liquid = Some(motion);
        self
    }
    pub fn presentation(mut self, presentation: MenuPresentation) -> Self {
        self.presentation = presentation;
        self
    }
    pub fn width(mut self, width: f32) -> Self {
        if width.is_finite() {
            self.width = width.max(1.0);
        }
        self
    }
    /// Customize colors, border, corners, spacing and shadow with egui's frame.
    pub fn frame(mut self, frame: egui::Frame) -> Self {
        self.frame = Some(frame);
        self
    }
    /// Override the inherited Vesper material for menu destinations.
    pub fn lighting(mut self, lighting: Option<Vesper>) -> Self {
        self.lighting = Some(lighting);
        self
    }

    /// Clicking the trigger toggles the menu. Selecting an enabled destination
    /// updates `selected`, closes the menu and returns the chosen destination.
    pub fn show(
        self,
        anchor: &Response,
        items: &[NavigationItem<'_>],
        selected: &mut Option<Id>,
    ) -> Option<Id> {
        if anchor.clicked() {
            anchor.request_focus();
            *self.open = !*self.open;
        }
        let was_open = *self.open;
        let viewport = anchor.ctx.content_rect();
        let lighting = self
            .lighting
            .unwrap_or_else(|| Vesper::context(&anchor.ctx));
        let motion = self
            .liquid
            .unwrap_or_else(|| crate::liquid::LiquidStyle::context_default(&anchor.ctx));
        let drawer = self.presentation == MenuPresentation::Drawer
            || (self.presentation == MenuPresentation::Adaptive && viewport.width() < 640.0);
        let full = self.presentation == MenuPresentation::FullScreen || drawer;
        let mut chosen = None;
        if full {
            let inset = 8.0_f32.min(viewport.size().min_elem() * 0.06);
            let drawer_size = drawer.then(|| {
                let safe_size = viewport.size() - Vec2::splat(2.0 * inset);
                vec2(
                    self.width.min(viewport.width() * 0.86).min(safe_size.x),
                    safe_size.y,
                )
                .max(Vec2::splat(1.0))
            });
            let frame = self.frame.unwrap_or_else(|| {
                let size = drawer_size.unwrap_or_else(|| viewport.size());
                let padding = if drawer {
                    (size.min_elem() * 0.06).clamp(3.0, 22.0) as i8
                } else {
                    18
                };
                let radius = SurfaceShape::Panel.radius(size);
                crate::surface::popup_frame(self.theme, padding).corner_radius(if drawer {
                    (radius + 8).min((size.min_elem() * 0.5) as u8)
                } else {
                    radius
                })
            });
            let mut dialog = Dialog::new(self.id, self.title)
                .theme(self.theme)
                .liquid(motion)
                .source_response(anchor)
                .frame(frame);
            if drawer {
                dialog = dialog.slide_from_left().area(
                    egui::Modal::default_area(self.id)
                        .anchor(egui::Align2::LEFT_TOP, Vec2::splat(inset)),
                );
            }
            dialog.show(&anchor.ctx, self.open, |ui| {
                let size = drawer_size
                    .map_or_else(
                        || viewport.size() - Vec2::splat(96.0),
                        |size| size - frame.total_margin().sum(),
                    )
                    .max(Vec2::ZERO);
                ui.set_width(size.x);
                ui.set_min_height(size.y);
                menu_contents(
                    ui,
                    self.id,
                    MenuContent {
                        title: self.title,
                        theme: self.theme,
                        items,
                        selected: *selected,
                        lighting,
                        drawer,
                    },
                    &mut chosen,
                    if drawer {
                        size.y
                    } else {
                        (size.y - 96.0).max(48.0)
                    },
                );
            });
        } else {
            Popover::new(self.id, self.title, self.open)
                .match_compact_trigger()
                .theme(self.theme)
                .liquid(motion)
                .width(self.width)
                .frame(
                    self.frame
                        .unwrap_or_else(|| crate::surface::popup_frame(self.theme, 10)),
                )
                .focus_on_open(true)
                .show(anchor, |ui| {
                    menu_contents(
                        ui,
                        self.id,
                        MenuContent {
                            title: self.title,
                            theme: self.theme,
                            items,
                            selected: *selected,
                            lighting,
                            drawer: false,
                        },
                        &mut chosen,
                        (viewport.height() - 112.0).clamp(44.0, 440.0),
                    )
                });
        }
        if was_open && !*self.open {
            anchor.request_focus();
            anchor.ctx.request_repaint();
        }
        if let Some(id) = chosen {
            *selected = Some(id);
            *self.open = false;
            anchor.request_focus();
            anchor.ctx.request_repaint();
        }
        chosen
    }
}

struct MenuContent<'a> {
    title: &'a str,
    theme: &'a Theme,
    items: &'a [NavigationItem<'a>],
    selected: Option<Id>,
    lighting: Option<Vesper>,
    drawer: bool,
}

fn menu_contents(
    ui: &mut Ui,
    id: Id,
    view: MenuContent<'_>,
    chosen: &mut Option<Id>,
    max_height: f32,
) -> Response {
    let MenuContent {
        title,
        theme,
        items,
        selected,
        lighting,
        drawer,
    } = view;
    let top = ui.next_widget_position().y;
    let close = ui
        .horizontal(|ui| {
            ui.spacing_mut().item_spacing.x = 4.0;
            ui.label(
                egui::RichText::new(title)
                    .size(if drawer { 21.0 } else { 16.0 })
                    .strong()
                    .color(theme.foreground),
            );
            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                let close = Button::new("Close", theme)
                    .variant(ButtonVariant::Ghost)
                    .lighting(lighting)
                    .show(ui);
                if close.clicked() {
                    ui.close();
                }
                close
            })
            .inner
        })
        .inner;
    let max_height = if drawer {
        if max_height >= 150.0 {
            ui.add_space(8.0);
            ui.label(
                egui::RichText::new("Find your next destination.").color(theme.muted_foreground),
            );
            ui.add_space(14.0);
        }
        (max_height - (ui.next_widget_position().y - top)).max(0.0)
    } else {
        let separator = ui.available_rect_before_wrap();
        ui.painter().hline(
            (separator.left() + 5.0)..=(separator.right() - 5.0),
            separator.top() + 2.0,
            Stroke::new(1.0, theme.border.gamma_multiply(0.75)),
        );
        ui.add_space(6.0);
        max_height
    };

    let mut responses = Vec::with_capacity(items.len());
    let mut direction = None;
    let min_height: f32 = match (drawer, theme.touch) {
        (true, true) => 54.0,
        (true, false) | (false, true) => 48.0,
        (false, false) => 42.0,
    };
    egui::ScrollArea::vertical()
        .id_salt(id.with("destinations"))
        .max_height(max_height)
        .auto_shrink([false, true])
        .show(ui, |ui| {
            ui.spacing_mut().item_spacing.y = if drawer { 8.0 } else { 1.0 };
            let rtl = ui.layout().prefer_right_to_left();
            let has_icons = items.iter().any(|item| item.icon.is_some());
            for item in items {
                let active = selected == Some(item.id);
                let enabled = ui.is_enabled() && item.enabled;
                let foreground = if drawer && active {
                    theme.primary_foreground
                } else {
                    theme.foreground
                };
                let color = if enabled {
                    foreground
                } else {
                    theme.disable(foreground)
                };
                let icon_space = if has_icons { 26.0 } else { 0.0 };
                let width = ui.available_width();
                let mut job = egui::text::LayoutJob::simple(
                    item.label.to_owned(),
                    egui::FontId::proportional(15.0),
                    color,
                    (width - 20.0 - icon_space - if drawer { 0.0 } else { 18.0 }).max(1.0),
                );
                job.wrap.max_rows = 2;
                job.wrap.break_anywhere = true;
                let label = ui.painter().layout_job(job);
                let height = min_height.max(label.size().y + 14.0);
                let rect = ui
                    .allocate_exact_size(vec2(width, height), Sense::hover())
                    .0;
                let response = ui
                    .add_enabled_ui(enabled, |ui| {
                        ui.interact(rect, id.with(item.id), Sense::click())
                    })
                    .inner;
                response.widget_info(|| {
                    egui::WidgetInfo::labeled(egui::Role::Button, enabled, item.label)
                });
                let press = crate::motion_controls::press_amount(&response, theme);
                let hover = crate::motion::animate_bool(
                    ui.ctx(),
                    response.id.with("hover"),
                    enabled && response.hovered(),
                    theme,
                );
                let surface = rect.shrink(1.0).scale_from_center(1.0 - 0.03 * press);
                let corner =
                    SurfaceShape::Card.radius(surface.size() * if drawer { 1.0 } else { 0.85 });
                if drawer || hover > 0.001 || press > 0.001 {
                    let emphasis = (hover * 0.10 + press * 0.13).clamp(0.0, 0.18);
                    let fill = if drawer {
                        let base = if active {
                            theme.primary
                        } else {
                            theme.secondary
                        };
                        let fill = base.lerp_to_gamma(foreground, emphasis);
                        if enabled { fill } else { theme.disable(fill) }
                    } else {
                        theme
                            .foreground
                            .gamma_multiply((hover * 0.06 + press * 0.09).clamp(0.0, 0.14))
                    };
                    ui.painter().add(crate::motion::press_shape(
                        egui::epaint::RectShape::filled(surface, corner, fill),
                        press,
                    ));
                }
                if let Some(light) = lighting
                    && let Some(shape) = light.interactive_shape(
                        ui.painter(),
                        theme,
                        &response,
                        surface,
                        crate::motion::press_corners(surface, corner.into(), press),
                    )
                {
                    ui.painter().add(shape);
                }
                if enabled && let Some(light) = lighting {
                    crate::lighting::spill::receive(
                        ui.painter(),
                        response.id,
                        surface,
                        crate::motion::press_corners(surface, corner.into(), press),
                        &light,
                        if active && drawer {
                            theme.primary
                        } else {
                            theme.secondary
                        },
                        theme.card,
                    );
                }
                if response.has_focus() {
                    ui.painter().rect_stroke(
                        surface,
                        corner,
                        Stroke::new(1.5, foreground),
                        egui::StrokeKind::Inside,
                    );
                }
                let y = rect.center().y;
                let offset = if rtl { -1.0 } else { 1.0 };
                let edge = if rtl { rect.right() } else { rect.left() };
                let left = if rtl {
                    edge - 10.0 - icon_space - label.size().x
                } else {
                    edge + 10.0 + icon_space
                };
                if let Some(icon) = item.icon {
                    icon(
                        ui.painter(),
                        Rect::from_center_size(pos2(edge + offset * 17.0, y), Vec2::splat(18.0)),
                        color,
                    );
                }
                ui.painter()
                    .galley(pos2(left, y - label.size().y * 0.5), label, color);
                if active && !drawer {
                    let check = pos2(
                        if rtl {
                            rect.left() + 12.0
                        } else {
                            rect.right() - 12.0
                        },
                        y,
                    );
                    let middle = check + vec2(-1.0, 3.0);
                    let stroke = Stroke::new(1.8, color);
                    ui.painter()
                        .line_segment([check + vec2(-4.0, 0.0), middle], stroke);
                    ui.painter()
                        .line_segment([middle, check + vec2(5.0, -4.0)], stroke);
                }
                ui.ctx().accesskit_node_builder(response.id, |node| {
                    node.set_role(egui::accesskit::Role::MenuItem);
                    node.set_label(item.label);
                    node.set_selected(active);
                });
                if response.clicked() {
                    *chosen = Some(item.id);
                    ui.close();
                }
                if response.gained_focus() {
                    response.scroll_to_me(None);
                }
                if response.has_focus() {
                    ui.memory_mut(|memory| {
                        memory.set_focus_lock_filter(
                            response.id,
                            egui::EventFilter {
                                vertical_arrows: true,
                                ..Default::default()
                            },
                        )
                    });
                    direction = ui.input(|input| {
                        if input.key_pressed(egui::Key::ArrowDown) {
                            Some((responses.len(), 1_isize))
                        } else if input.key_pressed(egui::Key::ArrowUp) {
                            Some((responses.len(), -1_isize))
                        } else {
                            None
                        }
                    });
                }
                responses.push((response, enabled));
            }
        });
    if let Some((from, delta)) = direction {
        for step in 1..=responses.len() {
            let index = (from as isize + step as isize * delta).rem_euclid(responses.len() as isize)
                as usize;
            if responses[index].1 {
                responses[index].0.request_focus();
                break;
            }
        }
    }
    responses
        .into_iter()
        .find(|(_, enabled)| *enabled)
        .map_or(close, |(response, _)| response)
}
