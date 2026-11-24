use egui::{Color32, ColorImage, Context, Frame, Rect, TextureHandle, TextureOptions, Ui, Vec2};
use temivu::{
    Button, ButtonVariant, Glass, GlassBackdrop, GlassTuning, LightSurface, Slider, Switch, Theme,
    Vesper, motion::ElasticDrag,
};

#[derive(Clone, Copy, PartialEq, Eq)]
enum Preset {
    Frost,
    ClearFrost,
    SoftPrism,
    Prism,
}

impl Preset {
    const ALL: [Self; 4] = [Self::Frost, Self::ClearFrost, Self::SoftPrism, Self::Prism];

    fn label(self) -> &'static str {
        match self {
            Self::Frost => "Frost",
            Self::ClearFrost => "Clear Frost",
            Self::SoftPrism => "Soft Prism",
            Self::Prism => "Prism",
        }
    }

    fn material(self) -> Glass {
        match self {
            Self::Frost => Glass::frost(),
            Self::ClearFrost => Glass::clear_frost(),
            Self::SoftPrism => Glass::soft_prism(),
            Self::Prism => Glass::prism(),
        }
    }

    fn is_frost(self) -> bool {
        matches!(self, Self::Frost | Self::ClearFrost)
    }
}

pub(super) struct GlassDemo {
    pub enabled: bool,
    tint: [f64; Preset::ALL.len()],
    preset: Preset,
    tuning: [GlassTuning; Preset::ALL.len()],
    artwork: Option<Artwork>,
    motion: [SampleMotion; Preset::ALL.len()],
    move_backdrop: bool,
    backdrop_phase: f32,
}

impl Default for GlassDemo {
    fn default() -> Self {
        Self {
            enabled: false,
            tint: Preset::ALL.map(|preset| f64::from(preset.material().tint())),
            preset: Preset::SoftPrism,
            tuning: Preset::ALL.map(|preset| preset.material().tuning()),
            artwork: None,
            motion: Default::default(),
            move_backdrop: false,
            backdrop_phase: 0.0,
        }
    }
}

impl GlassDemo {
    fn material(&self, preset: Preset) -> Glass {
        preset
            .material()
            .with_tint(self.tint[preset as usize] as f32)
            .with_tuning(self.tuning[preset as usize])
    }

    pub fn apply(&self, ctx: &Context, high_contrast: bool) {
        let glass = (self.enabled && !high_contrast).then(|| self.material(self.preset));
        Glass::set_context(ctx, glass);
    }

    pub fn settings(&mut self, ui: &mut Ui, theme: &Theme, high_contrast: bool) {
        ui.add_space(10.0);
        Switch::new(&mut self.enabled, theme)
            .label("Glass surfaces")
            .description("Choose a material for navigation and overlays.")
            .show(ui);
        if self.enabled && high_contrast {
            ui.label("High contrast uses opaque surfaces.");
        } else if self.enabled {
            ui.horizontal_wrapped(|ui| {
                for preset in Preset::ALL {
                    let variant = if self.preset == preset {
                        ButtonVariant::Secondary
                    } else {
                        ButtonVariant::Ghost
                    };
                    let response = Button::new(preset.label(), theme).variant(variant).show(ui);
                    if response.clicked() {
                        self.preset = preset;
                    }
                    ui.ctx().accesskit_node_builder(response.id, |node| {
                        node.set_selected(self.preset == preset);
                    });
                }
            });
            Slider::new(&mut self.tint[self.preset as usize], theme)
                .expect("the tint stays within the slider range")
                .label("Glass tint")
                .filled(true)
                .show(ui);
            ui.collapsing("Optical settings", |ui| {
                let tuning = &mut self.tuning[self.preset as usize];
                tuning_slider(ui, theme, "Backdrop blur", &mut tuning.blur);
                if !self.preset.is_frost() {
                    tuning_slider(ui, theme, "Refraction", &mut tuning.refraction);
                }
                tuning_slider(ui, theme, "Rim", &mut tuning.rim);
            });
        }
        self.apply(ui.ctx(), high_contrast);
    }

