use egui::{Align2, Color32, CornerRadius, FontId, Rect, Sense, Stroke, StrokeKind, Ui, Vec2};
use temivu::{
    Button, ButtonVariant, EffectSurface, Glass, LightQuality, LightSpill, LightTracking, Slider,
    Switch, Theme, Vesper,
};

const AMBER: Color32 = Color32::from_rgb(252, 219, 180);
const STAGE: Color32 = Color32::from_gray(34);
const INK: Color32 = Color32::from_gray(237);

pub(super) struct LightingDemo {
    enabled: bool,
    surfaces_enabled: bool,
    spread_enabled: bool,
    spill: LightSpill,
    custom_text: String,
    floating: bool,
    style: Vesper,
    switch_value: bool,
    stress: bool,
    sweep_started: Option<f64>,
}

impl Default for LightingDemo {
    fn default() -> Self {
        Self {
            enabled: true,
            surfaces_enabled: false,
            spread_enabled: true,
            spill: LightSpill::default(),
            custom_text: String::new(),
            floating: false,
            style: Vesper::expressive(),
            switch_value: false,
            stress: false,
            sweep_started: None,
        }
    }
}

impl LightingDemo {
    pub(super) fn enable_surfaces(&mut self) {
        self.surfaces_enabled = true;
    }

    pub(super) fn surface_lighting(&self, touch: bool) -> Option<Vesper> {
        self.surfaces_enabled.then_some(Vesper {
            color: None,
            quality: if touch && self.style.quality == LightQuality::Balanced {
                LightQuality::Low
            } else {
                self.style.quality
            },
            ..self.preview_style()
        })
    }

    fn preview_style(&self) -> Vesper {
        Vesper {
            spill: self.spread_enabled.then_some(self.spill),
            ..self.style
        }
    }

    pub fn show(&mut self, ui: &mut Ui, theme: &Theme) {
        super::studio_panel(
            ui,
            theme,
            "Vesper lighting",
            "A white highlight follows your touch, spreads across the surface and catches its edge.",
            |ui| {
                Switch::new(&mut self.enabled, theme)
                    .label("Enable lighting preview")
                    .show(ui);
                if Switch::new(&mut self.surfaces_enabled, theme)
                    .label("Light interactive controls")
                    .description("Apply Vesper to supported controls throughout Showcase")
                    .show(ui)
                    .changed()
                {
                    ui.ctx().request_repaint();
                }
                Switch::new(&mut self.spread_enabled, theme)
                    .label("Spread light to nearby surfaces")
                    .description("Turn off sharing while keeping the local highlight")
                    .show(ui);
                self.surface(ui, theme);
                ui.add_space(12.0);
                self.neighbors(ui, theme);
                ui.add_space(12.0);
                let glass = Glass::context(ui.ctx());
                let lighting = self.enabled.then(|| self.preview_style());
                EffectSurface::new(theme)
                    .id_salt("custom-effect-surface")
                    .glass(glass)
                    .lighting(lighting)
                    .show(ui, |ui| {
                        ui.label("Your own content, the same material");
                        ui.label("Glass and lighting also work around ordinary egui content.");
                        ui.add(
                            egui::TextEdit::singleline(&mut self.custom_text)
                                .hint_text("Try a custom field")
                                .desired_width(ui.available_width()),
                        );
                    });
                ui.add_space(8.0);
                self.floating_preview(ui, theme, glass, lighting);
                ui.add_space(8.0);
                ui.horizontal_wrapped(|ui| {
                    Button::new("Lit action", theme).lighting(lighting).show(ui);
                    Button::new("Without lighting", theme)
                        .variant(ButtonVariant::Outline)
                        .lighting(None)
                        .show(ui);
                    Switch::new(&mut self.switch_value, theme)
                        .label("Lit switch")
                        .lighting(lighting)
                        .show(ui);
                });
                ui.add_space(12.0);
                egui::CollapsingHeader::new("Lighting settings")
                    .default_open(true)
                    .show(ui, |ui| self.settings(ui, theme));
                ui.add_space(8.0);
                Switch::new(&mut self.stress, theme)
                    .label("Show simultaneous lights")
                    .show(ui);
                if self.stress {
                    self.stress_grid(ui, theme);
                } else {
                    self.sweep_started = None;
                }
            },
        );
    }

