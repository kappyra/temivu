//! Representative offline performance probe.
//!
//! Measures per-frame CPU time (layout, tessellation and paint callbacks) and
//! off-screen render time for several representative scenarios. Rendering uses
//! the software Vulkan adapter available to the review sandbox, so these are
//! CPU-bound measurements on one host, not proof of overall performance.
use std::hint::black_box;
use std::time::Instant;

use egui_kittest::Harness;
use temivu::{
    Badge, Button, ButtonVariant, Checkbox, Dialog, Header, Item, PaginationState, Popover,
    Progress, Select, SelectOption, Slider, Switch, TextField, Theme, Tile, pagination,
};

const WARMUP_FRAMES: usize = 5;
const SAMPLE_FRAMES: usize = 30;

struct Samples {
    cpu: Vec<f64>,
    render: Vec<f64>,
}

#[derive(Clone, Copy)]
struct Summary {
    min: f64,
    median: f64,
    p95: f64,
    max: f64,
    mean: f64,
}

impl Summary {
    fn of(mut values: Vec<f64>) -> Self {
        values.sort_by(f64::total_cmp);
        let sum: f64 = values.iter().sum();
        let mean = sum / values.len() as f64;
        Self {
            min: values[0],
            median: values[values.len() / 2],
            p95: values[(values.len() * 95 / 100).min(values.len() - 1)],
            max: values[values.len() - 1],
            mean,
        }
    }

    fn json(self) -> String {
        format!(
            "{{\"min\":{:.4},\"median\":{:.4},\"p95\":{:.4},\"max\":{:.4},\"mean\":{:.4}}}",
            self.min, self.median, self.p95, self.max, self.mean
        )
    }
}

fn bench<'a, State>(
    harness: &mut Harness<'a, State>,
    mut between_frames: impl FnMut(&mut Harness<'a, State>, usize),
) -> (Samples, usize) {
    for frame in 0..WARMUP_FRAMES {
        between_frames(harness, frame);
        harness.step();
    }
    let mut samples = Samples {
        cpu: Vec::with_capacity(SAMPLE_FRAMES),
        render: Vec::with_capacity(SAMPLE_FRAMES),
    };
    let mut pixels = 0;
    for frame in 0..SAMPLE_FRAMES {
        between_frames(harness, frame + WARMUP_FRAMES);
        let start = Instant::now();
        harness.step();
        samples.cpu.push(start.elapsed().as_secs_f64() * 1000.0);
        let start = Instant::now();
        let image = harness
            .render()
            .expect("off-screen graphics adapter required");
        samples.render.push(start.elapsed().as_secs_f64() * 1000.0);
        pixels = image.width() as usize * image.height() as usize;
        black_box(&image);
    }
    (samples, pixels)
}

fn report(name: &str, samples: &Samples, pixels: usize) -> String {
    let cpu = Summary::of(samples.cpu.clone());
    let render = Summary::of(samples.render.clone());
    println!(
        "{name}: frames={SAMPLE_FRAMES} pixels={pixels} cpu_ms median={:.3} p95={:.3} max={:.3} render_ms median={:.3} p95={:.3}",
        cpu.median, cpu.p95, cpu.max, render.median, render.p95
    );
    format!(
        "{{\"name\":\"{name}\",\"frames\":{SAMPLE_FRAMES},\"pixels\":{pixels},\"cpu_ms\":{},\"render_ms\":{}}}",
        cpu.json(),
        render.json()
    )
}