    pub fn preview(&mut self, ui: &mut Ui, theme: &Theme) {
        if Glass::context(ui.ctx()).is_none() {
            return;
        }
        super::studio_panel(
            ui,
            theme,
            "Glass studio",
            "Drag a sample or move the backdrop to compare the four materials.",
            |ui| {
                let full_motion = theme.motion.animations_enabled(ui.ctx());
                ui.add_enabled_ui(full_motion, |ui| {
                    Switch::new(&mut self.move_backdrop, theme)
                        .label("Move backdrop")
                        .show(ui);
                });
                ui.add_space(10.0);
                let materials = Preset::ALL.map(|preset| self.material(preset));
                let mut pan = None;
                let mut sample = |ui: &mut Ui, preset: Preset| {
                    let title = match preset {
                        Preset::Frost => "Frost glassmorphism",
                        Preset::ClearFrost => "Clear Frost glassmorphism",
                        Preset::SoftPrism => "Soft Prism optical glass",
                        Preset::Prism => "Prism optical glass",
                    };
                    ui.label(egui::RichText::new(title).strong());
                    ui.add_space(3.0);
                    let (rect, stage) = ui.allocate_exact_size(
                        Vec2::new(ui.available_width(), 164.0),
                        egui::Sense::hover(),
                    );
                    if !ui.is_rect_visible(rect) || !rect.is_positive() {
                        return;
                    }
                    let pan = *pan.get_or_insert_with(|| {
                        if self.move_backdrop && full_motion {
                            let dt = ui.input(|input| input.stable_dt).min(0.05);
                            self.backdrop_phase =
                                (self.backdrop_phase + dt * 0.68) % std::f32::consts::TAU;
                            ui.ctx().request_repaint();
                        }
                        egui::vec2(
                            0.52 * self.backdrop_phase.sin() - 0.35,
                            (self.backdrop_phase * 2.0).sin(),
                        )
                    });
                    let size = Vec2::new((rect.width() * 0.64).min(208.0), 62.0);
                    let index = preset as usize;
                    let sample = self.motion[index].sample(ui, &stage, size, title, theme);
                    let artwork = self.artwork.get_or_insert_with(|| Artwork::new(ui.ctx()));
                    artwork.paint(ui, theme, rect, sample, (preset, materials[index]), pan);
                };
                if ui.available_width() >= 620.0 {
                    for row in Preset::ALL.as_chunks::<2>().0 {
                        ui.columns(2, |columns| {
                            for (column, &preset) in columns.iter_mut().zip(row) {
                                sample(column, preset);
                            }
                        });
                        ui.add_space(12.0);
                    }
                } else {
                    for preset in Preset::ALL {
                        sample(ui, preset);
                        ui.add_space(12.0);
                    }
                }
            },
        );
    }
}

fn tuning_slider(ui: &mut Ui, theme: &Theme, label: &str, multiplier: &mut f32) {
    let mut position = f64::from(*multiplier) * 0.5;
    if Slider::new(&mut position, theme)
        .expect("the multiplier stays within its slider range")
        .label(label)
        .filled(true)
        .show(ui)
        .changed()
    {
        *multiplier = (position * 2.0) as f32;
    }
}

#[derive(Default)]
struct SampleMotion {
    position: Vec2,
}

impl SampleMotion {
    fn sample(
        &mut self,
        ui: &Ui,
        stage: &egui::Response,
        size: Vec2,
        label: &str,
        theme: &Theme,
    ) -> (Rect, egui::Response) {
        let travel = ((stage.rect.size() - size) * 0.5 - Vec2::splat(8.0)).max(Vec2::ZERO);
        let sample_at =
            |position: Vec2| Rect::from_center_size(stage.rect.center() + position * travel, size);
        let handle = ui.interact(
            sample_at(self.position),
            stage.id.with("glass-sample"),
            egui::Sense::click_and_drag(),
        );
        handle.widget_info(|| {
            egui::WidgetInfo::labeled(egui::Role::Unknown, true, format!("Drag {label}"))
        });
        if handle.has_focus() && !handle.dragged() {
            self.position += ui.input_mut(|input| {
                let mut step =
                    |key| f32::from(input.consume_key(egui::Modifiers::NONE, key)) * 0.14;
                egui::vec2(
                    step(egui::Key::ArrowRight) - step(egui::Key::ArrowLeft),
                    step(egui::Key::ArrowDown) - step(egui::Key::ArrowUp),
                )
            });
            self.position = self.position.clamp(-Vec2::splat(1.0), Vec2::splat(1.0));
        }
        let cursor = if handle.dragged() {
            egui::CursorIcon::Grabbing
        } else {
            egui::CursorIcon::Grab
        };
        let handle = handle.on_hover_cursor(cursor);
        let resting = sample_at(self.position);
        let offset = ElasticDrag::new(travel * (Vec2::splat(1.0) - self.position.abs()))
            .offset(&handle, theme.motion);
        (resting.translate(offset), handle)
    }
}

struct Artwork {
    scene: TextureHandle,
    filtered: [TextureHandle; Preset::ALL.len()],
}

