use egui_kittest::{Harness, kittest::Queryable};
use temivu::{
    Autocomplete, AutocompleteOption, Badge, Breadcrumb, BreadcrumbItem, Button, ButtonVariant,
    Calendar, CalendarState, Checkbox, Date, DateField, DateFieldState, DateTime, DateTimePicker,
    DateTimePickerState, Dialog, Header, Item, ItemVariant, LineCalendar, LineCalendarState,
    MultiSelect, NavigationItem, OtpField, PaginationState, Picker, PickerItem, Popover,
    PopoverMenu, Progress, RangeSlider, Resizable, ResizableAxis, Scaffold, Select, SelectGroup,
    SelectOption, SelectTile, SelectTileGroup, Sheet, Sidebar, SidebarGroup, SidebarItem,
    SidebarState, Slider, SliderMark, Switch, Tabs, Theme, Tile, Time, TimePicker, Toast,
    ToastEntry, ToastState, ToastVariant, Toaster, ToasterState, bottom_navigation, context_menu,
    pagination,
};

/// Output directory for the rendered sheets.
fn output_dir() -> std::path::PathBuf {
    let dir = std::env::var_os("TEMIVU_VISUAL_OUT")
        .map(std::path::PathBuf::from)
        .unwrap_or_else(|| std::path::PathBuf::from("harness/visual-review/out"));
    std::fs::create_dir_all(&dir).expect("create the visual-review output directory");
    dir
}

