# Changelog

All notable changes to this project will be documented in this file. The project
is experimental until 1.0; APIs may change between releases.

## [0.0.0] - 2026-11-01

### Added

- First public snapshot of the experimental Temivu library.
- Preliminary ports of 44 Forui widget modules and a neutral design-token theme
  (light, dark, touch scale, reduced motion, opt-in high contrast).
- Localization infrastructure generated from 113 upstream ARB files (37 messages).
- Five-page native `showcase` demo covering every component family.
- 402 Rust tests including `egui_kittest` interaction and accessibility tests,
  deterministic date/time logic, and theme contrast checks.
- Reviewed light/dark component baselines and an off-screen visual-review helper.
- Documentation for components, coverage, accessibility, migration, visual parity
  and dependency advisories.
- GitHub Actions workflows for quality, CodeQL and secret scanning.

### Known issues

- Visual and behavioural parity with upstream Forui is not proven.
- Accessibility has automated coverage only; no manual screen-reader audit.
- Overlay animations, gestures and parts of localization/localization formats are
  incomplete.
- `cargo-deny` reports three informational unmaintained advisories in the optional
  Linux accessibility chain; no compatible replacement is currently available.

[0.0.0]: https://github.com/vutienthan/temivu/releases/tag/v0.0.0
