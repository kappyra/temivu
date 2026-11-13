use egui_kittest::{Harness, kittest::Queryable};
use temivu::{
    Popover, Theme,
    liquid::{LiquidStyle, SurfaceTransition, paint_surface, rounded_rect_distance, smooth_union},
};

#[test]
fn signed_distance_and_smooth_union_have_consistent_boundaries() {
    let rect = egui::Rect::from_min_size(egui::pos2(0.0, 0.0), egui::vec2(100.0, 40.0));
    assert!(rounded_rect_distance(rect.center(), rect, 20.0) < 0.0);
    assert!(rounded_rect_distance(egui::pos2(50.0, 0.0), rect, 20.0).abs() < 0.001);
    assert!(rounded_rect_distance(egui::pos2(0.0, 0.0), rect, 20.0) > 0.0);
    for (a, b) in [(2.0, 2.0), (-2.0, 3.0), (8.0, 1.0)] {
        assert_eq!(smooth_union(a, b, 0.0), a.min(b));
        assert_eq!(smooth_union(a, b, 12.0), smooth_union(b, a, 12.0));
    }
    assert!(
        smooth_union(2.0, 2.0, 12.0) < 0.0,
        "nearby forms should have a connecting neck"
    );
}

#[test]
fn morph_emits_valid_bounded_geometry_and_exit_is_paint_only() {
    let mut harness = Harness::builder()
        .with_size(egui::vec2(500.0, 500.0))
        .build_ui_state(
            |ui, state: &mut (bool, usize)| {
                let theme = Theme::light();
                let anchor = ui.button("Open");
                Popover::new(egui::Id::unique("liquid"), "Liquid options", &mut state.0)
                    .theme(&theme)
                    .liquid(LiquidStyle {
                        transition: SurfaceTransition::Merge,
                        ..Default::default()
                    })
                    .width(240.0)
                    .show(&anchor, |ui| {
                        state.1 += 1;
                        ui.button("Apply")
                    });
            },
            (false, 0),
        );
    harness
        .ctx
        .all_styles_mut(|style| style.animation_time = 0.2);
    harness.input_mut().time = Some(0.0);
    harness.step();
    harness.state_mut().0 = true;
    harness.input_mut().time = Some(0.04);
    harness.step();
    harness.input_mut().time = Some(0.08);
    harness.step();
    fn check_meshes(shape: &egui::Shape) -> usize {
        match shape {
            egui::Shape::Mesh(mesh) => {
                assert!(mesh.is_valid());
                assert!(mesh.vertices.len() <= 131_072);
                assert!(mesh.vertices.iter().all(|vertex| vertex.pos.is_finite()));
                1
            }
            egui::Shape::Vec(shapes) => shapes.iter().map(check_meshes).sum(),
            _ => 0,
        }
    }
    let meshes: usize = harness
        .output()
        .shapes
        .iter()
        .map(|entry| check_meshes(&entry.shape))
        .sum();
    assert!(meshes > 0, "the liquid outline must be actual SDF geometry");
    let calls = harness.state().1;
    harness.state_mut().0 = false;
    harness.input_mut().time = Some(0.12);
    harness.step();
    assert_eq!(
        harness.state().1,
        calls,
        "exit rendering must not execute live controls"
    );
    assert!(
        harness.query_by_label("Apply").is_none(),
        "closing paint must not remain interactive"
    );
    harness
        .ctx
        .all_styles_mut(|style| style.animation_time = 0.0);
    harness.run_steps(3);
    assert!(
        !harness.ctx.has_requested_repaint(),
        "settled surfaces must be idle"
    );
}

fn opacity_at(mesh: &egui::Mesh, point: egui::Pos2) -> f32 {
    let cross = |a: egui::Vec2, b: egui::Vec2| a.x * b.y - a.y * b.x;
    mesh.indices
        .as_chunks::<3>()
        .0
        .iter()
        .filter_map(|index| {
            let [a, b, c] = std::array::from_fn(|i| mesh.vertices[index[i] as usize]);
            let area = cross(b.pos - a.pos, c.pos - a.pos);
            if area.abs() < 0.0001 {
                return None;
            }
            let u = cross(b.pos - point, c.pos - point) / area;
            let v = cross(c.pos - point, a.pos - point) / area;
            let w = 1.0 - u - v;
            (u.min(v).min(w) >= -0.0001).then_some(
                u * f32::from(a.color.a())
                    + v * f32::from(b.color.a())
                    + w * f32::from(c.color.a()),
            )
        })
        .fold(0.0, f32::max)
}

