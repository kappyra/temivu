# Development helpers

This directory contains the reusable parts of the tooling used while porting
Temivu. The full internal QA harness (CodeQL snapshotting, independent-review
bookkeeping, dependency/license inventories and gate profiles) is intentionally not
published; the generator and visual-review tooling below is self-contained enough
to be useful to contributors.

## Design-token and localization generators

`harness/port/` extracts the pinned Forui tokens and messages, checks the committed
Rust tables against a clean upstream checkout, and records the small JSON IR used
for that verification:

```bash
# Fetch the pinned source once (any checkout at the pinned revision works):
git clone https://github.com/duobaseio/forui.git /tmp/forui-source
git -C /tmp/forui-source checkout eea1e8e651d263f14ef43897ccbb7a2576549da5

TEMIVU_UPSTREAM=/tmp/forui-source python3 harness/port/verify_sources.py
python3 harness/port/generate_localizations.py --check
python3 harness/port/generate_palette.py --check
```

The generators require Python 3 only. Do not edit the generated Rust tables by hand;
change the generator and regenerate.

## Off-screen visual review

`harness/visual-review/` is an isolated helper crate that renders representative
component sheets off-screen with wgpu and compares PNGs against the reviewed
baselines. It needs a working Vulkan/GL adapter (the original environment used Mesa
lavapipe) and the same Rust toolchain:

```bash
python3 -m pip install pillow   # needed by compare.py only

# Render every sheet into harness/visual-review/out/<timestamp>/:
TEMIVU_VISUAL_OUT=harness/visual-review/out cargo run --manifest-path harness/visual-review/Cargo.toml --release

# Compare a render directory with the reviewed baselines:
python3 harness/visual-review/compare.py harness/visual-review/out/<timestamp>
```

The comparison is strict: every rendered image must match a reviewed baseline byte
for byte. Rendering success alone does not prove visual parity with Forui; see
`docs/VISUAL_PARITY.md`.

## Tests for the helpers

```bash
python3 -m unittest discover -s harness -p 'test_*.py'
```

These tests cover the token/localization extractors, the generated-source verifier,
the upstream-provenance verifier and the visual comparison limits. They do not cover
the unpublished gate scripts.
