use egui::{Color32, ColorImage, Context, Frame, Rect, TextureHandle, TextureOptions, Ui, Vec2};
use temivu::{
    Button, ButtonVariant, Glass, GlassBackdrop, GlassTuning, Slider, Switch, Theme,
    motion::ElasticDrag,
};

#[derive(Clone, Copy, PartialEq, Eq)]
enum Preset {
    Frost,
    SoftPrism,
    Prism,
}

impl Preset {
    const ALL: [Self; 3] = [Self::Frost, Self::SoftPrism, Self::Prism];

    fn label(self) -> &'static str {
        match self {
            Self::Frost => "Frost",
            Self::SoftPrism => "Soft Prism",
            Self::Prism => "Prism",
        }
    }

    fn material(self) -> Glass {
        match self {
            Self::Frost => Glass::frost(),
            Self::SoftPrism => Glass::soft_prism(),
            Self::Prism => Glass::prism(),
        }
    }
}

pub(super) struct GlassDemo {
    pub enabled: bool,
    tint: f64,
    preset: Preset,
    tuning: [GlassTuning; 3],
    artwork: Option<Artwork>,
    motion: [SampleMotion; 3],
}

impl Default for GlassDemo {
    fn default() -> Self {
        Self {
            enabled: false,
            tint: 0.28,
            preset: Preset::SoftPrism,
            tuning: Preset::ALL.map(|preset| preset.material().tuning()),
            artwork: None,
            motion: Default::default(),
        }
    }
}

impl GlassDemo {
    fn material(&self, preset: Preset) -> Glass {
        preset
            .material()
            .with_tint(self.tint as f32)
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
            Slider::new(&mut self.tint, theme)
                .expect("the tint stays within the slider range")
                .label("Glass tint")
                .filled(true)
                .show(ui);
            ui.collapsing("Optical settings", |ui| {
                let tuning = &mut self.tuning[self.preset as usize];
                tuning_slider(ui, theme, "Backdrop blur", &mut tuning.blur);
                if self.preset != Preset::Frost {
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
            "Pull and release each sample to compare the three materials.",
            |ui| {
                let materials = Preset::ALL.map(|preset| self.material(preset));
                let mut sample = |ui: &mut Ui, preset: Preset| {
                    let title = match preset {
                        Preset::Frost => "Frost glassmorphism",
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
                    let size = Vec2::new((rect.width() * 0.64).min(208.0), 62.0);
                    let index = preset as usize;
                    let sample = self.motion[index].sample(ui, &stage, size, title, theme);
                    let artwork = self.artwork.get_or_insert_with(|| Artwork::new(ui.ctx()));
                    artwork.paint(ui, theme, rect, sample, materials[index], preset);
                };
                if ui.available_width() >= 720.0 {
                    ui.columns(3, |columns| {
                        for (column, preset) in columns.iter_mut().zip(Preset::ALL) {
                            sample(column, preset);
                        }
                    });
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
    ) -> Rect {
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
        resting.translate(offset)
    }
}

struct Artwork {
    scene: TextureHandle,
    frost: TextureHandle,
    soft_prism: TextureHandle,
    prism: TextureHandle,
}

impl Artwork {
    fn new(ctx: &Context) -> Self {
        // The embedded asset is decoded once and shared by all three samples.
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
        let frost = to_color(fallback.blur(9.0).into_rgba8());
        let soft_prism = to_color(fallback.blur(4.0).into_rgba8());
        let prism = to_color(fallback.blur(1.5).into_rgba8());
        Self {
            scene: ctx.load_texture(
                "glass-studio",
                to_color(scene.into_rgba8()),
                TextureOptions::LINEAR,
            ),
            frost: ctx.load_texture("glass-studio-frost", frost, TextureOptions::LINEAR),
            soft_prism: ctx.load_texture(
                "glass-studio-soft-prism",
                soft_prism,
                TextureOptions::LINEAR,
            ),
            prism: ctx.load_texture("glass-studio-prism", prism, TextureOptions::LINEAR),
        }
    }

    fn paint(
        &self,
        ui: &Ui,
        theme: &Theme,
        rect: Rect,
        sample: Rect,
        glass: Glass,
        preset: Preset,
    ) {
        let photo = egui::vec2(800.0, 450.0);
        let scale = (rect.width() / photo.x).max(rect.height() / photo.y);
        let image_bounds = Rect::from_center_size(rect.center(), photo * scale);
        let uv = Rect::from_center_size(egui::pos2(0.5, 0.5), rect.size() / image_bounds.size());
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
                texture: match preset {
                    Preset::Frost => self.frost.id(),
                    Preset::SoftPrism => self.soft_prism.id(),
                    Preset::Prism => self.prism.id(),
                },
                bounds: image_bounds,
            }),
        );
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
                *sample = motion.sample(
                    ui,
                    &stage,
                    egui::vec2(160.0, 62.0),
                    "Glass sample",
                    &Theme::light(),
                );
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
