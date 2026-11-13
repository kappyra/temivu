use egui::{Align2, FontId, Pos2, Sense, Stroke, Ui, Vec2};
use temivu::motion::{Decay, MotionSpec, MotionTrack, Spring, SpringPreset, rubber_band};
use temivu::{Button, ButtonVariant, Theme};

pub(super) struct Lab {
    preset: SpringPreset,
    custom: bool,
    mass: f64,
    stiffness: f64,
    damping: f64,
    target: f32,
    momentum: bool,
    coasting: bool,
}

impl Default for Lab {
    fn default() -> Self {
        Self {
            preset: SpringPreset::Responsive,
            custom: false,
            mass: 1.0,
            stiffness: 500.0,
            damping: 34.0,
            target: 0.2,
            momentum: false,
            coasting: false,
        }
    }
}

impl Lab {
    pub(super) fn show(&mut self, ui: &mut Ui, theme: &Theme) {
        ui.horizontal_wrapped(|ui| {
            for (preset, label) in [
                (SpringPreset::Soft, "Soft"),
                (SpringPreset::Snappy, "Snappy"),
                (SpringPreset::Bouncy, "Bouncy"),
                (SpringPreset::Smooth, "Smooth"),
                (SpringPreset::Gentle, "Gentle"),
                (SpringPreset::Responsive, "Responsive"),
            ] {
                let selected = self.preset == preset && !self.custom;
                let button = Button::new(label, theme)
                    .variant(if selected {
                        ButtonVariant::Secondary
                    } else {
                        ButtonVariant::Ghost
                    })
                    .show(ui);
                ui.ctx()
                    .accesskit_node_builder(button.id, |node| node.set_selected(selected));
                if button.clicked() {
                    self.preset = preset;
                    self.custom = false;
                    self.reverse();
                }
            }
        });
        ui.checkbox(&mut self.custom, "Custom spring");
        if self.custom {
            ui.add(egui::Slider::new(&mut self.mass, 0.25..=3.0).text("Mass"));
            ui.add(egui::Slider::new(&mut self.stiffness, 80.0..=1400.0).text("Stiffness"));
            ui.add(egui::Slider::new(&mut self.damping, 4.0..=150.0).text("Damping"));
        }
        ui.checkbox(&mut self.momentum, "Coast on release");
        if Button::new("Reverse target", theme)
            .variant(ButtonVariant::Outline)
            .show(ui)
            .clicked()
        {
            self.reverse();
        }
        let preset = self.preset.spring(theme.motion);
        let spring = if self.custom {
            Spring::physical(self.mass, self.stiffness, self.damping).unwrap_or(preset)
        } else {
            preset
        };
        let regime = match spring.damping_ratio() {
            ratio if (ratio - 1.0).abs() < 1e-6 => "Critical",
            ratio if ratio < 1.0 => "Underdamped",
            _ => "Overdamped",
        };
        ui.label(format!("{regime}  ·  ζ {:.2}", spring.damping_ratio()));
        let (rect, response) = ui.allocate_exact_size(
            Vec2::new(ui.available_width().max(1.0), 76.0),
            Sense::drag(),
        );
        if !ui.is_rect_visible(rect) {
            return;
        }
        let track = MotionTrack::new(ui.ctx(), response.id, theme.motion);
        let decay = Decay::new(7.0).expect("valid demo friction");
        let rail = rect.shrink2(egui::vec2(24.0_f32.min(rect.width() * 0.2), 0.0));
        let value = if response.dragged()
            && let Some(point) = response.interact_pointer_pos()
        {
            let raw = (point.x - rail.left()) / rail.width();
            let clamped = raw.clamp(0.0, 1.0);
            let velocity = ui
                .input(|i| i.pointer.velocity().x / rail.width())
                .clamp(-12.0, 12.0);
            let position = if theme.motion.animations_enabled(ui.ctx()) {
                clamped + rubber_band(raw - clamped, 0.09)
            } else {
                clamped
            };
            self.target = clamped;
            self.coasting = false;
            track.follow(position, [f64::from(velocity)]);
            position
        } else {
            if response.drag_stopped()
                && let Some(sample) = track.inspect::<f32, 1>()
            {
                let projected =
                    decay.destination(f64::from(sample.value), sample.velocity[0]) as f32;
                self.coasting = self.momentum && (0.0..=1.0).contains(&sample.value);
                self.target = if !theme.motion.animations_enabled(ui.ctx()) {
                    sample.value.clamp(0.0, 1.0)
                } else if self.coasting {
                    projected.clamp(0.0, 1.0)
                } else if projected < 0.5 {
                    0.2
                } else {
                    0.8
                };
            }
            let spec = if self.coasting {
                MotionSpec::Decay(decay)
            } else {
                MotionSpec::Spring(spring)
            };
            track.sample(0.2, self.target, spec).value
        };
        let painter = ui.painter().with_clip_rect(rect.intersect(ui.clip_rect()));
        let at = |value| Pos2::new(egui::lerp(rail.x_range(), value), rail.center().y);
        painter.line_segment([at(0.0), at(1.0)], Stroke::new(3.0, theme.border));
        for stop in [0.2, 0.8] {
            painter.circle_filled(at(stop), 3.0, theme.muted_foreground);
        }
        painter.circle_filled(at(value), 13.0, theme.primary);
        painter.text(
            rect.center_bottom(),
            Align2::CENTER_BOTTOM,
            "Drag or reverse the target",
            FontId::proportional(11.0),
            theme.muted_foreground,
        );
    }

    fn reverse(&mut self) {
        self.target = if self.target < 0.5 { 0.8 } else { 0.2 };
        self.coasting = false;
    }
}