#[test]
fn droplet_overpaints_the_trigger_edge_at_the_join() {
    let ctx = egui::Context::default();
    let source = egui::Rect::from_min_size(egui::pos2(20.0, 20.0), egui::vec2(120.0, 40.0));
    let target = egui::Rect::from_min_size(egui::pos2(20.0, 68.0), egui::vec2(120.0, 100.0));
    let frame = egui::Frame::new()
        .fill(egui::Color32::WHITE)
        .corner_radius(20);
    let mut output = ctx.run_ui(egui::RawInput::default(), |ui| {
        paint_surface(
            ui.painter(),
            source,
            target,
            0.70,
            frame,
            LiquidStyle::droplet(),
        );
    });
    output.textures_delta.clear();
    let contour = output
        .shapes
        .iter()
        .find_map(|entry| match &entry.shape {
            egui::Shape::Vec(shapes) => shapes.iter().find_map(|shape| match shape {
                egui::Shape::Mesh(mesh) => Some(mesh),
                _ => None,
            }),
            _ => None,
        })
        .expect("joined contour");
    for y in [58.8, 59.8, 60.8, 63.0] {
        assert!(
            opacity_at(contour, egui::pos2(80.0, y)) > 248.0,
            "join at y={y}"
        );
    }
    assert_eq!(opacity_at(contour, source.center()), 0.0);
    assert_eq!(opacity_at(contour, egui::pos2(80.0, 20.0)), 0.0);
}

#[test]
fn droplet_join_tracks_the_trigger_and_panel_colors_in_both_directions() {
    use egui::{Color32, Shape};

    let ctx = egui::Context::default();
    let mut theme = Theme::light();
    let trigger = Color32::from_rgb(122, 68, 48);
    let panel = Color32::from_rgb(220, 236, 252);
    theme.primary = trigger;
    theme.apply(&ctx);
    let render = |time, mut open| {
        let mut output = ctx.run_ui(
            egui::RawInput {
                time: Some(time),
                screen_rect: Some(egui::Rect::from_min_size(
                    egui::Pos2::ZERO,
                    egui::vec2(500.0, 500.0),
                )),
                ..Default::default()
            },
            |ui| {
                let anchor = temivu::Button::new("Open settings", &theme).show(ui);
                Popover::new(egui::Id::unique("color-morph"), "Settings", &mut open)
                    .theme(&theme)
                    .frame(
                        egui::Frame::new()
                            .fill(panel)
                            .corner_radius(20)
                            .inner_margin(12),
                    )
                    .liquid(LiquidStyle::droplet())
                    .width(200.0)
                    .show(&anchor, |ui| ui.label("Popup content"));
            },
        );
        output.textures_delta.clear();
        output
    };

    let joined_colors = |output: egui::FullOutput| {
        output.shapes.iter().any(|entry| {
            let Shape::Vec(shapes) = &entry.shape else {
                return false;
            };
            shapes.iter().any(|shape| {
                let Shape::Mesh(mesh) = shape else {
                    return false;
                };
                let mut source = false;
                let mut middle = false;
                let mut destination = false;
                for vertex in &mesh.vertices {
                    let c = vertex.color;
                    source |= c == trigger;
                    destination |= c == panel;
                    middle |= c.a() == 255 && c.b() > trigger.b() && c.b() < panel.b();
                }
                source && middle && destination
            })
        })
    };

    render(0.0, false);
    let mut open = false;
    for i in 1..=50 {
        open |= joined_colors(render(f64::from(i) / 120.0, true));
    }
    assert!(open, "opening must blend the button and popup in one mesh");

    render(1.0, true);
    let mut close = false;
    for i in 1..=50 {
        close |= joined_colors(render(1.0 + f64::from(i) / 120.0, false));
    }
    assert!(close, "closing must blend back into the trigger");
}

