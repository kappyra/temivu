//! An experimental showcase of the migrated components; the library is far
//! from complete. Run with `cargo run --example showcase --features demo`.
#[path = "showcase/motion_gallery.rs"]
mod motion_gallery;
#[cfg(not(target_os = "android"))]
#[path = "showcase/video_demo.rs"]
mod video_demo;
#[cfg(target_os = "android")]
#[path = "showcase/video_android.rs"]
mod video_demo;
#[path = "showcase/widget_catalog.rs"]
mod widget_catalog;

use egui::Id;
use temivu::tooltip::tooltip;
use temivu::{
    AccordionItem, AccordionState, Alert, Autocomplete, AutocompleteOption, Avatar, Badge,
    BadgeStyle, BadgeVariant, Button, ButtonVariant, Calendar, CalendarState, Checkbox,
    CircularProgress, Date, DateField, DateFieldState, DateTime, DateTimePicker,
    DateTimePickerState, Dialog, FloatingNavigation, Item, ItemVariant, LineCalendar,
    LineCalendarState, MenuPresentation, Messages, MultiSelect, NavigationItem, NavigationMenu,
    OtpField, PageTransition, PaginationState, Picker, PickerItem, Popover, PopoverMenu, Progress,
    Radio, RangeSlider, Resizable, ResizableAxis, Scaffold, Select, SelectGroup, SelectMenuTile,
    SelectOption, SelectTile, SelectTileGroup, Sheet, Sidebar, SidebarGroup, SidebarItem,
    SidebarState, Slider, Switch, Tabs, TextField, Theme, Tile, Time, TimeField, TimeFieldState,
    TimePicker, Toast, ToastEntry, ToastState, ToastVariant, ToasterState, context_menu,
    pagination, pagination_localized,
};

/// The constant date used by the calendar, line-calendar and date-field demos.
fn demo_date() -> Date {
    Date::new(2026, 11, 1).expect("constant date is valid")
}

/// Locales offered by the localization demo; they cover left-to-right,
/// right-to-left and non-Latin scripts.
const LOCALES: [(&str, &str); 6] = [
    ("en", "English"),
    ("de", "Deutsch"),
    ("ar", "العربية"),
    ("ja", "日本語"),
    ("zh", "中文"),
    ("fr", "Français"),
];

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Page {
    Overview,
    Controls,
    Overlays,
    Data,
    Localization,
}

fn nav_icon(painter: &egui::Painter, rect: egui::Rect, color: egui::Color32) {
    let stroke = egui::Stroke::new(1.5, color);
    painter.rect_stroke(rect.shrink(2.0), 4, stroke, egui::StrokeKind::Inside);
    painter.circle_filled(rect.center(), 2.0, color);
}

fn controls_icon(p: &egui::Painter, r: egui::Rect, c: egui::Color32) {
    for y in [r.top() + 4.0, r.center().y, r.bottom() - 4.0] {
        p.line_segment(
            [
                egui::pos2(r.left() + 2.0, y),
                egui::pos2(r.right() - 2.0, y),
            ],
            egui::Stroke::new(1.5, c),
        );
    }
}

fn overlays_icon(p: &egui::Painter, r: egui::Rect, c: egui::Color32) {
    p.rect_stroke(
        r.shrink(2.0),
        3,
        egui::Stroke::new(1.5, c),
        egui::StrokeKind::Inside,
    );
    p.line_segment(
        [
            egui::pos2(r.left() + 3.0, r.top() + 6.0),
            egui::pos2(r.right() - 3.0, r.top() + 6.0),
        ],
        egui::Stroke::new(1.5, c),
    );
}

fn data_icon(p: &egui::Painter, r: egui::Rect, c: egui::Color32) {
    for i in 0..3 {
        let x = r.left() + 3.0 + i as f32 * 5.0;
        p.line_segment(
            [
                egui::pos2(x, r.bottom() - 2.0),
                egui::pos2(x, r.bottom() - 5.0 - i as f32 * 3.0),
            ],
            egui::Stroke::new(2.0, c),
        );
    }
}

fn language_icon(p: &egui::Painter, r: egui::Rect, c: egui::Color32) {
    p.circle_stroke(r.center(), 6.0, egui::Stroke::new(1.5, c));
    p.line_segment(
        [
            egui::pos2(r.left() + 2.0, r.center().y),
            egui::pos2(r.right() - 2.0, r.center().y),
        ],
        egui::Stroke::new(1.5, c),
    );
}

impl Page {
    const ALL: [Self; 5] = [
        Self::Overview,
        Self::Controls,
        Self::Overlays,
        Self::Data,
        Self::Localization,
    ];

    fn label(self) -> &'static str {
        match self {
            Self::Overview => "Overview",
            Self::Controls => "Controls",
            Self::Overlays => "Overlays",
            Self::Data => "Data",
            Self::Localization => "Localization",
        }
    }

    fn id(self) -> Id {
        match self {
            Self::Overview => Id::new("demo-page-overview"),
            Self::Controls => Id::new("demo-page-controls"),
            Self::Overlays => Id::new("demo-page-overlays"),
            Self::Data => Id::new("demo-page-data"),
            Self::Localization => Id::new("demo-page-localization"),
        }
    }

    fn from_id(id: Id) -> Self {
        Self::ALL
            .into_iter()
            .find(|page| page.id() == id)
            .unwrap_or(Self::Overview)
    }
}

/// State for the interactive demos. Kept apart from navigation and theme state
/// so the scaffold closures borrow disjoint fields.
struct DemoState {
    surface_motion: temivu::liquid::SurfaceTransition,
    surface_preview_open: bool,
    show_motion_gallery: bool,
    motion_gallery: motion_gallery::Gallery,
    catalog_group: widget_catalog::Category,
    catalog_query: String,
    video: video_demo::DemoVideo,
    breadcrumb_root: bool,
    catalog_navigation: Option<Id>,
    // Overview
    accepted: bool,
    notifications: bool,
    // Controls
    name: String,
    email: String,
    volume: temivu::Volume,
    price_range: (f64, f64),
    radio_alpha: bool,
    radio_beta: bool,
    tab: usize,
    tab_order: [usize; 3],
    choice: Option<Id>,
    accordion: Option<AccordionState>,
    // Overlays
    clicks: usize,
    toast: Option<ToastState>,
    toaster: ToasterState,
    next_toast: usize,
    dialog_open: bool,
    dialog_from_center: bool,
    dialog_result: Option<&'static str>,
    overlay_action: Option<&'static str>,
    draft_filter: bool,
    applied_filter: bool,
    popover_open: bool,
    popover_menu_open: bool,
    sheet_open: bool,
    // Second-factor input
    otp: String,
    // Data
    item_selected: bool,
    tile_selected: bool,
    pagination: Option<PaginationState>,
    split_fraction: f32,
    calendar: Option<CalendarState>,
    line_calendar: Option<LineCalendarState>,
    // Fields and navigation
    date_field: DateFieldState,
    preview_date: bool,
    time_field: TimeFieldState,
    sidebar: SidebarState,
    sidebar_selected: Option<Id>,
    select_tiles: Vec<Id>,
    menu_choice: Option<Id>,
    // Autocomplete, pickers and combined date/time
    query: String,
    channels: Vec<Id>,
    picker_choice: Option<Id>,
    time_picker_time: Time,
    date_time: Option<DateTimePickerState>,
}

