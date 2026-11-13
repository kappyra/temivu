use egui::{Id, RichText, Ui};
use temivu::{
    Button, ButtonVariant, CircularProgress, Direction, Item, PaginationState, PopoverMenu, Select,
    SelectOption, Switch, SwitchStyle, Tabs, Theme, ViewMode, direction_pad, layout_toggle,
    link_preview, navigation_arrows, pagination, quantity_stepper, search_box, toggle_button,
};

#[derive(Default)]
pub struct Gallery {
    lab: super::motion_lab::Lab,
    tab: usize,
    count: i32,
    view: ViewMode,
    following: bool,
    page: Option<PaginationState>,
    history: u8,
    notifications: bool,
    rail_switch: bool,
    expanded: bool,
    load_started: Option<f64>,
    load_completed: bool,
    menu: bool,
    action: Option<&'static str>,
    category: Option<Id>,
    search_open: bool,
    query: String,
    direction: Option<Direction>,
}

type Draw = fn(&mut Gallery, &mut Ui, &Theme);
type Example = (&'static str, &'static str, Draw);

impl Gallery {
    pub fn show(&mut self, ui: &mut Ui, theme: &Theme, filter: &str) -> bool {
        let examples: [Example; 15] = [
            (
                "Circular loading",
                "Watch the spinner or start a timed load.",
                Self::loading,
            ),
            (
                "Spring resize",
                "Stretch and retract a Droplet surface.",
                Self::resize,
            ),
            (
                "Switch",
                "Compare the stretching capsule and the thin rail.",
                Self::switch,
            ),
            (
                "Segments",
                "Drag or tap to move the selection.",
                Self::segments,
            ),
            (
                "Quantity",
                "Add the first item to see the controls join. Return to zero to fold them back.",
                Self::quantity,
            ),
            (
                "Layout",
                "Cycle through three ways to arrange a view.",
                Self::layout,
            ),
            (
                "Follow",
                "A pill with an animated state and icon.",
                Self::follow,
            ),
            (
                "Pagination",
                "The marker springs toward the active page.",
                Self::pages,
            ),
            (
                "History",
                "Arrows stretch on hover and press.",
                Self::history,
            ),
            (
                "Action menu",
                "Open and close an anchored surface.",
                Self::menu,
            ),
            (
                "Category",
                "Choose a category from the list.",
                Self::category,
            ),
            (
                "Link preview",
                "Hover or focus to reveal the destination.",
                Self::link,
            ),
            (
                "Search",
                "An icon springs open into an editable field.",
                Self::search,
            ),
            (
                "Direction pad",
                "Use the buttons or arrow keys when focused.",
                Self::direction,
            ),
            (
                "Motion lab",
                "Compare presets, change the spring, or release a drag with momentum.",
                Self::lab,
            ),
        ];
        let visible: Vec<_> = examples
            .iter()
            .copied()
            .filter(|(name, _, _)| filter.is_empty() || name.to_lowercase().contains(filter))
            .collect();
        if visible.is_empty() {
            return false;
        }
        ui.label(
            RichText::new(format!("{} of {} examples", visible.len(), examples.len()))
                .size(12.0)
                .color(theme.muted_foreground),
        );
        ui.add_space(6.0);
        let columns = if ui.available_width() >= 880.0 {
            3
        } else if ui.available_width() >= 575.0 {
            2
        } else {
            1
        };
        ui.spacing_mut().item_spacing.x = 12.0;
        for row in visible.chunks(columns) {
            ui.columns(columns, |cells| {
                for (cell, &(title, description, draw)) in cells.iter_mut().zip(row) {
                    cell.push_id(title, |ui| {
                        super::studio_panel(ui, theme, title, description, |ui| {
                            ui.set_min_height(76.0);
                            draw(self, ui, theme);
                        });
                    });
                }
            });
            ui.add_space(12.0);
        }
        true
    }

    fn lab(&mut self, ui: &mut Ui, theme: &Theme) {
        self.lab.show(ui, theme);
    }

    fn resize(&mut self, ui: &mut Ui, theme: &Theme) {
        let action = if self.expanded {
            "Shrink panel"
        } else {
            "Grow panel"
        };
        if Button::new(action, theme)
            .variant(ButtonVariant::Outline)
            .show(ui)
            .clicked()
        {
            self.expanded = !self.expanded;
        }
        let available = ui.available_width().max(1.0);
        let resize = temivu::liquid::DropletResize::new(
            ui.ctx(),
            ui.make_persistent_id("resize-preview"),
            egui::vec2(96.0_f32.min(available), 32.0),
            egui::vec2(200.0_f32.min(available), 70.0),
            self.expanded,
            theme,
        );
        let (rect, _) = ui.allocate_exact_size(resize.size, egui::Sense::hover());
        resize.paint(
            ui.painter(),
            rect,
            egui::Frame::new()
                .fill(theme.muted)
                .stroke(egui::Stroke::new(1.0, theme.border))
                .corner_radius(16),
        );
    }

    fn segments(&mut self, ui: &mut Ui, theme: &Theme) {
        Tabs::new(&mut self.tab, &["Studio", "Library"], theme)
            .expect("static tabs")
            .show(ui, |_, _| {});
    }

