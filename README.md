# Temivu

**An experimental native Rust/egui port of Forui components.**

[English](README.md) · [Tiếng Việt](README.vi.md) · [Русский](README.ru.md) · [中文](README.zh.md)

> **Status: early experiment, not production-ready.** Temivu is an independent
> study of how Forui's design language can translate to immediate-mode Rust and
> egui. All 44 upstream widget modules have a preliminary port and the bundled demo
> exercises every one of them, but no component is verified for full upstream
> parity and there is no stable release yet. Please read
> [Known limitations](#known-limitations) before using it.

![A representative component sheet rendered by the visual-review helper](docs/images/overview.png)

## Why I built this

I like Forui's visual language, but I write native desktop software in Rust. I
started Temivu to see whether I could get the same controls — buttons, forms,
selects, calendars, overlays, navigation — with the same spacing, colour tokens,
typography, keyboard behaviour and accessibility, without embedding Flutter.
The project is a faithful *study*, not a reimplementation of Forui's engine: it
adapts Forui's public tokens and behaviour to egui's immediate-mode model, and
the caller owns all persistent state.

I developed the port with an AI coding assistant (DeepSeek Harness) doing much of
the mechanical drafting and verification. I set the scope, decisions and review
bar, and I am responsible for what is published here.

## What is in the box

- **All 44 upstream Forui widget modules** (preliminary), plus three first-party
  extensions — accordion, alert, autocomplete, avatar,
  badge, bottom navigation, breadcrumb, button, calendar, card, checkbox,
  context menu, date field, date-time picker, dialog, divider, header, item,
  field label, line calendar, OTP field, pagination, picker, popover, popover
  menu, progress, radio, resizable, scaffold, select, multi-select, select group,
  select menu tile, select tile group, sheet, sidebar, slider, range slider, switch,
  tabs, text field, tile, time field, time picker, toast, toaster and tooltip.
- **A neutral design-token theme** (light, dark, touch scale, reduced motion and
  an opt-in high-contrast preset) with typography and size tokens.
- **Localization infrastructure** — 37 semantic messages generated from 113
  upstream ARB files, with English fallback and locale-subtag matching.
- **A five-page native demo** — `cargo run --features demo --example showcase` —
  covering every component family, themes, reduced motion and localization.
- **Tests** — 402 Rust tests, including real-input `egui_kittest` interaction
  tests for every component family, accessibility assertions, deterministic
  date/time logic and theme contrast checks.

![Calendar, select and toast examples](docs/images/calendar.png)

| Area | What to expect today |
| --- | --- |
| Theme & tokens | Stable enough to experiment with; colours and sizes adapted from Forui, high-contrast preset included |
| Inputs & buttons | AccessKit semantics and keyboard paths covered by kittest; not yet tested with a screen reader; icons are text-glyph approximations |
| Selects, pickers, autocomplete | Preliminary search, multi-select, calendar month/year picker, wheel picker, time/date pickers |
| Overlays | Dialog, sheet, popover, popover menu and toast/toaster work, but animations and gestures are incomplete |
| Visual parity | Some aspects are deliberately closer to upstream, but parity is not measured or proven; fonts, icons, blur and Flutter text metrics remain gaps |
| Accessibility | AccessKit roles/labels/values and keyboard paths are covered by tests; no manual screen-reader audit yet |

## Quick start

Temivu requires Rust 1.95 or newer (edition 2024). It is not published to
crates.io yet; use it as a path or git dependency.

```toml
[dependencies]
# from git:
temivu = { git = "https://github.com/vutienthan/temivu" }
# from a local clone instead:
# temivu = { path = "../temivu" }
egui = "=0.36.2"
```

> If you clone the repository instead, the `path = "…"` line is unnecessary.

```rust
use temivu::{Button, Dialog, Header, Theme};

fn page(ui: &mut egui::Ui, dialog_open: &mut bool) {
    let theme = Theme::light();
    Header::new("Example", &theme).show(ui, |_| {});
    if Button::new("Show details", &theme).show(ui).clicked() {
        *dialog_open = true;
    }
    Dialog::new(egui::Id::new("details-dialog"), "Details")
        .show(ui.ctx(), dialog_open, |ui| {
            ui.label("Details go here.");
        });
}
```

Call `Theme::apply` when the application theme changes. Persistent state is
caller-owned: keep it across frames and call each widget every frame, including
closed overlays, so dialogs and sheets can restore focus. IDs must be stable and
unique.

Run the demo on a supported graphical desktop:

```bash
cargo run --features demo --example showcase
```

Run the test suite (no display required):

```bash
cargo test --all-features
```

## Project layout

```text
src/               the library (47 widget modules plus foundation)
tests/             unit and egui_kittest interaction tests
examples/          the five-page native showcase
docs/              component notes, coverage matrix, migration and design docs
harness/port/      generated-token and localization extractors/verifiers
harness/visual-review/  off-screen screenshot helper, baselines and comparer
branding/          provisional project identity
third_party/       upstream license texts
```

The full internal QA harness I used while building this (CodeQL snapshotting,
independent-review bookkeeping, dependency/license inventories) is intentionally
not part of the public repository; the reusable generators and the visual-review
helper are included.

## Verification

I verify changes in layers: formatting, Clippy with warnings denied, the full
test suite and documentation tests on every change; CodeQL, an independent review
and a dependency audit at integration milestones; and rendered screenshots for
components whose visuals changed. The current state includes:

- 402 passing Rust tests and passing documentation tests;
- the initial GitHub Actions runs on `main` passed the quality and CodeQL
  workflows (the dependency job reports the known advisories without blocking);
- a CodeQL scan with clean extraction and only two reviewed false positives
  (boolean password-masking flags);
- a history scan with Gitleaks and no leaked credentials;
- reviewed light/dark component sheets in `harness/visual-review/baselines/`;
- documented dependency advisories in [docs/DEPENDENCY_ADVISORIES.md](docs/DEPENDENCY_ADVISORIES.md).

`cargo-deny` currently reports three *informational unmaintained* advisories
(`zvariant`, `zbus_names`, `zvariant_derive`) in the optional Linux accessibility
chain. They are not suppressed: no compatible upgrade or maintained replacement
exists as of November 2026, accessibility remains enabled, and the issue is
tracked openly. The GitHub workflows in `.github/workflows/` run the same
quality, dependency and CodeQL checks; the dependency job is allowed to fail so
it reports rather than hides this issue.

## Known limitations

- **Not production-ready.** APIs, visuals and behaviour will change; there is no
  release yet.
- **Upstream parity is unproven.** Fonts, vector icons, backdrop blur,
  superellipse corners, Flutter text metrics and animation curves differ from
  Forui. Screenshots in this repository are reviewed against previous versions of
  the project, not against upstream golden images.
- **Accessibility has automated coverage only.** AccessKit roles, labels, values,
  keyboard paths and contrast ratios are tested; no manual screen-reader or
  forced-colours audit has been performed.
- **Overlays are incomplete:** entrance/exit animation, swipe gestures and some
  focus edge cases remain.
- **Localization is partial:** 37 messages are generated; dates, times, numbers,
  plurals and RTL are not implemented.
- **Continuous integration is configured but the demo has not been exercised in a
  desktop session here.** `.github/workflows/` runs quality, CodeQL and secret scans on
  pushes; the latest results are in the Actions tab.

## Contributing

I would love this to become a community project, and I welcome help. You do not
need to be a Rust expert or know Forui in detail — curiosity is enough. Useful
ways to help:

- **Visual parity:** compare a component against the pinned Forui revision
  (`eea1e8e651d263f14ef43897ccbb7a2576549da5`) and describe differences.
- **Accessibility:** keyboard, screen-reader, forced-colours and contrast reviews.
- **Localization:** date/time formats, plurals, RTL and more locales.
- **Performance:** profiling, frame-time measurements and allocation hot spots.
- **Dependencies:** investigate a maintained replacement for the accessibility
  advisory chain — with tests and license review, never just to silence a warning.

Start with [CONTRIBUTING.md](CONTRIBUTING.md) and the issue tracker. If you are
unsure where to begin, open a discussion or ask for a "good first issue" — I will
politely help you find something that fits your interests. Please also read the
[Code of Conduct](CODE_OF_CONDUCT.md). Be kind; this is a learning project.

![Multi-select and toaster examples](docs/images/multi-select.png)

## Roadmap to 1.0

1. **Alpha** — all in-scope modules have tested APIs, the demo builds and runs,
   and documented behaviour gaps are tracked in [docs/COVERAGE.md](docs/COVERAGE.md).
2. **Beta** — accessibility baseline met, visual matrix compared with measurable
   thresholds, performance budgets defined on a documented reference host, and
   docs rewritten for users rather than implementers.
3. **1.0** — release gates pass with documented limitations accepted: no open
   high-severity advisories, complete notices, reproducible builds, and a stable
   public API. The detailed scope is in [docs/ROADMAP.md](docs/ROADMAP.md); feedback on it
   is very welcome.

## License and attribution

Temivu is MIT licensed — see [LICENSE](LICENSE). It is an independent project, **not
affiliated with or endorsed by Forui or Duobase IO**. Forui is MIT licensed; its
license text and the license texts of other referenced packages and fonts are
reproduced in [THIRD_PARTY_NOTICES.md](THIRD_PARTY_NOTICES.md) and `third_party/`.
The name, logo and mark are provisional and have not been trademark-cleared; see
[BRANDING.md](BRANDING.md).