impl Default for DemoState {
    fn default() -> Self {
        Self {
            surface_motion: temivu::liquid::SurfaceTransition::Bloom,
            surface_preview_open: false,
            show_motion_gallery: false,
            motion_gallery: motion_gallery::Gallery::default(),
            catalog_group: widget_catalog::Category::default(),
            catalog_query: String::new(),
            video: video_demo::DemoVideo::default(),
            breadcrumb_root: false,
            catalog_navigation: Some(Id::new("catalog-nav-home")),
            accepted: false,
            notifications: false,
            name: String::new(),
            email: String::new(),
            volume: temivu::Volume::default(),
            price_range: (0.2, 0.8),
            radio_alpha: true,
            radio_beta: false,
            tab: 0,
            tab_order: [0, 1, 2],
            choice: None,
            accordion: None,
            clicks: 0,
            toast: None,
            toaster: ToasterState::default(),
            next_toast: 0,
            dialog_open: false,
            dialog_from_center: false,
            dialog_result: None,
            overlay_action: None,
            draft_filter: false,
            applied_filter: false,
            popover_menu_open: false,
            popover_open: false,
            sheet_open: false,
            item_selected: false,
            tile_selected: false,
            pagination: None,
            split_fraction: 0.5,
            calendar: None,
            line_calendar: None,
            date_field: DateFieldState::default(),
            preview_date: false,
            time_field: TimeFieldState::default(),
            sidebar: SidebarState::default(),
            sidebar_selected: None,
            select_tiles: Vec::new(),
            menu_choice: None,
            query: String::new(),
            channels: Vec::new(),
            picker_choice: Some(Id::new("demo-medium")),
            time_picker_time: Time::new(9, 30).expect("constant time is valid"),
            date_time: None,
            otp: String::new(),
        }
    }
}

pub(crate) struct Showcase {
    menu_open: bool,
    island_open: bool,
    motion_preview: bool,
    preview_started: Option<f64>,
    fonts_installed: bool,
    dark: bool,
    neutral: bool,
    progress: f32,
    upload_started: Option<f64>,
    high_contrast: bool,
    applied_theme: Option<(bool, bool, bool)>,
    reduced_motion: bool,
    applied_reduced_motion: Option<bool>,
    destination: Option<Id>,
    gallery_return: Option<Page>,
    page_motion: PageTransition,
    locale: usize,
    state: DemoState,
}

impl Default for Showcase {
    fn default() -> Self {
        Self {
            menu_open: false,
            island_open: false,
            motion_preview: false,
            preview_started: None,
            fonts_installed: false,
            dark: false,
            neutral: true,
            progress: 0.0,
            upload_started: None,
            high_contrast: false,
            applied_theme: None,
            reduced_motion: false,
            applied_reduced_motion: None,
            destination: None,
            gallery_return: None,
            page_motion: PageTransition::new(Id::new("demo-page-motion")),
            locale: 0,
            state: DemoState::default(),
        }
    }
}

struct OverviewOptions<'a> {
    dark: &'a mut bool,
    neutral: &'a mut bool,
    progress: &'a mut f32,
    upload_started: &'a mut Option<f64>,
    high_contrast: &'a mut bool,
    reduced_motion: &'a mut bool,
}

fn studio_panel<R>(
    ui: &mut egui::Ui,
    theme: &Theme,
    title: &str,
    description: &str,
    content: impl FnOnce(&mut egui::Ui) -> R,
) -> R {
    egui::Frame::new()
        .fill(theme.card)
        .stroke(egui::Stroke::new(1.0, theme.border))
        .corner_radius(28)
        .inner_margin(egui::Margin::same(22))
        .show(ui, |ui| {
            ui.set_min_width((ui.available_width() - 1.0).max(0.0));
            ui.label(egui::RichText::new(title).size(18.0).strong());
            ui.label(
                egui::RichText::new(description)
                    .size(13.0)
                    .color(theme.muted_foreground),
            );
            ui.add_space(18.0);
            content(ui)
        })
        .inner
}

fn appearance_panel(
    ui: &mut egui::Ui,
    theme: &Theme,
    dark: &mut bool,
    neutral: &mut bool,
    high_contrast: &mut bool,
    reduced_motion: &mut bool,
) {
    studio_panel(
        ui,
        theme,
        "Make it yours",
        "A system that follows your preferences.",
        |ui| {
            Switch::new(dark, theme)
                .label("Dark theme")
                .description("A quieter surface after hours")
                .show(ui);
            ui.add_space(10.0);
            let mut mint = !*neutral;
            Switch::new(&mut mint, theme)
                .label("Mint accent")
                .description("Add a little color to the monochrome palette")
                .show(ui);
            *neutral = !mint;
            ui.add_space(10.0);
            Switch::new(high_contrast, theme)
                .label("High contrast")
                .show(ui);
            Switch::new(reduced_motion, theme)
                .label("Reduced motion")
                .show(ui);
        },
    );
}

fn activity_panel(
    ui: &mut egui::Ui,
    theme: &Theme,
    progress: &mut f32,
    upload_started: &mut Option<f64>,
) {
    let now = ui.input(|input| input.time);
    if let Some(started) = *upload_started {
        *progress = ((now - started) / 3.0).clamp(0.0, 1.0) as f32;
        if *progress >= 1.0 {
            *upload_started = None;
        } else {
            ui.ctx()
                .request_repaint_after(std::time::Duration::from_millis(16));
        }
    }
    studio_panel(
        ui,
        theme,
        "A little momentum",
        "Try a control. Feel the response.",
        |ui| {
            ui.horizontal(|ui| {
                Avatar::new(theme)
                    .label("Profile")
                    .fallback("TV")
                    .colors(theme.secondary, theme.foreground)
                    .show(ui);
                ui.vertical(|ui| {
                    ui.label(egui::RichText::new("Design workspace").strong());
                    ui.label(
                        egui::RichText::new("Everything, in its place.")
                            .size(12.0)
                            .color(theme.muted_foreground),
                    );
                });
            });
            ui.add_space(18.0);
            ui.horizontal_wrapped(|ui| {
                if Button::new("Start upload", theme)
                    .enabled(upload_started.is_none())
                    .show(ui)
                    .clicked()
                {
                    *upload_started = Some(now);
                    *progress = 0.0;
                }
                if Button::new("Reset", theme)
                    .variant(ButtonVariant::Outline)
                    .show(ui)
                    .clicked()
                {
                    *progress = 0.0;
                    *upload_started = None;
                }
            });
            ui.add_space(16.0);
            ui.horizontal(|ui| {
                CircularProgress::determinate(theme, *progress)
                    .expect("normalized upload progress")
                    .diameter(40.0)
                    .expect("static ring diameter")
                    .label("Upload ring")
                    .show(ui);
                ui.label(
                    egui::RichText::new(format!("Upload progress · {:.0}%", *progress * 100.0))
                        .size(12.0)
                        .color(theme.muted_foreground),
                );
            });
            Progress::determinate(theme, *progress)
                .expect("valid progress")
                .label("Upload progress")
                .thickness(8.0)
                .expect("valid thickness")
                .show(ui);
            ui.add_space(8.0);
            let mut adjusted = f64::from(*progress);
            if Slider::new(&mut adjusted, theme)
                .expect("normalized progress")
                .label("Adjust progress")
                .show(ui)
                .changed()
            {
                *progress = adjusted as f32;
                *upload_started = None;
            }
        },
    );
}

fn overview(options: OverviewOptions<'_>, _locale: usize, ui: &mut egui::Ui, theme: &Theme) {
    let OverviewOptions {
        dark,
        neutral,
        progress,
        upload_started,
        high_contrast,
        reduced_motion,
    } = options;
    ui.set_max_width(ui.available_width().min(1040.0));
    ui.spacing_mut().item_spacing = egui::vec2(20.0, 12.0);
    ui.add_space(20.0);
    ui.label(
        egui::RichText::new("TEMIVU / COMPONENT STUDIO")
            .monospace()
            .size(11.0)
            .color(theme.muted_foreground),
    );
    ui.add_space(8.0);
    ui.label(egui::RichText::new("Welcome to Temivu").size(34.0).strong());
    ui.label(
        egui::RichText::new("Thoughtful components for beautiful, accessible interfaces.")
            .size(15.0)
            .color(theme.muted_foreground),
    );
    ui.add_space(12.0);
    ui.horizontal_wrapped(|ui| {
        Badge::new(if *neutral { "MONOCHROME" } else { "MINT" }, theme)
            .radius(6)
            .show(ui);
        Badge::new("LIGHT & DARK", theme)
            .radius(6)
            .style(BadgeStyle::from_theme(theme, BadgeVariant::Outline, false))
            .show(ui);
        Badge::new("INTERACTIVE PREVIEW", theme)
            .radius(6)
            .style(BadgeStyle::from_theme(
                theme,
                BadgeVariant::Secondary,
                false,
            ))
            .show(ui);
    });
    ui.add_space(20.0);
    if ui.available_width() >= 760.0 {
        ui.columns(2, |columns| {
            activity_panel(&mut columns[0], theme, progress, upload_started);
            appearance_panel(
                &mut columns[1],
                theme,
                dark,
                neutral,
                high_contrast,
                reduced_motion,
            );
        });
    } else {
        activity_panel(ui, theme, progress, upload_started);
        ui.add_space(12.0);
        appearance_panel(ui, theme, dark, neutral, high_contrast, reduced_motion);
    }
    ui.add_space(14.0);
    Alert::new("Preview build", theme)
        .subtitle("Explore the Controls page for fields, choices and calendar interactions.")
        .show(ui);
}

