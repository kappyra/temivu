//! A first, intentionally incomplete port of Forui's button.
use crate::{Sizes, SlideSelect, SurfaceShape, Theme, Vesper};
use egui::{Color32, Response, RichText, Stroke, Ui, Vec2};

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum ButtonVariant {
    #[default]
    Primary,
    Secondary,
    Destructive,
    Outline,
    Ghost,
}
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum ButtonSize {
    ExtraSmall,
    Small,
    #[default]
    Medium,
    Large,
}
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum InputMode {
    #[default]
    Desktop,
    Touch,
}

impl ButtonSize {
    /// Content padding, matching Forui's `FButtonStyle` contentPadding.
    ///
    /// Desktop: xs 8/6, sm 10/9, md 10/11, lg 10/13 (horizontal/vertical).
    /// Touch: xs 10/9, sm 12/12, md 12/14, lg 12/16.
    pub fn padding(self, mode: InputMode) -> Vec2 {
        match (mode, self) {
            (InputMode::Desktop, Self::ExtraSmall) => Vec2::new(8.0, 6.0),
            (InputMode::Desktop, Self::Small) => Vec2::new(10.0, 9.0),
            (InputMode::Desktop, Self::Medium) => Vec2::new(10.0, 11.0),
            (InputMode::Desktop, Self::Large) => Vec2::new(10.0, 13.0),
            (InputMode::Touch, Self::ExtraSmall) => Vec2::new(10.0, 9.0),
            (InputMode::Touch, Self::Small) => Vec2::new(12.0, 12.0),
            (InputMode::Touch, Self::Medium) => Vec2::new(12.0, 14.0),
            (InputMode::Touch, Self::Large) => Vec2::new(12.0, 16.0),
        }
    }

    /// Minimum button height in logical pixels, matching upstream size tokens.
    pub fn height(self, mode: InputMode) -> f32 {
        let field = match mode {
            InputMode::Desktop => Sizes::desktop().field,
            InputMode::Touch => Sizes::touch().field,
        };
        match self {
            Self::ExtraSmall => field.xs,
            Self::Small => field.sm,
            Self::Medium => field.md,
            Self::Large => field.lg,
        }
    }
}

/// A text button. Response exposes egui's click, focus and pointer state.
/// Complex content, selected state, long-press and full style parity remain open.
pub struct Button<'a> {
    text: &'a str,
    theme: &'a Theme,
    variant: ButtonVariant,
    size: ButtonSize,
    input: InputMode,
    enabled: bool,
    icon: Option<&'a str>,
    radius: Option<u8>,
    lighting: Option<Option<Vesper>>,
    slide: Option<SlideSelect>,
}

impl<'a> Button<'a> {
    pub fn new(text: &'a str, theme: &'a Theme) -> Self {
        Self {
            text,
            theme,
            variant: ButtonVariant::Primary,
            size: ButtonSize::Medium,
            input: if theme.touch {
                InputMode::Touch
            } else {
                InputMode::Desktop
            },
            enabled: true,
            icon: None,
            radius: None,
            lighting: None,
            slide: None,
        }
    }
    pub fn variant(mut self, variant: ButtonVariant) -> Self {
        self.variant = variant;
        self
    }
    pub fn size(mut self, size: ButtonSize) -> Self {
        self.size = size;
        self
    }
    pub fn input_mode(mut self, input: InputMode) -> Self {
        self.input = input;
        self
    }
    pub fn enabled(mut self, enabled: bool) -> Self {
        self.enabled = enabled;
        self
    }
    /// An optional leading icon rendered as a text glyph. This approximates the
    /// upstream vector icon, and the glyph must exist in the active font (it is
    /// rendered as a missing-glyph box otherwise). The button's accessible name
    /// is overridden to the visible label text, so the glyph is not announced.
    pub fn icon(mut self, icon: &'a str) -> Self {
        self.icon = Some(icon);
        self
    }
    /// Override the default button corner radius.
    pub fn radius(mut self, radius: u8) -> Self {
        self.radius = Some(radius);
        self
    }
    /// Override the inherited lighting. Pass `None` to disable it for this button.
    pub fn lighting(mut self, lighting: Option<Vesper>) -> Self {
        self.lighting = Some(lighting);
        self
    }

    /// Allow a held press to move between buttons sharing the same group.
    pub fn slide_select(mut self, group: SlideSelect) -> Self {
        self.slide = Some(group);
        self
    }