fn main() {
    for (dark, name, zoom) in [
        (false, "light", 1.0),
        (true, "dark", 1.0),
        (false, "light-150", 1.5),
        (true, "dark-150", 1.5),
    ] {
        let theme = if dark { Theme::dark() } else { Theme::light() };
        let mut harness = Harness::builder()
            .with_size(egui::vec2(600.0, 480.0))
            .build_ui_state(
                move |ui, state: &mut (bool, PaginationState)| {
                    theme.apply(ui.ctx());
                    egui::CentralPanel::default().show(ui, |ui| {
                        Header::new("Temivu visual review", &theme).show(ui, |ui| {
                            Button::new("Settings", &theme)
                                .variant(ButtonVariant::Secondary)
                                .show(ui);
                        });
                        ui.add_space(8.0);
                        let crumbs = [
                            BreadcrumbItem::link("Home"),
                            BreadcrumbItem::current("Library"),
                        ];
                        Breadcrumb::new(&crumbs, &theme).show(ui);
                        ui.add_space(12.0);
                        ui.horizontal(|ui| {
                            Button::new("Primary", &theme).show(ui);
                            Button::new("Secondary", &theme)
                                .variant(ButtonVariant::Secondary)
                                .show(ui);
                            Button::new("Delete", &theme)
                                .variant(ButtonVariant::Destructive)
                                .show(ui);
                        });
                        ui.add_space(12.0);
                        Badge::new("Preview", &theme).show(ui);
                        Checkbox::new(&mut state.0, &theme, "Accept option").show(ui);
                        Progress::determinate(&theme, 0.6)
                            .expect("valid constant")
                            .label("Progress")
                            .show(ui);
                        ui.add_space(12.0);
                        pagination(ui, &mut state.1, &theme, "Previous", "Next");
                    });
                },
                (
                    false,
                    PaginationState::new(12, 4, 1, true).expect("valid constant"),
                ),
            );
        harness.ctx.set_zoom_factor(zoom);
        harness.run();
        let image = harness
            .render()
            .expect("off-screen graphics adapter required");
        image.save(format!("{}/{name}.png", output_dir().display())).expect("write PNG");
    }
    for (dark, name) in [(false, "dialog-light"), (true, "dialog-dark")] {
        let theme = if dark { Theme::dark() } else { Theme::light() };
        let mut harness = Harness::builder()
            .with_size(egui::vec2(600.0, 480.0))
            .build_ui(move |ui| {
                theme.apply(ui.ctx());
                egui::CentralPanel::default().show(ui, |ui| {
                    ui.heading("Background content");
                });
                let mut open = true;
                // Mirror Forui's dialog golden composition: a 16 point padded
                // column with display.md title and body.sm copy on a themed card.
                Dialog::new(egui::Id::new("review-dialog"), "Confirm action")
                    .theme(&theme)
                    .show(ui.ctx(), &mut open, |ui| {
                        egui::Frame::new()
                            .inner_margin(egui::Margin::same(16))
                            .show(ui, |ui| {
                                ui.label(
                                    egui::RichText::new("Are you absolutely sure?")
                                        .size(if theme.touch { 18.0 } else { 16.0 })
                                        .color(theme.foreground),
                                );
                                ui.add_space(8.0);
                                ui.label(
                                    egui::RichText::new("This action cannot be undone.")
                                        .size(theme.typography.body.size)
                                        .color(theme.muted_foreground),
                                );
                            });
                    });
            });
        harness.run();
        harness
            .render()
            .expect("off-screen graphics adapter required")
            .save(format!("{}/{name}.png", output_dir().display()))
            .expect("write PNG");
    }
    for (dark, name) in [(false, "switch-light-150"), (true, "switch-dark-150")] {
        let theme = if dark { Theme::dark() } else { Theme::light() };
        let mut harness = Harness::builder()
            .with_size(egui::vec2(300.0, 450.0))
            .build_ui_state(
                move |ui, values: &mut [bool; 2]| {
                    theme.apply(ui.ctx());
                    egui::CentralPanel::default().show(ui, |ui| {
                        for (leading, value) in [false, true].into_iter().zip(values) {
                            Switch::new(value, &theme)
                                .label("Automatically synchronize every update")
                                .description("Updates this device when connected")
                                .leading_label(leading)
                                .show(ui);
                            ui.add_space(12.0);
                        }
                    });
                },
                [false, true],
            );
        harness.ctx.set_zoom_factor(1.5);
        harness.run();
        harness
            .render()
            .expect("off-screen graphics adapter required")
            .save(format!("{}/{name}.png", output_dir().display()))
            .expect("write PNG");
    }
    for (dark, name) in [(false, "toast-light"), (true, "toast-dark")] {
        let theme = if dark { Theme::dark() } else { Theme::light() };
        let mut harness = Harness::builder()
            .with_size(egui::vec2(480.0, 320.0))
            .build_ui(move |ui| {
                theme.apply(ui.ctx());
                egui::CentralPanel::default().show(ui, |ui| {
                    let mut state = ToastState::new(None);
                    Toast::new(&mut state, &theme, "Changes saved", "Dismiss")
                        .description("Your changes are available on this device.")
                        .show(ui);
                });
            });
        harness.run();
        harness
            .render()
            .expect("off-screen graphics adapter required")
            .save(format!("{}/{name}.png", output_dir().display()))
            .expect("write PNG");
    }
    for (dark, name, zoom, size) in [
        (false, "scaffold-light", 1.0, egui::vec2(480.0, 320.0)),
        (true, "scaffold-dark", 1.0, egui::vec2(480.0, 320.0)),
        (false, "scaffold-light-150", 1.5, egui::vec2(300.0, 450.0)),
        (true, "scaffold-dark-150", 1.5, egui::vec2(300.0, 450.0)),
    ] {
        let theme = if dark { Theme::dark() } else { Theme::light() };
        let mut harness = Harness::builder().with_size(size).build_ui(move |ui| {
            theme.apply(ui.ctx());
            Scaffold::new(egui::Id::new("review-scaffold"), &theme)
                .sidebar(120.0)
                .unwrap()
                .header(true)
                .footer(true)
                .show(
                    ui,
                    |ui| {
                        ui.label("Navigation");
                    },
                    |ui| {
                        Header::new("Page title", &theme).show(ui, |_| {});
                    },
                    |ui| {
                        ui.label("Status");
                    },
                    |ui| {
                        ui.label("Main content");
                    },
                );
        });
        harness.ctx.set_zoom_factor(zoom);
        harness.run();
        harness
            .render()
            .expect("off-screen graphics adapter required")
            .save(format!("{}/{name}.png", output_dir().display()))
            .expect("write PNG");
    }
    for (dark, name, zoom, size) in [
        (false, "item-light", 1.0, egui::vec2(200.0, 300.0)),
        (true, "item-dark", 1.0, egui::vec2(200.0, 300.0)),
        (false, "item-light-150", 1.5, egui::vec2(300.0, 450.0)),
        (true, "item-dark-150", 1.5, egui::vec2(300.0, 450.0)),
    ] {
        let theme = if dark { Theme::dark() } else { Theme::light() };
        let mut harness = Harness::builder().with_size(size).build_ui(move |ui| {
            theme.apply(ui.ctx());
            egui::CentralPanel::default().show(ui, |ui| {
                Item::new("Account settings on this device", &theme)
                    .description("Manage your profile and preferences")
                    .selected(true)
                    .show(ui);
                Item::new("Delete account", &theme)
                    .variant(ItemVariant::Destructive)
                    .show(ui);
            });
        });
        harness.ctx.set_zoom_factor(zoom);
        harness.run();
        harness
            .render()
            .expect("off-screen graphics adapter required")
            .save(format!("{}/{name}.png", output_dir().display()))
            .expect("write PNG");
    }
    for (dark, name) in [(false, "touch-light"), (true, "touch-dark")] {
        let theme = if dark { Theme::dark() } else { Theme::light() }.touch();
        let mut harness = Harness::builder()
            .with_size(egui::vec2(600.0, 640.0))
            .build_ui_state(
                move |ui, state: &mut (bool, bool)| {
                    theme.apply(ui.ctx());
                    egui::CentralPanel::default().show(ui, |ui| {
                        Item::new("Account settings", &theme)
                            .description("Manage your profile")
                            .selected(true)
                            .show(ui);
                        Item::new("Delete account", &theme)
                            .variant(ItemVariant::Destructive)
                            .show(ui);
                        ui.add_space(8.0);
                        Tile::new("Bluetooth", &theme)
                            .description("Connect devices")
                            .selected(true)
                            .show(ui);
                        ui.add_space(8.0);
                        egui::Frame::new()
                            .inner_margin(egui::Margin::same(16))
                            .show(ui, |ui| {
                                Button::new("Primary", &theme).show(ui);
                                ui.add_space(8.0);
                                Checkbox::new(&mut state.0, &theme, "Accept option")
                                    .touch(true)
                                    .show(ui);
                                ui.add_space(8.0);
                                Switch::new(&mut state.1, &theme)
                                    .label("Airplane Mode")
                                    .description("Disable wireless connections")
                                    .show(ui);
                            });
                    });
                },
                (false, true),
            );
        harness.run();
        harness
            .render()
            .expect("off-screen graphics adapter required")
            .save(format!("{}/{name}.png", output_dir().display()))
            .expect("write PNG");
    }
    for (dark, name, zoom, size) in [
        (false, "tile-light", 1.0, egui::vec2(480.0, 220.0)),
        (true, "tile-dark", 1.0, egui::vec2(480.0, 220.0)),
        (false, "tile-light-150", 1.5, egui::vec2(300.0, 300.0)),
        (true, "tile-dark-150", 1.5, egui::vec2(300.0, 300.0)),
    ] {
        let theme = if dark { Theme::dark() } else { Theme::light() };
        let mut harness = Harness::builder().with_size(size).build_ui(move |ui| {
            theme.apply(ui.ctx());
            egui::CentralPanel::default().show(ui, |ui| {
                Tile::new("Bluetooth settings", &theme)
                    .description("Connect and manage nearby devices")
                    .selected(true)
                    .show(ui);
                Tile::new("Remove device", &theme)
                    .variant(ItemVariant::Destructive)
                    .show(ui);
            });
        });
        harness.ctx.set_zoom_factor(zoom);
        harness.run();
        harness
            .render()
            .expect("off-screen graphics adapter required")
            .save(format!("{}/{name}.png", output_dir().display()))
            .expect("write PNG");
    }
    for (dark, name, zoom, size) in [
        (false, "nav-light", 1.0, egui::vec2(480.0, 160.0)),
        (true, "nav-dark", 1.0, egui::vec2(480.0, 160.0)),
        (false, "nav-light-150", 1.5, egui::vec2(300.0, 180.0)),
        (true, "nav-dark-150", 1.5, egui::vec2(300.0, 180.0)),
    ] {
        let theme = if dark { Theme::dark() } else { Theme::light() };
        let mut harness = Harness::builder().with_size(size).build_ui_state(
            move |ui, selected: &mut Option<egui::Id>| {
                theme.apply(ui.ctx());
                egui::CentralPanel::default().show(ui, |ui| {
                    let items = [
                        NavigationItem::new(egui::Id::new("home"), "Home"),
                        NavigationItem::new(egui::Id::new("search"), "Search"),
                        NavigationItem::new(egui::Id::new("settings"), "Settings"),
                    ];
                    bottom_navigation(ui, egui::Id::new("review-nav"), &items, selected, &theme);
                });
            },
            Some(egui::Id::new("home")),
        );
        harness.ctx.set_zoom_factor(zoom);
        harness.run();
        harness
            .render()
            .expect("off-screen graphics adapter required")
            .save(format!("{}/{name}.png", output_dir().display()))
            .expect("write PNG");
    }
    for (dark, name, zoom) in [
        (false, "select-light", 1.0),
        (true, "select-dark", 1.0),
        (false, "select-light-150", 1.5),
        (true, "select-dark-150", 1.5),
    ] {
        let theme = if dark { Theme::dark() } else { Theme::light() };
        let mut harness = Harness::builder()
            .with_size(egui::vec2(300.0, 300.0))
            .build_ui_state(
                move |ui, selected: &mut Option<egui::Id>| {
                    theme.apply(ui.ctx());
                    egui::CentralPanel::default().show(ui, |ui| {
                        let options = [
                            SelectOption::new(egui::Id::new("red"), "Red"),
                            SelectOption::new(egui::Id::new("blue"), "Blue"),
                            SelectOption::new(
                                egui::Id::new("long"),
                                "A longer available selection",
                            ),
                        ];
                        Select::new(
                            egui::Id::new("review-select"),
                            "Color",
                            "Choose a color",
                            &options,
                            selected,
                        )
                        .show(ui);
                    });
                },
                None,
            );
        harness.ctx.set_zoom_factor(zoom);
        harness.run();
        harness
            .get_by_role_and_label(egui::accesskit::Role::ComboBox, "Color")
            .click();
        harness.run();
        harness.remove_cursor();
        harness.run();
        harness
            .render()
            .expect("off-screen graphics adapter required")
            .save(format!("{}/{name}.png", output_dir().display()))
            .expect("write PNG");
    }
    for (dark, name, zoom) in [
        (false, "context-light", 1.0),
        (true, "context-dark", 1.0),
        (false, "context-light-150", 1.5),
        (true, "context-dark-150", 1.5),
    ] {
        let theme = if dark { Theme::dark() } else { Theme::light() };
        let mut harness = Harness::builder()
            .with_size(egui::vec2(300.0, 220.0))
            .build_ui(move |ui| {
                theme.apply(ui.ctx());
                egui::CentralPanel::default().show(ui, |ui| {
                    let anchor = ui.button("Options");
                    context_menu(&anchor, |ui| {
                        let first = ui.button("Copy");
                        let _ = ui.button("Paste");
                        first
                    });
                });
            });
        harness.ctx.set_zoom_factor(zoom);
        harness.run();
        harness.get_by_label("Options").focus();
        harness.run();
        harness.key_press_modifiers(egui::Modifiers::SHIFT, egui::Key::F10);
        harness.run();
        harness.remove_cursor();
        harness.run();
        harness
            .render()
            .expect("off-screen graphics adapter required")
            .save(format!("{}/{name}.png", output_dir().display()))
            .expect("write PNG");
    }
    for (dark, name, zoom) in [
        (false, "popover-light", 1.0),
        (true, "popover-dark", 1.0),
        (false, "popover-light-150", 1.5),
        (true, "popover-dark-150", 1.5),
    ] {
        let theme = if dark { Theme::dark() } else { Theme::light() };
        let mut harness = Harness::builder()
            .with_size(egui::vec2(300.0, 220.0))
            .build_ui_state(
                move |ui, open: &mut bool| {
                    theme.apply(ui.ctx());
                    egui::CentralPanel::default().show(ui, |ui| {
                        let anchor = ui.button("Options");
                        let _ =
                            Popover::new(egui::Id::new("review-popover"), "Options panel", open)
                                .show(&anchor, |ui| {
                                    ui.label("Choose an option");
                                    ui.button("Apply")
                                });
                    });
                },
                true,
            );
        harness.ctx.set_zoom_factor(zoom);
        harness.run();
        harness
            .render()
            .expect("off-screen graphics adapter required")
            .save(format!("{}/{name}.png", output_dir().display()))
            .expect("write PNG");
    }

    for (dark, name) in [(false, "calendar-light"), (true, "calendar-dark")] {
        let theme = if dark { Theme::dark() } else { Theme::light() };
        let mut harness = Harness::builder()
            .with_size(egui::vec2(320.0, 340.0))
            .build_ui_state(
                move |ui, state: &mut CalendarState| {
                    theme.apply(ui.ctx());
                    egui::CentralPanel::default().show(ui, |ui| {
                        Calendar::new(state, &theme)
                            .today(Date::new(2026, 11, 1).expect("valid date"))
                            .show(ui);
                    });
                },
                CalendarState::new(
                    Date::new(2026, 11, 1).expect("valid month"),
                    Some(Date::new(2026, 11, 15).expect("valid date")),
                ),
            );
        harness.run();
        harness
            .render()
            .expect("off-screen graphics adapter required")
            .save(format!("{}/{name}.png", output_dir().display()))
            .expect("write PNG");
    }
    for (stage, name) in [(1, "calendar-month-picker"), (2, "calendar-year-picker")] {
        for (dark, suffix) in [(false, "light"), (true, "dark")] {
            let theme = if dark { Theme::dark() } else { Theme::light() };
            let mut harness = Harness::builder()
                .with_size(egui::vec2(320.0, 340.0))
                .build_ui_state(
                    move |ui, state: &mut CalendarState| {
                        theme.apply(ui.ctx());
                        egui::CentralPanel::default().show(ui, |ui| {
                            Calendar::new(state, &theme)
                                .today(Date::new(2026, 11, 1).expect("valid date"))
                                .show(ui);
                        });
                    },
                    CalendarState::new(
                        Date::new(2026, 11, 1).expect("valid month"),
                        Some(Date::new(2026, 11, 15).expect("valid date")),
                    ),
                );
            harness.run();
            harness.get_by_label("Choose month").click();
            harness.run();
            if stage == 2 {
                harness.get_by_label("Choose year").click();
                harness.run();
            }
            harness
                .render()
                .expect("off-screen graphics adapter required")
                .save(format!("{}/{name}-{suffix}.png", output_dir().display()))
                .expect("write PNG");
        }
    }
    for (dark, name) in [(false, "otp-light"), (true, "otp-dark")] {
        let theme = if dark { Theme::dark() } else { Theme::light() };
        let mut harness = Harness::builder()
            .with_size(egui::vec2(320.0, 140.0))
            .build_ui_state(
                move |ui, code: &mut String| {
                    theme.apply(ui.ctx());
                    egui::CentralPanel::default().show(ui, |ui| {
                        let _ = OtpField::new(code, &theme, "Verification code")
                            .length(6)
                            .show(ui);
                    });
                },
                String::from("12"),
            );
        harness.run();
        harness
            .render()
            .expect("off-screen graphics adapter required")
            .save(format!("{}/{name}.png", output_dir().display()))
            .expect("write PNG");
    }
    for (dark, name) in [(false, "sheet-light"), (true, "sheet-dark")] {
        let theme = if dark { Theme::dark() } else { Theme::light() };
        let mut harness = Harness::builder()
            .with_size(egui::vec2(480.0, 360.0))
            .build_ui_state(
                move |ui, open: &mut bool| {
                    theme.apply(ui.ctx());
                    egui::CentralPanel::default().show(ui, |ui| {
                        ui.label("Background content");
                    });
                    let _ = Sheet::new(egui::Id::new("review-sheet"), "Filters", &theme).show(
                        ui.ctx(),
                        open,
                        |ui| {
                            ui.label("Choose a filter");
                            ui.horizontal(|ui| {
                                let _ = Button::new("Apply", &theme).show(ui);
                                let _ = Button::new("Reset", &theme)
                                    .variant(ButtonVariant::Secondary)
                                    .show(ui);
                            });
                        },
                    );
                },
                true,
            );
        harness.run();
        harness
            .render()
            .expect("off-screen graphics adapter required")
            .save(format!("{}/{name}.png", output_dir().display()))
            .expect("write PNG");
    }
    for (dark, name) in [(false, "resizable-light"), (true, "resizable-dark")] {
        let theme = if dark { Theme::dark() } else { Theme::light() };
        let mut harness = Harness::builder()
            .with_size(egui::vec2(360.0, 220.0))
            .build_ui_state(
                move |ui, fraction: &mut f32| {
                    theme.apply(ui.ctx());
                    egui::CentralPanel::default().show(ui, |ui| {
                        ui.allocate_ui(egui::vec2(ui.available_width(), 150.0), |ui| {
                            let _ = Resizable::new(
                                egui::Id::new("review-split"),
                                ResizableAxis::Horizontal,
                                fraction,
                                &theme,
                            )
                            .min_extents(60.0, 60.0)
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
                    });
                },
                0.45f32,
            );
        harness.run();
        harness
            .render()
            .expect("off-screen graphics adapter required")
            .save(format!("{}/{name}.png", output_dir().display()))
            .expect("write PNG");
    }
    for (dark, name) in [(false, "menu-light"), (true, "menu-dark")] {
        let theme = if dark { Theme::dark() } else { Theme::light() };
        let mut harness = Harness::builder()
            .with_size(egui::vec2(300.0, 240.0))
            .build_ui_state(
                move |ui, open: &mut bool| {
                    theme.apply(ui.ctx());
                    egui::CentralPanel::default().show(ui, |ui| {
                        let anchor = ui.button("Actions");
                        let _ =
                            PopoverMenu::new(egui::Id::new("review-menu"), "Actions menu", open)
                                .show(&anchor, |ui, menu| {
                                    let _ = menu.item(ui, Item::new("Rename", &theme));
                                    let _ = menu.item(ui, Item::new("Duplicate", &theme));
                                    let _ =
                                        menu.item(ui, Item::new("Delete", &theme).enabled(false));
                                });
                    });
                },
                true,
            );
        harness.run();
        harness
            .render()
            .expect("off-screen graphics adapter required")
            .save(format!("{}/{name}.png", output_dir().display()))
            .expect("write PNG");
    }

    for (dark, name) in [(false, "autocomplete-light"), (true, "autocomplete-dark")] {
        let theme = if dark { Theme::dark() } else { Theme::light() };
        let candidates = [
            AutocompleteOption::new(egui::Id::new("review-apple"), "Apple"),
            AutocompleteOption::new(egui::Id::new("review-apricot"), "Apricot"),
            AutocompleteOption::new(egui::Id::new("review-banana"), "Banana"),
        ];
        let mut harness = Harness::builder()
            .with_size(egui::vec2(320.0, 220.0))
            .build_ui_state(
                move |ui, query: &mut String| {
                    theme.apply(ui.ctx());
                    egui::CentralPanel::default().show(ui, |ui| {
                        let _ = Autocomplete::new(query, &theme, "Fruit", &candidates)
                            .id(egui::Id::new("review-autocomplete"))
                            .show(ui);
                    });
                },
                String::from("Ban"),
            );
        harness.run();
        harness
            .get_by_role_and_label(egui::accesskit::Role::ComboBox, "Fruit")
            .focus();
        harness.run();
        harness
            .render()
            .expect("off-screen graphics adapter required")
            .save(format!("{}/{name}.png", output_dir().display()))
            .expect("write PNG");
    }
    for (dark, name) in [(false, "picker-light"), (true, "picker-dark")] {
        let theme = if dark { Theme::dark() } else { Theme::light() };
        let items = [
            PickerItem::new(egui::Id::new("review-small"), "Small"),
            PickerItem::new(egui::Id::new("review-medium"), "Medium"),
            PickerItem::new(egui::Id::new("review-large"), "Large"),
        ];
        let mut harness = Harness::builder()
            .with_size(egui::vec2(220.0, 240.0))
            .build_ui_state(
                move |ui, selected: &mut Option<egui::Id>| {
                    theme.apply(ui.ctx());
                    egui::CentralPanel::default().show(ui, |ui| {
                        let _ = Picker::new(selected, &items, &theme)
                            .label("Size")
                            .visible_rows(3)
                            .id(egui::Id::new("review-picker"))
                            .show(ui);
                    });
                },
                Some(egui::Id::new("review-medium")),
            );
        harness.run();
        harness
            .render()
            .expect("off-screen graphics adapter required")
            .save(format!("{}/{name}.png", output_dir().display()))
            .expect("write PNG");
    }
    for (dark, name) in [(false, "time-picker-light"), (true, "time-picker-dark")] {
        let theme = if dark { Theme::dark() } else { Theme::light() };
        let mut harness = Harness::builder()
            .with_size(egui::vec2(320.0, 240.0))
            .build_ui_state(
                move |ui, time: &mut Time| {
                    theme.apply(ui.ctx());
                    egui::CentralPanel::default().show(ui, |ui| {
                        let _ = TimePicker::new(time, &theme)
                            .id(egui::Id::new("review-time-picker"))
                            .show(ui);
                    });
                },
                Time::new(9, 30).expect("valid time"),
            );
        harness.run();
        harness
            .render()
            .expect("off-screen graphics adapter required")
            .save(format!("{}/{name}.png", output_dir().display()))
            .expect("write PNG");
    }
    for (dark, name) in [
        (false, "date-time-picker-light"),
        (true, "date-time-picker-dark"),
    ] {
        let theme = if dark { Theme::dark() } else { Theme::light() };
        let month = Date::new(2026, 12, 1).expect("valid date");
        let selected = DateTime::new(
            Date::new(2026, 12, 20).expect("valid date"),
            Time::new(9, 30).expect("valid time"),
        );
        let mut harness = Harness::builder()
            .with_size(egui::vec2(380.0, 460.0))
            .build_ui_state(
                move |ui, state: &mut DateTimePickerState| {
                    theme.apply(ui.ctx());
                    egui::CentralPanel::default().show(ui, |ui| {
                        let _ = DateTimePicker::new(state, &theme)
                            .today(month)
                            .id(egui::Id::new("review-date-time-picker"))
                            .show(ui);
                    });
                },
                DateTimePickerState::new(month, Some(selected)),
            );
        harness.run();
        harness
            .render()
            .expect("off-screen graphics adapter required")
            .save(format!("{}/{name}.png", output_dir().display()))
            .expect("write PNG");
    }
    for (dark, name) in [(false, "date-field-light"), (true, "date-field-dark")] {
        let theme = if dark { Theme::dark() } else { Theme::light() };
        let mut harness = Harness::builder()
            .with_size(egui::vec2(340.0, 200.0))
            .build_ui_state(
                move |ui, state: &mut DateFieldState| {
                    theme.apply(ui.ctx());
                    egui::CentralPanel::default().show(ui, |ui| {
                        let _ = DateField::new(state, &theme, "Start date")
                            .today(Date::new(2026, 11, 1).expect("valid date"))
                            .show(ui);
                    });
                },
                {
                    let mut state = DateFieldState::default();
                    state.set_text("2026-11-20");
                    state
                },
            );
        harness.run();
        harness
            .render()
            .expect("off-screen graphics adapter required")
            .save(format!("{}/{name}.png", output_dir().display()))
            .expect("write PNG");
    }
    for (dark, name) in [(false, "line-calendar-light"), (true, "line-calendar-dark")] {
        let theme = if dark { Theme::dark() } else { Theme::light() };
        let mut harness = Harness::builder()
            .with_size(egui::vec2(520.0, 170.0))
            .build_ui_state(
                move |ui, state: &mut LineCalendarState| {
                    theme.apply(ui.ctx());
                    egui::CentralPanel::default().show(ui, |ui| {
                        let _ = LineCalendar::new(state, &theme)
                            .today(Date::new(2026, 11, 1).expect("valid date"))
                            .days(12)
                            .show(ui);
                    });
                },
                LineCalendarState::new(
                    Date::new(2026, 11, 8).expect("valid date"),
                    Some(Date::new(2026, 11, 12).expect("valid date")),
                ),
            );
        harness.run();
        harness
            .render()
            .expect("off-screen graphics adapter required")
            .save(format!("{}/{name}.png", output_dir().display()))
            .expect("write PNG");
    }
    for (dark, name) in [(false, "sidebar-light"), (true, "sidebar-dark")] {
        let theme = if dark { Theme::dark() } else { Theme::light() };
        let mut harness = Harness::builder()
            .with_size(egui::vec2(320.0, 300.0))
            .build_ui_state(
                move |ui, state: &mut (SidebarState, Option<egui::Id>)| {
                    theme.apply(ui.ctx());
                    egui::CentralPanel::default().show(ui, |ui| {
                        let inbox = [
                            SidebarItem::new(egui::Id::new("review-inbox"), "Inbox"),
                            SidebarItem::new(egui::Id::new("review-sent"), "Sent"),
                        ];
                        let history =
                            [SidebarItem::new(egui::Id::new("review-archive"), "Archive")];
                        let groups = [
                            SidebarGroup::new(egui::Id::new("review-mail"), &inbox).label("Mail"),
                            SidebarGroup::new(egui::Id::new("review-history"), &history)
                                .label("History"),
                        ];
                        let _ = Sidebar::new(
                            egui::Id::new("review-sidebar"),
                            "Review navigation",
                            &theme,
                            &mut state.0,
                        )
                        .title("Temivu")
                        .show(ui, &groups, &mut state.1);
                    });
                },
                (SidebarState::default(), Some(egui::Id::new("review-inbox"))),
            );
        harness.run();
        harness
            .render()
            .expect("off-screen graphics adapter required")
            .save(format!("{}/{name}.png", output_dir().display()))
            .expect("write PNG");
    }
    for (dark, name) in [(false, "select-tiles-light"), (true, "select-tiles-dark")] {
        let theme = if dark { Theme::dark() } else { Theme::light() };
        let tiles = [
            SelectTile::new(egui::Id::new("review-wifi"), "Wi-Fi").description("Wireless"),
            SelectTile::new(egui::Id::new("review-bt"), "Bluetooth").description("Devices"),
        ];
        let mut harness = Harness::builder()
            .with_size(egui::vec2(380.0, 240.0))
            .build_ui_state(
                move |ui, selected: &mut Vec<egui::Id>| {
                    theme.apply(ui.ctx());
                    egui::CentralPanel::default().show(ui, |ui| {
                        let group = SelectGroup::new(
                            egui::Id::new("review-radios-group"),
                            "Radios",
                            &theme,
                        )
                        .description("Choose any that apply");
                        let _ = group.show(ui, |ui| {
                            let _ = SelectTileGroup::new(
                                egui::Id::new("review-radios"),
                                "Wireless",
                                &theme,
                                &tiles,
                                selected,
                            )
                            .show(ui);
                        });
                    });
                },
                vec![egui::Id::new("review-wifi")],
            );
        harness.run();
        harness
            .render()
            .expect("off-screen graphics adapter required")
            .save(format!("{}/{name}.png", output_dir().display()))
            .expect("write PNG");
    }

    for (dark, name) in [(false, "high-contrast-light"), (true, "high-contrast-dark")] {
        let theme = if dark { Theme::dark() } else { Theme::light() }.high_contrast();
        let mut harness = Harness::builder()
            .with_size(egui::vec2(420.0, 300.0))
            .build_ui_state(
                move |ui, accepted: &mut bool| {
                    theme.apply(ui.ctx());
                    egui::CentralPanel::default().show(ui, |ui| {
                        ui.heading("High contrast");
                        ui.horizontal(|ui| {
                            let _ = Button::new("Primary", &theme).show(ui);
                            let _ = Button::new("Delete", &theme)
                                .variant(ButtonVariant::Destructive)
                                .show(ui);
                        });
                        let _ = Item::new("Account settings", &theme)
                            .description("Manage your profile")
                            .selected(true)
                            .show(ui);
                        let _ = Item::new("Delete account", &theme)
                            .variant(ItemVariant::Destructive)
                            .show(ui);
                        Tabs::new(&mut 0usize, &["General", "Privacy"], &theme)
                            .expect("valid tabs")
                            .show(ui, |ui, _| {
                                ui.label("Tab content");
                            });
                        let _ = Checkbox::new(accepted, &theme, "Accept terms").show(ui);
                    });
                },
                true,
            );
        harness.run();
        harness
            .render()
            .expect("off-screen graphics adapter required")
            .save(format!("{}/{name}.png", output_dir().display()))
            .expect("write PNG");
    }

    for (dark, name, multi) in [
        (false, "multi-select-light", false),
        (true, "multi-select-dark", false),
        (false, "multi-select-search-light", true),
        (true, "multi-select-search-dark", true),
    ] {
        let theme = if dark { Theme::dark() } else { Theme::light() };
        let options = [
            SelectOption::new(egui::Id::new("review-email"), "Email digest"),
            SelectOption::new(egui::Id::new("review-sms"), "SMS alerts"),
            SelectOption::new(egui::Id::new("review-push"), "Push notifications"),
        ];
        let mut harness = Harness::builder()
            .with_size(egui::vec2(360.0, 320.0))
            .build_ui_state(
                move |ui, selected: &mut Vec<egui::Id>| {
                    theme.apply(ui.ctx());
                    egui::CentralPanel::default().show(ui, |ui| {
                        let select = MultiSelect::new(
                            egui::Id::new("review-channels"),
                            "Notification channels",
                            "Choose channels",
                            &options,
                            selected,
                        )
                        .searchable(multi);
                        let _ = select.show(ui);
                    });
                },
                vec![egui::Id::new("review-email")],
            );
        harness.run();
        harness
            .get_by_role_and_label(egui::accesskit::Role::ComboBox, "Notification channels")
            .click();
        harness.run();
        if multi {
            harness
                .get_by_role_and_label(egui::accesskit::Role::TextInput, "Search")
                .focus();
            harness.run();
        }
        harness
            .render()
            .expect("off-screen graphics adapter required")
            .save(format!("{}/{name}.png", output_dir().display()))
            .expect("write PNG");
    }

    for (dark, name) in [(false, "toaster-light"), (true, "toaster-dark")] {
        let theme = if dark { Theme::dark() } else { Theme::light() };
        let mut harness = Harness::builder()
            .with_size(egui::vec2(420.0, 240.0))
            .build_ui_state(
                move |ui, state: &mut ToasterState| {
                    theme.apply(ui.ctx());
                    egui::CentralPanel::default().show(ui, |ui| {
                        ui.label("Background content");
                        Toaster::new(state, &theme, "Dismiss").show(ui);
                    });
                },
                {
                    let mut state = ToasterState::new(3);
                    state.push(
                        ToastEntry::new(egui::Id::new("review-toast-1"), "Changes saved")
                            .description("Your draft is now up to date.")
                            .timeout(None),
                    );
                    state.push(
                        ToastEntry::new(egui::Id::new("review-toast-2"), "Upload failed")
                            .description("Check your connection and retry.")
                            .icon("!")
                            .variant(ToastVariant::Destructive)
                            .timeout(None),
                    );
                    state
                },
            );
        harness.run();
        harness
            .render()
            .expect("off-screen graphics adapter required")
            .save(format!("{}/{name}.png", output_dir().display()))
            .expect("write PNG");
    }

    for (dark, name) in [(false, "range-slider-light"), (true, "range-slider-dark")] {
        let theme = if dark { Theme::dark() } else { Theme::light() };
        let mut harness = Harness::builder()
            .with_size(egui::vec2(360.0, 140.0))
            .build_ui_state(
                move |ui, value: &mut (f64, f64)| {
                    theme.apply(ui.ctx());
                    egui::CentralPanel::default().show(ui, |ui| {
                        RangeSlider::new(value, &theme)
                            .expect("valid range")
                            .label("Price range")
                            .step(0.05)
                            .expect("valid step")
                            .show(ui);
                    });
                },
                (0.2, 0.8),
            );
        harness.run();
        harness
            .render()
            .expect("off-screen graphics adapter required")
            .save(format!("{}/{name}.png", output_dir().display()))
            .expect("write PNG");
    }

    for (dark, name) in [
        (false, "range-slider-vertical-light"),
        (true, "range-slider-vertical-dark"),
    ] {
        let theme = if dark { Theme::dark() } else { Theme::light() };
        let mut harness = Harness::builder()
            .with_size(egui::vec2(200.0, 300.0))
            .build_ui_state(
                move |ui, value: &mut (f64, f64)| {
                    theme.apply(ui.ctx());
                    egui::CentralPanel::default().show(ui, |ui| {
                        RangeSlider::new(value, &theme)
                            .expect("valid range")
                            .label("Level")
                            .vertical(true)
                            .show(ui);
                    });
                },
                (0.2, 0.8),
            );
        harness.run();
        harness
            .render()
            .expect("off-screen graphics adapter required")
            .save(format!("{}/{name}.png", output_dir().display()))
            .expect("write PNG");
    }

    for (dark, name) in [(false, "buttons-icons-light"), (true, "buttons-icons-dark")] {
        let theme = if dark { Theme::dark() } else { Theme::light() };
        let mut harness = Harness::builder()
            .with_size(egui::vec2(520.0, 160.0))
            .build_ui_state(
                move |ui, _: &mut ()| {
                    theme.apply(ui.ctx());
                    egui::CentralPanel::default().show(ui, |ui| {
                        ui.horizontal(|ui| {
                            let _ = Button::new("Continue", &theme).icon("+").show(ui);
                            let _ = Button::new("Back", &theme)
                                .icon("-")
                                .variant(ButtonVariant::Secondary)
                                .show(ui);
                            let _ = Button::new("Delete", &theme)
                                .icon("!")
                                .variant(ButtonVariant::Destructive)
                                .show(ui);
                            let _ = Button::new("Settings", &theme)
                                .icon("*")
                                .variant(ButtonVariant::Outline)
                                .show(ui);
                        });
                    });
                },
                (),
            );
        harness.run();
        harness
            .render()
            .expect("off-screen graphics adapter required")
            .save(format!("{}/{name}.png", output_dir().display()))
            .expect("write PNG");
    }

    for (dark, name) in [(false, "slider-marks-light"), (true, "slider-marks-dark")] {
        let theme = if dark { Theme::dark() } else { Theme::light() };
        let mut harness = Harness::builder()
            .with_size(egui::vec2(320.0, 120.0))
            .build_ui_state(
                move |ui, value: &mut f64| {
                    theme.apply(ui.ctx());
                    let marks = [
                        SliderMark::new(0.0).expect("mark").label("Min"),
                        SliderMark::new(0.5).expect("mark").label("Mid"),
                        SliderMark::new(1.0).expect("mark").label("Max"),
                    ];
                    egui::CentralPanel::default().show(ui, |ui| {
                        Slider::new(value, &theme)
                            .expect("normalized value")
                            .label("Level")
                            .marks(&marks)
                            .show(ui);
                    });
                },
                0.4_f64,
            );
        harness.run();
        harness
            .render()
            .expect("off-screen graphics adapter required")
            .save(format!("{}/{name}.png", output_dir().display()))
            .expect("write PNG");
    }
}