fn motion_controls(
    state: &mut DemoState,
    ui: &mut egui::Ui,
    theme: &Theme,
    gallery_link: bool,
) -> Option<egui::Response> {
    use temivu::liquid::{LiquidStyle, SurfaceTransition};
    let mut preview = None;
    let mut gallery = None;
    ui.horizontal_wrapped(|ui| {
        for (kind, label, description) in [
            (
                SurfaceTransition::Bloom,
                "Bloom",
                "Soft expansion from the whole trigger",
            ),
            (
                SurfaceTransition::Droplet,
                "Droplet",
                "Elastic separation and merging",
            ),
        ] {
            let response = Button::new(label, theme)
                .variant(if state.surface_motion == kind {
                    ButtonVariant::Primary
                } else {
                    ButtonVariant::Ghost
                })
                .show(ui)
                .on_hover_text(description);
            if response.clicked() {
                state.surface_motion = kind;
                ui.ctx().request_repaint();
            }
        }
        let anchor = Button::new("Preview", theme)
            .variant(ButtonVariant::Secondary)
            .show(ui);
        if anchor.clicked() {
            state.surface_preview_open = !state.surface_preview_open;
        }
        preview = Some(anchor);
        if gallery_link {
            let link = Button::new("Motion examples", theme)
                .variant(ButtonVariant::Outline)
                .show(ui);
            if link.clicked() {
                gallery = Some(link);
            }
        }
    });
    let motion = match state.surface_motion {
        SurfaceTransition::Bloom => LiquidStyle::bloom(),
        SurfaceTransition::Droplet => LiquidStyle::droplet(),
    };
    motion.set_context_default(ui.ctx());
    if let Some(anchor) = preview {
        Popover::new(
            Id::new("demo-surface-preview"),
            "Motion preview",
            &mut state.surface_preview_open,
        )
        .theme(theme)
        .liquid(motion)
        .width(280.0)
        .show(&anchor, |ui| {
            ui.label(
                egui::RichText::new(match state.surface_motion {
                    SurfaceTransition::Bloom => "Bloom",
                    SurfaceTransition::Droplet => "Droplet",
                })
                .size(18.0)
                .strong(),
            );
            ui.label("One surface, two ways to move.");
            ui.add_space(12.0);
            let close = Button::new("Done", theme).show(ui);
            if close.clicked() {
                ui.close();
            }
            close
        });
    }
    gallery
}

enum GalleryAction {
    Open(egui::Response),
    Close,
}

fn controls(
    state: &mut DemoState,
    ui: &mut egui::Ui,
    theme: &Theme,
    can_go_back: bool,
) -> Option<GalleryAction> {
    ui.set_max_width(ui.available_width().min(1040.0));
    ui.spacing_mut().item_spacing.y = 10.0;
    if state.show_motion_gallery {
        let leave = ui
            .horizontal_wrapped(|ui| {
                let clicked = !can_go_back
                    && Button::new("All controls", theme)
                        .variant(ButtonVariant::Outline)
                        .show(ui)
                        .clicked();
                ui.label(egui::RichText::new("Motion gallery").size(27.0).strong());
                clicked
            })
            .inner;
        widget_catalog::show(state, ui, theme);
        return leave.then_some(GalleryAction::Close);
    }
    ui.label(
        egui::RichText::new("COMPONENT STUDIO / CONTROLS")
            .monospace()
            .size(11.0)
            .color(theme.muted_foreground),
    );
    ui.label(
        egui::RichText::new("Controls and inputs")
            .size(30.0)
            .strong(),
    );
    ui.label(
        egui::RichText::new("Buttons, fields and choices for everyday tasks.")
            .color(theme.muted_foreground),
    );
    let gallery = motion_controls(state, ui, theme, true);
    ui.add_space(12.0);
    if ui.available_width() >= 760.0 {
        ui.columns(2, |columns| {
            control_project(state, &mut columns[0], theme);
            control_values(state, &mut columns[1], theme);
        });
        ui.add_space(16.0);
        ui.columns(2, |columns| {
            control_schedule(state, &mut columns[0], theme);
            control_choices(state, &mut columns[1], theme);
        });
    } else {
        control_project(state, ui, theme);
        ui.add_space(16.0);
        control_values(state, ui, theme);
        ui.add_space(16.0);
        control_schedule(state, ui, theme);
        ui.add_space(16.0);
        control_choices(state, ui, theme);
    }
    ui.add_space(16.0);
    control_details(state, ui, theme);
    gallery.map(GalleryAction::Open)
}

fn control_details(state: &mut DemoState, ui: &mut egui::Ui, theme: &Theme) {
    studio_panel(
        ui,
        theme,
        "The finer details",
        "Drag tabs to rearrange. Use Alt + arrow keys from the keyboard.",
        |ui| {
            Tabs::new(&mut state.tab, &["General", "Privacy", "About"], theme)
                .expect("three tabs with an in-range selection")
                .reorderable(&mut state.tab_order)
                .expect("permutation of three tabs")
                .show(ui, |ui, index| {
                    ui.label(match index {
                        0 => "General settings content",
                        1 => "Privacy settings content",
                        _ => "About this library",
                    });
                });
            ui.add_space(8.0);
            let accordion = state
                .accordion
                .get_or_insert_with(|| AccordionState::new(0, Some(2)).expect("valid limits"));
            AccordionItem::new(accordion, 0, "Details", theme).show(ui, |ui| {
                ui.label("Accordion content");
            });
            AccordionItem::new(accordion, 1, "More details", theme).show(ui, |ui| {
                ui.label("More accordion content");
            });
        },
    );
}

fn control_project(state: &mut DemoState, ui: &mut egui::Ui, theme: &Theme) {
    studio_panel(
        ui,
        theme,
        "Your next project",
        "Make something worth opening every day.",
        |ui| {
            ui.horizontal_wrapped(|ui| {
                if Button::new("Primary", theme).show(ui).clicked() {
                    state.clicks += 1;
                }
                Button::new("Secondary", theme)
                    .variant(ButtonVariant::Secondary)
                    .show(ui);
                Button::new("Delete", theme)
                    .variant(ButtonVariant::Destructive)
                    .show(ui);
            });
            let hovered = Button::new("Hover me", theme)
                .variant(ButtonVariant::Ghost)
                .show(ui);
            tooltip(hovered, "Soft motion. Precise feedback.");
            ui.add_space(8.0);
            TextField::new(&mut state.name, theme, "Name")
                .id(Id::new("demo-name"))
                .hint("Enter your name")
                .show(ui);
            ui.add_space(4.0);
            let email_validator = |value: &str| {
                (!value.is_empty() && !value.contains('@'))
                    .then(|| "Enter an address that contains @".to_owned())
            };
            TextField::new(&mut state.email, theme, "Email")
                .id(Id::new("demo-email-input"))
                .hint("name@example.com")
                .description("We only use this for notifications.")
                .validator(&email_validator)
                .show(ui);
            ui.add_space(4.0);
            Checkbox::new(&mut state.accepted, theme, "Accept terms").show(ui);
            Switch::new(&mut state.notifications, theme)
                .label("Enable notifications")
                .show(ui);
        },
    );
}

