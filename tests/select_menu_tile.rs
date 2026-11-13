use egui_kittest::{
    Harness,
    kittest::{NodeT, Queryable},
};
use temivu::{PopoverMenu, SelectMenuTile, Theme};

#[derive(Default)]
struct State {
    selected: Option<egui::Id>,
    open: bool,
    extra: bool,
}

fn view(ui: &mut egui::Ui, state: &mut State) {
    let theme = Theme::light();
    let State {
        selected,
        open,
        extra,
    } = state;
    if SelectMenuTile::new("Compact", &theme)
        .id(egui::Id::unique("compact-open"))
        .icon("=")
        .description("Show more rows")
        .selected(*selected == Some(egui::Id::unique("compact")))
        .show(ui)
        .clicked()
    {
        *selected = Some(egui::Id::unique("compact"));
    }
    let _ = extra;
    let _ = open;
}

#[test]
fn select_menu_tile_exposes_menu_item_role_label_and_selection() {
    let mut harness = Harness::builder()
        .with_size(egui::vec2(320.0, 240.0))
        .build_ui_state(view, State::default());
    harness.run();

    let tile = harness.get_by_role_and_label(egui::accesskit::Role::MenuItem, "Compact");
    assert_eq!(tile.accesskit_node().is_selected(), Some(false));
    assert_eq!(
        tile.accesskit_node().description().as_deref(),
        Some("Show more rows"),
        "the description should be exposed without leaking into the name"
    );
    assert!(
        tile.rect().height() >= 48.0,
        "tile rows keep the 48-point height"
    );

    tile.click();
    harness.run();
    assert_eq!(
        harness.state().selected,
        Some(egui::Id::unique("compact")),
        "a click is reported through the response"
    );
}

#[test]
fn disabled_select_menu_tile_ignores_pointer_and_stale_activation() {
    let mut harness = Harness::builder()
        .with_size(egui::vec2(320.0, 240.0))
        .build_ui_state(
            |ui, selected: &mut Option<egui::Id>| {
                if SelectMenuTile::new("Archive", &Theme::light())
                    .id(egui::Id::unique("archive"))
                    .selected(false)
                    .enabled(false)
                    .show(ui)
                    .clicked()
                {
                    *selected = Some(egui::Id::unique("archive"));
                }
            },
            None,
        );
    harness.run();
    let (target_node, target_tree) = {
        let tile = harness.get_by_label("Archive");
        assert!(tile.accesskit_node().is_disabled());
        tile.click();
        tile.accesskit_node().locate()
    };
    harness.run();
    assert_eq!(*harness.state(), None);
    harness.event(egui::Event::AccessKitActionRequest(
        egui::accesskit::ActionRequest {
            action: egui::accesskit::Action::Click,
            target_node,
            target_tree,
            data: None,
        },
    ));
    harness.run();
    assert_eq!(*harness.state(), None);
}

fn menu_view(ui: &mut egui::Ui, state: &mut State) {
    let theme = Theme::light();
    let State {
        selected,
        open,
        extra,
    } = state;
    let anchor = ui.button("Density");
    if anchor.clicked() {
        *open = !*open;
    }
    PopoverMenu::new(egui::Id::unique("density-menu"), "Density menu", open).show(
        &anchor,
        |ui, menu| {
            if *extra {
                let _ = menu.select_menu_tile(
                    ui,
                    SelectMenuTile::new("Extra", &theme).id(egui::Id::unique("extra")),
                );
            }
            if menu
                .select_menu_tile(
                    ui,
                    SelectMenuTile::new("Compact", &theme)
                        .id(egui::Id::unique("compact"))
                        .selected(*selected == Some(egui::Id::unique("compact"))),
                )
                .clicked()
            {
                *selected = Some(egui::Id::unique("compact"));
                ui.close();
            }
            let _ = menu.select_menu_tile(
                ui,
                SelectMenuTile::new("Comfortable", &theme)
                    .id(egui::Id::unique("comfortable"))
                    .enabled(false),
            );
            if menu
                .select_menu_tile(
                    ui,
                    SelectMenuTile::new("Roomy", &theme).id(egui::Id::unique("roommy")),
                )
                .clicked()
            {
                *selected = Some(egui::Id::unique("roommy"));
                ui.close();
            }
        },
    );
}

fn harness() -> Harness<'static, State> {
    Harness::builder()
        .with_size(egui::vec2(360.0, 320.0))
        .build_ui_state(menu_view, State::default())
}

#[test]
fn popover_menu_focuses_and_activates_a_select_menu_tile() {
    let mut harness = harness();
    harness.run();
    harness.get_by_label("Density").focus();
    harness.run();
    harness.key_press(egui::Key::Enter);
    harness.run();
    assert!(harness.state().open);
    assert!(
        harness
            .get_by_role_and_label(egui::accesskit::Role::MenuItem, "Compact")
            .is_focused(),
        "the first enabled select menu tile should receive focus on open"
    );
    assert!(
        harness
            .get_by_role_and_label(egui::accesskit::Role::MenuItem, "Comfortable")
            .accesskit_node()
            .is_disabled()
    );

    harness
        .get_by_role_and_label(egui::accesskit::Role::MenuItem, "Roomy")
        .click();
    harness.run();
    assert_eq!(harness.state().selected, Some(egui::Id::unique("roommy")));
    assert!(!harness.state().open, "ui.close() should dismiss the menu");
}

#[test]
fn stable_ids_keep_focus_when_menu_content_changes() {
    let mut harness = harness();
    harness.run();
    harness.get_by_label("Density").focus();
    harness.run();
    harness.key_press(egui::Key::Enter);
    harness.run();
    harness.get_by_label("Roomy").focus();
    harness.run();
    assert!(harness.get_by_label("Roomy").is_focused());

    harness.state_mut().extra = true;
    harness.run();
    assert!(
        harness.get_by_label("Roomy").is_focused(),
        "stable IDs should preserve menu focus when an earlier row is inserted"
    );
}
