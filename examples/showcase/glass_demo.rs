use egui::{Color32, ColorImage, Context, Frame, Rect, TextureHandle, TextureOptions, Ui, Vec2};
use temivu::{Button, ButtonVariant, Glass, GlassBackdrop, GlassMode, Slider, Switch, Theme};

pub(super) struct GlassDemo {
    pub enabled: bool,
    tint: f64,
    mode: GlassMode,
    artwork: Option<Artwork>,
}

impl Default for GlassDemo {
    fn default() -> Self {
        Self {
            enabled: false,
            tint: 0.28,
            mode: GlassMode::Prism,
            artwork: None,
        }
    }
}

impl GlassDemo {
    pub fn apply(&self, ctx: &Context, high_contrast: bool) {
        let glass =
            (self.enabled && !high_contrast).then(|| Glass::new(self.tint as f32).mode(self.mode));
        Glass::set_context(ctx, glass);
    }

    pub fn settings(&mut self, ui: &mut Ui, theme: &Theme, high_contrast: bool) {
        ui.add_space(10.0);
        Switch::new(&mut self.enabled, theme)
            .label("Glass surfaces")
            .description("Choose a material for navigation and overlays.")
            .show(ui);
        if self.enabled {
            if high_contrast {
                ui.label("High contrast uses opaque surfaces.");
            } else {
                ui.horizontal_wrapped(|ui| {
                    for (mode, label) in [(GlassMode::Frost, "Frost"), (GlassMode::Prism, "Prism")]
                    {
                        let variant = if self.mode == mode {
                            ButtonVariant::Secondary
                        } else {
                            ButtonVariant::Ghost
                        };
                        let response = Button::new(label, theme).variant(variant).show(ui);
                        if response.clicked() {
                            self.mode = mode;
                        }
                        ui.ctx().accesskit_node_builder(response.id, |node| {
                            node.set_selected(self.mode == mode)
                        });
                    }
                });
                ui.label(
                    egui::RichText::new(match self.mode {
                        GlassMode::Frost => "Frost diffuses the background evenly.",
                        GlassMode::Prism => "Prism bends the scene along the polished edge.",
                    })
                    .color(theme.muted_foreground),
                );
                Slider::new(&mut self.tint, theme)
                    .expect("the tint stays within the slider range")
                    .label("Glass tint")
                    .filled(true)
                    .show(ui);
            }
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
            "One backdrop, two materials. The tint slider applies to both samples.",
            |ui| {
                let tint = self.tint as f32;
                let mut sample = |ui: &mut Ui, mode| {
                    let (title, caption) = match mode {
                        GlassMode::Frost => (
                            "Frost glassmorphism",
                            "Soft diffusion across the entire surface.",
                        ),
                        GlassMode::Prism => (
                            "Prism optical glass",
                            "A clear center with a polished, refracting edge.",
                        ),
                    };
                    ui.label(egui::RichText::new(title).strong());
                    ui.label(
                        egui::RichText::new(caption)
                            .size(12.0)
                            .color(theme.muted_foreground),
                    );
                    ui.add_space(3.0);

                    let (rect, _) = ui.allocate_exact_size(
                        Vec2::new(ui.available_width(), 164.0),
                        egui::Sense::hover(),
                    );
                    if !ui.is_rect_visible(rect) || !rect.is_positive() {
                        return;
                    }
                    let artwork = self.artwork.get_or_insert_with(|| Artwork::new(ui.ctx()));
                    artwork.paint(ui, theme, rect, Glass::new(tint).mode(mode), mode);
                };

                if ui.available_width() >= 600.0 {
                    ui.columns(2, |columns| {
                        sample(&mut columns[0], GlassMode::Frost);
                        sample(&mut columns[1], GlassMode::Prism);
                    });
                } else {
                    sample(ui, GlassMode::Frost);
                    ui.add_space(16.0);
                    sample(ui, GlassMode::Prism);
                }
            },
        );
    }
}

struct Artwork {
    scene: TextureHandle,
    frost: TextureHandle,
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
        let frost = soften(&image, &[1, 4, 10, 18, 25, 28, 25, 18, 10, 4, 1]);
        let prism = soften(&image, &[1, 4, 6, 4, 1]);
        Self {
            scene: ctx.load_texture("glass-coast", image, TextureOptions::LINEAR),
            frost: ctx.load_texture("glass-coast-frost", frost, TextureOptions::LINEAR),
            prism: ctx.load_texture("glass-coast-prism", prism, TextureOptions::LINEAR),
        }
    }

    fn paint(&self, ui: &Ui, theme: &Theme, rect: Rect, glass: Glass, mode: GlassMode) {
        let uv = Rect::from_min_max(egui::Pos2::ZERO, egui::pos2(1.0, 1.0));
        ui.painter().add(
            egui::epaint::RectShape::filled(rect, 19, Color32::WHITE)
                .with_texture(self.scene.id(), uv),
        );
        let sample = Rect::from_center_size(
            rect.center(),
            Vec2::new((rect.width() * 0.64).min(208.0), 62.0),
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
                texture: match mode {
                    GlassMode::Frost => self.frost.id(),
                    GlassMode::Prism => self.prism.id(),
                },
                bounds: rect,
            }),
        );
        if sample.width() >= 75.0 {
            ui.painter().text(
                sample.center(),
                egui::Align2::CENTER_CENTER,
                match mode {
                    GlassMode::Frost => "Frost",
                    GlassMode::Prism => "Prism",
                },
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
