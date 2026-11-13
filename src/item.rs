//! Textual item port; rich content, grouping and full style parity remain open.
use crate::{Theme, Vesper};
use egui::{
    Color32, FontId, Galley, Response, Role, Sense, Stroke, StrokeKind, TextFormat, Ui, Vec2,
    WidgetInfo, text::LayoutJob,
};

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum ItemVariant {
    #[default]
    Primary,
    Destructive,
}

/// Resolved geometry and hover colors, shared with the tile port.
///
/// Forui's `FItemStyle` and `FTileStyle` both draw their content as a rounded
/// decoration, but with different fills, padding and borders. Keeping those
/// differences in one place avoids duplicating the item's paint code.
#[derive(Clone, Copy, Debug)]
pub(crate) struct ItemAppearance {
    /// Minimum row height in logical points.
    pub min_height: f32,
    /// Content padding inside the decorated container.
    pub content_padding: Vec2,
    /// Horizontal margin outside the decorated container (Forui item: 4).
    pub outer_margin: f32,
    /// Container corner radius (Forui `borderRadius.md` = 10).
    pub corner_radius: f32,
    /// Fill when idle.
    pub idle_fill: Color32,
    /// Fill for hovered, pressed and selected rows (Forui `colors.secondary`).
    pub active_fill: Color32,
    /// Container border (Forui item: none; tile: 1 px `colors.border`).
    pub stroke: Stroke,
}

impl ItemAppearance {
    /// Forui's item: page background, content padding 10/6.5 (desktop) or
    /// 10/12.5 (touch), a 4 point outer margin and no border.
    pub(crate) fn item(theme: &Theme) -> Self {
        Self {
            min_height: theme.sizes.item,
            content_padding: Vec2::new(10.0, if theme.touch { 12.5 } else { 6.5 }),
            outer_margin: 4.0,
            corner_radius: 10.0,
            idle_fill: theme.background,
            active_fill: theme.secondary,
            stroke: Stroke::NONE,
        }
    }

    /// Forui's tile: card background with a 1 point border, content padding
    /// 15/14.5, no outer margin and the 48 point tile height.
    pub(crate) fn tile(theme: &Theme) -> Self {
        Self {
            min_height: theme.sizes.tile,
            content_padding: Vec2::new(15.0, 14.5),
            outer_margin: 0.0,
            corner_radius: 10.0,
            idle_fill: theme.card,
            active_fill: theme.secondary,
            stroke: Stroke::new(1.0, theme.border),
        }
    }
}

/// A caller-controlled selectable item. A click is reported through its response.
pub struct Item<'a> {
    title: &'a str,
    description: Option<&'a str>,
    icon: Option<&'a str>,
    theme: &'a Theme,
    variant: ItemVariant,
    selected: bool,
    enabled: bool,
    id: Option<egui::Id>,
    appearance: Option<ItemAppearance>,
    flat: bool,
    radius: Option<u8>,
    lighting: Option<Option<Vesper>>,
}

impl<'a> Item<'a> {
    pub fn new(title: &'a str, theme: &'a Theme) -> Self {
        Self {
            title,
            description: None,
            icon: None,
            theme,
            variant: ItemVariant::Primary,
            selected: false,
            enabled: true,
            id: None,
            appearance: None,
            flat: false,
            radius: None,
            lighting: None,
        }
    }
    pub fn description(mut self, description: &'a str) -> Self {
        self.description = Some(description);
        self
    }
    /// An interim text glyph shown before the title, for example `"◉"`.
    ///
    /// The glyph is decorative and is not part of the accessible name. Real
    /// iconography is not migrated.
    pub fn icon(mut self, icon: &'a str) -> Self {
        self.icon = Some(icon);
        self
    }
    pub fn variant(mut self, variant: ItemVariant) -> Self {
        self.variant = variant;
        self
    }
    pub fn selected(mut self, selected: bool) -> Self {
        self.selected = selected;
        self
    }
    pub fn enabled(mut self, enabled: bool) -> Self {
        self.enabled = enabled;
        self
    }
    /// Use a stable, globally unique ID to preserve focus when rows move.
    pub fn id(mut self, id: egui::Id) -> Self {
        self.id = Some(id);
        self
    }
    /// Draw an inset interaction surface only when the row is active.
    pub fn flat(mut self, flat: bool) -> Self {
        self.flat = flat;
        self
    }

