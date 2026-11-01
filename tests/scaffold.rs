use egui_kittest::{Harness, kittest::Queryable};
use temivu::{Scaffold, Theme};

#[test]
fn regions_are_arranged_and_content_is_interactive() {
    let mut harness = Harness::builder()
        .with_size(egui::vec2(600.0, 400.0))
        .build_ui_state(
            |ui, clicked: &mut bool| {
                Scaffold::new(egui::Id::new("scaffold"), &Theme::light())
                    .sidebar(140.0)
                    .unwrap()
                    .header(true)
                    .footer(true)
                    .show(
                        ui,
                        |ui| {
                            ui.label("Sidebar");
                        },
                        |ui| {
                            ui.label("Header");
                        },
                        |ui| {
                            ui.label("Footer");
                        },
                        |ui| {
                            if ui.button("Content action").clicked() {
                                *clicked = true;
                            }
                        },
                    );
            },
            false,
        );
    harness.run();
    let side = harness.get_by_label("Sidebar").rect();
    let head = harness.get_by_label("Header").rect();
    let body = harness.get_by_label("Content action").rect();
    let foot = harness.get_by_label("Footer").rect();
    assert!(side.right() < head.left());
    assert!(head.bottom() <= body.top());
    assert!(body.bottom() <= foot.top());
    harness.get_by_label("Content action").click();
    harness.run();
    assert!(*harness.state());
}

#[test]
fn narrow_sidebar_leaves_room_for_content_and_invalid_sizes_fail() {
    assert!(
        Scaffold::new(egui::Id::new("invalid"), &Theme::light())
            .sidebar(f32::NAN)
            .is_err()
    );
    assert!(
        Scaffold::new(egui::Id::new("invalid"), &Theme::light())
            .sidebar(0.0)
            .is_err()
    );
    let mut harness = Harness::builder()
        .with_size(egui::vec2(200.0, 300.0))
        .build_ui(|ui| {
            Scaffold::new(egui::Id::new("narrow"), &Theme::light())
                .sidebar(180.0)
                .unwrap()
                .content_padding(false)
                .show(
                    ui,
                    |ui| {
                        ui.label("Side");
                    },
                    |_| {},
                    |_| {},
                    |ui| {
                        let _ = ui.button("Body");
                    },
                );
        });
    harness.run();
    assert!(harness.get_by_label("Body").rect().right() <= 200.0);
}

#[test]
fn disabled_regions_are_not_built() {
    let mut harness = Harness::builder().build_ui(|ui| {
        Scaffold::new(egui::Id::new("content-only"), &Theme::light()).show(
            ui,
            |_| panic!("sidebar must not build"),
            |_| panic!("header must not build"),
            |_| panic!("footer must not build"),
            |ui| {
                ui.label("Content only");
            },
        );
    });
    harness.run();
    harness.get_by_label("Content only");
}

#[test]
fn default_content_padding_matches_source_page_padding() {
    let position = |padding| {
        let mut harness = Harness::builder()
            .with_size(egui::vec2(300.0, 200.0))
            .build_ui(move |ui| {
                Scaffold::new(egui::Id::new("padding"), &Theme::light())
                    .content_padding(padding)
                    .show(
                        ui,
                        |_| {},
                        |_| {},
                        |_| {},
                        |ui| {
                            ui.label("Body");
                        },
                    );
            });
        harness.run();
        harness.get_by_label("Body").rect().min
    };
    let padded = position(true);
    let unpadded = position(false);
    // Forui `FStyle.pagePadding`: 12 horizontal, 8 vertical.
    assert!((padded.x - unpadded.x - 12.0).abs() < 0.01);
    assert!((padded.y - unpadded.y - 8.0).abs() < 0.01);
}
