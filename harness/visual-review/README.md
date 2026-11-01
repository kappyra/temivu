# Off-screen visual review

An isolated helper crate that renders representative component sheets off-screen and
writes light/dark (and 150% zoom, where useful) PNGs. It is used to review visual
changes and to reproduce the reviewed baselines committed in `baselines/`.

```bash
TEMIVU_VISUAL_OUT=harness/visual-review/out cargo run --manifest-path harness/visual-review/Cargo.toml --release
python3 harness/visual-review/compare.py harness/visual-review/out/<timestamp>
```

Requirements: a working off-screen graphics adapter (wgpu; the original environment
used Mesa's software Vulkan/lavapipe), Rust 1.95+ and network access on the first
build for crates. `compare.py` additionally needs Pillow.

The helper covers representative controls in light/dark (and touch/high-contrast
sheets), not every state, 150% zoom for every component, or upstream golden images.
Image review must be recorded separately; successful rendering alone does not pass
visual parity. See `docs/VISUAL_PARITY.md` for the current comparison against the
pinned Forui revision.
