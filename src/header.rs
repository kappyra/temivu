//! Preliminary root-header port; nested, RTL and full style parity remain open.
use crate::{InputMode, Theme};
use egui::{Frame, InnerResponse, Label, Margin, RichText, TextWrapMode, Ui, containers::Sides};

/// A root navigation header with a title and trailing actions.
pub struct Header<'a> {
    title: &'a str,
    theme: &'a Theme,
    input: InputMode,
}

impl<'a> Header<'a> {
    pub fn new(title: &'a str, theme: &'a Theme) -> Self {
        Self {
            title,
            theme,
            input: InputMode::Desktop,
        }
    }

    pub fn input_mode(mut self, input: InputMode) -> Self {
        self.input = input;
        self
    }

    /// Render trailing actions; they are given priority over a long title.
    ///
    /// The root header follows Forui's `FHeaderStyle`: a 54 point desktop
    /// (62 touch) minimum height, `display.xl2` title sizes (22 desktop / 30
    /// touch) and `pagePadding` with a 10 point bottom inset.
    pub fn show<R>(self, ui: &mut Ui, actions: impl FnOnce(&mut Ui) -> R) -> InnerResponse<R> {
        let touch = self.theme.touch || self.input == InputMode::Touch;
        let (height, title_size) = if touch { (62.0, 30.0) } else { (54.0, 22.0) };
        let padding = Margin {
            left: 12,
            right: 12,
            top: 8,
            bottom: 10,
        };
        let inner_height = (height - f32::from(padding.top) - f32::from(padding.bottom)).max(0.0);
        Frame::new().inner_margin(padding).show(ui, |ui| {
            let (_, action_result) = Sides::new()
                .height(inner_height)
                .shrink_left()
                .truncate()
                .show(
                    ui,
                    |ui| {
                        let title = RichText::new(self.title)
                            .size(title_size)
                            .strong()
                            .color(self.theme.foreground);
                        let response = ui.add(Label::new(title).wrap_mode(TextWrapMode::Truncate));
                        ui.ctx().accesskit_node_builder(response.id, |node| {
                            node.set_role(egui::accesskit::Role::Heading);
                            node.set_level(1);
                            node.set_label(self.title);
                        });
                    },
                    actions,
                );
            action_result
        })
    }
}