    pub fn show(self, ui: &mut Ui) -> Response {
        let (fill, foreground, border) = match self.variant {
            ButtonVariant::Primary => (
                self.theme.primary,
                self.theme.primary_foreground,
                Color32::TRANSPARENT,
            ),
            ButtonVariant::Secondary => (
                self.theme.secondary,
                self.theme.secondary_foreground,
                Color32::TRANSPARENT,
            ),
            ButtonVariant::Destructive => {
                let [r, g, b, _] = self.theme.destructive.to_srgba_unmultiplied();
                let alpha = if self.theme.dark { 51 } else { 26 };
                (
                    Color32::from_rgba_unmultiplied(r, g, b, alpha),
                    self.theme.destructive,
                    Color32::TRANSPARENT,
                )
            }
            ButtonVariant::Outline => (
                self.theme.card,
                self.theme.secondary_foreground,
                self.theme.border,
            ),
            ButtonVariant::Ghost => (
                Color32::TRANSPARENT,
                self.theme.secondary_foreground,
                Color32::TRANSPARENT,
            ),
        };
        let height = self.size.height(self.input);
        let default_radius = SurfaceShape::Control.radius(Vec2::splat(height));
        let radius = self.radius.unwrap_or(default_radius).min(default_radius);
        let hover_fill = match self.variant {
            ButtonVariant::Primary | ButtonVariant::Secondary => self.theme.hover(fill),
            ButtonVariant::Destructive => {
                let [r, g, b, _] = self.theme.destructive.to_srgba_unmultiplied();
                Color32::from_rgba_unmultiplied(r, g, b, if self.theme.dark { 77 } else { 51 })
            }
            ButtonVariant::Outline | ButtonVariant::Ghost => self.theme.secondary,
        };
        let content = if let Some(icon) = self.icon {
            let mut job = egui::text::LayoutJob::default();
            let format = egui::TextFormat {
                font_id: egui::FontId::proportional(self.theme.typography.body.size),
                color: foreground,
                ..Default::default()
            };
            job.append(icon, 0.0, format.clone());
            job.append(" ", 0.0, format.clone());
            job.append(self.text, 0.0, format);
            egui::WidgetText::from(job)
        } else {
            egui::WidgetText::from(
                RichText::new(self.text)
                    .size(self.theme.typography.body.size)
                    .color(foreground),
            )
        };
        let press_key = ui.next_auto_id().with("temivu-button-press");
        let previous = ui
            .ctx()
            .data(|data| data.get_temp::<(egui::Rect, egui::Id)>(press_key));
        let pressed = self.enabled
            && ui.is_enabled()
            && previous.is_some_and(|(rect, id)| {
                let focused = ui.memory(|memory| memory.has_focus(id));
                let pointer_over = ui.rect_contains_pointer(rect);
                self.slide.is_some_and(|group| group.is_previewed(ui, rect))
                    || ui.input(|input| {
                        (pointer_over
                            && input.pointer.primary_down()
                            && input
                                .pointer
                                .press_origin()
                                .is_some_and(|pos| rect.contains(pos)))
                            || (focused
                                && (input.key_down(egui::Key::Space)
                                    || input.key_down(egui::Key::Enter)))
                    })
            });
        let tapped = self.enabled
            && ui.is_enabled()
            && previous
                .and_then(|(_, id)| ui.ctx().read_response(id))
                .is_some_and(|response| response.clicked());
        let press = crate::motion::press(ui.ctx(), press_key, pressed, tapped, self.theme);
        let hover = crate::motion::animate_bool(
            ui.ctx(),
            press_key.with("surface"),
            self.enabled
                && ui.is_enabled()
                && previous.is_some_and(|(rect, _)| ui.rect_contains_pointer(rect)),
            self.theme,
        );
        let surface = Color32::from(
            egui::Rgba::from(fill) * (1.0 - hover) + egui::Rgba::from(hover_fill) * hover,
        );
        let button = egui::Button::new(content)
            .frame(true)
            .corner_radius(radius)
            .min_size(Vec2::splat(height));
        let button = if self.slide.is_some() {
            button.sense(egui::Sense::click_and_drag())
        } else {
            button
        };
        let center = previous.map_or(egui::Pos2::ZERO, |(rect, _)| rect.center());
        let transform = crate::motion::scale_around(center, crate::motion::press_scale(press));
        let mut response = ui
            .with_visual_transform(transform, |ui| {
                ui.spacing_mut().interact_size.y = height;
                ui.spacing_mut().button_padding = self.size.padding(self.input);
                let widgets = &mut ui.visuals_mut().widgets;
                widgets.inactive.weak_bg_fill = surface;
                widgets.hovered.weak_bg_fill = surface;
                widgets.active.weak_bg_fill = surface;
                widgets.noninteractive.weak_bg_fill = fill;
                for visual in [
                    &mut widgets.inactive,
                    &mut widgets.hovered,
                    &mut widgets.active,
                    &mut widgets.noninteractive,
                ] {
                    visual.bg_stroke = Stroke::new(1.0, border);
                    visual.expansion = 0.0;
                }
                let start = paint_start(ui.ctx(), ui.layer_id());
                let response = ui.add_enabled(self.enabled, button);
                deform_paint(ui.ctx(), ui.layer_id(), start, press);
                if let Some(style) = self.lighting.unwrap_or_else(|| Vesper::context(ui.ctx())) {
                    if let Some(light) = style.interactive_shape_on(
                        ui.painter(),
                        self.theme,
                        &response,
                        response.rect,
                        crate::motion::press_corners(response.rect, radius.into(), press),
                        surface,
                    ) {
                        // egui paints the button frame first. Keep the label and
                        // focus treatment above the interaction mesh.
                        ui.ctx().graphics_mut(|graphics| {
                            let list = graphics.entry(ui.layer_id());
                            if start.0 < list.next_idx().0 {
                                list.mutate_shape(start, |entry| {
                                    let frame =
                                        std::mem::replace(&mut entry.shape, egui::Shape::Noop);
                                    entry.shape = egui::Shape::Vec(vec![frame, light]);
                                });
                            }
                        });
                    }
                    if response.enabled() {
                        crate::lighting::spill::receive_background_at(
                            ui.painter(),
                            response.id,
                            response.rect,
                            crate::motion::press_corners(response.rect, radius.into(), press),
                            &style,
                            surface,
                            self.theme.card,
                            start,
                        );
                    }
                }
                response
            })
            .inner;
        if self
            .slide
            .is_some_and(|group| group.row(ui, &response).released)
        {
            response
                .flags
                .insert(egui::response::Flags::FAKE_PRIMARY_CLICKED);
        }
        ui.ctx()
            .data_mut(|data| data.insert_temp(press_key, (response.rect, response.id)));
        let painted = transform.mul_rect(response.rect);
        crate::motion::remember_anchor(&response, painted);
        crate::motion::remember_anchor_material(
            &response,
            (surface.a() > 0).then_some(crate::motion::AnchorMaterial {
                fill: surface,
                border,
            }),
        );
        if self.icon.is_some() {
            // Keep the accessible name identical to the visible label text.
            ui.ctx().accesskit_node_builder(response.id, |node| {
                node.set_label(self.text);
            });
        }
        let attention = crate::motion::animate_bool(
            ui.ctx(),
            response.id.with("hover-ring"),
            response.hovered() && self.enabled && self.variant != ButtonVariant::Ghost,
            self.theme,
        );
        let ring = if response.has_focus() {
            self.theme.foreground
        } else {
            self.theme.primary.gamma_multiply(attention * 0.23)
        };
        if ring.a() > 0 {
            let radius = crate::motion::press_corners(response.rect, radius.into(), press).nw;
            ui.painter().add(crate::motion::rounded_shape(
                painted.expand(2.0),
                egui::epaint::CornerRadiusF32::same(radius * transform.scaling + 2.0),
                Color32::TRANSPARENT,
                Stroke::new(2.0, ring),
                egui::StrokeKind::Outside,
            ));
        }
        response
    }
}