    fn quantity(&mut self, ui: &mut Ui, theme: &Theme) {
        quantity_stepper(
            ui,
            theme,
            &mut self.count,
            0..=99,
            "Decrease quantity",
            "Increase quantity",
        );
    }

    fn layout(&mut self, ui: &mut Ui, theme: &Theme) {
        layout_toggle(ui, theme, &mut self.view, ["Columns", "Grid", "List"]);
    }

    fn follow(&mut self, ui: &mut Ui, theme: &Theme) {
        toggle_button(ui, theme, &mut self.following, ["Follow", "Following"]);
    }

    fn pages(&mut self, ui: &mut Ui, theme: &Theme) {
        let pages = self
            .page
            .get_or_insert_with(|| PaginationState::new(4, 0, 1, true).expect("static pagination"));
        pagination(ui, pages, theme, "‹", "›");
    }

    fn history(&mut self, ui: &mut Ui, theme: &Theme) {
        ui.horizontal(|ui| {
            let (back, next) = navigation_arrows(
                ui,
                theme,
                ("Go back", self.history > 0),
                ("Go forward", self.history < 2),
            );
            if back.clicked() {
                self.history = self.history.saturating_sub(1);
            }
            if next.clicked() {
                self.history = (self.history + 1).min(2);
            }
            ui.label(["Start", "Browse", "Details"][usize::from(self.history)]);
        });
    }

    fn switch(&mut self, ui: &mut Ui, theme: &Theme) {
        ui.horizontal_wrapped(|ui| {
            Switch::new(&mut self.notifications, theme)
                .label("Capsule")
                .show(ui);
            Switch::new(&mut self.rail_switch, theme)
                .style(SwitchStyle::Rail)
                .label("Rail")
                .show(ui);
        });
    }

    fn loading(&mut self, ui: &mut Ui, theme: &Theme) {
        let now = ui.input(|input| input.time);
        let progress = self
            .load_started
            .map(|start| ((now - start) / 2.5).clamp(0.0, 1.0) as f32);
        if progress == Some(1.0) {
            self.load_started = None;
            self.load_completed = true;
        }
        ui.horizontal_wrapped(|ui| {
            let ring = if let Some(value) = progress.or(self.load_completed.then_some(1.0)) {
                if value < 1.0 {
                    ui.ctx()
                        .request_repaint_after(std::time::Duration::from_millis(16));
                }
                CircularProgress::determinate(theme, value).expect("normalized loading progress")
            } else {
                CircularProgress::new(theme)
            };
            ring.diameter(40.0)
                .expect("static ring diameter")
                .label("Circular progress")
                .show(ui);
            if let Some(value) = progress.or(self.load_completed.then_some(1.0)) {
                ui.label(format!("{:.0}%", value * 100.0));
            }
            if Button::new("Start loading", theme)
                .variant(ButtonVariant::Secondary)
                .enabled(self.load_started.is_none())
                .show(ui)
                .clicked()
            {
                self.load_started = Some(now);
                self.load_completed = false;
                ui.ctx().request_repaint();
            }
        });
    }

    fn menu(&mut self, ui: &mut Ui, theme: &Theme) {
        let anchor = ui
            .horizontal(|ui| {
                let anchor = Button::new("Actions", theme)
                    .icon("...")
                    .variant(ButtonVariant::Secondary)
                    .show(ui);
                if let Some(action) = self.action {
                    ui.label(action);
                }
                anchor
            })
            .inner;
        if anchor.clicked() {
            self.menu = !self.menu;
        }
        PopoverMenu::new(
            Id::unique("motion-gallery-menu"),
            "Item actions",
            &mut self.menu,
        )
        .show(&anchor, |ui, menu| {
            for (label, status) in [
                ("Rename", "Rename selected"),
                ("Archive", "Archive selected"),
            ] {
                if menu.item(ui, Item::new(label, theme)).clicked() {
                    self.action = Some(status);
                    ui.close();
                }
            }
        });
    }

    fn category(&mut self, ui: &mut Ui, theme: &Theme) {
        let options = ["Design", "Development", "Research"]
            .map(|label| SelectOption::new(Id::unique(label), label));
        Select::new(
            Id::unique("motion-gallery-category"),
            "Collection",
            "Choose category",
            &options,
            &mut self.category,
        )
        .theme(theme)
        .show(ui);
    }

    fn link(&mut self, ui: &mut Ui, theme: &Theme) {
        // Demo preview only; the button makes no external requests.
        let link = Button::new("Project portfolio", theme)
            .variant(ButtonVariant::Outline)
            .show(ui);
        link_preview(link, "studio.example/portfolio", theme);
    }

    fn search(&mut self, ui: &mut Ui, theme: &Theme) {
        search_box(
            ui,
            theme,
            &mut self.search_open,
            &mut self.query,
            "Search components",
            "Close search",
        );
    }

    fn direction(&mut self, ui: &mut Ui, theme: &Theme) {
        if let Some(direction) = direction_pad(ui, theme, ["Up", "Left", "Right", "Down"]) {
            self.direction = Some(direction);
        }
        if let Some(direction) = self.direction {
            ui.label(
                RichText::new(match direction {
                    Direction::Up => "Up",
                    Direction::Left => "Left",
                    Direction::Right => "Right",
                    Direction::Down => "Down",
                })
                .color(theme.muted_foreground),
            );
        }
    }
}
