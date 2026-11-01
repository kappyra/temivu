# Contributing to Temivu

Thank you for being here. Temivu is an early experiment and I would genuinely
love help — from code and reviews to screenshots, translations and bug reports.
You do not need to be a Rust expert or know Forui; curiosity is enough.

## Before you start

- Read the [README](README.md) and the [Code of Conduct](CODE_OF_CONDUCT.md).
- Look through [docs/COVERAGE.md](docs/COVERAGE.md) and
  [docs/COMPONENTS.md](docs/COMPONENTS.md) to see what is preliminary.
- Open an issue or a discussion before a large change, so we can agree on scope
  and avoid duplicate work. Small fixes and documentation can go straight to a PR.

New here? Ask for a **good first issue**; I will happily point you to something
that matches your interests.

## Development setup

Temivu requires Rust 1.95 or newer (edition 2024). The only runtime dependency is
`egui`; tests use `egui_kittest`; the optional demo uses `eframe`.

```bash
git clone https://github.com/vutienthan/temivu
cd temivu
cargo check --all-features
cargo test --all-features
cargo run --features demo --example showcase   # requires a desktop
```

Formatting and lints are mandatory:

```bash
cargo fmt --check
cargo clippy --all-targets --all-features -- -D warnings
cargo test --all-features
```

Please keep `unsafe` out of the crate. The project follows a caller-owned-state
model: widgets receive `&mut` state, IDs must be stable, and widgets are called
every frame (including when an overlay is closed).

## What to contribute

- **Visual parity:** compare a component with Forui at
  `eea1e8e651d263f14ef43897ccbb7a2576549da5` and describe differences with
  screenshots and the theme/size/zoom used.
- **Accessibility:** keyboard paths, AccessKit semantics, focus restoration,
  contrast and reduced-motion review. Automated `egui_kittest` assertions are
  especially welcome.
- **Localization:** dates, times, numbers, plurals, RTL and additional locales.
- **Performance:** frame-time measurements, allocation hot spots and profiling on
  a documented machine.
- **Dependencies:** investigate the advisory chain in
  [docs/DEPENDENCY_ADVISORIES.md](docs/DEPENDENCY_ADVISORIES.md). A replacement
  must keep accessibility working, pass tests, keep the license review honest and
  never merely silence a warning.
- **Docs/demo:** corrections, clearer examples and new demo pages are valued.

## Tests and evidence

- Add or update tests with every behaviour change. Prefer `egui_kittest` for
  interaction, focus, keyboard and accessibility behaviour.
- For visual changes, run the off-screen sheet helper if your environment supports
  it (see `harness/visual-review/` (requires `python3 -m pip install pillow` for the comparison script)) and mention what you compared. If you cannot
  render, say so explicitly in the PR — that is fine, not a failure.
- Update the relevant `docs/*.md` and `docs/COVERAGE.md` when you close a gap or
  discover a new limitation. Honest documentation is more valuable than a green
  checkbox.

## Pull requests

- Keep changes focused; separate mechanical refactors from behaviour changes.
- Describe the motivation, the behaviour change and the tests you ran.
- Include screenshots for visual changes when possible.
- CI runs formatting, Clippy, tests, documentation tests, CodeQL and a history
  secret scan. The dependency-audit job is expected to report the known
  accessibility advisories; it is allowed to fail visibly.
- By contributing you agree that your contribution is licensed under the project
  MIT license (see [LICENSE](LICENSE)).

## Communication

Be kind, assume good intent, and help each other learn. Harassment and
disrespect are not acceptable; see the [Code of Conduct](CODE_OF_CONDUCT.md).
For security issues, please **do not open a public issue** — follow
[SECURITY.md](SECURITY.md) instead.

Thank you for considering a contribution. Every report, screenshot, translation
and review makes this project better.