fn environment() -> String {
    let rss_kib = std::fs::read_to_string("/proc/self/status")
        .ok()
        .and_then(|status| {
            status.lines().find_map(|line| {
                line.strip_prefix("VmHWM:")
                    .map(|value| value.trim().trim_end_matches(" kB").trim().to_owned())
            })
        })
        .unwrap_or_else(|| "unknown".to_owned());
    let load = std::fs::read_to_string("/proc/loadavg").unwrap_or_default();
    let cpus = std::thread::available_parallelism().map_or(0, |count| count.get());
    let rustc = std::process::Command::new("rustc")
        .arg("--version")
        .output()
        .ok()
        .and_then(|output| String::from_utf8(output.stdout).ok())
        .map_or_else(
            || "unavailable".to_owned(),
            |version| version.trim().to_owned(),
        );
    format!(
        "{{\"cpus\":{cpus},\"loadavg\":\"{}\",\"max_rss_kib\":\"{rss_kib}\",\"rustc\":\"{rustc}\",\"mode\":\"release\",\"renderer\":\"wgpu software Vulkan (lavapipe)\"}}",
        load.trim()
    )
}

fn item_list() -> String {
    let theme = Theme::light();
    let titles: Vec<String> = (0..500).map(|index| format!("Item {index}")).collect();
    let mut harness = Harness::builder()
        .with_size(egui::vec2(800.0, 600.0))
        .build_ui_state(
            move |ui, frame: &mut usize| {
                theme.apply(ui.ctx());
                egui::CentralPanel::default().show(ui, |ui| {
                    egui::ScrollArea::vertical()
                        .vertical_scroll_offset((*frame % 40) as f32 * 12.0)
                        .show(ui, |ui| {
                            for title in &titles {
                                Item::new(title, &theme)
                                    .description("A caller-owned selectable row")
                                    .show(ui);
                            }
                        });
                });
            },
            0usize,
        );
    let (samples, pixels) = bench(&mut harness, |harness, frame| {
        *harness.state_mut() = frame;
    });
    report("item-list-500-scroll", &samples, pixels)
}

fn tile_list() -> String {
    let theme = Theme::light();
    let titles: Vec<String> = (0..200).map(|index| format!("Tile {index}")).collect();
    let mut harness = Harness::builder()
        .with_size(egui::vec2(420.0, 600.0))
        .build_ui_state(
            move |ui, frame: &mut usize| {
                theme.apply(ui.ctx());
                egui::CentralPanel::default().show(ui, |ui| {
                    egui::ScrollArea::vertical()
                        .vertical_scroll_offset((*frame % 30) as f32 * 16.0)
                        .show(ui, |ui| {
                            for title in &titles {
                                Tile::new(title, &theme)
                                    .description("Connected device")
                                    .show(ui);
                            }
                        });
                });
            },
            0usize,
        );
    let (samples, pixels) = bench(&mut harness, |harness, frame| {
        *harness.state_mut() = frame;
    });
    report("tile-list-200-scroll", &samples, pixels)
}

fn pagination_large() -> String {
    let theme = Theme::light();
    let mut harness = Harness::builder()
        .with_size(egui::vec2(800.0, 200.0))
        .build_ui_state(
            move |ui, state: &mut PaginationState| {
                theme.apply(ui.ctx());
                egui::CentralPanel::default().show(ui, |ui| {
                    pagination(ui, state, &theme, "Previous", "Next");
                });
            },
            PaginationState::new(1_000_000, 500_000, 10, true).expect("valid constant"),
        );
    let (samples, pixels) = bench(&mut harness, |harness, frame| {
        let total = harness.state().pages();
        let page = (harness.state().page() + 7 + frame % 13) % total;
        harness
            .state_mut()
            .set_page(page)
            .expect("page is in range");
    });
    report("pagination-1m-pages", &samples, pixels)
}

fn dialog_overlay() -> String {
    let theme = Theme::light();
    let mut harness = Harness::builder()
        .with_size(egui::vec2(800.0, 600.0))
        .build_ui_state(
            move |ui, open: &mut bool| {
                theme.apply(ui.ctx());
                egui::CentralPanel::default().show(ui, |ui| {
                    for index in 0..20 {
                        ui.label(format!("Background row {index}"));
                    }
                });
                Dialog::new(egui::Id::new("perf-dialog"), "Confirm action").show(
                    ui.ctx(),
                    open,
                    |ui| {
                        ui.heading("Confirm action");
                        ui.label("Review the details before continuing.");
                        ui.label("This action cannot be undone.");
                        ui.horizontal(|ui| {
                            Button::new("Continue", &theme).show(ui);
                            Button::new("Cancel", &theme)
                                .variant(ButtonVariant::Secondary)
                                .show(ui);
                        });
                    },
                );
            },
            true,
        );
    let (samples, pixels) = bench(&mut harness, |harness, _| {
        *harness.state_mut() = true;
    });
    report("dialog-overlay", &samples, pixels)
}

