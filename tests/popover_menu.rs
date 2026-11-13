use egui_kittest::{
    Harness,
    kittest::{NodeT, Queryable},
};
use temivu::{Item, PopoverMenu, Theme};

#[derive(Default)]
struct State {
    open: bool,
    activated: &'static str,
    other: usize,
    extra_item: bool,
}

fn view(ui: &mut egui::Ui, state: &mut State) {
    let theme = Theme::light();
    let State {
        open,
        activated,
        other,
        extra_item,
    } = state;
    if ui.button("Other").clicked() {
        *other += 1;
    }
    let anchor = ui.button("Actions");
    if anchor.clicked() {
        *open = !*open;
    }
    PopoverMenu::new(egui::Id::unique("actions-menu"), "Actions menu", open).show(
        &anchor,
        |ui, menu| {
            if *extra_item {
                let _ = menu.item(ui, Item::new("Help", &theme).id(egui::Id::unique("help")));
            }
            let _ = menu.item(
                ui,
                Item::new("Archive", &theme)
                    .id(egui::Id::unique("archive"))
                    .enabled(false),
            );
            if menu
                .item(
                    ui,
                    Item::new("Rename", &theme).id(egui::Id::unique("rename")),
                )
                .clicked()
            {
                *activated = "rename";
                ui.close();
            }
            if menu
                .item(
                    ui,
                    Item::new("Delete", &theme).id(egui::Id::unique("delete")),
                )
                .clicked()
            {
                *activated = "delete";
                ui.close();
            }
        },
    );
}

fn open_from_keyboard(harness: &mut Harness<State>) {
    harness.get_by_label("Actions").focus();
    harness.run();
    harness.key_press(egui::Key::Enter);
    harness.run();
}

#[test]
fn keyboard_open_focuses_first_enabled_item_and_escape_restores_trigger_focus() {
    let mut harness = Harness::builder().build_ui_state(view, State::default());
    harness.run();
    open_from_keyboard(&mut harness);
    assert!(harness.state().open);

    let menu = harness.get_by_role_and_label(egui::accesskit::Role::Menu, "Actions menu");
    assert_eq!(menu.accesskit_node().role(), egui::accesskit::Role::Menu);

    let archive = harness.get_by_role_and_label(egui::accesskit::Role::MenuItem, "Archive");
    assert!(archive.accesskit_node().is_disabled());
    assert!(!archive.is_focused(), "a disabled item must not be focused");
    assert!(
        harness
            .get_by_role_and_label(egui::accesskit::Role::MenuItem, "Rename")
            .is_focused(),
        "the first enabled item should receive focus when the menu opens"
    );

    harness.key_press(egui::Key::Escape);
    harness.run();
    assert!(!harness.state().open);
    assert!(harness.query_by_label("Rename").is_none());
    assert!(harness.get_by_label("Actions").is_focused());
}

#[test]
fn enabled_item_activates_with_pointer_and_closes_the_menu() {
    let mut harness = Harness::builder().build_ui_state(view, State::default());
    harness.run();
    harness.get_by_label("Actions").click();
    harness.run();
    assert!(harness.state().open);

    harness.get_by_label("Rename").click();
    harness.run();
    assert_eq!(harness.state().activated, "rename");
    assert!(!harness.state().open);
    assert!(harness.query_by_label("Rename").is_none());
}

#[test]
fn outside_click_dismisses_without_restoring_trigger_focus() {
    let mut harness = Harness::builder().build_ui_state(view, State::default());
    harness.run();
    harness.get_by_label("Actions").click();
    harness.run();
    assert!(harness.state().open);

    harness.get_by_label("Other").click();
    harness.run();
    assert!(!harness.state().open);
    assert_eq!(harness.state().other, 1);
    assert!(!harness.get_by_label("Actions").is_focused());
}

#[test]
fn disabled_item_ignores_accessibility_activation_and_keeps_menu_open() {
    let mut harness = Harness::builder().build_ui_state(view, State::default());
    harness.run();
    open_from_keyboard(&mut harness);

    let archive = harness.get_by_role_and_label(egui::accesskit::Role::MenuItem, "Archive");
    assert!(archive.accesskit_node().is_disabled());
    archive.click_accesskit();
    harness.run();
    assert_eq!(harness.state().activated, "");
    assert!(
        harness.state().open,
        "a disabled item must not dismiss the menu"
    );
}

#[test]
fn stable_item_ids_keep_focus_when_content_changes() {
    let mut harness = Harness::builder().build_ui_state(view, State::default());
    harness.run();
    open_from_keyboard(&mut harness);
    harness.get_by_label("Rename").focus();
    harness.run();
    assert!(harness.get_by_label("Rename").is_focused());

    harness.state_mut().extra_item = true;
    harness.run();
    assert!(
        harness.get_by_label("Rename").is_focused(),
        "stable item IDs should preserve focus when an earlier item is inserted"
    );
}

#[test]
fn programmatic_open_does_not_steal_unrelated_focus() {
    let mut harness = Harness::builder().build_ui_state(view, State::default());
    harness.run();
    harness.get_by_label("Other").focus();
    harness.run();

    harness.state_mut().open = true;
    harness.run();
    harness.get_by_role_and_label(egui::accesskit::Role::Menu, "Actions menu");
    assert!(harness.get_by_label("Other").is_focused());
}