#[test]
fn compact_command_menu_replaces_the_trigger_with_one_moving_outline() {
    let ctx = egui::Context::default();
    let theme = Theme::light();
    theme.apply(&ctx);
    LiquidStyle::droplet().set_context_default(&ctx);
    let screen = egui::Rect::from_min_size(egui::Pos2::ZERO, egui::vec2(440.0, 420.0));
    let mut trigger = egui::Rect::NOTHING;
    let mut menu = None;
    let mut render = |time, mut open| {
        let mut output = ctx.run_ui(
            egui::RawInput {
                time: Some(time),
                screen_rect: Some(screen),
                ..Default::default()
            },
            |ui| {
                let anchor = ui
                    .with_layout(egui::Layout::right_to_left(egui::Align::TOP), |ui| {
                        ui.add_sized([44.0, 44.0], egui::Button::new("⋯"))
                    })
                    .inner;
                trigger = anchor.rect;
                menu = temivu::PopoverMenu::new(
                    egui::Id::unique("compact-command"),
                    "Commands",
                    &mut open,
                )
                .theme(&theme)
                .show(&anchor, |ui, menu| {
                    menu.item(ui, temivu::Item::new("Duplicate", &theme));
                    menu.item(ui, temivu::Item::new("Rename", &theme));
                })
                .map(|response| response.rect);
            },
        );
        output.textures_delta.clear();
        (output, trigger, menu)
    };

    render(0.0, false);
    render(0.1, true);
    let (moving, trigger, menu) = render(0.23, true);
    let target = menu.expect("open command menu");
    assert!(
        target.expand(1.0).contains_rect(trigger),
        "target={target:?}, trigger={trigger:?}"
    );
    assert!(screen.contains_rect(target));
    let bounds = moving
        .shapes
        .iter()
        .find_map(|entry| match &entry.shape {
            egui::Shape::Vec(shapes) => shapes.iter().find_map(|shape| match shape {
                egui::Shape::Path(path) if path.fill == theme.card => {
                    Some(egui::Rect::from_points(&path.points))
                }
                _ => None,
            }),
            _ => None,
        })
        .expect("one animated rounded outline");
    assert!(bounds.width() > trigger.width() && bounds.width() < target.width());
    assert!(bounds.intersects(trigger));
}

#[test]
#[ignore = "manual geometry CPU probe"]
fn geometry_timing_probe() {
    let ctx = egui::Context::default();
    let source = egui::Rect::from_min_size(egui::pos2(20.0, 20.0), egui::vec2(360.0, 44.0));
    let target = egui::Rect::from_min_size(egui::pos2(20.0, 76.0), egui::vec2(360.0, 300.0));
    let frame = egui::Frame::new()
        .fill(egui::Color32::WHITE)
        .stroke(egui::Stroke::new(1.0, egui::Color32::GRAY))
        .corner_radius(32);
    let start = std::time::Instant::now();
    for index in 0..120 {
        let mut output = ctx.run_ui(egui::RawInput::default(), |ui| {
            paint_surface(
                ui.painter(),
                source,
                target,
                index as f32 / 119.0,
                frame,
                LiquidStyle {
                    transition: SurfaceTransition::Merge,
                    ..Default::default()
                },
            );
        });
        output.textures_delta.clear();
    }
    eprintln!(
        "SDF transition: {:.3} ms per frame (CPU, debug)",
        start.elapsed().as_secs_f64() * 1000.0 / 120.0
    );
}

