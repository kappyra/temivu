//! A styled trigger sharing egui's popup-memory convention with ComboBox.
//! Option/search behavior remains in Select/MultiSelect; only the surface moves.
use egui::{Id, Response, Ui};

pub(crate) struct LiquidCombo<'a> {
    id: Id,
    text: &'a str,
    width: f32,
    motion: Option<crate::liquid::LiquidStyle>,
    theme: Option<&'a crate::Theme>,
    frame: Option<egui::Frame>,
}
pub(crate) struct ComboOutput {
    pub response: Response,
}
impl<'a> LiquidCombo<'a> {
    pub fn new(id: Id, text: &'a str, width: f32) -> Self {
        Self {
            id,
            text,
            width,
            motion: None,
            theme: None,
            frame: None,
        }
    }
    pub fn liquid(mut self, motion: Option<crate::liquid::LiquidStyle>) -> Self {
        self.motion = motion;
        self
    }
    pub fn theme(mut self, theme: Option<&'a crate::Theme>) -> Self {
        self.theme = theme;
        self
    }
    pub fn frame(mut self, frame: Option<egui::Frame>) -> Self {
        self.frame = frame;
        self
    }
    pub fn show_ui<R>(self, ui: &mut Ui, content: impl FnOnce(&mut Ui) -> R) -> ComboOutput {
        let popup_id = self.id.with("popup");
        let was_open = egui::Popup::is_id_open(ui.ctx(), popup_id);
        let theme = self
            .theme
            .cloned()
            .unwrap_or_else(|| crate::surface::context_theme(ui.ctx()));
        let response = ui
            .push_id(self.id.with("trigger"), |ui| {
                crate::button::press_native(ui, &theme, |ui| {
                    ui.add_sized(
                        [self.width, 44.0],
                        egui::Button::new("")
                            .fill(ui.visuals().extreme_bg_color)
                            .stroke(egui::Stroke::new(
                                if was_open { 1.5 } else { 1.0 },
                                if was_open {
                                    ui.visuals().text_color()
                                } else {
                                    ui.visuals().widgets.inactive.bg_stroke.color
                                },
                            ))
                            .corner_radius(24),
                    )
                })
            })
            .inner;
        let keyboard_open = response.has_focus()
            && ui.input(|input| {
                input.key_pressed(egui::Key::ArrowDown) || input.key_pressed(egui::Key::ArrowUp)
            });
        if ui.is_enabled() && (response.clicked() || keyboard_open) {
            egui::Popup::toggle_id(ui.ctx(), popup_id);
        }
        let mut open = ui.is_enabled() && egui::Popup::is_id_open(ui.ctx(), popup_id);
        let label = ui.painter().layout(
            self.text.to_owned(),
            egui::TextStyle::Button.resolve(ui.style()),
            ui.visuals().text_color(),
            (response.rect.width() - 52.0).max(1.0),
        );
        ui.painter()
            .with_clip_rect(response.rect.shrink(10.0))
            .galley(
                egui::pos2(
                    response.rect.left() + 12.0,
                    response.rect.center().y - label.size().y / 2.0,
                ),
                label,
                ui.visuals().text_color(),
            );
        crate::surface::chevron(
            ui,
            egui::Rect::from_center_size(
                egui::pos2(response.rect.right() - 20.0, response.rect.center().y),
                egui::Vec2::splat(16.0),
            ),
            &ui.visuals().widgets.inactive,
            crate::motion::spring_bool(ui.ctx(), response.id.with("chevron"), open, &theme),
        );
        ui.ctx().accesskit_node_builder(response.id, |node| {
            node.set_role(egui::accesskit::Role::ComboBox);
            node.set_expanded(open);
        });
        crate::liquid::popup(
            &response,
            popup_id,
            &mut open,
            crate::liquid::PopupSurface {
                label: None,
                match_compact_trigger: false,
                width: Some(response.rect.width()),
                frame: self
                    .frame
                    .unwrap_or_else(|| crate::surface::popup_frame(&theme, 12)),
                motion: self
                    .motion
                    .unwrap_or_else(|| crate::liquid::LiquidStyle::context_default(ui.ctx())),
                close_behavior: egui::PopupCloseBehavior::CloseOnClickOutside,
            },
            &theme,
            |ui| {
                egui::ScrollArea::vertical()
                    .max_height((ui.ctx().content_rect().height() - 180.0).clamp(80.0, 360.0))
                    .show(ui, content)
                    .inner
            },
        );
        if !open {
            egui::Popup::close_id(ui.ctx(), popup_id);
        } else {
            egui::Popup::open_id(ui.ctx(), popup_id);
        }
        ComboOutput { response }
    }
}
