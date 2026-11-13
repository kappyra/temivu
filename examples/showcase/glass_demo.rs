use egui::{Color32, ColorImage, Context, Frame, Rect, TextureHandle, TextureOptions, Ui, Vec2};
use temivu::{
    Button, ButtonVariant, Glass, GlassBackdrop, GlassMode, GlassTuning, Slider, Switch, Theme,
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
            Self::Frost => Glass::default().mode(GlassMode::Frost),
            Self::SoftPrism => Glass::soft_prism(),
            Self::Prism => Glass::default(),
        }
    }
}

pub(super) struct GlassDemo {
    pub enabled: bool,
    tint: f64,
    preset: Preset,
    prism_preset: Preset,
    tuning: [GlassTuning; 3],
    artwork: Option<Artwork>,
    frost_motion: SampleMotion,
    prism_motion: SampleMotion,
}

impl Default for GlassDemo {
    fn default() -> Self {
        Self {
            enabled: false,
            tint: 0.28,
            preset: Preset::SoftPrism,
            prism_preset: Preset::SoftPrism,
            tuning: Preset::ALL.map(|preset| preset.material().tuning()),
            artwork: None,
            frost_motion: SampleMotion::default(),
            prism_motion: SampleMotion::default(),
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
                        if preset != Preset::Frost {
                            self.prism_preset = preset;
                        }
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
            "Drag either sample across the scene to compare its effect on the background.",
            |ui| {
                let frost = self.material(Preset::Frost);
                let prism = self.material(self.prism_preset);
                let mut sample = |ui: &mut Ui, preset, glass| {
                    let (title, motion) = if preset == Preset::Frost {
                        ("Frost glassmorphism", &mut self.frost_motion)
                    } else {
                        ("Prism optical glass", &mut self.prism_motion)
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
                    let sample = motion.sample(ui, &stage, size, title);
                    let artwork = self.artwork.get_or_insert_with(|| Artwork::new(ui.ctx()));
                    artwork.paint(ui, theme, rect, sample, glass, preset);
                };
                if ui.available_width() >= 600.0 {
                    ui.columns(2, |columns| {
                        sample(&mut columns[0], Preset::Frost, frost);
                        sample(&mut columns[1], self.prism_preset, prism);
                    });
                } else {
                    sample(ui, Preset::Frost, frost);
                    ui.add_space(16.0);
                    sample(ui, self.prism_preset, prism);
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
    drag_origin: Option<Vec2>,
}

impl SampleMotion {
    fn sample(&mut self, ui: &Ui, stage: &egui::Response, size: Vec2, label: &str) -> Rect {
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
        if let Some(delta) = handle.total_drag_delta() {
            let origin = *self.drag_origin.get_or_insert(self.position);
            self.position = origin + delta / travel.max(Vec2::splat(1.0));
        } else {
            self.drag_origin = None;
            if handle.has_focus() {
                self.position += ui.input_mut(|input| {
                    let mut step =
                        |key| f32::from(input.consume_key(egui::Modifiers::NONE, key)) * 0.14;
                    egui::vec2(
                        step(egui::Key::ArrowRight) - step(egui::Key::ArrowLeft),
                        step(egui::Key::ArrowDown) - step(egui::Key::ArrowUp),
                    )
                });
            }
        }
        self.position = self.position.clamp(-Vec2::splat(1.0), Vec2::splat(1.0));
        let cursor = if handle.dragged() {
            egui::CursorIcon::Grabbing
        } else {
            egui::CursorIcon::Grab
        };
        handle.on_hover_cursor(cursor);
        sample_at(self.position)
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
        let [width, height] = [192, 112];
        let mut image = ColorImage::filled([width, height], Color32::BLACK);
        for y in 0..height {
            for x in 0..width {
                let u = x as f32 / (width - 1) as f32;
                let v = y as f32 / (height - 1) as f32;
                image[(x, y)] = coast(u, v);
            }
        }
        // Backends without a live effect reuse continuous, prefiltered versions
        // of the same artwork. The native Showcase samples the original scene.
        let frost = soften(
            &image,
            &[1, 3, 7, 12, 20, 28, 34, 36, 34, 28, 20, 12, 7, 3, 1],
        );
        let soft_prism = soften(&image, &[1, 3, 8, 13, 16, 13, 8, 3, 1]);
        let prism = soften(&image, &[1, 4, 6, 4, 1]);
        Self {
            scene: ctx.load_texture("glass-coast", image, TextureOptions::LINEAR),
            frost: ctx.load_texture("glass-coast-frost", frost, TextureOptions::LINEAR),
            soft_prism: ctx.load_texture(
                "glass-coast-soft-prism",
                soft_prism,
                TextureOptions::LINEAR,
            ),
            prism: ctx.load_texture("glass-coast-prism", prism, TextureOptions::LINEAR),
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
        let uv = Rect::from_min_max(egui::Pos2::ZERO, egui::pos2(1.0, 1.0));
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
                bounds: rect,
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

fn coast(u: f32, v: f32) -> Color32 {
    let sky = Color32::from_rgb(51, 65, 107);
    let light = Color32::from_rgb(241, 175, 138);
    let water = Color32::from_rgb(23, 70, 79);
    let smooth = |lo: f32, hi: f32, value: f32| {
        let t = ((value - lo) / (hi - lo)).clamp(0.0, 1.0);
        t * t * (3.0 - 2.0 * t)
    };
    let ridge = |center: f32, width: f32| (-((u - center) / width).powi(2)).exp();

    let mut color = sky.lerp_to_gamma(light, (v * 1.19 + 0.12 * u).clamp(0.0, 1.0));
    let sun = ((u - 0.63) * 2.4).hypot(v - 0.33);
    color = color.lerp_to_gamma(
        Color32::from_rgb(255, 225, 169),
        0.93 * (1.0 - smooth(0.096, 0.110, sun)),
    );

    let far_shore = 0.65 - 0.25 * ridge(0.04, 0.30) - 0.11 * ridge(0.40, 0.21);
    color = color.lerp_to_gamma(
        Color32::from_rgb(87, 107, 121),
        smooth(-0.012, 0.014, v - far_shore),
    );
    let near_shore = 0.74 - 0.24 * ridge(1.0, 0.25);
    color = color.lerp_to_gamma(
        Color32::from_rgb(44, 93, 102),
        smooth(-0.009, 0.012, v - near_shore),
    );
    color = color.lerp_to_gamma(water, smooth(0.67, 1.0, v) * 0.94);

    // A small sail crosses the edge of the sample and shows the difference
    // between diffused detail and the displacement of a sharper silhouette.
    let sail = (0.45..=0.615).contains(&v) && (0.728 - 0.29 * (v - 0.45)..=0.728).contains(&u);
    let hull = (0.623..=0.640).contains(&v)
        && (0.665 + (v - 0.623) * 0.3..=0.742 - (v - 0.623) * 0.3).contains(&u);
    if sail {
        color = color.lerp_to_gamma(Color32::from_rgb(255, 236, 207), 0.93);
    }
    if hull {
        color = color.lerp_to_gamma(Color32::from_rgb(22, 51, 67), 0.95);
    }
    color
}

fn soften(source: &ColorImage, weights: &[u16]) -> ColorImage {
    let [width, height] = source.size;
    let midpoint = (weights.len() / 2) as isize;
    let sum: u16 = weights.iter().sum();
    let pass = |input: &ColorImage, horizontal: bool| {
        let mut output = ColorImage::filled(input.size, Color32::BLACK);
        for y in 0..height {
            for x in 0..width {
                let mut rgb = [0; 3];
                for (index, weight) in weights.iter().enumerate() {
                    let offset = index as isize - midpoint;
                    let (px, py) = if horizontal {
                        (x.saturating_add_signed(offset).min(width - 1), y)
                    } else {
                        (x, y.saturating_add_signed(offset).min(height - 1))
                    };
                    for (total, channel) in rgb.iter_mut().zip(input[(px, py)].to_array()) {
                        *total += u16::from(channel) * weight;
                    }
                }
                let [r, g, b] = rgb.map(|total| (total / sum) as u8);
                output[(x, y)] = Color32::from_rgb(r, g, b);
            }
        }
        output
    };
    pass(&pass(source, true), false)
}

#[cfg(test)]
mod tests {
    use super::*;
    use egui_kittest::Harness;

    #[test]
    fn a_glass_sample_moves_with_the_pointer_and_stays_inside_the_artwork() {
        let mut harness = Harness::builder().build_ui_state(
            |ui, (motion, artwork, sample)| {
                let (rect, stage) =
                    ui.allocate_exact_size(egui::vec2(280.0, 164.0), egui::Sense::hover());
                *artwork = rect;
                *sample = motion.sample(ui, &stage, egui::vec2(160.0, 62.0), "Glass sample");
            },
            (SampleMotion::default(), Rect::NOTHING, Rect::NOTHING),
        );
        harness.run_steps(2);
        let center = harness.state().2.center();
        harness.drag_at(center);
        harness.event(egui::Event::PointerMoved(center + egui::vec2(120.0, 0.0)));
        harness.run_steps(2);
        let (_, artwork, sample) = harness.state();
        assert!(sample.center().x > center.x + 40.0);
        assert!(artwork.contains_rect(*sample));
    }
}