    /// Set the resting corner radius in logical points.
    pub fn radius(mut self, radius: u8) -> Self {
        self.radius = Some(radius);
        self
    }

    /// Override inherited lighting. Pass `None` to disable it for this row.
    pub fn lighting(mut self, lighting: Option<Vesper>) -> Self {
        self.lighting = Some(lighting);
        self
    }
    /// Whether this item is enabled. Unlike [`Response::enabled`], this is
    /// accurate during egui's invisible popup sizing pass.
    pub fn is_enabled(&self) -> bool {
        self.enabled
    }
    /// Whether this item is shown as selected.
    pub fn is_selected(&self) -> bool {
        self.selected
    }

    /// Replace the resolved appearance; used by the tile port.
    pub(crate) fn appearance(mut self, appearance: ItemAppearance) -> Self {
        self.appearance = Some(appearance);
        self
    }

    pub fn show(self, ui: &mut Ui) -> Response {
        if let Some(id) = self.id {
            ui.scope_builder(egui::UiBuilder::new().scope_id(id), |ui| {
                self.show_inner(ui)
            })
            .inner
        } else {
            self.show_inner(ui)
        }
    }

    fn show_inner(self, ui: &mut Ui) -> Response {
        let theme = self.theme;
        let mut appearance = self
            .appearance
            .unwrap_or_else(|| ItemAppearance::item(theme));
        if self.flat {
            appearance.idle_fill = Color32::TRANSPARENT;
            appearance.stroke = Stroke::NONE;
            appearance.outer_margin = 0.0;
        }
        let ui_enabled = ui.is_enabled();
        let enabled = self.enabled && ui_enabled;
        let base_title_color = match self.variant {
            ItemVariant::Primary => theme.foreground,
            ItemVariant::Destructive => theme.destructive,
        };
        // Dim explicitly only when this item is disabled inside an enabled parent.
        // A disabled parent Ui is already dimmed by egui, so dimming again would
        // halve the contrast twice (mirrors the switch's guard).
        let dim = !self.enabled && ui_enabled;
        let title_color = if dim {
            theme.disable(base_title_color)
        } else {
            base_title_color
        };
        let subtitle_color = if dim {
            theme.disable(theme.muted_foreground)
        } else {
            theme.muted_foreground
        };
        // Upstream paints muted/destructive text on the `secondary` fill, which
        // is 4.35:1 in the light palette. Nudge the active-row colors to WCAG AA
        // while keeping their hue; idle rows are unchanged.
        let active_title_color = theme.readable_on(title_color, appearance.active_fill, 4.5);
        let active_subtitle_color = theme.readable_on(subtitle_color, appearance.active_fill, 4.5);

        let width = ui.available_width();
        let max_width = (width - 2.0 * appearance.outer_margin).max(1.0);
        let text_width = (max_width - 2.0 * appearance.content_padding.x).max(1.0);
        let title_galley = layout_title(
            ui,
            self.icon,
            self.title,
            FontId::proportional(theme.typography.body.size),
            title_color,
            theme.muted_foreground,
            text_width,
        );
        let subtitle_galley = self.description.map(|description| {
            layout(
                ui,
                description,
                FontId::proportional(theme.typography.caption.size),
                subtitle_color,
                text_width,
            )
        });
        // Item content style: titleSpacing 4 (desktop and tile default: 3).
        let title_spacing = if appearance.outer_margin == 0.0 {
            3.0
        } else {
            4.0
        };
        let text_height = title_galley.size().y
            + subtitle_galley
                .as_ref()
                .map_or(0.0, |galley| title_spacing + galley.size().y);
        let height = (2.0 * appearance.content_padding.y + text_height).max(appearance.min_height);
        let sense = if enabled {
            Sense::click()
        } else {
            Sense::hover()
        };
        let (rect, response) = ui.allocate_exact_size(Vec2::new(width, height), sense);
        response
            .widget_info(|| WidgetInfo::selected(Role::Button, enabled, self.selected, self.title));
        ui.ctx().accesskit_node_builder(response.id, |node| {
            node.set_label(self.title);
            node.clear_toggled();
            node.set_selected(self.selected);
            if let Some(description) = self.description {
                node.set_description(description);
            }
        });
        if ui.is_rect_visible(rect) {
            let container = egui::Rect::from_min_max(
                egui::pos2(rect.left() + appearance.outer_margin, rect.top()),
                egui::pos2(rect.right() - appearance.outer_margin, rect.bottom()),
            );
            let active = enabled
                && (self.selected || response.hovered() || response.is_pointer_button_down_on());
            let amount = crate::motion::spring_bool(
                ui.ctx(),
                response.id.with("item-selection"),
                active,
                theme,
            )
            .clamp(0.0, 1.0);
            let press = crate::motion_controls::press_amount(&response, theme);
            let visual = container.scale_from_center(1.0 - 0.028 * press);
            let radius = self.radius.unwrap_or(appearance.corner_radius as u8);
            let fill = egui::Color32::from(
                egui::Rgba::from(appearance.idle_fill) * (1.0 - amount)
                    + egui::Rgba::from(appearance.active_fill) * amount,
            );
            let title_color = if amount > 0.5 {
                active_title_color
            } else {
                title_color
            };
            let subtitle_color = if amount > 0.5 {
                active_subtitle_color
            } else {
                subtitle_color
            };
            ui.painter().add(crate::motion::press_shape(
                egui::epaint::RectShape::new(
                    visual,
                    radius,
                    fill,
                    appearance.stroke,
                    StrokeKind::Inside,
                ),
                press,
            ));
            if enabled
                && let Some(style) = self.lighting.unwrap_or_else(|| Vesper::context(ui.ctx()))
            {
                let corners = crate::motion::press_corners(visual, radius.into(), press);
                if let Some(light) = style.interactive_shape_on(
                    ui.painter(),
                    theme,
                    &response,
                    visual,
                    corners,
                    fill,
                ) {
                    ui.painter().add(light);
                }
                crate::lighting::spill::receive(
                    ui.painter(),
                    response.id,
                    visual,
                    corners,
                    &style,
                    fill,
                    theme.card,
                );
            }
            crate::motion::remember_anchor(&response, visual);
            if response.has_focus() {
                ui.painter().add(crate::motion::press_shape(
                    egui::epaint::RectShape::stroke(
                        visual,
                        radius,
                        Stroke::new(2.0, theme.primary),
                        StrokeKind::Inside,
                    ),
                    press,
                ));
            }
            let mut y = rect.center().y - text_height / 2.0;
            let x = container.left() + appearance.content_padding.x;
            ui.painter()
                .galley(egui::pos2(x, y), title_galley.clone(), title_color);
            y += title_galley.size().y + title_spacing;
            if let Some(galley) = subtitle_galley {
                ui.painter()
                    .galley(egui::pos2(x, y), galley, subtitle_color);
            }
        }
        response
    }
}

