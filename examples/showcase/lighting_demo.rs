use egui::{Align2, Color32, CornerRadius, FontId, Rect, Sense, Stroke, StrokeKind, Ui, Vec2};
use temivu::{Button, ButtonVariant, LightQuality, LightTracking, Slider, Switch, Theme, Vesper};

const PEARL: Color32 = Color32::from_rgb(176, 193, 255);
const AMBER: Color32 = Color32::from_rgb(255, 193, 145);
const STAGE: Color32 = Color32::from_rgb(22, 27, 44);
const INK: Color32 = Color32::from_rgb(228, 233, 251);
const SECONDARY_INK: Color32 = Color32::from_rgb(166, 179, 216);

pub(super) struct LightingDemo {
    enabled: bool,
    style: Vesper,
    switch_value: bool,
    stress: bool,
    sweep_started: Option<f64>,
}

impl Default for LightingDemo {
    fn default() -> Self {
        let mut style = Vesper::expressive();
        style.color = Some(PEARL);
        Self {
            enabled: true,
            style,
            switch_value: false,
            stress: false,
            sweep_started: None,
        }
    }
}

impl LightingDemo {
    pub fn show(&mut self, ui: &mut Ui, theme: &Theme) {
        super::studio_panel(
            ui,
            theme,
            "Vesper lighting",
            "A light that follows interaction, with a soft surface wash and a sharper contour.",
            |ui| {
                Switch::new(&mut self.enabled, theme)
                    .label("Enable lighting preview")
                    .show(ui);
                self.surface(ui, theme);
                ui.add_space(8.0);
                ui.horizontal_wrapped(|ui| {
                    Button::new("Lit action", theme)
                        .lighting(self.enabled.then_some(self.style))
                        .show(ui);
                    Button::new("Without lighting", theme)
                        .variant(ButtonVariant::Outline)
                        .lighting(None)
                        .show(ui);
                    Switch::new(&mut self.switch_value, theme)
                        .label("Lit switch")
                        .lighting(self.enabled.then_some(self.style))
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
            egui::WidgetInfo::labeled(egui::WidgetType::Button, true, "Vesper interactive surface")
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
        let painter = ui.painter();
        painter.rect(
            surface,
            corners,
            STAGE,
            Stroke::new(1.0, Color32::from_rgb(52, 62, 88)),
            StrokeKind::Inside,
        );
        if self.enabled {
            self.style
                .paint(painter, theme, &response, surface, corners);
        }

        let center = surface.center() - egui::vec2(0.0, 11.0);
        let mark_radius = 24.0 - press * 2.0;
        for scale in [1.0, 1.34, 1.70] {
            painter.circle_stroke(
                center,
                mark_radius * scale,
                Stroke::new(
                    0.85,
                    SECONDARY_INK.gamma_multiply(0.34 - 0.13 * (scale - 1.0)),
                ),
            );
        }
        painter.circle_filled(center, 3.2, INK);
        painter.text(
            center + egui::vec2(0.0, 55.0),
            Align2::CENTER_CENTER,
            "Hover, press or drag",
            FontId::proportional(13.0),
            INK,
        );
        if response.has_focus() {
            painter.rect_stroke(
                surface.shrink(3.0),
                22,
                Stroke::new(1.5, INK),
                StrokeKind::Inside,
            );
        }
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
                self.style.color = Some(PEARL);
            }
            if Button::new("Expressive", theme)
                .variant(ButtonVariant::Outline)
                .show(ui)
                .clicked()
            {
                self.style = Vesper::expressive();
                self.style.color = Some(AMBER);
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
            for (color, label) in [(PEARL, "Pearl"), (AMBER, "Amber")] {
                if option(ui, theme, label, self.style.color == Some(color)) {
                    self.style.color = Some(color);
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
            .with_size(egui::vec2(390.0, 1600.0))
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
