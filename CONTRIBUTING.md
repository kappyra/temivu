# Contributing to Temivu

Bug reports, documentation fixes and focused code changes are welcome. Open an
issue first for changes that affect the public API or multiple component families.

## Local checks

Rust 1.95 or later is required. Run the following commands from the repository root:

```bash
cargo fmt --all -- --check
cargo clippy --locked --all-targets --all-features -- -D warnings
cargo test --locked --all-features
```

For changes to layout or motion, run
`cargo run --locked --features demo --example showcase` and record the affected
theme, window size and interaction. Use `egui_kittest` when a change needs coverage
for keyboard input, focus or accessibility. Keep state caller-owned, use stable
widget IDs and respect reduced motion. The crate forbids `unsafe` code.

## Pull requests

Fork the repository and open a pull request against `main`. Only the maintainer
has direct write access and decides which pull requests to merge. Describe the
behavior change and the checks that ran. Include before-and-after screenshots or
a short recording for visual changes.

Contributions are accepted under the project’s [MIT license](LICENSE). Retain
applicable notices when proposing third-party code or assets. Follow the
[Code of Conduct](CODE_OF_CONDUCT.md) and report suspected vulnerabilities using
the [security policy](SECURITY.md).
