//! An experimental showcase of the migrated components; the library is far
//! from complete. Run with `cargo run --example showcase --features demo`.
use egui::Id;
use temivu::tooltip::tooltip;
use temivu::{
    AccordionItem, AccordionState, Alert, AlertStyle, AlertVariant, Autocomplete,
    AutocompleteOption, Avatar, Badge, BadgeStyle, BadgeVariant, Breadcrumb, BreadcrumbItem,
    Breakpoints, Button, ButtonVariant, Calendar, CalendarState, Card, Checkbox, CircularProgress,
    Date, DateField, DateFieldState, DateTime, DateTimePicker, DateTimePickerState, Dialog,
    Divider, DividerAxis, FieldLabel, Header, Item, ItemVariant, LabelLayout, LineCalendar,
    LineCalendarState, Messages, MultiSelect, NavigationItem, OtpField, PaginationState, Picker,
    PickerItem, Popover, PopoverMenu, Progress, Radio, RangeSlider, Resizable, ResizableAxis,
    Scaffold, Select, SelectGroup, SelectMenuTile, SelectOption, SelectTile, SelectTileGroup,
    Sheet, Sidebar, SidebarGroup, SidebarItem, SidebarState, Slider, SliderMark, Switch, Tabs,
    TextField, Theme, Tile, Time, TimeField, TimeFieldState, TimePicker, Toast, ToastEntry,
    ToastState, ToastVariant, Toaster, ToasterState, ViewportClass, bottom_navigation,
    context_menu, pagination, pagination_localized,
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
    // Overview
    accepted: bool,
    notifications: bool,
    // Controls
    name: String,
    email: String,
    volume: f64,
    price_range: (f64, f64),
    radio_alpha: bool,
    radio_beta: bool,
    tab: usize,
    choice: Option<Id>,
    accordion: Option<AccordionState>,
    // Overlays
    clicks: usize,
    toast: Option<ToastState>,
    toaster: ToasterState,
    next_toast: usize,
    dialog_open: bool,
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
            accepted: false,
            notifications: false,
            name: String::new(),
            email: String::new(),
            volume: 0.0,
            price_range: (0.2, 0.8),
            radio_alpha: true,
            radio_beta: false,
            tab: 0,
            choice: None,
            accordion: None,
            clicks: 0,
            toast: None,
            toaster: ToasterState::default(),
            next_toast: 0,
            dialog_open: false,
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

#[derive(Default)]
pub(crate) struct Showcase {
    dark: bool,
    high_contrast: bool,
    applied_theme: Option<(bool, bool)>,
    reduced_motion: bool,
    applied_reduced_motion: Option<bool>,
    destination: Option<Id>,
    locale: usize,
    state: DemoState,
}

fn overview(
    dark: &mut bool,
    high_contrast: &mut bool,
    reduced_motion: &mut bool,
    locale: usize,
    ui: &mut egui::Ui,
    theme: &Theme,
) {
    ui.heading("Welcome to Temivu");
    ui.label("An experimental Forui-inspired component library for egui.");
    ui.add_space(8.0);
    Card::new(theme).show(ui, |ui, _| {
        ui.label("This demo exercises the preliminary components. None of them is complete.");
    });
    ui.add_space(8.0);
    ui.horizontal(|ui| {
        for (variant, label) in [
            (BadgeVariant::Primary, "Primary"),
            (BadgeVariant::Secondary, "Secondary"),
            (BadgeVariant::Outline, "Outline"),
            (BadgeVariant::Destructive, "Destructive"),
        ] {
            Badge::new(label, theme)
                .style(BadgeStyle::from_theme(theme, variant, false))
                .show(ui);
        }
    });
    ui.add_space(8.0);
    Alert::new("Preview build", theme)
        .subtitle("Some components are still being migrated.")
        .style(AlertStyle::from_theme(theme, AlertVariant::Primary))
        .show(ui);
    ui.add_space(8.0);
    ui.horizontal(|ui| {
        Avatar::new(theme).label("Profile").fallback("TV").show(ui);
        Divider::new(theme, DividerAxis::Vertical).show(ui);
        CircularProgress::new(theme)
            .label(Messages::for_locale(LOCALES[locale].0).progress)
            .show(ui);
        Progress::determinate(theme, 0.6)
            .expect("constant progress is valid")
            .label("Upload progress")
            .show(ui);
    });
    ui.add_space(8.0);
    let crumbs = [
        BreadcrumbItem::link("Settings"),
        BreadcrumbItem::current("Overview"),
    ];
    Breadcrumb::new(&crumbs, theme).show(ui);
    ui.add_space(8.0);
    ui.horizontal(|ui| {
        Switch::new(dark, theme).label("Dark theme").show(ui);
        Switch::new(high_contrast, theme)
            .label("High contrast")
            .show(ui);
        Switch::new(reduced_motion, theme)
            .label("Reduced motion")
            .show(ui);
    });
    let width = ui.available_width();
    let class = Breakpoints::default()
        .classify(width)
        .expect("available width is finite and non-negative");
    ui.label(format!(
        "Available content width {width:.0} points: {}",
        match class {
            ViewportClass::BelowSmall => "below small",
            ViewportClass::Small => "small",
            ViewportClass::Medium => "medium",
            ViewportClass::Large => "large",
            ViewportClass::ExtraLarge => "extra large",
            ViewportClass::ExtraExtraLarge => "extra extra large",
        }
    ));
}

fn controls(state: &mut DemoState, ui: &mut egui::Ui, theme: &Theme) {
    ui.heading("Controls and inputs");
    ui.add_space(8.0);
    ui.horizontal(|ui| {
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
    let hovered = Button::new("Hover me", theme).show(ui);
    tooltip(hovered, "A preliminary tooltip");
    ui.add_space(8.0);
    TextField::new(&mut state.name, theme, "Name")
        .hint("Enter your name")
        .show(ui);
    ui.add_space(4.0);
    let email_error = (!state.email.is_empty() && !state.email.contains('@'))
        .then_some("Enter an address that contains @");
    let mut email_label = FieldLabel::new(theme, LabelLayout::Vertical)
        .label("Email")
        .description("We only use this for notifications.");
    if let Some(error) = email_error {
        email_label = email_label.error(error);
    }
    email_label.show(ui, |ui| {
        ui.add(egui::TextEdit::singleline(&mut state.email).hint_text("name@example.com"))
    });
    ui.add_space(4.0);
    Checkbox::new(&mut state.accepted, theme, "Accept terms").show(ui);
    Switch::new(&mut state.notifications, theme)
        .label("Enable notifications")
        .show(ui);
    let mut alpha_clicked = false;
    let mut beta_clicked = false;
    ui.horizontal(|ui| {
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
    let volume_marks = [
        SliderMark::new(0.0).expect("mark").label("Mute"),
        SliderMark::new(0.5).expect("mark").label("Mid"),
        SliderMark::new(1.0).expect("mark").label("Max"),
    ];
    Slider::new(&mut state.volume, theme)
        .expect("normalized value")
        .label("Volume")
        .marks(&volume_marks)
        .show(ui);
    ui.add_space(8.0);
    RangeSlider::new(&mut state.price_range, theme)
        .expect("normalized range")
        .label("Price range")
        .step(0.05)
        .expect("valid step")
        .show(ui);
    ui.add_space(8.0);
    let otp = OtpField::new(&mut state.otp, theme, "Verification code")
        .length(6)
        .description("A six-digit code")
        .show(ui);
    if otp.completed {
        ui.label("The code is complete.");
    }
    ui.add_space(8.0);
    let _ = DateField::new(&mut state.date_field, theme, "Start date")
        .hint("YYYY-MM-DD")
        .today(demo_date())
        .show(ui);
    let _ = TimeField::new(&mut state.time_field, theme, "Start time").show(ui);
    ui.add_space(8.0);
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
    Tabs::new(&mut state.tab, &["General", "Privacy", "About"], theme)
        .expect("three tabs with an in-range selection")
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
}

fn overlays(state: &mut DemoState, ui: &mut egui::Ui, theme: &Theme) {
    ui.heading("Overlays and dialogs");
    ui.add_space(8.0);
    let click_button = Button::new("Click me", theme).icon("+").show(ui);
    if click_button.clicked() {
        state.clicks += 1;
    }
    context_menu(&click_button, |ui| {
        let reset = ui.button("Reset clicks");
        if reset.clicked() {
            state.clicks = 0;
            ui.close();
        }
        reset
    });
    ui.label(format!("Clicks: {}", state.clicks));
    ui.label("Right-click the button or press Shift+F10 for its context menu.");
    ui.add_space(8.0);
    let anchor = Button::new("Show popover", theme).show(ui);
    if anchor.clicked() {
        state.popover_open = !state.popover_open;
    }
    Popover::new(
        Id::new("demo-popover"),
        "Demo popover",
        &mut state.popover_open,
    )
    .focus_on_open(true)
    .show(&anchor, |ui| {
        ui.label("Popover content");
        let action = ui.button("Popover action");
        if action.clicked() {
            ui.close();
        }
        action
    });
    ui.add_space(8.0);
    let menu_anchor = Button::new("Show menu", theme).show(ui);
    if menu_anchor.clicked() {
        state.popover_menu_open = !state.popover_menu_open;
    }
    PopoverMenu::new(
        Id::new("demo-popover-menu"),
        "Demo menu",
        &mut state.popover_menu_open,
    )
    .show(&menu_anchor, |ui, menu| {
        if menu
            .item(
                ui,
                Item::new("Rename", theme).id(Id::new("demo-menu-rename")),
            )
            .clicked()
        {
            ui.close();
        }
        let _ = menu.item(ui, Item::new("Archive", theme).enabled(false));
        let pinned = state.menu_choice == Some(Id::new("demo-menu-pin"));
        if menu
            .select_menu_tile(
                ui,
                SelectMenuTile::new("Pin to top", theme)
                    .id(Id::new("demo-menu-pin"))
                    .selected(pinned),
            )
            .clicked()
        {
            state.menu_choice = if pinned {
                None
            } else {
                Some(Id::new("demo-menu-pin"))
            };
            ui.close();
        }
        if menu
            .item(
                ui,
                Item::new("Delete", theme).id(Id::new("demo-menu-delete")),
            )
            .clicked()
        {
            ui.close();
        }
    });
    ui.add_space(8.0);
    if Button::new("Open sheet", theme).show(ui).clicked() {
        state.sheet_open = true;
    }
    Sheet::new(Id::new("demo-sheet"), "Filters", theme).show(
        ui.ctx(),
        &mut state.sheet_open,
        |ui| {
            ui.label("Sheet content");
            if Button::new("Apply", theme).show(ui).clicked() {
                ui.close();
            }
        },
    );
    ui.add_space(8.0);
    if Button::new("Open dialog", theme).show(ui).clicked() {
        state.dialog_open = true;
    }
    Dialog::new(Id::new("demo-dialog"), "Confirm action").show(
        ui.ctx(),
        &mut state.dialog_open,
        |ui| {
            ui.heading("Confirm action");
            ui.label("Review the details before continuing.");
            ui.horizontal(|ui| {
                if Button::new("Continue", theme).show(ui).clicked() {
                    ui.close();
                }
                if Button::new("Cancel", theme)
                    .variant(ButtonVariant::Secondary)
                    .show(ui)
                    .clicked()
                {
                    ui.close();
                }
            });
        },
    );
    ui.add_space(8.0);
    if Button::new("Show notification", theme).show(ui).clicked() {
        state.toast = Some(ToastState::default());
    }
    if let Some(toast_state) = &mut state.toast {
        Toast::new(toast_state, theme, "Notification", "Dismiss")
            .description("This notification closes when dismissed.")
            .show(ui);
        if !toast_state.is_visible() {
            state.toast = None;
        }
    }
    ui.add_space(8.0);
    if Button::new("Queue notifications", theme).show(ui).clicked() {
        state.next_toast += 1;
        let index = state.next_toast;
        state.toaster.push(
            ToastEntry::new(
                Id::new(("queued-toast", index)),
                format!("Queued notification {index}"),
            )
            .description("Queued by the toaster overlay")
            .icon(if index.is_multiple_of(2) { "!" } else { "i" })
            .variant(if index.is_multiple_of(2) {
                ToastVariant::Destructive
            } else {
                ToastVariant::Primary
            })
            .timeout(None),
        );
    }
    Toaster::new(&mut state.toaster, theme, "Dismiss").show(ui);
}

fn data(state: &mut DemoState, ui: &mut egui::Ui, theme: &Theme) {
    ui.heading("Data displays");
    ui.add_space(8.0);
    if Item::new("Example item", theme)
        .description("Select this row")
        .selected(state.item_selected)
        .show(ui)
        .clicked()
    {
        state.item_selected = !state.item_selected;
    }
    Item::new("Delete account", theme)
        .variant(ItemVariant::Destructive)
        .show(ui);
    ui.add_space(8.0);
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
    let pagination_state = state
        .pagination
        .get_or_insert_with(|| PaginationState::new(12, 1, 1, true).expect("valid state"));
    pagination(ui, pagination_state, theme, "Previous", "Next");
    ui.add_space(8.0);
    ui.label("Resizable panes");
    ui.allocate_ui(egui::vec2(ui.available_width(), 110.0), |ui| {
        Resizable::new(
            Id::new("demo-split"),
            ResizableAxis::Horizontal,
            &mut state.split_fraction,
            theme,
        )
        .min_extents(120.0, 120.0)
        .show(
            ui,
            |ui| {
                ui.label("Left pane");
            },
            |ui| {
                ui.label("Right pane");
            },
        );
    });
    ui.add_space(8.0);
    let calendar_state = state.calendar.get_or_insert_with(|| {
        CalendarState::new(Date::new(2026, 11, 1).expect("valid month"), None)
    });
    Calendar::new(calendar_state, theme)
        .today(demo_date())
        .show(ui);
    ui.add_space(8.0);
    ui.label("Navigation sidebar");
    let sidebar_width = Sidebar::recommended_width(&state.sidebar).min(ui.available_width());
    ui.allocate_ui(egui::vec2(sidebar_width, 220.0), |ui| {
        let inbox = [
            SidebarItem::new(Id::new("demo-inbox"), "Inbox"),
            SidebarItem::new(Id::new("demo-sent"), "Sent"),
        ];
        let history = [SidebarItem::new(Id::new("demo-archive"), "Archive")];
        let groups = [
            SidebarGroup::new(Id::new("demo-mail"), &inbox).label("Mail"),
            SidebarGroup::new(Id::new("demo-history"), &history).label("History"),
        ];
        let _ = Sidebar::new(
            Id::new("demo-sidebar"),
            "Demo navigation",
            theme,
            &mut state.sidebar,
        )
        .title("Temivu")
        .show(ui, &groups, &mut state.sidebar_selected);
    });
    ui.add_space(8.0);
    let tiles = [
        SelectTile::new(Id::new("demo-wifi"), "Airplane mode").description("Disable radios"),
        SelectTile::new(Id::new("demo-bluetooth"), "Power saving")
            .description("Reduce battery use"),
    ];
    let _ = SelectTileGroup::new(
        Id::new("demo-radios"),
        "Radios",
        theme,
        &tiles,
        &mut state.select_tiles,
    )
    .description("Choose any that apply")
    .show(ui);
    ui.add_space(8.0);
    ui.add_space(8.0);
    let _ = TimePicker::new(&mut state.time_picker_time, theme)
        .id(Id::new("demo-time-picker"))
        .show(ui);
    ui.add_space(8.0);
    let selected_time = state.time_picker_time;
    let december = Date::new(2026, 12, 1).expect("constant date is valid");
    let date_time = state.date_time.get_or_insert_with(|| {
        let selected = Date::new(2026, 12, 20).expect("constant date is valid");
        DateTimePickerState::new(december, Some(DateTime::new(selected, selected_time)))
    });
    let _ = DateTimePicker::new(date_time, theme)
        .today(demo_date())
        .seconds(true)
        .show(ui);
    ui.add_space(8.0);
    let line_calendar = state.line_calendar.get_or_insert_with(|| {
        LineCalendarState::new(
            Date::new(2026, 11, 8).expect("valid date"),
            Some(Date::new(2026, 11, 12).expect("valid date")),
        )
    });
    let _ = LineCalendar::new(line_calendar, theme)
        .today(demo_date())
        .days(10)
        .show(ui);
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
        let mut theme = if self.dark {
            Theme::dark()
        } else {
            Theme::light()
        };
        if self.high_contrast {
            theme = theme.high_contrast();
        }
        if self.applied_theme != Some((self.dark, self.high_contrast)) {
            theme.apply(ui.ctx());
            self.applied_theme = Some((self.dark, self.high_contrast));
        }
        if self.applied_reduced_motion != Some(self.reduced_motion) {
            let animation_time = if self.reduced_motion { 0.0 } else { 0.1 };
            ui.ctx()
                .all_styles_mut(|style| style.animation_time = animation_time);
            self.applied_reduced_motion = Some(self.reduced_motion);
        }
        let current = Page::from_id(*self.destination.get_or_insert(Page::Overview.id()));
        Scaffold::new(Id::new("demo-scaffold"), &theme)
            .header(true)
            .footer(true)
            .show(
                ui,
                |_ui| {},
                |ui| {
                    Header::new("Temivu", &theme).show(ui, |_| {});
                },
                |ui| {
                    let items = Page::ALL.map(|page| NavigationItem::new(page.id(), page.label()));
                    bottom_navigation(
                        ui,
                        Id::new("demo-navigation"),
                        &items,
                        &mut self.destination,
                        &theme,
                    );
                },
                |ui| {
                    egui::ScrollArea::vertical().show(ui, |ui| match current {
                        Page::Overview => overview(
                            &mut self.dark,
                            &mut self.high_contrast,
                            &mut self.reduced_motion,
                            self.locale,
                            ui,
                            &theme,
                        ),
                        Page::Controls => controls(&mut self.state, ui, &theme),
                        Page::Overlays => overlays(&mut self.state, ui, &theme),
                        Page::Data => data(&mut self.state, ui, &theme),
                        Page::Localization => {
                            localization(&mut self.state, &mut self.locale, ui, &theme)
                        }
                    });
                },
            );
    }
}

fn main() -> eframe::Result {
    eframe::run_native(
        "Temivu showcase",
        eframe::NativeOptions::default(),
        Box::new(|_| Ok(Box::<Showcase>::default())),
    )
}
