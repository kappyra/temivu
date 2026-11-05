//! Caller-owned tabs with animated selection, drag selection and optional reordering.
use crate::Theme;
use egui::{Frame, Key, RichText, Stroke, Ui};

pub struct Tabs<'a> {
    selected: &'a mut usize,
    labels: &'a [&'a str],
    theme: &'a Theme,
    order: Option<&'a mut [usize]>,
    draggable: bool,
}

impl<'a> Tabs<'a> {
    /// Selection refers to the label index, including after reordering.
    pub fn new(
        selected: &'a mut usize,
        labels: &'a [&'a str],
        theme: &'a Theme,
    ) -> Result<Self, &'static str> {
        if labels.is_empty() {
            return Err("tabs require at least one entry");
        }
        if *selected >= labels.len() {
            return Err("selected tab is out of range");
        }
        Ok(Self {
            selected,
            labels,
            theme,
            order: None,
            draggable: true,
        })
    }

    /// Allow dragging across headers to select a tab, using mouse or touch.
    pub fn draggable(mut self, draggable: bool) -> Self {
        self.draggable = draggable;
        self
    }

    /// Keep the visual order in caller-owned state. Drag a header, or use
    /// Alt+Left/Right while focused, to rearrange it without changing its content.
    /// `order` must be a permutation of the label indices.
    pub fn reorderable(mut self, order: &'a mut [usize]) -> Result<Self, &'static str> {
        if order.len() != self.labels.len() {
            return Err("tab order must include every label");
        }
        let mut seen = vec![false; order.len()];
        for &index in order.iter() {
            if index >= seen.len() || seen[index] {
                return Err("tab order must be a permutation");
            }
            seen[index] = true;
        }
        self.order = Some(order);
        Ok(self)
    }

    pub fn show<R>(mut self, ui: &mut Ui, content: impl FnOnce(&mut Ui, usize) -> R) -> R {
        let motion_id = ui.next_auto_id().with("temivu-tab-indicator");
        let mut move_focus = None;
        let mut reorder = None;
        let mut ids = Vec::with_capacity(self.labels.len());
        let frame = Frame::new()
            .fill(self.theme.muted)
            .stroke(Stroke::new(1.0, self.theme.muted))
            .corner_radius(10)
            .inner_margin(4)
            .show(ui, |ui| {
                ui.horizontal(|ui| {
                    ui.spacing_mut().item_spacing.x = 0.0;
                    let width = (ui.available_width() / self.labels.len() as f32).max(1.0);
                    let start = ui.cursor().min;
                    let strip = egui::Rect::from_min_size(
                        start,
                        egui::vec2(width * self.labels.len() as f32, 44.0),
                    );
                    let last_slot = (self.labels.len() - 1) as f32;
                    let selected = *self.selected;
                    let selected_position = self
                        .order
                        .as_deref()
                        .and_then(|order| order.iter().position(|&index| index == selected))
                        .unwrap_or(selected);
                    let ctx = ui.ctx().clone();
                    let spring_position = |id, slot: usize| {
                        crate::motion::spring_f32(&ctx, id, slot as f32, slot as f32, self.theme)
                            .clamp(0.0, last_slot)
                    };
                    let position = spring_position(motion_id, selected_position);
                    let indicator = egui::Rect::from_min_size(
                        start + egui::vec2(width * position, 0.0),
                        egui::vec2(width, 44.0),
                    );
                    ui.painter().rect_filled(indicator, 8, self.theme.card);
                    ui.painter().rect_stroke(
                        indicator,
                        8,
                        Stroke::new(1.0, self.theme.border),
                        egui::StrokeKind::Inside,
                    );
                    for position in 0..self.labels.len() {
                        let index = self
                            .order
                            .as_deref()
                            .map_or(position, |order| order[position]);
                        let selected = selected == index;
                        let shift = if self.order.is_some() {
                            let visual_slot = spring_position(motion_id.with(index), position);
                            egui::vec2((visual_slot - position as f32) * width, 0.0)
                        } else {
                            egui::Vec2::ZERO
                        };
                        let text = RichText::new(self.labels[index])
                            .size(self.theme.typography.body.size)
                            .color(if selected {
                                self.theme.foreground
                            } else {
                                self.theme.muted_foreground
                            });
                        let layer = ui.layer_id();
                        let paint_start = (shift != egui::Vec2::ZERO)
                            .then(|| ctx.graphics_mut(|graphics| graphics.entry(layer).next_idx()));
                        let response = ui
                            .push_id(index, |ui| {
                                crate::button::press_native(ui, self.theme, |ui| {
                                    ui.add_sized(
                                        [width, 44.0],
                                        egui::Button::new(text)
                                            .truncate()
                                            .fill(egui::Color32::TRANSPARENT)
                                            .stroke(Stroke::NONE)
                                            .corner_radius(8)
                                            .sense(if self.draggable {
                                                egui::Sense::click_and_drag()
                                            } else {
                                                egui::Sense::click()
                                            }),
                                    )
                                })
                            })
                            .inner;
                        if let Some(paint_start) = paint_start {
                            // Move the paint only. The logical slot and the widget ID
                            // stay stable for keyboard focus and the active drag.
                            let clip = strip.intersect(ui.clip_rect());
                            ctx.graphics_mut(|graphics| {
                                let list = graphics.entry(layer);
                                let end = list.next_idx();
                                list.transform_range(
                                    paint_start,
                                    end,
                                    egui::emath::TSTransform::from_translation(shift),
                                );
                                for i in paint_start.0..end.0 {
                                    list.mutate_shape(egui::layers::ShapeIdx(i), |entry| {
                                        entry.clip_rect = entry.clip_rect.intersect(clip);
                                    });
                                }
                            });
                        }
                        ui.ctx().accesskit_node_builder(response.id, |node| {
                            node.set_role(egui::accesskit::Role::Tab);
                            node.set_selected(selected);
                        });
                        ids.push(response.id);
                        if response.clicked() && *self.selected != index {
                            *self.selected = index;
                            ui.ctx().request_repaint();
                        }
                        if response.dragged()
                            && let Some(pointer) = response.interact_pointer_pos()
                        {
                            let target = (((pointer.x - start.x) / width).floor().max(0.0)
                                as usize)
                                .min(self.labels.len() - 1);
                            let next = if self.order.is_some() {
                                if position != target {
                                    reorder = Some((position, target));
                                }
                                index
                            } else {
                                target
                            };
                            if *self.selected != next {
                                *self.selected = next;
                                ui.ctx().request_repaint();
                            }
                        }
                        if response.has_focus() {
                            ui.memory_mut(|memory| {
                                memory.set_focus_lock_filter(
                                    response.id,
                                    egui::EventFilter {
                                        horizontal_arrows: true,
                                        ..Default::default()
                                    },
                                )
                            });
                            ui.painter().rect_stroke(
                                response.rect.translate(shift).shrink(1.0),
                                8,
                                Stroke::new(2.0, self.theme.foreground),
                                egui::StrokeKind::Inside,
                            );
                            let direction = ui.input(|input| {
                                if input.key_pressed(Key::ArrowRight) {
                                    Some(1)
                                } else if input.key_pressed(Key::ArrowLeft) {
                                    Some(-1)
                                } else {
                                    None
                                }
                            });
                            if let Some(direction) = direction {
                                let target = (position as isize + direction)
                                    .rem_euclid(self.labels.len() as isize)
                                    as usize;
                                if self.order.is_some() && ui.input(|input| input.modifiers.alt) {
                                    reorder = Some((position, target));
                                } else {
                                    move_focus = Some(target);
                                }
                            }
                        }
                    }
                });
            });
        ui.ctx().accesskit_node_builder(frame.response.id, |node| {
            node.set_role(egui::accesskit::Role::TabList)
        });
        if let Some(position) = move_focus {
            *self.selected = self
                .order
                .as_deref()
                .map_or(position, |order| order[position]);
            ui.memory_mut(|memory| memory.request_focus(ids[position]));
            ui.ctx().request_repaint();
        }
        if let Some((from, to)) = reorder
            && let Some(order) = self.order.as_deref_mut()
        {
            let index = order[from];
            if from < to {
                order.copy_within(from + 1..=to, from);
            } else {
                order.copy_within(to..from, to + 1);
            }
            order[to] = index;
            ui.ctx().request_repaint();
        }
        ui.add_space(12.0);
        content(ui, *self.selected)
    }
}
