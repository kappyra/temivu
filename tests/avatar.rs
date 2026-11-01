use egui_kittest::{
    Harness,
    kittest::{NodeT, Queryable},
};
use temivu::{Avatar, Theme};

#[test]
fn invalid_sizes_are_rejected() {
    let theme = Theme::light();
    for size in [0.0, -1.0, f32::NAN, f32::INFINITY] {
        assert!(Avatar::new(&theme).size(size).is_err());
    }
    assert!(
        Avatar::new(&theme)
            .image(egui::TextureId::User(1), egui::vec2(1.0, 0.0))
            .is_err()
    );
}

#[test]
fn fallback_and_image_have_accessible_labels_and_expected_geometry() {
    let mut harness = Harness::builder().build_ui(|ui| {
        let theme = Theme::light();
        Avatar::new(&theme)
            .label("Alice avatar")
            .fallback("AL")
            .show(ui);
        Avatar::new(&theme)
            .label("Bob avatar")
            .size(60.0)
            .unwrap()
            .image(egui::TextureId::User(123), egui::vec2(200.0, 100.0))
            .unwrap()
            .show(ui);
    });
    harness.run();
    let alice = harness.get_by_label("Alice avatar");
    assert_eq!(alice.accesskit_node().role(), egui::accesskit::Role::Image);
    assert_eq!(alice.rect().size(), egui::vec2(40.0, 40.0));
    assert_eq!(
        harness.get_by_label("Bob avatar").rect().size(),
        egui::vec2(60.0, 60.0)
    );
    let meshes: Vec<_> = harness
        .output()
        .shapes
        .iter()
        .filter_map(|s| match &s.shape {
            egui::Shape::Mesh(mesh) if mesh.texture_id == egui::TextureId::User(123) => Some(mesh),
            _ => None,
        })
        .collect();
    assert_eq!(meshes.len(), 1);
    let uv_min = meshes[0]
        .vertices
        .iter()
        .map(|v| v.uv.x)
        .fold(f32::INFINITY, f32::min);
    let uv_max = meshes[0]
        .vertices
        .iter()
        .map(|v| v.uv.x)
        .fold(f32::NEG_INFINITY, f32::max);
    assert!((uv_min - 0.25).abs() < 0.001 && (uv_max - 0.75).abs() < 0.001);
}
