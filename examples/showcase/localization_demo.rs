use egui::{FontId, RichText, Ui};
use temivu::{Button, ButtonVariant, Messages, PaginationState, Theme};

const LOCALES: [(&str, &str); 6] = [
    ("en", "English"),
    ("de", "Deutsch"),
    ("ar", "العربية"),
    ("ja", "日本語"),
    ("zh", "中文"),
    ("fr", "Français"),
];

const SCRIPT_FONTS: [(&str, &[u8]); 3] = [
    (
        "ar",
        include_bytes!("../../third_party/showcase-fonts/ar.ttf"),
    ),
    (
        "ja",
        include_bytes!("../../third_party/showcase-fonts/ja.otf"),
    ),
    (
        "zh",
        include_bytes!("../../third_party/showcase-fonts/zh.otf"),
    ),
];

pub(super) fn install_fonts(ctx: &egui::Context) {
    use egui::epaint::text::{FontInsert, FontPriority, InsertFontFamily};

    for (code, bytes) in SCRIPT_FONTS {
        let name = format!("showcase-{code}");
        ctx.add_font(FontInsert {
            data: egui::FontData::from_static(bytes),
            families: vec![
                InsertFontFamily {
                    family: egui::FontFamily::Proportional,
                    priority: FontPriority::Lowest,
                },
                InsertFontFamily {
                    family: egui::FontFamily::Name(name.clone().into()),
                    priority: FontPriority::Highest,
                },
            ],
            name,
        });
    }
}

pub(super) struct LocalizationDemo {
    locale: usize,
    pagination: PaginationState,
}

impl Default for LocalizationDemo {
    fn default() -> Self {
        Self {
            locale: 0,
            pagination: PaginationState::new(12, 0, 0, false).expect("valid preview pages"),
        }
    }
}

impl LocalizationDemo {
    pub(super) fn show(&mut self, ui: &mut Ui, theme: &Theme) {
        super::page_intro(
            ui,
            theme,
            "COMPONENT STUDIO / LOCALIZATION",
            "Localization and themes",
            "Preview how the same controls read across languages.",
        );
        ui.add_space(16.0);
        super::studio_panel(
            ui,
            theme,
            "Language",
            "Choose a language for the preview.",
            |ui| {
                let available = ui.available_width();
                let columns = if available < 230.0 {
                    1
                } else if available < 510.0 {
                    2
                } else {
                    3
                };
                let gap = 8.0;
                let width = ((available - gap * (columns - 1) as f32) / columns as f32).max(1.0);
                for (row, locales) in LOCALES.chunks(columns).enumerate() {
                    ui.horizontal(|ui| {
                        ui.spacing_mut().item_spacing.x = gap;
                        for (column, (code, name)) in locales.iter().enumerate() {
                            let index = row * columns + column;
                            let selected = self.locale == index;
                            let label = format!("{name} ({code})");
                            let text_color = if selected {
                                theme.primary_foreground
                            } else {
                                theme.foreground
                            };
                            let fill = if selected { theme.primary } else { theme.card };
                            let height = if theme.touch { 46.0 } else { 38.0 };
                            let response = ui.add_sized(
                                [width, height],
                                egui::Button::new(RichText::new(&label).color(text_color))
                                    .fill(fill)
                                    .stroke(egui::Stroke::new(1.0, theme.border))
                                    .corner_radius(
                                        temivu::SurfaceShape::Control
                                            .radius(egui::vec2(width, height)),
                                    )
                                    .wrap(),
                            );
                            ui.ctx().accesskit_node_builder(response.id, |node| {
                                node.set_selected(selected)
                            });
                            if response.clicked() {
                                self.locale = index;
                            }
                        }
                    });
                    if row + 1 < LOCALES.len().div_ceil(columns) {
                        ui.add_space(5.0);
                    }
                }
            },
        );
        ui.add_space(12.0);

        let (code, _) = LOCALES[self.locale];
        let messages = Messages::for_locale(code);
        let preview_font = FontId::new(
            21.0,
            if SCRIPT_FONTS.iter().any(|(script, _)| code == *script) {
                egui::FontFamily::Name(format!("showcase-{code}").into())
            } else {
                egui::FontFamily::Proportional
            },
        );

        super::studio_panel(
            ui,
            theme,
            "Live preview",
            "Localized status and navigation.",
            |ui| {
                ui.label(
                    RichText::new("STATUS")
                        .size(11.0)
                        .color(theme.muted_foreground),
                );
                ui.label(RichText::new(messages.progress).font(preview_font.clone()));
                ui.add_space(12.0);
                ui.separator();
                ui.add_space(12.0);
                ui.vertical_centered(|ui| {
                    let number = self.pagination.page() + 1;
                    ui.label(
                        RichText::new(messages.page_label(number))
                            .font(preview_font)
                            .strong(),
                    );
                    ui.label(
                        RichText::new(format!("{number} / {}", self.pagination.pages()))
                            .size(12.0)
                            .color(theme.muted_foreground),
                    );
                });
                ui.add_space(12.0);
                ui.horizontal_wrapped(|ui| {
                    ui.spacing_mut().item_spacing.x = 8.0;
                    let previous = Button::new(messages.previous, theme)
                        .variant(ButtonVariant::Outline)
                        .enabled(self.pagination.page() > 0)
                        .show(ui);
                    let next = Button::new(messages.next, theme)
                        .variant(ButtonVariant::Secondary)
                        .enabled(self.pagination.page() + 1 < self.pagination.pages())
                        .show(ui);
                    if previous.clicked() {
                        self.pagination.previous();
                    }
                    if next.clicked() {
                        self.pagination.next();
                    }
                });
            },
        );
    }
}