#[test]
fn popup_content_remains_visible_through_reopen_and_exit() {
    fn text_visible(shape: &egui::Shape, clip: egui::Rect) -> bool {
        match shape {
            egui::Shape::Vec(shapes) => shapes.iter().any(|shape| text_visible(shape, clip)),
            egui::Shape::Text(text) if text.galley.text() == "Visible content" => {
                text.opacity_factor > 0.1
                    && text.fallback_color.a() > 20
                    && shape.visual_bounding_rect().intersects(clip)
            }
            _ => false,
        }
    }
    let ctx = egui::Context::default();
    let theme = Theme::light();
    theme.apply(&ctx);
    let mut visible = Vec::new();
    for index in 0..100 {
        let time = index as f64 / 60.0;
        let mut open = (0.0..0.6).contains(&time) || (0.9..1.5).contains(&time);
        let mut output = ctx.run_ui(
            egui::RawInput {
                time: Some(time),
                screen_rect: Some(egui::Rect::from_min_size(
                    egui::Pos2::ZERO,
                    egui::vec2(500.0, 500.0),
                )),
                ..Default::default()
            },
            |ui| {
                let anchor = ui.button("Open");
                Popover::new(egui::Id::unique("reopen"), "Options", &mut open)
                    .theme(&theme)
                    .width(240.0)
                    .show(&anchor, |ui| ui.label("Visible content"));
            },
        );
        visible.push(
            output
                .shapes
                .iter()
                .any(|entry| text_visible(&entry.shape, entry.clip_rect)),
        );
        output.textures_delta.clear();
    }
    for index in [12, 24, 35, 37, 66, 78, 89, 91] {
        assert!(visible[index], "missing visible content at frame {index}");
    }
}

#[test]
fn calendar_content_survives_above_anchor_reopen() {
    let ctx = egui::Context::default();
    let theme = Theme::light();
    theme.apply(&ctx);
    let mut state = temivu::DateFieldState::new();
    for index in 0..100 {
        let time = index as f64 / 60.0;
        state.set_open((0.0..0.6).contains(&time) || (0.9..1.5).contains(&time));
        let mut output = ctx.run_ui(
            egui::RawInput {
                time: Some(time),
                screen_rect: Some(egui::Rect::from_min_size(
                    egui::Pos2::ZERO,
                    egui::vec2(500.0, 600.0),
                )),
                ..Default::default()
            },
            |ui| {
                ui.add_space(380.0);
                temivu::DateField::new(&mut state, &theme, "Date")
                    .today(temivu::Date::new(2026, 11, 1).unwrap())
                    .show(ui);
            },
        );
        if [12, 24, 35, 37, 66, 78, 89, 91].contains(&index) {
            let texts: Vec<_> = output
                .shapes
                .iter()
                .filter_map(|entry| match &entry.shape {
                    egui::Shape::Text(text) => Some((text, entry.clip_rect)),
                    _ => None,
                })
                .collect();
            assert!(
                texts.iter().any(|(text, clip)| text.galley.text() == "15"
                    && text.opacity_factor > 0.1
                    && text.fallback_color.a() > 20
                    && egui::Shape::Text((*text).clone())
                        .visual_bounding_rect()
                        .intersects(*clip)),
                "missing date at frame {index}: {:?}",
                texts
                    .iter()
                    .map(|(t, c)| (t.galley.text(), t.pos, t.opacity_factor, *c))
                    .collect::<Vec<_>>()
            );
        }
        output.textures_delta.clear();
    }
}

#[test]
fn settled_surface_uses_outer_bounds_without_adding_padding_twice() {
    let ctx = egui::Context::default();
    let target = egui::Rect::from_min_size(egui::pos2(20.0, 76.0), egui::vec2(280.0, 300.0));
    let frame = egui::Frame::new()
        .fill(egui::Color32::WHITE)
        .stroke(egui::Stroke::new(1.0, egui::Color32::GRAY))
        .inner_margin(16)
        .corner_radius(32);
    let mut output = ctx.run_ui(egui::RawInput::default(), |ui| {
        paint_surface(
            ui.painter(),
            target.translate(egui::vec2(0.0, -60.0)),
            target,
            1.0,
            frame,
            LiquidStyle::default(),
        );
    });
    assert!(
        output
            .shapes
            .iter()
            .any(|entry| matches!(&entry.shape, egui::Shape::Rect(rect) if rect.rect == target))
    );
    output.textures_delta.clear();
}

