<p align="center">
  <picture>
    <source media="(prefers-color-scheme: dark)" srcset="branding/temivu-wordmark-matted.png">
    <img src="branding/temivu-wordmark.png" alt="Temivu" width="448">
  </picture>
</p>

<p align="center"><strong>UI components with responsive layouts and spring-based motion for Rust and egui.</strong></p>

<p align="center">
  <a href="#overview">Overview</a> ·
  <a href="#demo">Demo</a> ·
  <a href="#quick-start">Quick start</a> ·
  <a href="#documentation">Documentation</a> ·
  <a href="CHANGELOG.md">Changelog</a> ·
  <a href="CONTRIBUTING.md">Contributing</a>
</p>

## Overview

Temivu helps developers build interfaces for egui applications with a consistent visual language and caller-owned state. Controls, navigation, overlays and data views share theme tokens and motion settings. The library can be integrated into existing egui layouts one widget at a time.

| Area | What the library provides |
| --- | --- |
| Controls | Buttons, switches, form inputs, selects, sliders, segments and progress indicators |
| Navigation | Floating bottom navigation, drawers, menus, tabs and shared-source page transitions |
| Overlays | Dialogs, sheets, popovers, tooltips and queued notifications |
| Data and media | Calendar and time controls, pagination, resizable panels and video player controls |
| Foundation | Monochrome and mint themes, touch sizing, localization and reduced-motion settings |

Temivu began as a [Rust/egui port](docs/MIGRATION.md) of Forui’s Dart/Flutter components. Most widgets now use Temivu’s own rendering, styling and motion. Remaining Forui-derived behavior is being replaced incrementally, and the Forui MIT notice remains included. The visual and motion design draws on [shadcn/ui](https://ui.shadcn.com/), [SwiftUI and Apple’s iOS 26 Liquid Glass sessions](https://developer.apple.com/videos/play/wwdc2025/219/), [iOS 27 navigation design](https://developer.apple.com/videos/play/wwdc2026/269/) and [Material 3 Expressive](https://design.google/library/expressive-material-design-google-research).

## Demo

https://github.com/user-attachments/assets/3a0d534d-143f-424b-b976-2acb06b394eb

[Watch the demo](https://github.com/user-attachments/assets/3a0d534d-143f-424b-b976-2acb06b394eb) · [Download MP4](https://raw.githubusercontent.com/kappyra/temivu/main/docs/demo/temivu_demo.mp4) · [Overview screenshot](docs/demo/overview.png) · [Widget catalog screenshot](docs/demo/catalog.png) · [Dialog screenshot](docs/demo/dialog.png)

The first recording walks through the widget catalog, overlays, data views and localization. The second demonstrates the video player’s timeline, playback speed and volume controls.

## Quick start

Rust 1.95 or later is required. Temivu uses egui 0.37.0. The library is available as source from [GitHub Releases](https://github.com/kappyra/temivu/releases/latest).

```bash
git clone https://github.com/kappyra/temivu.git
cd temivu
cargo run --locked --features demo --example showcase
```

Prebuilt Showcase previews are available in the **Artifacts** section of a successful
[Showcase builds run](https://github.com/kappyra/temivu/actions/workflows/showcase.yml).
Sign in to GitHub to download them. Linux and Windows previews target x86-64. The
Android APK supports arm64 and x86-64 and uses a new debug signing key for each run.
Uninstall an older preview before installing an APK signed with a different key.

The showcase needs a desktop graphics session. On Linux, to play a local video, open **Widgets → Media**. Select **Choose video…**, or enter a local filename and select **Open path** or press Enter. The chooser uses Zenity on Linux. Playback requires system Python with PyGObject, GStreamer 1.0 and plugins for the chosen media format.

The library’s `VideoPlayer` accepts a texture and emits playback actions for the host’s media backend.

Use a Temivu component in any egui view:

```rust
use temivu::{Button, Switch, Theme};

fn settings(ui: &mut egui::Ui, theme: &Theme, notifications: &mut bool) -> bool {
    Switch::new(notifications, theme)
        .label("Notifications")
        .show(ui);
    Button::new("Save changes", theme).show(ui).clicked()
}
```

Create a theme such as `Theme::light()` in the host, apply it to the egui context when the preference changes and keep application state between frames. Use stable widget IDs for stateful controls and transitions.

## Documentation

- [Component inventory and coverage](docs/COVERAGE.md)
- [Navigation menus and responsive presentation](docs/NAVIGATION_MENU.md)
- [Motion system and custom animation API](docs/MOTION.md)
- [Droplet motion](docs/LIQUID_MOTION.md)
- [Optional glass surfaces](docs/GLASS.md)
- [Floating surfaces and responsive geometry](docs/FLOATING_SURFACES.md)
- [Vesper interaction lighting](docs/VESPER.md)
- [Calendar API and keyboard behavior](docs/CALENDAR.md)
- [Accessibility and verification scope](docs/ACCESSIBILITY.md)
- [Changes since the first public version](CHANGELOG.md)

The showcase recording was captured on Linux. The showcase can also be [built for Android](examples/showcase/android/). Web validation remains open. The API is pre-1.0. The [dependency advisory record](docs/DEPENDENCY_ADVISORIES.md) tracks three open informational advisories in the optional Linux accessibility stack.

## License

Temivu is distributed under the [MIT License](LICENSE). The [third-party notices](THIRD_PARTY_NOTICES.md) preserve Forui attribution and the terms for bundled fonts and other assets. The [first public release](https://github.com/kappyra/temivu/releases/tag/v0.0.0) remains available as a [source ZIP](https://github.com/kappyra/temivu/releases/download/v0.0.0/temivu-v0.0.0-source.zip).

<p align="center"><img src="branding/temivu-mark.svg" alt="Temivu symbol" width="56"></p>
