//! Headless animation comparison. This harness also runs against Temivu 0.1.3.
use std::{hint::black_box, time::Instant};

use egui::{Context, Id, RawInput};
use temivu::{Theme, motion::spring_f32};

fn run(count: usize, animated: bool) -> f64 {
    let ctx = Context::default();
    let theme = Theme::light();
    let ids: Vec<_> = (0..count).map(Id::new).collect();
    let frame = |index: i32| {
        let target = if animated && (index / 24) % 2 == 0 {
            1.0
        } else {
            0.0
        };
        let mut output = ctx.run_ui(
            RawInput {
                time: Some(f64::from(index) / 120.0),
                ..Default::default()
            },
            |_| {
                for &id in &ids {
                    black_box(spring_f32(&ctx, id, 0.0, target, &theme));
                }
            },
        );
        output.textures_delta.clear();
    };
    for index in 0..120 {
        frame(index);
    }
    let start = Instant::now();
    for index in 120..600 {
        frame(index);
    }
    start.elapsed().as_secs_f64() * 1_000_000.0 / 480.0
}

fn main() {
    for count in [0, 1, 128, 1024] {
        for animated in [false, true] {
            if count == 0 && animated {
                continue;
            }
            let mut samples = std::array::from_fn::<_, 7, _>(|_| run(count, animated));
            samples.sort_by(f64::total_cmp);
            println!(
                "{count:>4} {:>6} {:>8.2} us/frame",
                if animated { "active" } else { "idle" },
                samples[3]
            );
        }
    }
}