fn control_values(state: &mut DemoState, ui: &mut egui::Ui, theme: &Theme) {
    studio_panel(
        ui,
        theme,
        "Choices & values",
        "Fine-tune the details.",
        |ui| {
            let mut alpha_clicked = false;
            let mut beta_clicked = false;
            ui.horizontal_wrapped(|ui| {
                alpha_clicked = Radio::new(&mut state.radio_alpha, theme, "Option A")
                    .show(ui)
                    .clicked();
                beta_clicked = Radio::new(&mut state.radio_beta, theme, "Option B")
                    .show(ui)
                    .clicked();
            });
            // `Radio` toggles standalone state; the demo enforces group semantics until
            // a radio-group API exists, so clicking the selected option keeps it on.
            if alpha_clicked {
                state.radio_alpha = true;
                state.radio_beta = false;
            }
            if beta_clicked {
                state.radio_beta = true;
                state.radio_alpha = false;
            }
            ui.add_space(4.0);
            ui.add_space(6.0);
            ui.label(egui::RichText::new("Volume").strong());
            temivu::volume_control(ui, theme, &mut state.volume);
            ui.add_space(8.0);
            ui.label("Set the minimum and maximum price");
            RangeSlider::new(&mut state.price_range, theme)
                .expect("normalized range")
                .label("Price range")
                .step(0.05)
                .expect("valid step")
                .show(ui);
            ui.add_space(8.0);
        },
    );
}

fn control_schedule(state: &mut DemoState, ui: &mut egui::Ui, theme: &Theme) {
    studio_panel(
        ui,
        theme,
        "A time for everything",
        "Dates, time and verification in one place.",
        |ui| {
            let otp = OtpField::new(&mut state.otp, theme, "Verification code")
                .length(6)
                .description("A six-digit code")
                .show(ui);
            if otp.completed {
                ui.label("The code is complete.");
            }
            ui.add_space(8.0);
            let date = DateField::new(&mut state.date_field, theme, "Start date")
                .hint("YYYY-MM-DD")
                .today(demo_date())
                .show(ui);
            if state.preview_date {
                state.preview_date = false;
                state.date_field.set_open(true);
                ui.scroll_to_rect(date.response.rect, Some(egui::Align::Center));
                ui.ctx().request_repaint();
            }
            let _ = TimeField::new(&mut state.time_field, theme, "Start time").show(ui);
            ui.add_space(8.0);
        },
    );
}

fn control_choices(state: &mut DemoState, ui: &mut egui::Ui, theme: &Theme) {
    studio_panel(
        ui,
        theme,
        "Find your preference",
        "Search, choose and make it yours.",
        |ui| {
            let options = [
                SelectOption::new(Id::new("demo-red"), "Red"),
                SelectOption::new(Id::new("demo-blue"), "Blue"),
            ];
            SelectGroup::new(Id::new("demo-select-group"), "Appearance", theme)
                .description("Choose the accent color")
                .show(ui, |ui| {
                    Select::new(
                        Id::new("demo-select"),
                        "Color",
                        "Choose a color",
                        &options,
                        &mut state.choice,
                    )
                    .theme(theme)
                    .searchable(true)
                    .show(ui);
                });
            ui.add_space(8.0);
            let channels = [
                SelectOption::new(Id::new("demo-email"), "Email digest"),
                SelectOption::new(Id::new("demo-sms"), "SMS alerts"),
                SelectOption::new(Id::new("demo-push"), "Push notifications"),
            ];
            MultiSelect::new(
                Id::new("demo-channels"),
                "Notification channels",
                "Choose channels",
                &channels,
                &mut state.channels,
            )
            .searchable(true)
            .show(ui);
            ui.add_space(8.0);
            let fruits = [
                AutocompleteOption::new(Id::new("demo-apple"), "Apple"),
                AutocompleteOption::new(Id::new("demo-apricot"), "Apricot"),
                AutocompleteOption::new(Id::new("demo-banana"), "Banana"),
                AutocompleteOption::new(Id::new("demo-blueberry"), "Blueberry"),
            ];
            let _ = Autocomplete::new(&mut state.query, theme, "Fruit", &fruits)
                .hint("Type to search")
                .id(Id::new("demo-autocomplete"))
                .show(ui);
            ui.add_space(8.0);
            let sizes = [
                PickerItem::new(Id::new("demo-small"), "Small"),
                PickerItem::new(Id::new("demo-medium"), "Medium"),
                PickerItem::new(Id::new("demo-large"), "Large"),
            ];
            let _ = Picker::new(&mut state.picker_choice, &sizes, theme)
                .label("Size")
                .visible_rows(3)
                .id(Id::new("demo-picker"))
                .show(ui);
            ui.add_space(8.0);
        },
    );
}

fn page_intro(ui: &mut egui::Ui, theme: &Theme, section: &str, title: &str, description: &str) {
    ui.label(
        egui::RichText::new(section)
            .monospace()
            .size(11.0)
            .color(theme.muted_foreground),
    );
    ui.label(
        egui::RichText::new(title)
            .size(30.0)
            .strong()
            .color(theme.foreground),
    );
    ui.label(egui::RichText::new(description).color(theme.muted_foreground));
    ui.add_space(12.0);
}

fn studio_pair(
    state: &mut DemoState,
    ui: &mut egui::Ui,
    theme: &Theme,
    first: fn(&mut DemoState, &mut egui::Ui, &Theme),
    second: fn(&mut DemoState, &mut egui::Ui, &Theme),
) {
    if ui.available_width() >= 780.0 {
        ui.columns(2, |columns| {
            first(state, &mut columns[0], theme);
            second(state, &mut columns[1], theme);
        });
    } else {
        first(state, ui, theme);
        ui.add_space(14.0);
        second(state, ui, theme);
    }
    ui.add_space(14.0);
}

fn overlays(state: &mut DemoState, ui: &mut egui::Ui, theme: &Theme) {
    page_intro(
        ui,
        theme,
        "COMPONENT STUDIO / OVERLAYS",
        "Overlays and dialogs",
        "Compare anchored surfaces, confirmations and notifications.",
    );
    let _ = motion_controls(state, ui, theme, false);
    ui.add_space(14.0);
    studio_pair(state, ui, theme, overlay_dialog, overlay_sheet);
    studio_pair(state, ui, theme, overlay_actions, overlay_notifications);
    show_toast_stack(state, ui, theme);
}

fn show_toast_stack(state: &mut DemoState, ui: &mut egui::Ui, theme: &Theme) {
    if !state.toaster.is_empty() {
        let narrow = ui.ctx().content_rect().width() < 640.0;
        let (anchor, offset) = if narrow {
            (
                egui::Align2::CENTER_BOTTOM,
                egui::vec2(0.0, -FloatingNavigation::BOTTOM_CLEARANCE),
            )
        } else {
            (
                egui::Align2::RIGHT_BOTTOM,
                egui::vec2(-20.0, -FloatingNavigation::BOTTOM_CLEARANCE),
            )
        };
        state.toaster.show_with(
            ui,
            theme,
            "Dismiss",
            Id::new("demo-overlay-notifications"),
            anchor,
            offset,
        );
    }
}

