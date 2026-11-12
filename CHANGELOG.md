# Changelog

All notable changes to this project will be documented in this file. The project
is experimental until 1.0; APIs may change between releases.

## [0.1.3] - 2026-11-12

- Improved Vesper contrast on pale surfaces and maintained pointer and touch
  feedback throughout a hold. Nearby light can be disabled separately.
- Added reusable effect surfaces, configurable Droplet and press primitives,
  and spring-driven foreground placement for custom egui content.
- Added Soft Prism alongside Prism, refined Frost, and made the glass samples
  draggable with adjustable tint, blur, refraction, and rim.
- Refined safe-area navigation, focus outlines, calendar and time controls,
  paging, and multilingual display in Showcase.

## [0.1.2] - 2026-11-07

- Notification previews show a `+N` count for entries beyond the preview limit.
  The notification center lists the full queue and supports individual dismissal
  and clearing multiple entries.
- Notification cards and the center adapt to desktop and touch layouts. Short
  messages keep the center compact.
- Bottom sheets float above the viewport with rounded corners and clearance for
  keyboard focus rings.
- Droplet returns to the latest point on the trigger when a second press closes
  an anchored popup. Single-line fields center text and hints vertically.

## [0.1.1] - 2026-11-07

- Droplet opens and closes anchored popovers and dialogs at the point where the
  trigger was clicked or tapped. Keyboard activation starts at the center.
- Showcase pages switch immediately. Search keeps its stretch on opening and
  closes into a round button while the text and close icon fade out.
- Calendar navigation, date fields, notification cards and bottom sheets
  adapt to narrow viewports. The sheet supports drag dismissal.
- The Android Showcase can be built for ARM64 and x86-64.

## [0.1.0] - 2026-11-05

### Added

- Spring-based press feedback, shape transitions and shared-source navigation
  animations. Bloom and Droplet provide two opening and closing motion presets.
- Responsive drawer and floating navigation, a floating bottom bar with
  press-and-drag selection, and an interactive widget catalog in the showcase.
- Video player controls for seeking, volume, playback speed and fullscreen.
  The optional Linux showcase includes a local GStreamer playback backend, a
  file chooser and keyboard submission of local paths.
- Segments, a quantity stepper, directional controls and circular progress.

### Changed

- The showcase defaults to monochrome light and dark themes. Mint remains an
  opt-in preset.
- Reworked controls, calendar and date inputs, collection and paging layouts,
  tabs, dialogs and other overlay surfaces for more consistent spacing and motion.
- Refreshed the English documentation, identity and showcase media.
- Retired the development Harness from the active source tree. The first release
  and its historical source remain available under the original tag.

### Verification scope

- Native showcase verified on Linux. Web, mobile and manual screen-reader
  validation remain open.
- The three informational Linux accessibility dependency advisories remain open.
  See [dependency advisories](docs/DEPENDENCY_ADVISORIES.md).

## [0.0.0] - 2026-11-01

### Added

- First public snapshot of the experimental Temivu library.
- All 44 upstream Forui widget modules ported (47 first-party widget modules, including MultiSelect, RangeSlider and Toaster) and a neutral design-token theme
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

[0.0.0]: https://github.com/kappyra/temivu/releases/tag/v0.0.0
[0.1.0]: https://github.com/kappyra/temivu/releases/tag/v0.1.0
[0.1.1]: https://github.com/kappyra/temivu/releases/tag/v0.1.1
[0.1.2]: https://github.com/kappyra/temivu/releases/tag/v0.1.2
[0.1.3]: https://github.com/kappyra/temivu/releases/tag/v0.1.3
