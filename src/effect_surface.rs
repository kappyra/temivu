//! Opt-in visual effects for arbitrary egui content.
use crate::{Glass, GlassProfile, SurfaceShape, Theme, Vesper};
use egui::{AsIdSalt, Frame, IdSalt, InnerResponse, Sense, Shape, Stroke, Ui, UiBuilder};

/// A framed egui container with optional glass and interaction lighting.
///
/// Effects are disabled by default. The container paints them behind its
/// content and leaves child buttons, fields, and drag targets in charge of
/// their own input. Use an egui `Area` to place the container above other UI.
///
/// ```
/// # fn search(ui: &mut egui::Ui, theme: &temivu::Theme, query: &mut String) {
/// use temivu::{EffectSurface, Glass, Vesper};
/// EffectSurface::new(theme)
///     .id_salt("search-surface")
///     .glass(Glass::context(ui.ctx()))
///     .lighting(Some(Vesper::subtle()))
///     .show(ui, |ui| ui.text_edit_singleline(query));
/// # }
/// ```
pub struct EffectSurface<'a> {
    theme: &'a Theme,
    frame: Frame,
    glass: Option<Glass>,
    glass_profile: GlassProfile,
    lighting: Option<Vesper>,
    sense: Sense,
    id_salt: Option<IdSalt>,
}

impl<'a> EffectSurface<'a> {
    pub fn new(theme: &'a Theme) -> Self {
        Self {
            theme,
            frame: Frame::new()
                .fill(theme.card)
                .stroke(Stroke::new(1.0, theme.border))
                .corner_radius(SurfaceShape::Card.radius(egui::vec2(240.0, 120.0)))
                .inner_margin(12),
            glass: None,
            glass_profile: GlassProfile::Floating,
            lighting: None,
            sense: Sense::hover(),
            id_salt: None,
        }
    }

    /// Override the background, insets, stroke, corner shape, and shadow.
    pub fn frame(mut self, frame: Frame) -> Self {
        self.frame = frame;
        self
    }

    /// Add the standard floating elevation to the current frame.
    pub fn elevated(mut self) -> Self {
        self.frame.shadow = egui::epaint::Shadow {
            offset: [0, 7],
            blur: 22,
            spread: 0,
            color: egui::Color32::from_black_alpha(if self.theme.dark { 70 } else { 30 }),
        };
        self
    }

    /// Apply a glass material. `None` uses the plain frame.
    pub fn glass(mut self, glass: Option<Glass>) -> Self {
        self.glass = glass;
        self
    }

    /// Choose the backdrop treatment for the glass renderer.
    pub fn glass_profile(mut self, profile: GlassProfile) -> Self {
        self.glass_profile = profile;
        self
    }

    /// Apply Vesper lighting to the frame. `None` leaves the frame unlit.
    pub fn lighting(mut self, lighting: Option<Vesper>) -> Self {
        self.lighting = lighting;
        self
    }

    /// Set how the background responds to input. Interactive children take
    /// priority over the background. The default senses hover.
    pub fn sense(mut self, sense: Sense) -> Self {
        self.sense = sense;
        self
    }

    /// Give the surface a stable identity unique within its parent UI.
    pub fn id_salt(mut self, id_salt: impl AsIdSalt) -> Self {
        self.id_salt = Some(IdSalt::new(id_salt));
        self
    }

    pub(crate) fn theme(&self) -> &Theme {
        self.theme
    }

    pub fn show<R>(self, ui: &mut Ui, content: impl FnOnce(&mut Ui) -> R) -> InnerResponse<R> {
        let mut builder = UiBuilder::new().sense(self.sense);
        if let Some(salt) = self.id_salt {
            builder = builder.id_salt(salt);
        }
        let effects_allowed = ui.is_enabled() && ui.is_visible() && ui.clip_rect().is_positive();
        let mut spill_slot = None;
        let mut light_slot = None;
        let surface = ui.scope_builder(builder, |ui| {
            let content = |ui: &mut Ui| {
                if effects_allowed && let Some(lighting) = self.lighting {
                    if lighting.spill.is_some() {
                        spill_slot = Some(ui.painter().add(Shape::Noop));
                    }
                    light_slot = Some(ui.painter().add(Shape::Noop));
                }
                content(ui)
            };
            match self.glass.filter(|_| effects_allowed) {
                Some(glass) => glass.show(ui, self.theme, self.frame, self.glass_profile, content),
                None => self.frame.show(ui, content),
            }
        });

        let rect = surface.inner.response.rect - self.frame.outer_margin;
        if let (Some(lighting), Some(slot)) = (self.lighting, light_slot)
            && ui.clip_rect().intersect(rect).is_positive()
        {
            let corners = self.frame.corner_radius.into();
            if let Some(spill_slot) = spill_slot {
                crate::lighting::spill::receive_at(
                    ui.painter(),
                    surface.response.id,
                    rect,
                    corners,
                    &lighting,
                    self.frame.fill,
                    self.theme.card,
                    spill_slot,
                );
            }
            // This passive probe also follows hover over a child. The
            // background response still owns clicks, drags and focus.
            let hover = ui.interact(
                rect,
                surface.response.id.with("vesper-hover"),
                Sense::hover(),
            );
            let mut interaction = surface.response.union(hover);
            // Lighting may observe a child-owned hold without claiming input.
            interaction.sense = Sense::hover();
            if let Some(shape) = lighting.interactive_shape_on(
                ui.painter(),
                self.theme,
                &interaction,
                rect,
                corners,
                self.frame.fill,
            ) {
                ui.painter().set(slot, shape);
            }
        }

        InnerResponse::new(surface.inner.inner, surface.response)
    }
}
