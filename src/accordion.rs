//! Accordion with caller-owned expansion and a paint-only reveal.
use crate::Theme;
use egui::{Response, RichText, Stroke, Ui};
use std::sync::atomic::{AtomicU64, Ordering};

static NEXT_ACCORDION_ID: AtomicU64 = AtomicU64::new(0);

fn new_id() -> egui::Id {
    egui::Id::new((
        "temivu-accordion",
        NEXT_ACCORDION_ID.fetch_add(1, Ordering::Relaxed),
    ))
}

/// Tracks expansion order to evict the least recently expanded item when max is reached.
/// Cloning creates a distinct widget identity while preserving expansion state.
#[derive(Debug)]
pub struct AccordionState {
    id: egui::Id,
    min: usize,
    max: Option<usize>,
    expanded: Vec<usize>,
}

impl Clone for AccordionState {
    fn clone(&self) -> Self {
        Self {
            id: new_id(),
            min: self.min,
            max: self.max,
            expanded: self.expanded.clone(),
        }
    }
}

impl AccordionState {
    pub fn new(min: usize, max: Option<usize>) -> Result<Self, &'static str> {
        if max.is_some_and(|max| min > max) {
            return Err("minimum exceeds maximum");
        }
        Ok(Self {
            id: new_id(),
            min,
            max,
            expanded: Vec::new(),
        })
    }
    /// Override the generated identity with a globally unique, stable ID.
    pub fn id(mut self, id: egui::Id) -> Self {
        self.id = id;
        self
    }
    pub fn is_expanded(&self, index: usize) -> bool {
        self.expanded.contains(&index)
    }
    pub fn expanded(&self) -> &[usize] {
        &self.expanded
    }
    /// Return true only if the expansion state changed.
    pub fn toggle(&mut self, index: usize) -> bool {
        if let Some(position) = self.expanded.iter().position(|&i| i == index) {
            if self.expanded.len() <= self.min {
                return false;
            }
            self.expanded.remove(position);
            return true;
        }
        if let Some(max) = self.max {
            if max == 0 {
                return false;
            }
            if self.expanded.len() >= max {
                self.expanded.remove(0);
            }
        }
        self.expanded.push(index);
        true
    }
}

/// An accordion item. Each item must use a stable, unique index within its state.
pub struct AccordionItem<'a> {
    state: &'a mut AccordionState,
    index: usize,
    title: &'a str,
    theme: &'a Theme,
}

impl<'a> AccordionItem<'a> {
    pub fn new(
        state: &'a mut AccordionState,
        index: usize,
        title: &'a str,
        theme: &'a Theme,
    ) -> Self {
        Self {
            state,
            index,
            title,
            theme,
        }
    }
    /// Content is only evaluated while the item is open; this prevents hidden interaction.
    pub fn show(self, ui: &mut Ui, content: impl FnOnce(&mut Ui)) -> Response {
        let id = self.state.id.with(self.index);
        ui.scope_builder(egui::UiBuilder::new().id(id), |ui| {
            let mut response = crate::button::press_native(ui, self.theme, |ui| {
                ui.add(
                    egui::Button::new(RichText::new(self.title).color(self.theme.foreground))
                        .frame(false)
                        .min_size(egui::vec2(ui.available_width(), 48.0)),
                )
            });
            if response.clicked() && self.state.toggle(self.index) {
                response.mark_changed();
            }
            let open = self.state.is_expanded(self.index);
            ui.ctx().accesskit_node_builder(response.id, |node| {
                node.set_expanded(open);
            });
            if response.has_focus() {
                ui.painter().rect_stroke(
                    response.rect.expand(1.0),
                    4,
                    Stroke::new(2.0, self.theme.foreground),
                    egui::StrokeKind::Outside,
                );
            }
            let target = f32::from(open);
            let reveal =
                crate::motion::MotionTrack::new(ui.ctx(), id.with("reveal"), self.theme.motion);
            if open {
                let opacity = reveal
                    .sample(
                        target,
                        target,
                        crate::motion::MotionRole::Feedback.tween(self.theme.motion),
                    )
                    .value;
                ui.vertical(|ui| {
                    ui.multiply_opacity(opacity);
                    content(ui);
                    ui.add_space(16.0);
                });
            } else {
                reveal.follow(0.0_f32, [0.0]);
            }
            response
        })
        .inner
    }
}