#[test]
fn bloom_popup_expands_from_the_trigger_without_shrinking_text() {
    let ctx = egui::Context::default();
    let theme = Theme::light();
    theme.apply(&ctx);
    let mut target = None;
    let mut during = None;
    for frame in 0..50 {
        let time = frame as f64 / 60.0;
        let mut open = true;
        let mut output = ctx.run_ui(
            egui::RawInput {
                time: Some(time),
                ..Default::default()
            },
            |ui| {
                let anchor = ui.button("Open");
                target = Popover::new(egui::Id::unique("expand"), "Options", &mut open)
                    .theme(&theme)
                    .liquid(LiquidStyle::bloom())
                    .width(240.0)
                    .show(&anchor, |ui| ui.label("Normal sized text"))
                    .map(|r| r.response.rect);
            },
        );
        if frame == 8 {
            fn find(shape: &egui::Shape) -> Option<&egui::epaint::TextShape> {
                match shape {
                    egui::Shape::Vec(shapes) => shapes.iter().find_map(find),
                    egui::Shape::Text(text) if text.galley.text() == "Normal sized text" => {
                        Some(text)
                    }
                    _ => None,
                }
            }
            during = output
                .shapes
                .iter()
                .find_map(|entry| find(&entry.shape))
                .map(|text| text.galley.size());
        }
        if frame == 49 {
            let settled = output
                .shapes
                .iter()
                .find_map(|entry| match &entry.shape {
                    egui::Shape::Text(text) if text.galley.text() == "Normal sized text" => {
                        Some(text.galley.size())
                    }
                    _ => None,
                })
                .unwrap();
            let during = during.expect("text is painted during opening");
            assert!(during.x >= settled.x * 0.95 && during.y >= settled.y * 0.95);
            assert!(target.is_some());
        }
        output.textures_delta.clear();
    }
    let source = egui::Rect::from_min_size(egui::pos2(20.0, 20.0), egui::vec2(300.0, 44.0));
    let target = source.translate(egui::vec2(0.0, 80.0));
    assert_eq!(temivu::liquid::surface_rect(source, target, 0.0), source);
    assert_eq!(temivu::liquid::surface_rect(source, target, 1.0), target);
}

#[test]
fn droplet_body_is_finite_and_mirrors_for_above_and_below_popups() {
    use temivu::liquid::droplet_rect;
    fn mirror(rect: egui::Rect) -> egui::Rect {
        egui::Rect::from_min_max(
            egui::pos2(rect.left(), -rect.bottom()),
            egui::pos2(rect.right(), -rect.top()),
        )
    }
    let source = egui::Rect::from_min_size(egui::pos2(30.0, 100.0), egui::vec2(240.0, 44.0));
    let target = egui::Rect::from_min_size(egui::pos2(20.0, 156.0), egui::vec2(280.0, 320.0));
    let closed = droplet_rect(source, target, 0.0);
    assert!(
        source.contains_rect(closed),
        "the initial body starts inside the trigger"
    );
    let emerging = droplet_rect(source, target, 0.12);
    assert!(
        emerging.bottom() > source.bottom(),
        "the leading edge emerges first"
    );
    assert!((emerging.width() - closed.width()).abs() < 0.001);
    for step in 0..=102 {
        let amount = step as f32 / 100.0;
        let body = droplet_rect(source, target, amount);
        assert!(body.is_finite() && body.is_positive());
        let reflected = mirror(droplet_rect(mirror(source), mirror(target), amount));
        assert!(
            body.min.distance(reflected.min) < 0.001 && body.max.distance(reflected.max) < 0.001
        );
    }
    assert_eq!(droplet_rect(source, target, 1.0), target);
}

