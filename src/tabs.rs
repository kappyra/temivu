//! Preliminary Forui tabs: caller-owned selection, equal-width headers and
//! selected content. Swipe gestures, transitions and full tab-list semantics
//! remain unimplemented.
use crate::Theme;
use egui::{Frame, Key, RichText, Stroke, Ui};

pub struct Tabs<'a> {
    selected: &'a mut usize,
    labels: &'a [&'a str],
    theme: &'a Theme,
}

impl<'a> Tabs<'a> {
    /// Reject empty tab lists and out-of-range selection, matching upstream's
    /// nonempty-list requirement without panicking during rendering.
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
        })
    }

    /// Renders the headers and invokes the content callback for the selected index.
    pub fn show<R>(self, ui: &mut Ui, content: impl FnOnce(&mut Ui, usize) -> R) -> R {
        let mut move_focus = None;
        let mut ids = Vec::with_capacity(self.labels.len());
        Frame::new()
            .fill(self.theme.muted)
            // Forui's tab container has a 1 point muted border and md radius.
            .stroke(Stroke::new(1.0, self.theme.muted))
            .corner_radius(10)
            .inner_margin(4)
            .show(ui, |ui| {
                ui.horizontal(|ui| {
                    // Upstream tabs sit edge to edge inside the 4 point padding.
                    ui.spacing_mut().item_spacing.x = 0.0;
                    let width = (ui.available_width() / self.labels.len() as f32).max(0.0);
                    for (index, label) in self.labels.iter().enumerate() {
                        let selected = *self.selected == index;
                        let text = RichText::new(*label)
                            .size(self.theme.typography.body.size)
                            .color(if selected {
                                self.theme.foreground
                            } else {
                                self.theme.muted_foreground
                            });
                        let fill = if selected {
                            self.theme.background
                        } else {
                            egui::Color32::TRANSPARENT
                        };
                        let response = ui.add_sized(
                            [width, 36.0],
                            egui::Button::new(text)
                                .fill(fill)
                                .stroke(Stroke::NONE)
                                .corner_radius(10),
                        );
                        ui.ctx().accesskit_node_builder(response.id, |node| {
                            node.set_role(egui::accesskit::Role::Tab);
                            node.set_selected(selected);
                        });
                        ids.push(response.id);
                        if response.clicked() {
                            *self.selected = index;
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
                                response.rect.expand(1.0),
                                10,
                                Stroke::new(2.0, self.theme.foreground),
                                egui::StrokeKind::Outside,
                            );
                            if ui.input(|input| input.key_pressed(Key::ArrowRight)) {
                                move_focus = Some((index + 1) % self.labels.len());
                            } else if ui.input(|input| input.key_pressed(Key::ArrowLeft)) {
                                move_focus =
                                    Some((index + self.labels.len() - 1) % self.labels.len());
                            }
                        }
                    }
                });
            });
        if let Some(index) = move_focus {
            *self.selected = index;
            ui.ctx()
                .memory_mut(|memory| memory.request_focus(ids[index]));
        }
        ui.add_space(10.0);
        content(ui, *self.selected)
    }
}
