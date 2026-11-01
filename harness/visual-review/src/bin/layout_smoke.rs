//! Offline layout smoke measurement; excludes rendering and is not a release gate.
use std::{hint::black_box, time::Instant};
use temivu::{PaginationState, Theme, pagination};

fn main() {
    for (label, siblings, samples) in [("typical", 1, 50), ("large-window", 1000, 10)] {
        let ctx = egui::Context::default();
        let theme = Theme::light();
        theme.apply(&ctx);
        let mut state = PaginationState::new(1_000_000, 500_000, siblings, true).unwrap();
        // Separate page-list construction from layout so allocation cost is
        // not confused with the cost of creating and laying out widgets.
        let iterations = 1000;
        let list_start = Instant::now();
        for _ in 0..iterations {
            black_box(state.items());
        }
        let list_ms = list_start.elapsed().as_secs_f64() * 1000.0 / f64::from(iterations);
        let mut elapsed = Vec::with_capacity(samples);
        for iteration in 0..samples + 5 {
            let input = egui::RawInput {
                screen_rect: Some(egui::Rect::from_min_size(
                    egui::Pos2::ZERO,
                    egui::vec2(800.0, 600.0),
                )),
                ..Default::default()
            };
            let start = Instant::now();
            let mut output = ctx.run_ui(input, |ui| {
                pagination(ui, &mut state, &theme, "Previous", "Next");
            });
            black_box(output.shapes.len());
            // This measures layout without a renderer; discard texture updates explicitly.
            output.textures_delta.clear();
            if iteration >= 5 {
                elapsed.push(start.elapsed().as_secs_f64() * 1000.0);
            }
        }
        elapsed.sort_by(f64::total_cmp);
        let median = elapsed[elapsed.len() / 2];
        let p95 = elapsed[(elapsed.len() * 95 / 100).min(elapsed.len() - 1)];
        println!(
            "{label}: samples={samples} median_ms={median:.3} p95_ms={p95:.3} item_list_mean_ms={list_ms:.5}"
        );
    }
}
