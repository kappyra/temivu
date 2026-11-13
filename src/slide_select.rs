//! Shared press-and-slide selection for related controls.

use egui::{Id, Rect, Response, Ui};

/// The pointer's interaction with a row in a [SlideSelect] group.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct SlideSelection {
    /// The held pointer is over this row. Use this for visual feedback.
    pub previewed: bool,
    /// The pointer finished a drag over this row. Commit the action now.
    pub released: bool,
}

/// Lets users press a control, slide to another, and activate it on release.
///
/// Use a stable group ID and visit each eligible row once per frame. Give each
/// control click-and-drag sense, then handle ordinary clicks alongside
/// [SlideSelection::released]. Reserve this gesture for related controls.
#[derive(Clone, Copy, Debug)]
pub struct SlideSelect {
    id: Id,
}

impl SlideSelect {
    pub fn new(id: Id) -> Self {
        Self {
            id: id.with("temivu-slide-select"),
        }
    }

    pub(crate) fn is_previewed(self, ui: &Ui, rect: Rect) -> bool {
        let ctx = ui.ctx();
        let origin = ctx.data(|data| data.get_temp::<Id>(self.id));
        origin.is_some_and(|id| ctx.dragged_id() == Some(id))
            && ui.input(|input| input.pointer.primary_down())
            && ui.rect_contains_pointer(rect)
    }

    pub fn row(self, ui: &Ui, response: &Response) -> SlideSelection {
        if !ui.is_enabled() || !ui.is_visible() || !response.enabled() {
            return SlideSelection::default();
        }
        let ctx = ui.ctx();
        let (pressed, held, released, pointer, cancelled) = ui.input(|input| {
            (
                input.pointer.primary_pressed(),
                input.pointer.primary_down(),
                input.pointer.primary_released(),
                input.pointer.interact_pos(),
                input.events.iter().any(|event| {
                    matches!(
                        event,
                        egui::Event::Touch {
                            phase: egui::TouchPhase::Cancel,
                            ..
                        }
                    )
                }) || !input.focused,
            )
        });
        if cancelled || (!held && !released) {
            ctx.data_mut(|data| data.remove::<Id>(self.id));
            return SlideSelection::default();
        }
        if pressed && response.is_pointer_button_down_on() {
            ctx.data_mut(|data| data.insert_temp(self.id, response.id));
        }
        let origin = ctx.data(|data| data.get_temp::<Id>(self.id));
        let target = pointer.is_some_and(|point| {
            ctx.memory(|memory| memory.allows_interaction(response.layer_id))
                && ctx.layer_id_at(point) == Some(response.layer_id)
                && response.interact_rect.contains(
                    ctx.layer_transform_from_global(response.layer_id)
                        .map_or(point, |transform| transform * point),
                )
        });
        let owns_drag = origin.is_some_and(|origin| {
            if released {
                ctx.drag_stopped_id() == Some(origin)
            } else {
                ctx.dragged_id() == Some(origin)
            }
        });
        let released = released && target && owns_drag;
        if released {
            ctx.data_mut(|data| data.remove::<Id>(self.id));
        }
        SlideSelection {
            previewed: held && target && (owns_drag || origin == Some(response.id)),
            released,
        }
    }
}
