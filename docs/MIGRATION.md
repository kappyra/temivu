# Porting notes

Temivu adapts the public surface of Forui to Rust and egui. This page records how
the pinned upstream revision was studied and what that means for the port.

## Pinned baseline

The reference is Forui 0.27.7 at
`eea1e8e651d263f14ef43897ccbb7a2576549da5`, not a moving branch. Its public surface
includes 44 widget modules plus foundation, theme, localization and assets.
`COMPONENTS.md` is a starting inventory, not a complete semantic specification.
There are 325 Dart source files under `lib/src`, a large test corpus and a set of
golden images used for visual comparison.

## Approach

1. Extract and freeze the design tokens (colours, radii, sizes, typography,
   breakpoints, motion) and the semantic localization messages. Small
   source-hash-keyed extractors live in `harness/port/`; the generated Rust tables
   are committed and verified in CI against the pinned checkout.
2. Map Flutter's widget model to egui's immediate-mode model. Persistent state
   (selection, visibility, focus, scroll) is caller-owned; widget IDs must be
   stable and unique; widgets are called every frame, including closed overlays so
   focus can be restored.
3. Port components in dependency order: foundation and theme, then small controls,
   composition/form fields, overlays and navigation, then calendars and pickers.
   Each port includes kittest interaction tests, accessibility assertions and a
   rendered sheet for visual review.
4. Study upstream source, tests and golden images for spacing, state, keyboard,
   focus, accessibility and animation. Port what egui can express; record gaps
   openly in `docs/COVERAGE.md` and the component notes instead of approximating
   silently.
5. Re-check the pinned revision before a parity pass; record measured differences
   in `docs/VISUAL_PARITY.md`. Rendering the project's own sheets proves
   reproducibility, not upstream parity.

## Deliberate differences

- egui has no Flutter layout engine, so text metrics, font weights, vector icons,
  backdrop blur and superellipse corners differ. These are tracked, not hidden.
- Some Forui animations cannot be represented directly in immediate mode; reduced
  motion is honored and animations are implemented only where they are stable.
- The crate forbids `unsafe` and keeps its dependency set deliberately small
  (`egui` at runtime, `egui_kittest` for tests, optional `eframe` for the demo).
- State is caller-owned rather than controller-managed, which fits egui idioms but
  changes the API compared with Forui.

## Remaining work

See `docs/COVERAGE.md` for the per-module gap list, `docs/ACCESSIBILITY.md` for the
accessibility audit, `docs/VISUAL_PARITY.md` for the parity comparison and
`docs/DEPENDENCY_ADVISORIES.md` for the known dependency issue.
