use egui::{Color32, CursorIcon, Pos2, Rect, Sense, Shape, TextureHandle, Ui, Vec2, pos2, vec2};
use temivu::{
    Button, ButtonVariant, SharedElementDrive, SharedElementEndpoint, SharedElementFade,
    SharedElementFit, SharedElementPose, SharedElementSide, SharedElementSnapshot,
    SharedElementStyle, SharedElementTransition, Theme,
};

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
enum Destination {
    #[default]
    Dialog,
    Popover,
    Image,
    Panel,
}

impl Destination {
    const ALL: [Self; 4] = [Self::Dialog, Self::Popover, Self::Image, Self::Panel];

    fn label(self) -> &'static str {
        match self {
            Self::Dialog => "Dialog",
            Self::Popover => "Popover",
            Self::Image => "Image viewer",
            Self::Panel => "Panel",
        }
    }

    fn bounds(self, stage: Rect) -> Rect {
        let w = stage.width();
        let h = stage.height();
        match self {
            Self::Dialog => {
                Rect::from_center_size(stage.center(), vec2((w * 0.75).min(420.0), h * 0.73))
            }
            Self::Popover => Rect::from_center_size(
                stage.center() + vec2(w * 0.13, 24.0),
                vec2((w * 0.61).min(360.0), h * 0.52),
            ),
            Self::Image => stage.shrink(8.0),
            Self::Panel => Rect::from_min_max(
                pos2(stage.right() - (w * 0.68).min(400.0), stage.top() + 7.0),
                stage.right_bottom() - vec2(7.0, 7.0),
            ),
        }
    }
}

#[derive(Clone, Copy)]
struct Grab {
    pointer: Pos2,
    progress: f32,
}

#[derive(Default)]
pub(super) struct SharedElements {
    destination: Destination,
    target: SharedElementSide,
    pose: Option<SharedElementPose>,
    grab: Option<Grab>,
    text: Option<TextSnapshots>,
    artwork: Option<Artwork>,
}

impl SharedElements {
    pub fn show(&mut self, ui: &mut Ui, theme: &Theme) {
        super::studio_panel(
            ui,
            theme,
            "Shared element",
            "Drag the surface horizontally to take over, or toggle it during motion.",
            |ui| {
                ui.horizontal_wrapped(|ui| {
                    for choice in Destination::ALL {
                        let variant = if self.destination == choice {
                            ButtonVariant::Primary
                        } else {
                            ButtonVariant::Outline
                        };
                        if Button::new(choice.label(), theme)
                            .variant(variant)
                            .show(ui)
                            .clicked()
                        {
                            self.destination = choice;
                        }
                    }
                });
                ui.add_space(10.0);
                let label = if self.target == SharedElementSide::Source {
                    "Expand card"
                } else {
                    "Collapse to card"
                };
                let reverse = Button::new(label, theme)
                    .variant(ButtonVariant::Secondary)
                    .show(ui)
                    .clicked();
                ui.add_space(8.0);
                self.stage(ui, theme, reverse);
            },
        );
    }

    fn stage(&mut self, ui: &mut Ui, theme: &Theme, reverse: bool) {
        let width = ui.available_width().max(1.0);
        let (stage, response) = ui.allocate_exact_size(vec2(width, 244.0), Sense::hover());
        let src_width = (width * 0.46).min(190.0);
        let source =
            Rect::from_min_size(stage.left_top() + vec2(10.0, 70.0), vec2(src_width, 82.0));
        let destination = self.destination.bounds(stage);
        let image = self.destination == Destination::Image;
        let from = SharedElementEndpoint::new(source, 18.0, theme.card);
        let to = SharedElementEndpoint::new(
            destination,
            24.0,
            if image { theme.background } else { theme.card },
        );
        let motion = SharedElementTransition::new(response.id).style(SharedElementStyle {
            fade: if self.destination == Destination::Popover {
                SharedElementFade::Cross
            } else {
                SharedElementFade::Through
            },
            fit: if image {
                SharedElementFit::Cover
            } else {
                SharedElementFit::Contain
            },
            scrim: Color32::from_black_alpha(if theme.dark { 42 } else { 19 }),
            ..Default::default()
        });
        let hit = self.pose.map_or(source, |pose| pose.rect).intersect(stage);
        let handle = ui
            .interact(hit, response.id.with("surface"), Sense::click_and_drag())
            .on_hover_cursor(if self.grab.is_some() {
                CursorIcon::Grabbing
            } else {
                CursorIcon::Grab
            });
        handle.widget_info(|| {
            egui::WidgetInfo::labeled(egui::Role::Button, ui.is_enabled(), "Shared surface")
        });
        let ctx = ui.ctx();
        if reverse || handle.clicked() {
            self.target = if self.target == SharedElementSide::Source {
                SharedElementSide::Destination
            } else {
                SharedElementSide::Source
            };
            self.grab = None;
        }
        let (pointer, speed, down) = ctx.input(|input| {
            (
                input.pointer.interact_pos(),
                input.pointer.velocity().x,
                input.pointer.primary_down(),
            )
        });
        if handle.drag_started()
            && let Some(pointer) = pointer
        {
            self.grab = Some(Grab {
                pointer,
                progress: self.pose.map_or(0.0, |pose| pose.progress),
            });
        }
        let mut pose = None;
        if let Some(grab) = self.grab {
            let travel = (width * 0.63).clamp(110.0, 340.0);
            let delta = pointer.map_or(0.0, |p| p.x - grab.pointer.x);
            let progress = (grab.progress + delta / travel).clamp(0.0, 1.0);
            let velocity = speed / travel;
            pose = motion.sample(
                ctx,
                theme,
                from,
                to,
                SharedElementDrive::Follow { progress, velocity },
            );
            if !down {
                self.grab = None;
                self.target = if progress + velocity * 0.13 >= 0.5 {
                    SharedElementSide::Destination
                } else {
                    SharedElementSide::Source
                };
            }
        }
        if self.grab.is_none() {
            pose = motion.sample(
                ctx,
                theme,
                from,
                to,
                SharedElementDrive::SettleTo(self.target),
            );
        }
        self.pose = pose;
        let Some(pose) = pose.filter(|_| ui.is_rect_visible(stage)) else {
            return;
        };
        let painter = ui.painter().with_clip_rect(stage);
        painter.rect_filled(stage, 18, theme.muted);
        let content = if image {
            let artwork = self.artwork.get_or_insert_with(|| Artwork::new(ui));
            [Some(&artwork.content), Some(&artwork.content)]
        } else {
            let key = (
                self.destination,
                theme.foreground,
                theme.muted_foreground,
                ui.ctx().pixels_per_point(),
            );
            if self.text.as_ref().is_none_or(|text| text.key != key) {
                self.text = Some(TextSnapshots::new(ui, key));
            }
            let text = self.text.as_ref().unwrap();
            [Some(&text.source), Some(&text.destination)]
        };
        motion.paint(&painter, pose, content);
    }
}