fn layout(
    ui: &Ui,
    text: &str,
    font_id: FontId,
    color: Color32,
    max_width: f32,
) -> std::sync::Arc<Galley> {
    let mut job = LayoutJob::default();
    job.wrap.max_width = max_width;
    job.wrap.break_anywhere = true;
    job.append(
        text,
        0.0,
        TextFormat {
            font_id,
            color,
            ..Default::default()
        },
    );
    ui.fonts_mut(|fonts| fonts.layout_job(job))
}

/// Lay out an optional decorative glyph followed by the title.
fn layout_title(
    ui: &Ui,
    icon: Option<&str>,
    title: &str,
    font_id: FontId,
    color: Color32,
    icon_color: Color32,
    max_width: f32,
) -> std::sync::Arc<Galley> {
    let mut job = LayoutJob::default();
    job.wrap.max_width = max_width;
    job.wrap.break_anywhere = true;
    if let Some(icon) = icon {
        job.append(
            icon,
            0.0,
            TextFormat {
                font_id: font_id.clone(),
                color: icon_color,
                ..Default::default()
            },
        );
        job.append("  ", 0.0, TextFormat::default());
    }
    job.append(
        title,
        0.0,
        TextFormat {
            font_id,
            color,
            ..Default::default()
        },
    );
    ui.fonts_mut(|fonts| fonts.layout_job(job))
}