    fn floating_preview(
        &mut self,
        ui: &mut Ui,
        theme: &Theme,
        glass: Option<Glass>,
        light: Option<Vesper>,
    ) {
        if Button::new("Launch floating preview", theme)
            .variant(ButtonVariant::Outline)
            .show(ui)
            .clicked()
        {
            self.floating = !self.floating;
        }
        let id = ui.make_persistent_id("custom-flight");
        let surface = EffectSurface::new(theme)
            .elevated()
            .glass(glass)
            .lighting(light);
        temivu::FloatingMotion::flight(egui::vec2(-54.0, 34.0)).show(
            ui.ctx(),
            id,
            (Align2::RIGHT_TOP, egui::vec2(-16.0, 90.0)),
            self.floating,
            surface,
            |ui| {
                ui.horizontal(|ui| {
                    ui.label("Floating preview");
                    if Button::new("Close", theme)
                        .variant(ButtonVariant::Ghost)
                        .show(ui)
                        .clicked()
                    {
                        self.floating = false;
                    }
                })
            },
        );
    }

    fn surface(&self, ui: &mut Ui, theme: &Theme) {
        let height = if ui.available_width() < 420.0 {
            184.0
        } else {
            224.0
        };
        let (rect, response) = ui.allocate_exact_size(
            Vec2::new(ui.available_width(), height),
            Sense::click_and_drag(),
        );
        response.widget_info(|| {
            egui::WidgetInfo::labeled(egui::Role::Button, true, "Vesper interactive surface")
        });
        if !ui.is_rect_visible(rect) {
            return;
        }

        let press = if self.enabled && self.style.effects.depth {
            temivu::motion::spring_f32(
                ui.ctx(),
                response.id.with("surface-press"),
                0.0,
                f32::from(response.is_pointer_button_down_on() || response.dragged()),
                theme,
            )
            .clamp(0.0, 1.0)
        } else {
            0.0
        };
        let surface = rect.scale_from_center(1.0 - 0.016 * press);
        let corners = CornerRadius::same(24);
        let fill = if theme.dark {
            STAGE
        } else {
            Color32::from_gray(250)
        };
        let text = if theme.dark { INK } else { theme.foreground };
        let painter = ui.painter();
        painter.rect(
            surface,
            corners,
            fill,
            Stroke::new(1.0, theme.border),
            StrokeKind::Inside,
        );
        if self.enabled {
            self.preview_style()
                .paint_on(painter, theme, &response, surface, corners, fill);
        }

        painter.text(
            surface.center(),
            Align2::CENTER_CENTER,
            "Hover, hold or drag",
            FontId::proportional(13.0),
            text,
        );
        if response.has_focus() {
            painter.rect_stroke(
                surface.shrink(3.0),
                22,
                Stroke::new(1.5, text),
                StrokeKind::Inside,
            );
        }
    }

    fn neighbors(&self, ui: &mut Ui, theme: &Theme) {
        ui.label(egui::RichText::new("Connected surfaces").strong());
        ui.label("Move across the dark tile. Nearby edges catch the same light.");
        let (stage, _) =
            ui.allocate_exact_size(egui::vec2(ui.available_width(), 192.0), Sense::hover());
        if !ui.is_rect_visible(stage) {
            return;
        }
        let painter = ui.painter();
        let base = if theme.dark {
            Color32::from_gray(22)
        } else {
            Color32::from_gray(225)
        };
        painter.rect_filled(stage, 20, base);
        let style = self.preview_style();
        if self.enabled {
            style.receive(
                painter,
                theme,
                ui.scope_id().with("lit-environment"),
                stage,
                20.into(),
                base,
            );
        }
        let inner = stage.shrink(14.0);
        let gap = 12.0;
        let left_width = ((inner.width() - gap) * 0.48).max(1.0);
        let emitter = Rect::from_min_size(inner.min, egui::vec2(left_width, inner.height()));
        let right = Rect::from_min_max(egui::pos2(emitter.right() + gap, inner.top()), inner.max);
        // Register the receiving objects first to demonstrate same-frame sharing
        // independent of the source's drawing order.
        for (index, (label, fill)) in [
            ("Nearby card", Color32::from_gray(248)),
            ("Another object", Color32::from_gray(72)),
        ]
        .into_iter()
        .enumerate()
        {
            let height = (right.height() - gap) * 0.5;
            let rect = Rect::from_min_size(
                right.min + egui::vec2(0.0, index as f32 * (height + gap)),
                egui::vec2(right.width(), height),
            );
            painter.rect_filled(rect, 16, fill);
            if self.enabled {
                style.receive(
                    painter,
                    theme,
                    ui.scope_id().with(("neighbor", index)),
                    rect,
                    16.into(),
                    fill,
                );
            }
            painter.text(
                rect.center(),
                Align2::CENTER_CENTER,
                label,
                FontId::proportional(if right.width() < 150.0 { 11.0 } else { 14.0 }),
                if index == 0 {
                    Color32::from_gray(30)
                } else {
                    INK
                },
            );
        }
        let response = ui.interact(
            emitter,
            ui.scope_id().with("spill-source"),
            Sense::click_and_drag(),
        );
        response.widget_info(|| {
            egui::WidgetInfo::labeled(egui::Role::Button, true, "Shared light source")
        });
        painter.rect_filled(emitter, 20, STAGE);
        if self.enabled {
            style.paint_on(painter, theme, &response, emitter, 20.into(), STAGE);
        }
        painter.text(
            emitter.center(),
            Align2::CENTER_CENTER,
            "Move the light",
            FontId::proportional(13.0),
            INK,
        );
    }

