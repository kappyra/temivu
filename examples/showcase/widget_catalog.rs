//! One registry of sections and aliases, backed by the same demos as the main pages.
use super::{
    DemoState, control_choices, control_details, control_project, control_schedule, control_values,
    data_calendar, data_collection, data_date_strip, data_datetime, data_explorer, data_pagination,
    data_time, overlay_actions, overlay_dialog, overlay_notifications, overlay_sheet,
    show_toast_stack, studio_panel,
};
use egui::{Id, Ui};
use temivu::{
    Alert, Avatar, Badge, Breadcrumb, BreadcrumbItem, Button, ButtonVariant, Card, Checkbox,
    CircularProgress, Divider, DividerAxis, FieldLabel, Header, LabelLayout, Motion,
    NavigationItem, Progress, TextField, Theme, bottom_navigation,
};

#[derive(Clone, Copy, Default, PartialEq, Eq)]
pub enum Category {
    #[default]
    Motion,
    Media,
    Inputs,
    Overlays,
    Data,
    Foundations,
}

impl Category {
    const ALL: [(Self, &'static str); 6] = [
        (Self::Motion, "Motion"),
        (Self::Media, "Media"),
        (Self::Inputs, "Inputs"),
        (Self::Overlays, "Surfaces"),
        (Self::Data, "Records"),
        (Self::Foundations, "Foundations"),
    ];
}

type Draw = fn(&mut DemoState, &mut Ui, &Theme);
type Entry = (Category, &'static str, Draw);
use Category::*;

const VIDEO_NAMES: &str = "video player playback media volume sound audio mute fullscreen";
const EXAMPLES: &[Entry] = &[
    (Media, VIDEO_NAMES, video),
    (
        Inputs,
        "button text field checkbox switch tooltip name email",
        control_project,
    ),
    (
        Inputs,
        "radio option volume slider range price",
        control_values,
    ),
    (
        Inputs,
        "otp field calendar date field time field",
        control_schedule,
    ),
    (
        Inputs,
        "select group multi select autocomplete picker choices",
        control_choices,
    ),
    (
        Inputs,
        "tabs segments accordion finer details",
        control_details,
    ),
    (Overlays, "dialog modal confirm", overlay_dialog),
    (Overlays, "sheet filter", overlay_sheet),
    (
        Overlays,
        "popover menu context menu item select menu tile anchored actions",
        overlay_actions,
    ),
    (Overlays, "toast toaster notification", notifications),
    (
        Data,
        "item tile select tile group collection",
        data_collection,
    ),
    (Data, "pagination paged records", data_pagination),
    (Data, "calendar date month year", data_calendar),
    (Data, "line calendar date strip", data_date_strip),
    (Data, "sidebar resizable split explorer", data_explorer),
    (Data, "time picker", data_time),
    (Data, "date time picker schedule", data_datetime),
    (
        Foundations,
        "card header divider breadcrumb avatar badge alert field label",
        foundations,
    ),
    (Foundations, "progress circular spinner loading", indicators),
    (
        Foundations,
        "bottom navigation floating navigation scaffold navigation menu drawer island",
        navigation,
    ),
];

fn matches(query: &str, names: &str) -> bool {
    query.split_whitespace().all(|term| names.contains(term))
}

pub fn media_visible(state: &DemoState) -> bool {
    let query = state.catalog_query.trim();
    if query.is_empty() {
        state.catalog_group == Media
    } else {
        matches(&query.to_lowercase(), VIDEO_NAMES)
    }
}

pub fn show(state: &mut DemoState, ui: &mut Ui, theme: &Theme) {
    ui.scope(|ui| {
        ui.set_max_width(ui.available_width().min(420.0));
        TextField::new(&mut state.catalog_query, theme, "Find a widget")
            .hint("Video, calendar, toast, direction pad…")
            .max_chars(64)
            .show(ui);
    });
    let width = ui.available_width();
    let columns = if width >= 680.0 {
        6
    } else if width >= 312.0 {
        3
    } else {
        2
    };
    for row in Category::ALL.chunks(columns) {
        ui.columns(columns, |cells| {
            for (cell, &(group, label)) in cells.iter_mut().zip(row) {
                if Button::new(label, theme)
                    .variant(
                        if state.catalog_query.trim().is_empty() && group == state.catalog_group {
                            ButtonVariant::Primary
                        } else {
                            ButtonVariant::Outline
                        },
                    )
                    .show(cell)
                    .clicked()
                {
                    state.catalog_group = group;
                    state.catalog_query.clear();
                    cell.ctx().request_repaint();
                }
            }
        });
    }
    ui.add_space(10.0);
    let query = state.catalog_query.trim().to_lowercase();
    let mut found = false;
    if state.catalog_group == Motion && query.is_empty() {
        super::motion_controls(state, ui, theme, false);
        ui.add_space(6.0);
    }
    if state.catalog_group == Motion || !query.is_empty() {
        found = state.motion_gallery.show(ui, theme, &query);
    }
    for &(group, names, draw) in EXAMPLES {
        let show = if query.is_empty() {
            group == state.catalog_group
        } else {
            matches(&query, names)
        };
        if show {
            found = true;
            ui.push_id(names, |ui| draw(state, ui, theme));
            ui.add_space(12.0);
        }
    }
    if !found {
        ui.label("No widgets match your search.");
    }
}

fn video(state: &mut DemoState, ui: &mut Ui, theme: &Theme) {
    studio_panel(
        ui,
        theme,
        "Video player",
        "Play a local video with sound, seek, and change the playback speed.",
        |ui| {
            state.video.show(ui, theme);
        },
    );
}

fn notifications(state: &mut DemoState, ui: &mut Ui, theme: &Theme) {
    overlay_notifications(state, ui, theme);
    show_toast_stack(state, ui, theme);
}

fn foundations(state: &mut DemoState, ui: &mut Ui, theme: &Theme) {
    studio_panel(
        ui,
        theme,
        "Surfaces and typography",
        "Compose layout, navigation cues and labeled inputs.",
        |ui| {
            Header::new("Workspace", theme).show(ui, |ui| {
                Avatar::new(theme).label("Profile").fallback("TV").show(ui);
            });
            Card::new(theme)
                .inner_padding(egui::Margin::same(16))
                .show(ui, |ui, style| {
                    ui.label(
                        egui::RichText::new("Workspace card")
                            .color(style.title_color)
                            .strong(),
                    );
                    let crumbs = if state.breadcrumb_root {
                        vec![BreadcrumbItem::current("Workspace")]
                    } else {
                        vec![
                            BreadcrumbItem::link("Workspace"),
                            BreadcrumbItem::current("Components"),
                        ]
                    };
                    if Breadcrumb::new(&crumbs, theme).show(ui).is_some() {
                        state.breadcrumb_root = true;
                    }
                    if state.breadcrumb_root
                        && Button::new("Open components", theme).show(ui).clicked()
                    {
                        state.breadcrumb_root = false;
                    }
                    Divider::new(theme, DividerAxis::Horizontal).show(ui);
                    Badge::new("PREVIEW", theme).show(ui);
                    ui.add_space(8.0);
                    FieldLabel::new(theme, LabelLayout::Vertical)
                        .label("Workspace visibility")
                        .description("This preference is used only in the demo.")
                        .show(ui, |ui| {
                            Checkbox::new(&mut state.accepted, theme, "Share preview").show(ui)
                        });
                });
            ui.add_space(8.0);
            Alert::new("Workspace ready", theme)
                .subtitle("Your components are available in this catalog.")
                .show(ui);
        },
    );
}

fn indicators(_: &mut DemoState, ui: &mut Ui, theme: &Theme) {
    studio_panel(
        ui,
        theme,
        "Progress indicators",
        "Compare a fixed value with an indeterminate indicator.",
        |ui| {
            Progress::determinate(theme, 0.64)
                .expect("fixed preview value")
                .label("Sample progress")
                .show(ui);
            ui.add_space(8.0);
            CircularProgress::new(theme)
                .label("Indeterminate progress")
                .motion(Motion::Disabled)
                .show(ui);
        },
    );
}

fn navigation(state: &mut DemoState, ui: &mut Ui, theme: &Theme) {
    studio_panel(
        ui,
        theme,
        "Navigation and scaffold",
        "The app frame demonstrates Scaffold, FloatingNavigation and both header menus.",
        |ui| {
            let items = [
                NavigationItem::new(Id::new("catalog-nav-home"), "Home"),
                NavigationItem::new(Id::new("catalog-nav-library"), "Library"),
            ];
            bottom_navigation(
                ui,
                Id::new("catalog-nav"),
                &items,
                &mut state.catalog_navigation,
                theme,
            );
        },
    );
}
