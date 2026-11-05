# Third-party notices

Temivu adapts neutral palette values, selected widget behavior and 37
localization messages from 113 Forui ARB locale files. The reference is
[Forui at revision `eea1e8e`](https://github.com/duobaseio/forui/tree/eea1e8e651d263f14ef43897ccbb7a2576549da5),
copyright (c) 2026 Duobase IO.

## Forui MIT license

Forui

MIT License

Copyright (c) 2026 Duobase IO

Permission is hereby granted, free of charge, to any person obtaining a copy
of this software and associated documentation files (the "Software"), to deal
in the Software without restriction, including without limitation the rights
to use, copy, modify, merge, publish, distribute, sublicense, and/or sell
copies of the Software, and to permit persons to whom the Software is
furnished to do so, subject to the following conditions:

The above copyright notice and this permission notice shall be included in all
copies or substantial portions of the Software.

THE SOFTWARE IS PROVIDED "AS IS", WITHOUT WARRANTY OF ANY KIND, EXPRESS OR
IMPLIED, INCLUDING BUT NOT LIMITED TO THE WARRANTIES OF MERCHANTABILITY,
FITNESS FOR A PARTICULAR PURPOSE AND NONINFRINGEMENT. IN NO EVENT SHALL THE
AUTHORS OR COPYRIGHT HOLDERS BE LIABLE FOR ANY CLAIM, DAMAGES OR OTHER
LIABILITY, WHETHER IN AN ACTION OF CONTRACT, TORT OR OTHERWISE, ARISING FROM,
OUT OF OR IN CONNECTION WITH THE SOFTWARE OR THE USE OR OTHER DEALINGS IN THE
SOFTWARE.



## Inter

The Inter font used in the showcase is distributed under the SIL Open Font
License 1.1. The bundled [font license and copyright notice](third_party/inter/OFL.txt)
accompany [Inter.ttf](third_party/inter/Inter.ttf). The font is sourced from the
[Inter project](https://github.com/rsms/inter).

## egui default fonts

The locked dependency `epaint_default_fonts` 0.36.2 bundles Hack, Noto Emoji, Ubuntu Light and an emoji icon font. Its font license and attribution text files are reproduced in `third_party/egui-fonts/` from the published crate. These fonts are subject to their own terms; they are not relicensed as MIT. The complete transitive dependency and asset notice review is still outstanding.

The optional native showcase adds dependencies under the Boost Software License 1.0 and ISC License. They remain under their own licenses. Full transitive notice collection remains incomplete. The dependency audit also reports unmaintained `zbus_names`, `zvariant`, and `zvariant_derive` through the Linux accessibility stack; these findings are not suppressed and the dependency gate remains blocked until resolved.

The license expressions declared by the locked dependencies can be inspected with `cargo metadata --all-features --format-version 1`; declarations are not proof of license text or distribution compliance.

The collected upstream license texts for the cached dependency packages are stored
under `third_party/upstream-licenses/` (mapping, revisions and hashes in its
`README.md` and `provenance.json`). These copies cover the packages checked during
the port; they are not a complete distributable notice bundle for every transitive
dependency of the optional demo feature.

## Recorded gaps

- `dispatch` 0.2.0 declares MIT but contains no license text or notice file at its
  recorded revision or current `master`. No text was fabricated; the record is in
  `third_party/upstream-licenses/dispatch/0.2.0/README.md`.
- The `objc2` family's upstream licensing notes raise an unresolved question about
  Apple-SDK-derived works; package manifests declare MIT or
  `Zlib OR Apache-2.0 OR MIT`. See `third_party/upstream-licenses/objc2-packages.json`
  and `docs/ASSETS.md`.
- The optional Linux accessibility chain carries three informational "unmaintained"
  advisories (RUSTSEC-2026-0370/0371/0372). They are not suppressed; accessibility
  remains enabled. See `docs/DEPENDENCY_ADVISORIES.md`.

This file and `docs/ASSETS.md` are provenance records, not legal advice or a
completed distribution review.