    fn settings(&mut self, ui: &mut Ui, theme: &Theme) {
        ui.horizontal_wrapped(|ui| {
            ui.label("Preset");
            if Button::new("Quiet", theme)
                .variant(ButtonVariant::Outline)
                .show(ui)
                .clicked()
            {
                self.style = Vesper::subtle();
            }
            if Button::new("Expressive", theme)
                .variant(ButtonVariant::Outline)
                .show(ui)
                .clicked()
            {
                self.style = Vesper::expressive();
            }
        });
        ui.horizontal_wrapped(|ui| {
            ui.label("Quality");
            for (quality, label) in [
                (LightQuality::Low, "Low"),
                (LightQuality::Balanced, "Balanced"),
                (LightQuality::High, "High"),
            ] {
                if option(ui, theme, label, quality == self.style.quality) {
                    self.style.quality = quality;
                }
            }
        });
        ui.horizontal_wrapped(|ui| {
            ui.label("Tracking");
            for (tracking, label) in [
                (LightTracking::Direct, "Direct"),
                (LightTracking::Smooth, "Smooth"),
                (LightTracking::Spring, "Spring"),
            ] {
                if option(ui, theme, label, tracking == self.style.tracking) {
                    self.style.tracking = tracking;
                }
            }
        });
        ui.horizontal_wrapped(|ui| {
            ui.label("Color");
            for (color, label) in [(None, "White"), (Some(AMBER), "Warm")] {
                if option(ui, theme, label, self.style.color == color) {
                    self.style.color = color;
                }
            }
        });
        ui.horizontal_wrapped(|ui| {
            let effects = &mut self.style.effects;
            for (active, label) in [
                (&mut effects.follow, "Follow"),
                (&mut effects.edge, "Contour"),
                (&mut effects.scatter, "Scatter"),
                (&mut effects.flow, "Flow"),
                (&mut effects.particles, "Motes"),
                (&mut effects.depth, "Depth"),
            ] {
                if option(ui, theme, label, *active) {
                    *active = !*active;
                }
            }
        });
        ui.add_space(8.0);
        if self.spread_enabled {
            normalized(
                ui,
                theme,
                "Light spread reach",
                &mut self.spill.radius,
                48.0..=280.0,
            );
            normalized(
                ui,
                theme,
                "Light spread strength",
                &mut self.spill.intensity,
                0.0..=1.0,
            );
        }
        normalized(
            ui,
            theme,
            "Light intensity",
            &mut self.style.intensity,
            0.0..=1.0,
        );
        normalized(
            ui,
            theme,
            "Light radius",
            &mut self.style.radius,
            24.0..=180.0,
        );
        normalized(
            ui,
            theme,
            "Light softness",
            &mut self.style.softness,
            0.0..=1.0,
        );
        normalized(
            ui,
            theme,
            "Tracking inertia",
            &mut self.style.inertia,
            0.0..=1.0,
        );
        egui::CollapsingHeader::new("Advanced light parameters").show(ui, |ui| {
            normalized(
                ui,
                theme,
                "Surface opacity",
                &mut self.style.opacity,
                0.0..=1.0,
            );
            normalized(ui, theme, "Falloff", &mut self.style.attenuation, 0.5..=3.0);
            normalized(
                ui,
                theme,
                "Scatter speed",
                &mut self.style.propagation,
                0.1..=2.0,
            );
            normalized(
                ui,
                theme,
                "Surface elevation",
                &mut self.style.elevation,
                0.0..=1.0,
            );
        });
    }