impl Artwork {
    fn new(ctx: &Context) -> Self {
        // Decode the artwork once and share it across the samples.
        let scene = image::load_from_memory_with_format(
            include_bytes!("assets/glass-studio.png"),
            image::ImageFormat::Png,
        )
        .expect("valid embedded Glass Studio image");
        let to_color = |image: image::RgbaImage| {
            ColorImage::from_rgba_unmultiplied(
                [image.width() as usize, image.height() as usize],
                image.as_raw(),
            )
        };
        let fallback = scene.resize_exact(320, 180, image::imageops::FilterType::Triangle);
        let filtered = Preset::ALL.map(|preset| {
            let sigma = match preset {
                Preset::Frost => 8.0,
                Preset::ClearFrost => 4.8,
                Preset::SoftPrism => 4.2,
                Preset::Prism => 1.7,
            };
            ctx.load_texture(
                preset.label(),
                to_color(fallback.blur(sigma).into_rgba8()),
                TextureOptions::LINEAR,
            )
        });
        Self {
            scene: ctx.load_texture(
                "glass-studio",
                to_color(scene.into_rgba8()),
                TextureOptions::LINEAR,
            ),
            filtered,
        }
    }

    fn paint(
        &self,
        ui: &Ui,
        theme: &Theme,
        rect: Rect,
        (sample, handle): (Rect, egui::Response),
        (preset, glass): (Preset, Glass),
        pan: Vec2,
    ) {
        let photo = egui::vec2(800.0, 450.0);
        let scale = (rect.width() / photo.x).max(rect.height() / photo.y) * 1.25;
        let size = photo * scale;
        let offset = (size - rect.size()) * 0.42 * pan;
        let image_bounds = Rect::from_center_size(rect.center() + offset, size);
        let uv = Rect::from_min_max(
            ((rect.min - image_bounds.min) / size).to_pos2(),
            ((rect.max - image_bounds.min) / size).to_pos2(),
        );
        ui.painter().add(
            egui::epaint::RectShape::filled(rect, 19, Color32::WHITE)
                .with_texture(self.scene.id(), uv),
        );
        glass.paint(
            ui.painter(),
            theme,
            sample,
            Frame::new()
                .fill(theme.card)
                .stroke(egui::Stroke::new(1.0, theme.border))
                .corner_radius(26),
            Some(GlassBackdrop {
                texture: self.filtered[preset as usize].id(),
                bounds: image_bounds,
            }),
        );
        if let Some(lighting) = Vesper::context(ui.ctx()) {
            lighting.paint_on_surface(
                ui.painter(),
                theme,
                &handle,
                sample,
                26.into(),
                LightSurface::Glass,
            );
        }
        if sample.width() >= 75.0 {
            ui.painter().text(
                sample.center(),
                egui::Align2::CENTER_CENTER,
                preset.label(),
                egui::FontId::proportional(14.0),
                theme.foreground,
            );
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use egui_kittest::Harness;

    #[test]
    fn a_glass_sample_resists_pulling_and_bounces_back_inside_the_artwork() {
        let mut harness = Harness::builder().with_step_dt(1.0 / 60.0).build_ui_state(
            |ui, (motion, artwork, sample)| {
                let (rect, stage) =
                    ui.allocate_exact_size(egui::vec2(280.0, 164.0), egui::Sense::hover());
                *artwork = rect;
                *sample = motion
                    .sample(
                        ui,
                        &stage,
                        egui::vec2(160.0, 62.0),
                        "Glass sample",
                        &Theme::light(),
                    )
                    .0;
            },
            (SampleMotion::default(), Rect::NOTHING, Rect::NOTHING),
        );
        harness
            .ctx
            .all_styles_mut(|style| style.animation_time = 0.2);
        harness.run_steps(2);
        let center = harness.state().2.center();
        harness.drag_at(center);
        let mut previous = 0.0;
        let mut step = f32::INFINITY;
        for distance in [40.0, 80.0, 120.0] {
            harness.event(egui::Event::PointerMoved(
                center + egui::vec2(distance, 0.0),
            ));
            harness.run_steps(2);
            let (_, artwork, sample) = harness.state();
            let travel = sample.center().x - center.x;
            assert!(travel > previous && travel - previous < step);
            assert!(artwork.contains_rect(*sample));
            step = travel - previous;
            previous = travel;
        }
        harness.event(egui::Event::PointerButton {
            pos: center + egui::vec2(120.0, 0.0),
            button: egui::PointerButton::Primary,
            pressed: false,
            modifiers: egui::Modifiers::NONE,
        });
        let mut bounced = false;
        for _ in 0..70 {
            harness.step();
            bounced |= harness.state().2.center().x < center.x - 0.25;
        }
        assert!(bounced);
        assert!(harness.state().2.center().distance(center) < 0.1);
    }
}
