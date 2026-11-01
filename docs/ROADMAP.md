# Roadmap and 1.0 criteria

Temivu is an experimental port. This page defines what "done" means so progress can
be measured instead of argued about. The exact scope is open for discussion; please
open an issue or discussion if you disagree with a threshold.

## Scope for 1.0

- Pinned Forui revision `eea1e8e651d263f14ef43897ccbb7a2576549da5`, Rust 1.95+ and
  egui `0.36.2`.
- Linux desktop (X11 and Wayland) with desktop and touch size variants, light and
  dark themes, the opt-in high-contrast preset and reduced-motion support.
- English plus the existing localization infrastructure (37 semantic messages and
  locale fallback). Full ICU coverage (plurals, dates, times, numbers, RTL) may be
  declared out of scope with a documented limitation.
- All 44 upstream widget modules (47 first-party widget modules, including the
  MultiSelect, RangeSlider and Toaster extensions) have documented in-scope APIs, a
  working demo entry, tests and a visual reference sheet.

Explicitly out of scope unless separately approved: hardware-GPU performance
guarantees, formal screen-reader certification, full vector-icon/font-weight parity,
backdrop blur, and full RTL mirroring.

## Milestones

### Alpha — "usable and honest"

- [ ] Every in-scope module has a tested public API and a showcase entry.
- [ ] `cargo test --all-features` and documentation tests pass; Clippy is clean.
- [ ] No known panics on invalid public input; all rejected inputs return errors.
- [ ] `docs/COVERAGE.md` accurately lists every known behaviour/visual gap.
- [ ] CodeQL, Gitleaks and dependency checks are recorded with reviewed findings.

### Beta — "accessible and measurable"

- [ ] Every interactive widget exposes AccessKit role, label(s) and state, and is
  reachable and operable by keyboard; focus order and restoration are tested.
- [ ] Contrast ratios are computed and asserted for every text/surface token pair;
  normal text meets WCAG AA (4.5:1) or has a documented, user-visible exception.
- [ ] Reduced motion and the high-contrast preset are documented and tested.
- [ ] A visual matrix is defined (component × state × theme × scale) and compared
  with the pinned upstream goldens using explicit thresholds (e.g. perceptual
  similarity or pixel-difference limits); accepted deviations are listed.
- [ ] Performance budgets are defined on a documented reference machine (frame-time
  p95, render time, memory, no unbounded growth over 600 frames) and measured by a
  committed script.
- [ ] A manual accessibility checklist is completed or its absence is recorded as an
  accepted limitation.

### 1.0 — "release-ready for an experimental library"

- [ ] All above gates pass, or every remaining blocker is explicitly accepted and
  documented as a limitation of the release.
- [ ] Dependency policy satisfied: no unpatched vulnerability reachable from the
  library/demo; informational unmaintained advisories documented with an owner,
  review date and replacement plan.
- [ ] License/notice bundle complete for the distributed source and demo, or gaps
  explicitly accepted; accessibility remains enabled and nothing is suppressed.
- [ ] Reproducible build instructions, changelog, semver-stable public API and a
  final independent review of the frozen release snapshot.
- [ ] The demo runs on a real desktop session and the result is recorded.

## Tracking

Progress lives in `docs/COVERAGE.md` (per-module gaps) and the component notes under
`docs/`. Issues labelled `roadmap` group remaining work by milestone; a contribution
is welcome at any stage.
