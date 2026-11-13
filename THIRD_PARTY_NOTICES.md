# Third-party notices

## Forui

Temivu adapts the neutral palette, typography, sizing, breakpoint and motion
tokens, selected widget behavior, and 37 localization messages from 113 Forui
ARB locale files. Forui is copyright (c) 2026 Duobase IO. The full
[Forui MIT notice](third_party/forui/LICENSE-MIT) accompanies these adaptations.

The notice matches the MIT section of both the [root license](https://github.com/duobaseio/forui/blob/eea1e8e651d263f14ef43897ccbb7a2576549da5/LICENSE)
and the [widget package license](https://github.com/duobaseio/forui/blob/eea1e8e651d263f14ef43897ccbb7a2576549da5/forui/LICENSE)
at revision `eea1e8e651d263f14ef43897ccbb7a2576549da5`. The Forui font
and icon bundles are excluded from this source distribution.

## Inter

Temivu bundles the unmodified [Inter variable font](third_party/inter/Inter.ttf)
from [Google Fonts](https://github.com/google/fonts/blob/e1d6480102fed30739fead0faee463101f892c8f/ofl/inter/Inter%5Bopsz%2Cwght%5D.ttf).
It is copyright 2020 The Inter Project Authors and remains under SIL Open Font
License 1.1. The [copyright and full license](third_party/inter/OFL.txt) are
included with the font.

## Showcase script fonts

Showcase embeds renamed, reduced glyph sets for its Japanese, Simplified Chinese,
and Arabic previews. The [source credits and SIL Open Font License 1.1 texts](third_party/showcase-fonts/)
accompany the subsets and are included in the Android package.

## egui default fonts

The locked dependency `epaint_default_fonts` 0.37.0 bundles Hack, Ubuntu Light and an icon font subset. The optional monochrome emoji fonts are included in the published crate. Its font license and attribution text files are reproduced in `third_party/egui-fonts/` from the published crate. These fonts are subject to their own terms; they are not relicensed as MIT. The complete transitive dependency and asset notice review is still outstanding.

The optional native showcase adds dependencies under the Boost Software License 1.0 and ISC License. They remain under their own licenses. Full transitive notice collection remains incomplete. The dependency audit also reports unmaintained `zbus_names`, `zvariant`, and `zvariant_derive` through the Linux accessibility stack; these findings are not suppressed and the dependency gate remains blocked until resolved.

The license expressions declared by the locked dependencies can be inspected with `cargo metadata --all-features --format-version 1`; declarations are not proof of license text or distribution compliance.

The collected upstream license texts for the cached dependency packages are stored
under `third_party/upstream-licenses/` (mapping, revisions and hashes in its
`README.md` and `provenance.json`). These copies cover the packages checked during
the port; they are not a complete distributable notice bundle for every transitive
dependency of the optional demo feature.

## Recorded gaps

- `dispatch` 0.2.0 declares MIT in its upstream manifest. The macOS preview
  bundles that manifest and the standard MIT text from SPDX. A standalone
  upstream license file and copyright line are still missing. See the
  [dependency notice](third_party/upstream-licenses/dispatch/0.2.0/README.md).
- The `objc2` family's upstream licensing notes raise an unresolved question about
  Apple-SDK-derived works; package manifests declare MIT or
  `Zlib OR Apache-2.0 OR MIT`. See `third_party/upstream-licenses/objc2-packages.json`
  and `docs/ASSETS.md`.
- The optional Linux accessibility chain carries three informational "unmaintained"
  advisories (RUSTSEC-2026-0370/0371/0372). They are not suppressed; accessibility
  remains enabled. See `docs/DEPENDENCY_ADVISORIES.md`.

This file and `docs/ASSETS.md` are provenance records, not legal advice or a
completed distribution review.
