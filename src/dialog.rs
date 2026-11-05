//! Modal dialog port; full route and focus parity remain open. A themed dialog
//! draws Forui's card, border and md radius over the theme barrier color.
use crate::{Theme, liquid::OverlayTravel};
use egui::{Context, Frame, Id, Stroke};

#[derive(Clone, Default)]
struct PreviousFocus(Option<Id>);

fn restore_focus(ctx: &Context, key: Id) {
    if let Some(PreviousFocus(Some(id))) =
        ctx.data_mut(|data| data.remove_temp::<PreviousFocus>(key))
    {
        ctx.memory_mut(|memory| memory.request_focus(id));
    }
}

/// A caller-owned modal. The ID must remain unique and stable while open.
/// Opening multiple modals in the same frame has undefined focus ordering;
/// open at most one new modal per frame.
pub struct Dialog<'a> {
    id: Id,
    label: &'a str,
    barrier_dismissible: bool,
    theme: Option<&'a Theme>,
    radius: u8,
    padding: egui::Margin,
    frame: Option<Frame>,
    area: Option<egui::Area>,
    source: Option<egui::Rect>,
    travel: OverlayTravel,
    liquid: Option<crate::liquid::LiquidStyle>,
}

impl<'a> Dialog<'a> {
    pub fn new(id: Id, label: &'a str) -> Self {
        Self {
            id,
            label,
            barrier_dismissible: true,
            theme: None,
            radius: 32,
            padding: egui::Margin::same(24),
            frame: None,
            area: None,
            source: None,
            travel: OverlayTravel::Source,
            liquid: None,
        }
    }

    pub fn barrier_dismissible(mut self, dismissible: bool) -> Self {
        self.barrier_dismissible = dismissible;
        self
    }

    /// Use the theme's card, border and barrier colors. Radius and padding
    /// can be customized through the existing frame-related builders.
    pub fn theme(mut self, theme: &'a Theme) -> Self {
        self.theme = Some(theme);
        self
    }

    /// Set the corner radius of the themed surface.
    pub fn radius(mut self, radius: u8) -> Self {
        self.radius = radius;
        self
    }

    /// Set the inner spacing of the themed surface.
    pub fn padding(mut self, padding: egui::Margin) -> Self {
        self.padding = padding;
        self
    }

    /// Override the complete surface using egui's frame configuration.
    pub fn frame(mut self, frame: Frame) -> Self {
        self.frame = Some(frame);
        self
    }

    /// Override the context's opening/closing animation preset.
    pub fn liquid(mut self, motion: crate::liquid::LiquidStyle) -> Self {
        self.liquid = Some(motion);
        self
    }

    /// Match the entrance/exit to a trigger rectangle.
    pub fn source(mut self, source: egui::Rect) -> Self {
        self.source = Some(source);
        self
    }
    /// Match the transition to the source control's painted bounds.
    pub fn source_response(self, response: &egui::Response) -> Self {
        self.source(crate::motion::anchor_rect(response))
    }

    /// Reuse egui's positioning API, for example to place a navigation drawer.
    pub fn area(mut self, area: egui::Area) -> Self {
        self.area = Some(area);
        self
    }
    pub(crate) fn slide_from_left(mut self) -> Self {
        self.travel = OverlayTravel::Left;
        self
    }

    pub(crate) fn slide_from_bottom(mut self) -> Self {
        self.travel = OverlayTravel::Bottom;
        self
    }

    /// Returns the content result while open; updates `open` on dismissal.
    /// Call every frame, including after closing, so focus can be restored.
    pub fn show<R>(
        self,
        ctx: &Context,
        open: &mut bool,
        content: impl FnOnce(&mut egui::Ui) -> R,
    ) -> Option<R> {
        let focus_key = self.id.with("temivu-dialog-previous-focus");
        let theme = self
            .theme
            .cloned()
            .unwrap_or_else(|| crate::surface::context_theme(ctx));
        let newly_opened = *open
            && ctx
                .data(|data| data.get_temp::<PreviousFocus>(focus_key))
                .is_none();
        let source_key = self.id.with("transition-source");
        let stored_source = ctx.data(|data| data.get_temp::<egui::Rect>(source_key));
        let source = if newly_opened {
            self.source
        } else {
            stored_source.or(self.source)
        }
        .filter(|rect| rect.is_finite() && rect.is_positive())
        .unwrap_or_else(|| {
            egui::Rect::from_center_size(
                ctx.content_rect().center() + egui::vec2(0.0, 16.0),
                egui::vec2(48.0, 32.0),
            )
        });
        let area = self
            .area
            .unwrap_or_else(|| egui::Modal::default_area(self.id))
            .sense(egui::Sense::focusable_noninteractive())
            .fade_in(false);
        let motion = self
            .liquid
            .unwrap_or_else(|| crate::liquid::LiquidStyle::context_default(ctx));
        let travel = if matches!(self.travel, OverlayTravel::Source) && self.source.is_none() {
            OverlayTravel::Center
        } else {
            self.travel
        };
        let transition = crate::liquid::OverlayPass::begin(
            ctx,
            area.layer(),
            *open,
            source,
            travel,
            motion,
            &theme,
        );
        if !*open {
            restore_focus(ctx, focus_key);
            transition.paint_exit(ctx);
            return None;
        }
        if newly_opened {
            let previous = ctx.memory(|memory| memory.focused());
            ctx.data_mut(|data| {
                data.insert_temp(source_key, source);
                data.insert_temp(focus_key, PreviousFocus(previous));
            });
        }
        let frame = self.frame.unwrap_or_else(|| {
            if let Some(theme) = self.theme {
                Frame::new()
                    .fill(theme.card)
                    .stroke(Stroke::new(1.0, theme.border))
                    .corner_radius(self.radius)
                    .inner_margin(self.padding)
            } else {
                Frame::popup(&ctx.global_style())
                    .corner_radius(self.radius)
                    .inner_margin(self.padding)
            }
        });
        let mut modal = egui::Modal::new(self.id).area(area).frame(Frame::NONE);
        if let Some(theme) = self.theme {
            modal = modal.backdrop_color(theme.barrier);
        }
        let start = transition.start(ctx);
        let response = modal.show(ctx, |ui| {
            let sizing = ui.is_sizing_pass();
            if sizing {
                ui.ctx().request_discard("modal surface sizing");
            }
            let surface = transition.start(ctx);
            let mut prepared = frame.begin(ui);
            let alpha = prepared.content_ui.opacity();
            let inner = prepared
                .content_ui
                .add_enabled_ui(transition.interactive(), |ui| {
                    ui.set_opacity(alpha);
                    content(ui)
                })
                .inner;
            let rect = frame.widget_rect(prepared.content_ui.min_rect());
            prepared.end(ui);
            if !sizing {
                transition.finish(ctx, start, surface, rect, frame);
            }
            inner
        });
        ctx.accesskit_node_builder(response.response.id, |node| {
            node.set_role(egui::accesskit::Role::Dialog);
            node.set_label(self.label);
            node.set_modal();
        });
        if newly_opened {
            response.response.request_focus();
        }
        // Escape applies only to the topmost modal and must not preempt popups.
        let escape = response.is_top_modal
            && !response.any_popup_open
            && ctx.input_mut(|input| input.consume_key(egui::Modifiers::NONE, egui::Key::Escape));
        if response.response.should_close()
            || escape
            || (self.barrier_dismissible
                && transition.interactive()
                && response.backdrop_response.clicked())
        {
            *open = false;
            // The modal still owns focus in this frame. Restore after it disappears.
            ctx.request_repaint();
        }
        Some(response.inner)
    }
}