#[test]
fn context_preset_is_inherited_and_widget_override_takes_precedence() {
    fn has_mesh(shape: &egui::Shape) -> bool {
        match shape {
            egui::Shape::Mesh(_) => true,
            egui::Shape::Vec(shapes) => shapes.iter().any(has_mesh),
            _ => false,
        }
    }
    fn sample(explicit_bloom: bool) -> bool {
        let ctx = egui::Context::default();
        let theme = Theme::light();
        theme.apply(&ctx);
        LiquidStyle::droplet().set_context_default(&ctx);
        let mut meshes = false;
        for (time, state) in [(0.0, false), (30.0, true), (30.12, true)] {
            let mut open = state;
            let mut output = ctx.run_ui(
                egui::RawInput {
                    time: Some(time),
                    ..Default::default()
                },
                |ui| {
                    let anchor = ui.button("Open");
                    let mut popup = Popover::new(egui::Id::unique("inherit"), "Options", &mut open)
                        .width(240.0)
                        .theme(&theme);
                    if explicit_bloom {
                        popup = popup.liquid(LiquidStyle::bloom());
                    }
                    popup.show(&anchor, |ui| ui.button("Done"));
                },
            );
            meshes = output.shapes.iter().any(|entry| has_mesh(&entry.shape));
            output.textures_delta.clear();
        }
        meshes
    }
    assert!(
        sample(false),
        "the context Droplet preset must reach an unconfigured popup"
    );
    assert!(
        !sample(true),
        "an explicit Bloom popup must retain its original rounded-frame rendering"
    );
}

#[test]
fn three_droplets_form_one_filled_surface_and_separate_again() {
    use egui::{Color32, Frame, Pos2, Rect, Shape, pos2, vec2};
    use temivu::liquid::{DropletPart, droplet_group_progress, paint_droplet_group};

    let ctx = egui::Context::default();
    let theme = Theme::light();
    theme.apply(&ctx);
    let style = LiquidStyle::droplet();
    let id = egui::Id::unique("group");
    let parts = [80.0, 150.0, 220.0].map(|x| DropletPart {
        rect: Rect::from_center_size(pos2(x, 100.0), vec2(50.0, 40.0)),
        corners: 20.into(),
    });
    let target = Rect::from_center_size(pos2(150.0, 120.0), vec2(180.0, 72.0));
    let fill = Color32::from_rgb(192, 214, 221);
    let frame = Frame::new().fill(fill).corner_radius(28);
    let mut phase = 0.0;
    let mut render = |time, merged| {
        let mut output = ctx.run_ui(
            egui::RawInput {
                time: Some(time),
                screen_rect: Some(Rect::from_min_size(Pos2::ZERO, vec2(300.0, 240.0))),
                ..Default::default()
            },
            |ui| {
                phase = droplet_group_progress(ui.ctx(), id, merged, style, &theme);
                paint_droplet_group(ui.painter(), &parts, target, phase, frame, style);
            },
        );
        output.textures_delta.clear();
        (phase, output)
    };
    let count_cards = |output: &egui::FullOutput| {
        output
            .shapes
            .iter()
            .filter(|entry| matches!(&entry.shape, Shape::Rect(rect) if rect.fill == fill))
            .count()
    };
    let (_, split) = render(0.0, false);
    assert_eq!(count_cards(&split), 3);
    render(0.01, true);
    let (mid, joining) = render(0.10, true);
    assert!((0.40..0.90).contains(&mid), "phase={mid}");
    let mesh = joining
        .shapes
        .iter()
        .find_map(|entry| match &entry.shape {
            Shape::Vec(shapes) => shapes.iter().find_map(|shape| match shape {
                Shape::Mesh(mesh) => Some(mesh),
                _ => None,
            }),
            _ => None,
        })
        .expect("one mesh for all three surfaces");
    assert!(mesh.is_valid());
    assert!(mesh.vertices.len() < 20_000);
    assert!(mesh.vertices.iter().all(|vertex| vertex.pos.is_finite()));
    for x in (124..=176).step_by(4) {
        assert!(
            opacity_at(mesh, pos2(x as f32, 113.0)) > 240.0,
            "join at x={x}"
        );
    }
    let (retarget, _) = render(0.10, false);
    assert_eq!(retarget, mid, "reversing preserves the current pose");
    let (_, split) = render(1.0, false);
    assert_eq!(count_cards(&split), 3);
    render(1.01, true);
    let (_, joined) = render(2.0, true);
    assert_eq!(count_cards(&joined), 1);
    assert!(joined.shapes.iter().any(|entry| {
        matches!(&entry.shape, Shape::Rect(rect) if rect.fill == fill && rect.rect == target)
    }));
}