fn overlay_dialog(state: &mut DemoState, ui: &mut egui::Ui, theme: &Theme) {
    studio_panel(
        ui,
        theme,
        "Confirmation dialog",
        "Pause to confirm a decision.",
        |ui| {
            let (trigger, centered) = ui
                .horizontal_wrapped(|ui| {
                    (
                        Button::new("Open dialog", theme).show(ui),
                        Button::new("From center", theme)
                            .variant(ButtonVariant::Outline)
                            .show(ui),
                    )
                })
                .inner;
            if trigger.clicked() || centered.clicked() {
                state.dialog_open = true;
                state.dialog_from_center = centered.clicked();
            }
            ui.label(
                egui::RichText::new(state.dialog_result.unwrap_or("Ready to review"))
                    .color(theme.muted_foreground),
            );
            let mut result = None;
            let screen_width = ui.ctx().content_rect().width();
            let width = (screen_width - 76.0).clamp(1.0, 360.0);
            let frame = egui::Frame::new()
                .fill(theme.card)
                .stroke(egui::Stroke::new(1.0, theme.border))
                .inner_margin(24)
                .corner_radius(30)
                .shadow(egui::epaint::Shadow {
                    offset: [0, 8],
                    blur: 24,
                    spread: 0,
                    color: egui::Color32::from_black_alpha(if theme.dark { 90 } else { 40 }),
                });
            let dialog = Dialog::new(Id::new("demo-dialog"), "Confirm action")
                .theme(theme)
                .frame(frame);
            let dialog = if state.dialog_from_center {
                dialog
            } else {
                dialog.source_response(&trigger)
            };
            dialog.show(ui.ctx(), &mut state.dialog_open, |ui| {
                ui.set_width(width);
                Badge::new("REVIEW", theme).show(ui);
                ui.add_space(8.0);
                ui.label(egui::RichText::new("Confirm action").size(24.0).strong());
                ui.label(
                    egui::RichText::new("Review the details before continuing.")
                        .color(theme.muted_foreground),
                );
                ui.add_space(18.0);
                ui.horizontal_wrapped(|ui| {
                    if Button::new("Continue", theme).show(ui).clicked() {
                        result = Some("Action confirmed");
                        ui.close();
                    }
                    if Button::new("Cancel", theme)
                        .variant(ButtonVariant::Outline)
                        .show(ui)
                        .clicked()
                    {
                        result = Some("Action cancelled");
                        ui.close();
                    }
                });
            });
            if result.is_some() {
                state.dialog_result = result;
            }
        },
    );
}

fn overlay_sheet(state: &mut DemoState, ui: &mut egui::Ui, theme: &Theme) {
    studio_panel(
        ui,
        theme,
        "Bottom sheet",
        "Adjust filters in a surface that slides up.",
        |ui| {
            let trigger = ui
                .horizontal(|ui| {
                    Button::new("Open sheet", theme)
                        .variant(ButtonVariant::Outline)
                        .show(ui)
                })
                .inner;
            if trigger.clicked() {
                state.draft_filter = state.applied_filter;
                state.sheet_open = true;
            }
            ui.label(if state.applied_filter {
                "Showing available items"
            } else {
                "Showing all items"
            });
            Sheet::new(Id::new("demo-sheet"), "Filters", theme).show(
                ui.ctx(),
                &mut state.sheet_open,
                |ui| {
                    ui.label(egui::RichText::new("Sheet content").color(theme.muted_foreground));
                    Switch::new(&mut state.draft_filter, theme)
                        .label("Only available items")
                        .show(ui);
                    ui.add_space(12.0);
                    if Button::new("Apply", theme).show(ui).clicked() {
                        state.applied_filter = state.draft_filter;
                        ui.close();
                    }
                },
            );
        },
    );
}

fn overlay_actions(state: &mut DemoState, ui: &mut egui::Ui, theme: &Theme) {
    studio_panel(
        ui,
        theme,
        "Anchored actions",
        "A popover, action menu and context menu.",
        |ui| {
            let (anchor, menu_anchor) = ui
                .horizontal_wrapped(|ui| {
                    (
                        Button::new("Show popover", theme)
                            .variant(ButtonVariant::Secondary)
                            .show(ui),
                        Button::new("Show menu", theme)
                            .variant(ButtonVariant::Outline)
                            .show(ui),
                    )
                })
                .inner;
            if anchor.clicked() {
                state.popover_open = !state.popover_open;
            }
            Popover::new(
                Id::new("demo-popover"),
                "Demo popover",
                &mut state.popover_open,
            )
            .theme(theme)
            .width(248.0)
            .focus_on_open(true)
            .show(&anchor, |ui| {
                ui.label(egui::RichText::new("Popover content").strong());
                ui.label("Quick actions stay close to their trigger.");
                ui.add_space(8.0);
                let action = Button::new("Popover action", theme).show(ui);
                if action.clicked() {
                    state.overlay_action = Some("Popover action applied");
                    ui.close();
                }
                action
            });
            if menu_anchor.clicked() {
                state.popover_menu_open = !state.popover_menu_open;
            }
            let pin_id = Id::new("demo-menu-pin");
            PopoverMenu::new(
                Id::new("demo-popover-menu"),
                "Demo menu",
                &mut state.popover_menu_open,
            )
            .theme(theme)
            .width(220.0)
            .show(&menu_anchor, |ui, menu| {
                if menu
                    .item(
                        ui,
                        Item::new("Rename", theme).id(Id::new("demo-menu-rename")),
                    )
                    .clicked()
                {
                    state.overlay_action = Some("Rename selected");
                    ui.close();
                }
                let _ = menu.item(ui, Item::new("Archive", theme).enabled(false));
                let pinned = state.menu_choice == Some(pin_id);
                if menu
                    .select_menu_tile(
                        ui,
                        SelectMenuTile::new("Pin to top", theme)
                            .id(pin_id)
                            .selected(pinned),
                    )
                    .clicked()
                {
                    state.menu_choice = (!pinned).then_some(pin_id);
                    state.overlay_action = Some(if pinned {
                        "Item unpinned"
                    } else {
                        "Item pinned"
                    });
                    ui.close();
                }
                if menu
                    .item(
                        ui,
                        Item::new("Delete", theme)
                            .variant(ItemVariant::Destructive)
                            .id(Id::new("demo-menu-delete")),
                    )
                    .clicked()
                {
                    state.overlay_action = Some("Delete selected");
                    ui.close();
                }
            });
            if let Some(status) = state.overlay_action {
                ui.label(egui::RichText::new(status).color(theme.muted_foreground));
            }
            ui.add_space(12.0);
            ui.horizontal_wrapped(|ui| {
                let button = Button::new("Click me", theme)
                    .icon("+")
                    .variant(ButtonVariant::Secondary)
                    .show(ui);
                if button.clicked() {
                    state.clicks = state.clicks.saturating_add(1);
                }
                context_menu(&button, |ui| {
                    let reset = Button::new("Reset clicks", theme)
                        .variant(ButtonVariant::Ghost)
                        .show(ui);
                    if reset.clicked() {
                        state.clicks = 0;
                        ui.close();
                    }
                    reset
                });
                Badge::new(&format!("Clicks: {}", state.clicks), theme).show(ui);
            });
            ui.label(
                egui::RichText::new("Right-click or press Shift+F10 to reset.")
                    .size(12.0)
                    .color(theme.muted_foreground),
            );
        },
    );
}

fn overlay_notifications(state: &mut DemoState, ui: &mut egui::Ui, theme: &Theme) {
    studio_panel(
        ui,
        theme,
        "Notifications",
        "Send one inline notice or stack messages above the page.",
        |ui| {
            ui.horizontal_wrapped(|ui| {
                if Button::new("Show notification", theme)
                    .variant(ButtonVariant::Secondary)
                    .show(ui)
                    .clicked()
                {
                    state.toast = Some(ToastState::default());
                }
                if Button::new("Queue notifications", theme)
                    .variant(ButtonVariant::Outline)
                    .show(ui)
                    .clicked()
                {
                    state.next_toast += 1;
                    let index = state.next_toast;
                    state.toaster.push(
                        ToastEntry::new(
                            Id::new(("queued-toast", index)),
                            format!("Queued notification {index}"),
                        )
                        .description(if index.is_multiple_of(2) {
                            "An item needs your attention."
                        } else {
                            "Your update is ready."
                        })
                        .icon(if index.is_multiple_of(2) { "!" } else { "i" })
                        .variant(if index.is_multiple_of(2) {
                            ToastVariant::Destructive
                        } else {
                            ToastVariant::Primary
                        })
                        .timeout(None),
                    );
                }
            });
            if let Some(toast) = &mut state.toast {
                ui.add_space(8.0);
                Toast::new(toast, theme, "Notification", "Dismiss")
                    .description("This notification closes when dismissed.")
                    .show(ui);
                if !toast.is_visible() {
                    state.toast = None;
                }
            }
        },
    );
}

fn data(state: &mut DemoState, ui: &mut egui::Ui, theme: &Theme) {
    page_intro(
        ui,
        theme,
        "COMPONENT STUDIO / DATA",
        "Data displays",
        "A compact workspace for records, folders and scheduling.",
    );
    studio_pair(state, ui, theme, data_collection, data_pagination);
    studio_pair(state, ui, theme, data_calendar, data_explorer);
    data_date_strip(state, ui, theme);
    ui.add_space(14.0);
    studio_pair(state, ui, theme, data_time, data_datetime);
}

