//! Reusable foreground travel and lift for arbitrary egui content.
use crate::{EffectSurface, liquid::LiquidStyle};
use egui::{Align2, Area, Context, Id, InnerResponse, Order, Ui, Vec2, vec2};

/// Spring-driven placement for a floating surface. The application supplies
/// visibility and placement and can reuse an ordinary egui UI inside it.
#[derive(Clone, Copy, Debug)]
pub struct FloatingMotion {
    /// Offset from the settled position at the start of the entrance.
    pub start_offset: Vec2,
    /// Placement order. Defaults to the foreground.
    pub order: Order,
    /// Spring and reduced-motion configuration shared with Droplet.
    pub spring: LiquidStyle,
}

impl Default for FloatingMotion {
    fn default() -> Self {
        Self::lift()
    }
}

impl FloatingMotion {
    /// Raise a nearby surface into place.
    pub fn lift() -> Self {
        Self {
            start_offset: vec2(0.0, 18.0),
            order: Order::Foreground,
            spring: LiquidStyle::droplet(),
        }
    }

    /// Move from a caller-selected offset to the settled position.
    pub fn flight(start_offset: Vec2) -> Self {
        Self {
            start_offset,
            ..Self::lift()
        }
    }

    /// Draw inside a safe-area-constrained foreground area. The anchor and
    /// offset locate the final surface. Call each frame, including when closed.
    /// Exit content remains visible briefly with input disabled.
    pub fn show<R>(
        self,
        ctx: &Context,
        id: Id,
        anchor: (Align2, Vec2),
        open: bool,
        surface: EffectSurface<'_>,
        content: impl FnOnce(&mut Ui) -> R,
    ) -> Option<InnerResponse<R>> {
        let phase = self
            .spring
            .progress(ctx, id.with("float-motion"), open, surface.theme());
        if phase <= 0.001 {
            return None;
        }
        let travel = if self.start_offset.is_finite() {
            self.start_offset
        } else {
            Vec2::ZERO
        };
        let base = if anchor.1.is_finite() {
            anchor.1
        } else {
            Vec2::ZERO
        };
        let offset = base + travel * (1.0 - phase);
        Some(
            Area::new(id)
                .order(self.order)
                .anchor(anchor.0, offset)
                .constrain_to(ctx.content_rect())
                .interactable(open)
                .fade_in(false)
                .show(ctx, |ui| {
                    ui.multiply_opacity(phase.min(1.0));
                    // Keep the material during the exit. Disable only the content's input.
                    surface.show(ui, |ui| ui.add_enabled_ui(open, content).inner)
                })
                .inner,
        )
    }
}