struct FormState {
    name: String,
    email: String,
    accepted: bool,
    enabled: bool,
    volume: f64,
    choice: Option<egui::Id>,
}

fn form_page() -> String {
    let theme = Theme::light();
    let options = [
        SelectOption::new(egui::Id::new("perf-red"), "Red"),
        SelectOption::new(egui::Id::new("perf-blue"), "Blue"),
    ];
    let mut harness = Harness::builder()
        .with_size(egui::vec2(800.0, 600.0))
        .build_ui_state(
            move |ui, state: &mut FormState| {
                theme.apply(ui.ctx());
                egui::CentralPanel::default().show(ui, |ui| {
                    Header::new("Account settings", &theme).show(ui, |_| {});
                    TextField::new(&mut state.name, &theme, "Name")
                        .hint("Enter your name")
                        .show(ui);
                    TextField::new(&mut state.email, &theme, "Email")
                        .hint("name@example.com")
                        .show(ui);
                    Checkbox::new(&mut state.accepted, &theme, "Accept terms").show(ui);
                    Switch::new(&mut state.enabled, &theme)
                        .label("Enable notifications")
                        .show(ui);
                    Slider::new(&mut state.volume, &theme)
                        .expect("value is normalized")
                        .label("Volume")
                        .show(ui);
                    Select::new(
                        egui::Id::new("perf-select"),
                        "Color",
                        "Choose a color",
                        &options,
                        &mut state.choice,
                    )
                    .show(ui);
                    Badge::new("Preview", &theme).show(ui);
                    Progress::determinate(&theme, 0.6)
                        .expect("valid constant")
                        .label("Saving")
                        .show(ui);
                    if Button::new("Save changes", &theme).show(ui).clicked() {
                        state.accepted = !state.accepted;
                    }
                });
            },
            FormState {
                name: "Ada".to_owned(),
                email: "ada@example.com".to_owned(),
                accepted: true,
                enabled: false,
                volume: 0.42,
                choice: Some(egui::Id::new("perf-blue")),
            },
        );
    let (samples, pixels) = bench(&mut harness, |harness, frame| {
        harness.state_mut().volume = 0.4 + (frame % 20) as f64 / 100.0;
    });
    report("form-page", &samples, pixels)
}

fn popover_menu() -> String {
    let theme = Theme::light();
    let mut harness = Harness::builder()
        .with_size(egui::vec2(600.0, 480.0))
        .build_ui_state(
            move |ui, open: &mut bool| {
                theme.apply(ui.ctx());
                egui::CentralPanel::default().show(ui, |ui| {
                    let anchor = ui.button("Options");
                    Popover::new(egui::Id::new("perf-popover"), "Options", open)
                        .focus_on_open(false)
                        .show(&anchor, |ui| {
                            for index in 0..20 {
                                let _ = ui.button(format!("Action {index}"));
                            }
                            ui.button("Apply")
                        });
                });
            },
            true,
        );
    let (samples, pixels) = bench(&mut harness, |harness, _| {
        *harness.state_mut() = true;
    });
    report("popover-20-actions", &samples, pixels)
}

fn main() {
    let scenarios = [
        item_list(),
        tile_list(),
        pagination_large(),
        dialog_overlay(),
        form_page(),
        popover_menu(),
    ];
    println!(
        "PERF_JSON {{\"environment\":{},\"scenarios\":[{}]}}",
        environment(),
        scenarios.join(",")
    );
}