fn data_date_strip(state: &mut DemoState, ui: &mut egui::Ui, theme: &Theme) {
    studio_panel(
        ui,
        theme,
        "Date strip",
        "Move through days without leaving the page.",
        |ui| {
            let calendar = state.line_calendar.get_or_insert_with(|| {
                LineCalendarState::new(
                    Date::new(2026, 11, 8).expect("valid date"),
                    Some(Date::new(2026, 11, 12).expect("valid date")),
                )
            });
            LineCalendar::new(calendar, theme)
                .today(demo_date())
                .days(10)
                .show(ui);
        },
    );
}

fn data_collection(state: &mut DemoState, ui: &mut egui::Ui, theme: &Theme) {
    studio_panel(
        ui,
        theme,
        "Collection",
        "Choose what stays active in this workspace.",
        |ui| {
            let count = usize::from(state.item_selected)
                + usize::from(state.tile_selected)
                + state.select_tiles.len();
            ui.horizontal_wrapped(|ui| {
                Badge::new(&format!("{count} selected"), theme).show(ui);
                ui.label(
                    egui::RichText::new("WORKSPACE")
                        .monospace()
                        .size(11.0)
                        .color(theme.muted_foreground),
                );
            });
            ui.add_space(8.0);
            if Item::new("Example item", theme)
                .icon("+")
                .description("Select this row")
                .selected(state.item_selected)
                .show(ui)
                .clicked()
            {
                state.item_selected = !state.item_selected;
            }
            ui.add_space(6.0);
            if Tile::new("Bluetooth", theme)
                .id(Id::new("demo-bluetooth-tile"))
                .description("Connect devices")
                .selected(state.tile_selected)
                .show(ui)
                .clicked()
            {
                state.tile_selected = !state.tile_selected;
            }
            ui.add_space(8.0);
            let tiles = [
                SelectTile::new(Id::new("demo-wifi"), "Airplane mode")
                    .description("Disable radios"),
                SelectTile::new(Id::new("demo-bluetooth"), "Power saving")
                    .description("Reduce battery use"),
            ];
            SelectTileGroup::new(
                Id::new("demo-radios"),
                "Radios",
                theme,
                &tiles,
                &mut state.select_tiles,
            )
            .description("Choose any that apply")
            .show(ui);
            let has_selection =
                state.item_selected || state.tile_selected || !state.select_tiles.is_empty();
            ui.add_space(6.0);
            if Item::new("Clear selection", theme)
                .variant(ItemVariant::Destructive)
                .enabled(has_selection)
                .show(ui)
                .clicked()
            {
                state.item_selected = false;
                state.tile_selected = false;
                state.select_tiles.clear();
                ui.ctx().request_repaint();
            }
        },
    );
}

fn data_pagination(state: &mut DemoState, ui: &mut egui::Ui, theme: &Theme) {
    const PAGE_SIZE: usize = 6;
    studio_panel(
        ui,
        theme,
        "Paged records",
        "Browse a collection in groups of six.",
        |ui| {
            let pages = state
                .pagination
                .get_or_insert_with(|| PaginationState::new(12, 1, 1, true).expect("valid state"));
            ui.horizontal_wrapped(|ui| {
                Badge::new(
                    &format!("Page {} of {}", pages.page() + 1, pages.pages()),
                    theme,
                )
                .show(ui);
                ui.label(
                    egui::RichText::new(format!("{} records", pages.pages() * PAGE_SIZE))
                        .color(theme.muted_foreground),
                );
            });
            ui.add_space(10.0);
            let page = pages.page() as f32;
            let slide = temivu::motion::spring_f32(
                ui.ctx(),
                ui.make_persistent_id("record-motion"),
                page,
                page,
                theme,
            );
            let shift = (slide - page).clamp(-1.0, 1.0);
            let first = pages.page() * PAGE_SIZE + 1;
            ui.with_visual_transform(
                egui::emath::TSTransform::from_translation(egui::vec2(6.0 * shift, 0.0)),
                |ui| {
                    ui.multiply_opacity(1.0 - 0.40 * shift.abs());
                    ui.horizontal(|ui| {
                        ui.label(
                            egui::RichText::new("RECORD / STATUS")
                                .monospace()
                                .size(11.0)
                                .color(theme.muted_foreground),
                        );
                    });
                    for record in first..first + PAGE_SIZE {
                        egui::Frame::new()
                            .fill(if record.is_multiple_of(2) {
                                theme.muted
                            } else {
                                theme.card
                            })
                            .stroke(egui::Stroke::new(1.0, theme.border))
                            .corner_radius(13)
                            .inner_margin(8)
                            .show(ui, |ui| {
                                ui.set_min_width(ui.available_width());
                                egui::Sides::new().height(18.0).show(
                                    ui,
                                    |ui| {
                                        ui.label(
                                            egui::RichText::new(format!("Record {record:02}"))
                                                .strong(),
                                        );
                                    },
                                    |ui| {
                                        ui.label(
                                            egui::RichText::new(if record.is_multiple_of(3) {
                                                "In review"
                                            } else {
                                                "Ready"
                                            })
                                            .size(11.0)
                                            .color(theme.muted_foreground),
                                        );
                                    },
                                );
                            });
                    }
                },
            );
            ui.add_space(12.0);
            pagination(ui, pages, theme, "Previous", "Next");
        },
    );
}

fn data_calendar(state: &mut DemoState, ui: &mut egui::Ui, theme: &Theme) {
    studio_panel(
        ui,
        theme,
        "Calendar",
        "Browse months and choose a date.",
        |ui| {
            let calendar = state
                .calendar
                .get_or_insert_with(|| CalendarState::new(demo_date(), None));
            egui::Frame::new()
                .fill(theme.muted)
                .corner_radius(19)
                .inner_margin(8)
                .show(ui, |ui| {
                    Calendar::new(calendar, theme).today(demo_date()).show(ui);
                });
            ui.add_space(8.0);
            let selected = calendar
                .selected()
                .map_or_else(|| "No date selected".to_owned(), |date| date.to_string());
            Badge::new(&selected, theme).show(ui);
        },
    );
}

fn data_explorer(state: &mut DemoState, ui: &mut egui::Ui, theme: &Theme) {
    studio_panel(
        ui,
        theme,
        "Navigation sidebar",
        "Explore folders and resize the work area.",
        |ui| {
            let wide = ui.available_width() >= 360.0;
            let axis = if wide {
                ResizableAxis::Horizontal
            } else {
                ResizableAxis::Vertical
            };
            let (rect, _) = ui.allocate_exact_size(
                egui::vec2(ui.available_width(), if wide { 244.0 } else { 330.0 }),
                egui::Sense::hover(),
            );
            let mut split = ui.new_child(egui::UiBuilder::new().max_rect(rect));
            split.set_clip_rect(split.clip_rect().intersect(rect));
            let inbox = [
                SidebarItem::new(Id::new("demo-inbox"), "Inbox"),
                SidebarItem::new(Id::new("demo-sent"), "Sent"),
            ];
            let history = [SidebarItem::new(Id::new("demo-archive"), "Archive")];
            let groups = [
                SidebarGroup::new(Id::new("demo-mail"), &inbox).label("Mail"),
                SidebarGroup::new(Id::new("demo-history"), &history).label("History"),
            ];
            let previous = state.sidebar_selected;
            Resizable::new(
                Id::new("demo-split"),
                axis,
                &mut state.split_fraction,
                theme,
            )
            .label("Resizable panes")
            .min_extents(156.0, 100.0)
            .show(
                &mut split,
                |ui| {
                    ui.label(
                        egui::RichText::new("Left pane")
                            .size(12.0)
                            .color(theme.muted_foreground),
                    );
                    egui::ScrollArea::vertical()
                        .id_salt("demo-folders")
                        .show(ui, |ui| {
                            Sidebar::new(
                                Id::new("demo-sidebar"),
                                "Demo navigation",
                                theme,
                                &mut state.sidebar,
                            )
                            .title("Workspace")
                            .show(
                                ui,
                                &groups,
                                &mut state.sidebar_selected,
                            );
                        });
                },
                |ui| {
                    ui.add_space(4.0);
                    ui.label(
                        egui::RichText::new("Right pane")
                            .size(12.0)
                            .color(theme.muted_foreground),
                    );
                    if let Some(folder) = inbox
                        .iter()
                        .chain(&history)
                        .find(|item| Some(item.id) == previous)
                    {
                        ui.label(egui::RichText::new(folder.label).size(20.0).strong());
                        Badge::new(
                            if folder.label == "Inbox" {
                                "2 items"
                            } else {
                                "0 items"
                            },
                            theme,
                        )
                        .show(ui);
                        ui.add_space(6.0);
                        if folder.label == "Inbox" {
                            ui.label("Project brief");
                            ui.label("Design notes");
                        } else {
                            ui.label(
                                egui::RichText::new("No items yet").color(theme.muted_foreground),
                            );
                        }
                    } else {
                        ui.label("Select a folder");
                    }
                },
            );
            if previous != state.sidebar_selected {
                ui.ctx().request_repaint();
            }
        },
    );
}