struct TextSnapshots {
    key: (Destination, Color32, Color32, f32),
    source: SharedElementSnapshot,
    destination: SharedElementSnapshot,
}

impl TextSnapshots {
    fn new(ui: &Ui, key: (Destination, Color32, Color32, f32)) -> Self {
        let (destination, ink, muted, _) = key;
        let text = |at, title: &str, size, color| {
            Shape::galley(
                at,
                ui.painter().layout_no_wrap(
                    title.to_owned(),
                    egui::FontId::proportional(size),
                    color,
                ),
                color,
            )
        };
        let source = Rect::from_min_size(Pos2::ZERO, vec2(162.0, 82.0));
        let source = SharedElementSnapshot::from_shapes(
            ui.ctx(),
            source,
            [
                text(pos2(14.0, 20.0), "Field notes", 22.0, ink),
                text(pos2(14.0, 49.0), "Open collection", 13.0, muted),
            ],
        );
        let bounds = Rect::from_min_size(Pos2::ZERO, vec2(300.0, 134.0));
        let destination = SharedElementSnapshot::from_shapes(
            ui.ctx(),
            bounds,
            [
                text(pos2(20.0, 18.0), destination.label(), 25.0, ink),
                text(pos2(20.0, 62.0), "River survey", 16.0, ink),
                text(pos2(20.0, 88.0), "3 observations", 14.0, muted),
            ],
        );
        Self {
            key,
            source,
            destination,
        }
    }
}

struct Artwork {
    // The tessellated content retains an ID. This handle owns its texture.
    _texture: TextureHandle,
    content: SharedElementSnapshot,
}

impl Artwork {
    fn new(ui: &Ui) -> Self {
        let image = image::load_from_memory_with_format(
            include_bytes!("assets/glass-studio.png"),
            image::ImageFormat::Png,
        )
        .expect("the bundled artwork is a PNG")
        .into_rgba8();
        let size = [image.width() as usize, image.height() as usize];
        let texture = ui.ctx().load_texture(
            "shared-element-artwork",
            egui::ColorImage::from_rgba_unmultiplied(size, image.as_raw()),
            egui::TextureOptions::LINEAR,
        );
        let rect = Rect::from_min_size(Pos2::ZERO, Vec2::new(size[0] as f32, size[1] as f32));
        let shape = egui::epaint::RectShape::filled(rect, 0, Color32::WHITE)
            .with_texture(texture.id(), Rect::from_min_max(Pos2::ZERO, pos2(1.0, 1.0)));
        let content = SharedElementSnapshot::from_shapes(ui.ctx(), rect, [shape.into()]);
        Self {
            _texture: texture,
            content,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use egui_kittest::{Harness, kittest::Queryable};

    #[test]
    fn narrow_demo_can_reverse_and_expand_the_shared_image() {
        let mut harness = Harness::builder()
            .with_size(vec2(390.0, 620.0))
            .with_step_dt(1.0 / 60.0)
            .build_ui_state(
                |ui, state: &mut SharedElements| state.show(ui, &Theme::light()),
                SharedElements::default(),
            );
        harness
            .ctx
            .all_styles_mut(|style| style.animation_time = 0.2);
        harness.run_steps(3);
        harness.get_by_label("Expand card").click();
        harness.run_steps(5);
        let progress = harness.state().pose.unwrap().progress;
        assert!(progress > 0.0 && progress < 1.0);
        harness.get_by_label("Collapse to card").click();
        harness.run_steps(50);
        assert_eq!(
            harness.state().pose.unwrap().settled,
            Some(SharedElementSide::Source)
        );
        harness.get_by_label("Image viewer").click();
        harness.run_steps(2);
        harness.get_by_label("Expand card").click();
        harness.run_steps(50);
        assert_eq!(
            harness.state().pose.unwrap().settled,
            Some(SharedElementSide::Destination)
        );
        assert!(harness.state().artwork.is_some());
    }
}