    fn stress_grid(&mut self, ui: &mut Ui, theme: &Theme) {
        ui.label("24 lights with one shared sweep.");
        let motion = theme.motion.animations_enabled(ui.ctx());
        let now = ui.input(|input| input.time);
        if Button::new("Run sweep", theme)
            .variant(ButtonVariant::Outline)
            .enabled(self.enabled && motion)
            .show(ui)
            .clicked()
        {
            self.sweep_started = Some(now);
        }
        let mut phase = 0.0;
        if let Some(start) = self.sweep_started {
            let elapsed = (now - start).max(0.0);
            if self.enabled && motion && elapsed < 2.4 {
                phase = elapsed as f32 * std::f32::consts::TAU / 2.4;
                ui.ctx().request_repaint();
            } else {
                self.sweep_started = None;
            }
        }

        let width = ui.available_width();
        let columns = if width < 420.0 { 4 } else { 8 };
        let gap = 6.0;
        let size = ((width - (columns - 1) as f32 * gap) / columns as f32).max(1.0);
        let (grid, _) = ui.allocate_exact_size(
            egui::vec2(width, (24 / columns) as f32 * (size + gap) - gap),
            Sense::hover(),
        );
        let corners = CornerRadius::same(12);
        for index in 0..24 {
            let cell = Rect::from_min_size(
                grid.min
                    + egui::vec2(
                        (index % columns) as f32 * (size + gap),
                        (index / columns) as f32 * (size + gap),
                    ),
                Vec2::splat(size),
            );
            if !ui.is_rect_visible(cell) {
                continue;
            }
            ui.painter().rect_filled(cell, corners, STAGE);
            if self.enabled {
                let angle = phase + index as f32 * 0.32;
                let (y, x) = angle.sin_cos();
                let source = cell.center() + egui::vec2(x, y) * (size * 0.26);
                self.style
                    .paint_at(ui.painter(), theme, cell, corners, source, 0.85);
            }
        }
    }
}

fn option(ui: &mut Ui, theme: &Theme, label: &str, selected: bool) -> bool {
    let variant = if selected {
        ButtonVariant::Secondary
    } else {
        ButtonVariant::Ghost
    };
    let response = Button::new(label, theme).variant(variant).show(ui);
    ui.ctx()
        .accesskit_node_builder(response.id, |node| node.set_selected(selected));
    response.clicked()
}

fn normalized(
    ui: &mut Ui,
    theme: &Theme,
    label: &str,
    value: &mut f32,
    range: std::ops::RangeInclusive<f32>,
) {
    let width = range.end() - range.start();
    let mut position = ((*value - range.start()) / width).clamp(0.0, 1.0) as f64;
    if Slider::new(&mut position, theme)
        .expect("the normalized value stays in range")
        .label(label)
        .show(ui)
        .changed()
    {
        *value = range.start() + position as f32 * width;
    }
}

#[cfg(test)]
mod tests {
    use super::{LightQuality, LightingDemo, Theme};
    use egui_kittest::{Harness, kittest::Queryable};

    #[test]
    fn lighting_studio_controls_work_in_a_narrow_viewport() {
        let mut harness = Harness::builder()
            .with_size(egui::vec2(390.0, 2400.0))
            .build_ui_state(
                |ui, demo: &mut LightingDemo| demo.show(ui, &Theme::light().touch()),
                LightingDemo::default(),
            );
        harness.run_steps(3);
        let surface = harness.get_by_label("Vesper interactive surface").rect();
        assert!(surface.width() > 200.0 && surface.right() < 390.0);

        harness.get_by_label("Lit switch").click();
        harness.run_steps(3);
        assert!(harness.state().switch_value);

        harness.get_by_label("Low").click();
        harness.run_steps(3);
        assert_eq!(harness.state().style.quality, LightQuality::Low);

        let previous = harness.state().style.effects.particles;
        harness.get_by_label("Motes").click();
        harness.run_steps(3);
        assert_eq!(harness.state().style.effects.particles, !previous);

        harness.get_by_label("Show simultaneous lights").click();
        harness.run_steps(3);
        harness.get_by_label("Run sweep");
        harness.get_by_label("Enable lighting preview").click();
        harness.run_steps(3);
        assert!(!harness.state().enabled);
    }
}