/// Give internally composed egui buttons the same paint-only press feedback.
pub(crate) fn press_native(
    ui: &mut Ui,
    theme: &Theme,
    draw: impl FnOnce(&mut Ui) -> Response,
) -> Response {
    let layer = ui.layer_id();
    let ctx = ui.ctx().clone();
    let start = paint_start(&ctx, layer);
    let response = draw(ui);
    let material = ctx.graphics(|graphics| {
        graphics
            .get(layer)?
            .all_entries()
            .skip(start.0)
            .find_map(|entry| {
                crate::motion::AnchorMaterial::from_shape(&entry.shape, response.rect)
            })
    });
    crate::motion::remember_anchor_material(&response, material);
    let press = crate::motion_controls::press_amount(&response, theme);
    if press.abs() < 0.001 {
        crate::motion::remember_anchor(&response, response.rect);
        return response;
    }
    let transform =
        crate::motion::scale_around(response.rect.center(), crate::motion::press_scale(press));
    deform_paint(&ctx, layer, start, press);
    ctx.graphics_mut(|graphics| {
        let list = graphics.entry(layer);
        list.transform_range(start, list.next_idx(), transform);
    });
    crate::motion::remember_anchor(&response, transform.mul_rect(response.rect));
    response
}

fn paint_start(ctx: &egui::Context, layer: egui::LayerId) -> egui::layers::ShapeIdx {
    ctx.graphics(|graphics| {
        graphics
            .get(layer)
            .map_or(egui::layers::ShapeIdx(0), |list| list.next_idx())
    })
}

fn deform_paint(
    ctx: &egui::Context,
    layer: egui::LayerId,
    start: egui::layers::ShapeIdx,
    press: f32,
) {
    if press <= 0.001 {
        return;
    }
    fn deform(shape: &mut egui::Shape, press: f32) {
        match shape {
            egui::Shape::Rect(rect)
                if rect.brush.is_none() && rect.blur_width == 0.0 && rect.angle == 0.0 =>
            {
                *shape = crate::motion::press_shape(rect.clone(), press);
            }
            egui::Shape::Vec(shapes) => shapes.iter_mut().for_each(|shape| deform(shape, press)),
            _ => {}
        }
    }
    ctx.graphics_mut(|graphics| {
        let list = graphics.entry(layer);
        for index in start.0..list.next_idx().0 {
            list.mutate_shape(egui::layers::ShapeIdx(index), |entry| {
                deform(&mut entry.shape, press);
            });
        }
    });
}