fn data_time(state: &mut DemoState, ui: &mut egui::Ui, theme: &Theme) {
    studio_panel(
        ui,
        theme,
        "Time picker",
        "Set the time with scroll, touch or the arrow keys.",
        |ui| {
            ui.label(
                egui::RichText::new(state.time_picker_time.to_string())
                    .monospace()
                    .size(30.0)
                    .strong(),
            );
            ui.add_space(8.0);
            TimePicker::new(&mut state.time_picker_time, theme)
                .id(Id::new("demo-time-picker"))
                .show(ui);
        },
    );
}

fn data_datetime(state: &mut DemoState, ui: &mut egui::Ui, theme: &Theme) {
    studio_panel(
        ui,
        theme,
        "Schedule",
        "Combine a calendar date and precise time.",
        |ui| {
            let date_time = state.date_time.get_or_insert_with(|| {
                let december = Date::new(2026, 12, 1).expect("constant date is valid");
                let selected = Date::new(2026, 12, 20).expect("constant date is valid");
                DateTimePickerState::new(
                    december,
                    Some(DateTime::new(selected, state.time_picker_time)),
                )
            });
            DateTimePicker::new(date_time, theme)
                .today(demo_date())
                .seconds(true)
                .show(ui);
        },
    );
}

fn localization(state: &mut DemoState, locale: &mut usize, ui: &mut egui::Ui, theme: &Theme) {
    let messages = Messages::for_locale(LOCALES[*locale].0);
    ui.heading("Localization and themes");
    ui.label("Strings generated from the 113 pinned Forui ARB locale files.");
    ui.horizontal_wrapped(|ui| {
        for (index, (code, name)) in LOCALES.iter().enumerate() {
            if ui
                .selectable_label(*locale == index, format!("{name} ({code})"))
                .clicked()
            {
                *locale = index;
            }
        }
    });
    ui.add_space(8.0);
    ui.label(format!("progressSemanticsLabel = {}", messages.progress));
    ui.label(format!(
        "paginationPreviousSemanticsLabel = {}",
        messages.previous
    ));
    ui.add_space(8.0);
    let pagination_state = state
        .pagination
        .get_or_insert_with(|| PaginationState::new(12, 1, 1, true).expect("valid state"));
    pagination_localized(ui, pagination_state, theme, &messages);
    ui.add_space(8.0);
    ui.label("The page buttons expose localized labels to assistive technology.");
}