#[test]
fn wide_popup_uses_the_current_click_and_retains_the_contact_on_external_close() {
    fn sample(fraction: f32, close_at: Option<f32>, glass: bool) -> (f32, f32) {
        let mut harness = Harness::builder()
            .with_size(egui::vec2(500.0, 420.0))
            .build_ui_state(
                move |ui, open| {
                    temivu::Glass::set_context(ui.ctx(), glass.then(temivu::Glass::default));
                    let anchor = ui.add_sized([300.0, 44.0], egui::Button::new("Wide trigger"));
                    if anchor.clicked() {
                        *open = !*open;
                    }
                    Popover::new(egui::Id::unique("contact"), "Options", open)
                        .width(200.0)
                        .show(&anchor, |ui| ui.label("Content"));
                },
                false,
            );
        harness
            .ctx
            .all_styles_mut(|style| style.animation_time = 0.2);
        let trigger = harness.get_by_label("Wide trigger").rect();
        let pointer = egui::pos2(egui::lerp(trigger.x_range(), fraction), trigger.center().y);
        harness.hover_at(pointer);
        harness.drag_at(pointer);
        harness.input_mut().time = Some(0.1);
        harness.step();
        harness.event(egui::Event::PointerButton {
            pos: pointer,
            button: egui::PointerButton::Primary,
            pressed: false,
            modifiers: egui::Modifiers::NONE,
        });
        harness.input_mut().time = Some(0.12);
        harness.step();
        assert!(*harness.state());
        let surface_center = |harness: &Harness<'_, bool>| {
            harness
                .output()
                .shapes
                .iter()
                .find_map(|entry| {
                    let egui::Shape::Vec(shapes) = &entry.shape else {
                        return None;
                    };
                    shapes.iter().find_map(|shape| match shape {
                        egui::Shape::Mesh(mesh) if !mesh.indices.is_empty() => {
                            Some(mesh.calc_bounds().center().x)
                        }
                        egui::Shape::Path(path) if path.fill.a() > 0 => {
                            Some(egui::Rect::from_points(&path.points).center().x)
                        }
                        _ => None,
                    })
                })
                .unwrap_or_else(|| {
                    panic!(
                        "visible Droplet contour at {}",
                        harness.ctx.input(|input| input.time)
                    )
                })
        };
        harness.input_mut().time = Some(0.19);
        harness.step();
        let opening = surface_center(&harness);
        harness.input_mut().time = Some(1.0);
        harness.step();
        if let Some(close_at) = close_at {
            let pointer = egui::pos2(egui::lerp(trigger.x_range(), close_at), trigger.center().y);
            harness.hover_at(pointer);
            harness.drag_at(pointer);
            harness.input_mut().time = Some(1.02);
            harness.step();
            harness.event(egui::Event::PointerButton {
                pos: pointer,
                button: egui::PointerButton::Primary,
                pressed: false,
                modifiers: egui::Modifiers::NONE,
            });
        } else {
            *harness.state_mut() = false;
            harness.hover_at(egui::pos2(480.0, 400.0));
        }
        harness.input_mut().time = Some(1.04);
        harness.step();
        assert!(!*harness.state());
        harness.input_mut().time = Some(1.20);
        harness.step();
        (opening, surface_center(&harness))
    }

    let left = sample(0.15, None, false);
    let middle = sample(0.5, None, false);
    let right = sample(0.85, None, false);
    assert!(left.0 + 50.0 < middle.0 && middle.0 + 50.0 < right.0);
    assert!(left.1 + 50.0 < middle.1 && middle.1 + 50.0 < right.1);
    let left_to_right = sample(0.15, Some(0.85), false);
    let right_to_left = sample(0.85, Some(0.15), false);
    assert!(right_to_left.1 + 100.0 < left_to_right.1);
    assert_eq!(sample(0.15, Some(0.85), true), left_to_right);
    assert_eq!(sample(0.85, Some(0.15), true), right_to_left);
}