impl eframe::App for Showcase {
    fn ui(&mut self, ui: &mut egui::Ui, _frame: &mut eframe::Frame) {
        if !self.fonts_installed {
            Theme::install_inter(ui.ctx());
            self.fonts_installed = true;
        }
        let mut theme = if self.neutral && self.dark {
            Theme::dark()
        } else if self.neutral {
            Theme::light()
        } else if self.dark {
            Theme::mint_dark()
        } else {
            Theme::mint_light()
        };
        #[cfg(target_os = "android")]
        {
            theme = theme.touch();
        }
        if self.high_contrast {
            theme = theme.high_contrast();
        }
        if self.applied_theme != Some((self.dark, self.high_contrast, self.neutral)) {
            theme.apply(ui.ctx());
            self.applied_theme = Some((self.dark, self.high_contrast, self.neutral));
        }
        if self.applied_reduced_motion != Some(self.reduced_motion) {
            let animation_time = if self.reduced_motion { 0.0 } else { 0.1 };
            ui.ctx()
                .all_styles_mut(|style| style.animation_time = animation_time);
            self.applied_reduced_motion = Some(self.reduced_motion);
        }
        match self.state.surface_motion {
            temivu::liquid::SurfaceTransition::Bloom => temivu::liquid::LiquidStyle::bloom(),
            temivu::liquid::SurfaceTransition::Droplet => temivu::liquid::LiquidStyle::droplet(),
        }
        .set_context_default(ui.ctx());
        // The root Ui exists before this frame applies a changed theme.
        ui.set_style(ui.ctx().global_style());
        if self.motion_preview {
            let now = ui.input(|input| input.time);
            let start = *self.preview_started.get_or_insert(now);
            let elapsed = now - start;
            self.state
                .date_field
                .set_open(elapsed < 1.0 || (2.0..3.0).contains(&elapsed));
            if elapsed >= 4.0 {
                self.motion_preview = false;
            } else {
                ui.ctx().request_repaint();
            }
        }
        let current = Page::from_id(*self.destination.get_or_insert(Page::Overview.id()));
        let video_visible = current == Page::Controls
            && self.state.show_motion_gallery
            && widget_catalog::media_visible(&self.state);
        self.state.video.visible(video_visible, ui.ctx());
        if video_visible && ui.input(|input| input.viewport().fullscreen.unwrap_or(false)) {
            self.state.video.show(ui, &theme);
            return;
        }
        let mut menu_choice = None;
        let mut open_gallery = None;
        let mut gallery_action = None;
        let mut header_back = false;
        let can_go_back = self.gallery_return.is_some();
        let keyboard_back = can_go_back
            && current == Page::Controls
            && self.state.show_motion_gallery
            && !ui.ctx().any_popup_open()
            && ui.input_mut(|input| input.consume_key(egui::Modifiers::NONE, egui::Key::Escape));
        let mut menu_selected = self.destination;
        let items = Page::ALL.map(|page| {
            let icon: temivu::bottom_navigation::NavigationIcon = match page {
                Page::Overview => nav_icon,
                Page::Controls => controls_icon,
                Page::Overlays => overlays_icon,
                Page::Data => data_icon,
                Page::Localization => language_icon,
            };
            NavigationItem::new(page.id(), page.label()).icon(icon)
        });
        Scaffold::new(Id::new("demo-scaffold"), &theme)
            .header(true)
            .show(
                ui,
                |_ui| {},
                |ui| {
                    egui::Frame::new()
                        .inner_margin(egui::Margin::symmetric(12, 10))
                        .show(ui, |ui| {
                            ui.horizontal(|ui| {
                                let navigation = ui.add_sized(
                                    [44.0, 44.0],
                                    egui::Button::new("")
                                        .fill(theme.card)
                                        .stroke(egui::Stroke::new(1.0, theme.border))
                                        .corner_radius(255),
                                );
                                let center = navigation.rect.center();
                                let navigation_label = if can_go_back {
                                    let stroke = egui::Stroke::new(1.8, theme.foreground);
                                    let tip = center - egui::vec2(4.0, 0.0);
                                    for y in [-6.0, 6.0] {
                                        ui.painter().line_segment(
                                            [tip, center + egui::vec2(2.0, y)],
                                            stroke,
                                        );
                                    }
                                    header_back = navigation.clicked();
                                    "Back"
                                } else {
                                    controls_icon(
                                        ui.painter(),
                                        egui::Rect::from_center_size(
                                            center,
                                            egui::Vec2::splat(20.0),
                                        ),
                                        theme.foreground,
                                    );
                                    "Menu"
                                };
                                ui.ctx().accesskit_node_builder(navigation.id, |node| {
                                    node.set_label(navigation_label);
                                });
                                ui.label(
                                    egui::RichText::new("Temivu")
                                        .size(22.0)
                                        .strong()
                                        .color(theme.foreground),
                                );
                                if !can_go_back
                                    && let Some(destination) = NavigationMenu::new(
                                        Id::new("demo-full-menu"),
                                        "Temivu",
                                        &mut self.menu_open,
                                        &theme,
                                    )
                                    .presentation(MenuPresentation::Drawer)
                                    .show(&navigation, &items, &mut menu_selected)
                                {
                                    menu_choice = Some(destination);
                                }
                                ui.with_layout(
                                    egui::Layout::right_to_left(egui::Align::Center),
                                    |ui| {
                                        let island = Button::new("", &theme)
                                            .variant(ButtonVariant::Outline)
                                            .show(ui)
                                            .on_hover_text("Quick menu");
                                        for x in [-5.0, 0.0, 5.0] {
                                            ui.painter().circle_filled(
                                                island.rect.center() + egui::vec2(x, 0.0),
                                                1.5,
                                                theme.foreground,
                                            );
                                        }
                                        ui.ctx().accesskit_node_builder(island.id, |node| {
                                            node.set_label("Quick menu");
                                        });
                                        if let Some(destination) = NavigationMenu::new(
                                            Id::new("demo-island-menu"),
                                            "Quick navigation",
                                            &mut self.island_open,
                                            &theme,
                                        )
                                        .presentation(MenuPresentation::Floating)
                                        .show(&island, &items, &mut menu_selected)
                                        {
                                            menu_choice = Some(destination);
                                        }
                                        let widgets = Button::new("Widgets", &theme)
                                            .variant(ButtonVariant::Outline)
                                            .show(ui);
                                        if widgets.clicked() {
                                            open_gallery = Some(widgets);
                                        }
                                    },
                                );
                            });
                        });
                },
                |_ui| {},
                |ui| {
                    self.page_motion.show(ui, &theme, |ui| {
                        egui::ScrollArea::vertical()
                            .id_salt(
                                current.id().with(
                                    current == Page::Controls && self.state.show_motion_gallery,
                                ),
                            )
                            .show(ui, |ui| {
                                let width = ui.available_width().min(1040.0);
                                ui.horizontal(|ui| {
                                    ui.add_space(((ui.available_width() - width) / 2.0).max(0.0));
                                    ui.allocate_ui_with_layout(
                                        egui::vec2(width, 0.0),
                                        egui::Layout::top_down(egui::Align::Min),
                                        |ui| match current {
                                            Page::Overview => overview(
                                                OverviewOptions {
                                                    dark: &mut self.dark,
                                                    neutral: &mut self.neutral,
                                                    progress: &mut self.progress,
                                                    upload_started: &mut self.upload_started,
                                                    high_contrast: &mut self.high_contrast,
                                                    reduced_motion: &mut self.reduced_motion,
                                                },
                                                self.locale,
                                                ui,
                                                &theme,
                                            ),
                                            Page::Controls => {
                                                gallery_action = controls(
                                                    &mut self.state,
                                                    ui,
                                                    &theme,
                                                    can_go_back,
                                                );
                                            }
                                            Page::Overlays => overlays(&mut self.state, ui, &theme),
                                            Page::Data => data(&mut self.state, ui, &theme),
                                            Page::Localization => localization(
                                                &mut self.state,
                                                &mut self.locale,
                                                ui,
                                                &theme,
                                            ),
                                        },
                                    );
                                });
                                ui.add_space(FloatingNavigation::BOTTOM_CLEARANCE);
                            });
                    });
                },
            );
        FloatingNavigation::new(Id::new("demo-navigation"), &theme)
            .select_on_drag(true)
            .show(ui, &items, &mut self.destination);
        if self.applied_theme != Some((self.dark, self.high_contrast, self.neutral)) {
            ui.ctx().request_repaint();
        }
        let open_gallery = open_gallery.or(match &gallery_action {
            Some(GalleryAction::Open(anchor)) => Some(anchor.clone()),
            _ => None,
        });
        if let Some(anchor) = open_gallery {
            if current != Page::Controls || !self.state.show_motion_gallery {
                self.page_motion.open_from(&anchor, &theme);
                self.gallery_return = Some(current);
                self.state.show_motion_gallery = true;
            }
            self.destination = Some(Page::Controls.id());
            ui.ctx().request_repaint();
        } else if header_back
            || keyboard_back
            || matches!(gallery_action, Some(GalleryAction::Close))
        {
            if let Some(previous) = self.gallery_return.take() {
                self.page_motion.close(ui.ctx(), &theme);
                self.destination = Some(previous.id());
            }
            self.state.show_motion_gallery = false;
            ui.ctx().request_repaint();
        } else {
            if let Some(destination) = menu_choice {
                self.destination = Some(destination);
            }
            if let Some(next) = self.destination.map(Page::from_id)
                && next != current
            {
                self.page_motion
                    .slide(ui.ctx(), (next as u8) > (current as u8), &theme);
                if self.gallery_return.take().is_some() {
                    self.state.show_motion_gallery = false;
                }
                ui.ctx().request_repaint();
            }
        }
    }
}

#[cfg(not(target_os = "android"))]
fn main() -> eframe::Result {
    let arguments: Vec<_> = std::env::args().skip(1).collect();
    let mobile = arguments.iter().any(|argument| argument == "--mobile");
    let droplet = arguments.iter().any(|argument| argument == "--droplet");
    let vsync = !arguments.iter().any(|argument| argument == "--no-vsync");
    let motion_preview = arguments
        .iter()
        .any(|argument| argument == "--motion-preview");
    let preview_date = motion_preview || arguments.iter().any(|argument| argument == "--calendar");
    let video_path = arguments
        .windows(2)
        .find(|pair| pair[0] == "--video")
        .map(|pair| pair[1].clone());
    let show_motion_gallery = video_path.is_some()
        || arguments
            .iter()
            .any(|argument| argument == "--motion-gallery");
    let controls = preview_date
        || show_motion_gallery
        || arguments.iter().any(|argument| argument == "--controls");
    let preview_dialog = arguments.iter().any(|argument| argument == "--dialog");
    let overlays = preview_dialog || arguments.iter().any(|argument| argument == "--overlays");
    let data = arguments.iter().any(|argument| argument == "--data");
    let dark = arguments.iter().any(|argument| argument == "--dark");
    let menu_open = arguments.iter().any(|argument| argument == "--menu");
    let island_open = arguments.iter().any(|argument| argument == "--island");
    let size = if mobile {
        [390.0, 720.0]
    } else {
        [1100.0, 720.0]
    };
    let title = if preview_date {
        "Temivu — calendar preview"
    } else if menu_open {
        "Temivu — drawer preview"
    } else if island_open {
        "Temivu — island menu preview"
    } else {
        "Temivu showcase"
    };
    eframe::run_native(
        title,
        eframe::NativeOptions {
            glow_options: eframe::egui_glow::GlowConfiguration {
                vsync,
                ..Default::default()
            },
            viewport: egui::ViewportBuilder::default()
                .with_inner_size(size)
                .with_min_inner_size([360.0, 480.0]),
            ..Default::default()
        },
        Box::new(move |_| {
            Ok(Box::new(Showcase {
                dark,
                menu_open,
                island_open,
                motion_preview,
                state: DemoState {
                    preview_date,
                    dialog_open: preview_dialog,
                    show_motion_gallery,
                    catalog_group: if video_path.is_some() {
                        widget_catalog::Category::Media
                    } else {
                        widget_catalog::Category::default()
                    },
                    video: video_demo::DemoVideo::with_path(video_path),
                    surface_motion: if droplet {
                        temivu::liquid::SurfaceTransition::Droplet
                    } else {
                        temivu::liquid::SurfaceTransition::Bloom
                    },
                    ..Default::default()
                },
                destination: Some(
                    if controls {
                        Page::Controls
                    } else if overlays {
                        Page::Overlays
                    } else if data {
                        Page::Data
                    } else {
                        Page::Overview
                    }
                    .id(),
                ),
                ..Default::default()
            }))
        }),
    )
}
