# Upstream license text provenance

These texts were retrieved from upstream GitHub repositories at the Git revisions recorded by the corresponding locked crates' `.cargo_vcs_info.json` files in the local Cargo cache. URLs and SHA-256 hashes identify the copies; metadata alone does not prove the packaged crate exactly matches the repository revision. These records are not a complete third-party license, copyright, asset or distribution audit.

## egui family 0.36.2

The cached `ecolor`, `eframe`, `egui`, `egui-wgpu`, `egui-winit`, `egui_glow`, `egui_kittest`, `emath`, `epaint` and `epaint_default_fonts` 0.36.2 packages record revision `49682f8baa058bf49e011035cfbd6e825f88a5ef` of <https://github.com/emilk/egui>.

- [MIT text](https://github.com/emilk/egui/blob/49682f8baa058bf49e011035cfbd6e825f88a5ef/LICENSE-MIT): `egui/LICENSE-MIT`, SHA-256 `95ca92f5f8ea5231f1580b3a2a799e8260af3114b900e1def5355a7f44bcf60c`.
- [Apache text](https://github.com/emilk/egui/blob/49682f8baa058bf49e011035cfbd6e825f88a5ef/LICENSE-APACHE): `egui/LICENSE-APACHE`, SHA-256 `8173d5c29b4f956d532781d2b86e4e30f83e6b7878dce18c919451d6ba707c90`.

The default-font package declares additional font terms; its separate font notices are retained in `third_party/egui-fonts/` and still require complete asset review.

## AccessKit family

The following cached packages record revisions of <https://github.com/AccessKit/accesskit>:

- `a55d3e1a18bb9ef0e4bccc9083fb13c3e0ad8969`: `accesskit` 0.24.1.
- `f40dfc01a0c0e76de535969f82fb35e19513737d`: `accesskit_atspi_common` 0.18.1, `accesskit_consumer` 0.36.0, `accesskit_unix` 0.21.1.
- `1bbcf100942bac96c2c3a4a91cb67b0b20201a24`: `accesskit_consumer` 0.35.0, `accesskit_windows` 0.32.1, `accesskit_winit` 0.32.2.
- `c88605b96d04431f9c3c792464a0f2f253480e94`: `accesskit_consumer` 0.38.0 and `accesskit_macos` 0.26.3.

For each listed revision, `accesskit/<revision>/LICENSE-MIT` and `LICENSE-APACHE` were copied from `https://github.com/AccessKit/accesskit/blob/<revision>/LICENSE-MIT` and `LICENSE-APACHE`, respectively. All four MIT copies have SHA-256 `23f18e03dc49df91622fe2a76176497404e46ced8a715d9d2b67a7446571cca3`, and all four Apache copies have SHA-256 `62c7a1e35f56406896d7aa7ca52d0cc0d272ac022b5d2796e7d6905db8a3636a`. Check each package's declared expression; in particular, `accesskit_winit` 0.32.2 declares Apache-2.0 only.

The remaining packages without a matching root-level cached license file and their applicable notices are still unresolved.

## objc2 family

`objc2-packages.json` maps 27 cached packages without root-level license/notice files to the
Git revisions reported in their crate VCS metadata, the upstream root license document at that
revision, and its SHA-256. All eight distinct collected files were re-checked byte-for-byte
against `raw.githubusercontent.com` at the recorded revisions on 2026-11-01 (three `LICENSE.txt`
copies with MIT text "Copyright (c) Steven Sheldon", sha256
`e353f37b12aefbb9f9b29490e837cfee05d9bda70804b3562839a3285c1df1e5`; five `LICENSE.md` copies
with the licensing description, sha256
`7f976f7e9cb2d87df7230606feb932c3f21ac0e664045a775b600046ff850c54`). Older revisions carry an
MIT text; newer revisions carry a `LICENSE.md` that describes package-dependent MIT or
Zlib/Apache/MIT options and links to standard license texts. The latter is an upstream licensing
description, not itself a full copy of each license text.

The upstream licensing description states that the crates are derived from Apple SDKs and raises
uncertainty about distribution of derived works
(<https://github.com/madsmtm/objc2/blob/7b1abfd750a2cacaea71d6a56ecfb83cb7de560b/LICENSE.md>).
Upstream issue <https://github.com/madsmtm/objc2/issues/23> (open) records a pending relicensing
intent. Later upstream `master` carries full `LICENSE-MIT.txt`, `LICENSE-ZLIB.txt` and
`LICENSE-APACHE.txt` files that are absent at the locked revisions; they were deliberately not
added here so the collection does not mix revisions. This is an upstream concern, not a legal
conclusion; the applicable terms, notices and asset provenance remain unresolved and need
review. Do not mark the license/asset gate passed from this inventory.

## Other packages

`other-packages.json` maps the remaining packages identified by the earlier root-file inventory to
their cached crate VCS revisions and the upstream root license/notice files retrieved at those
revisions. `r-efi` 5.3.0 and 6.0.0 include licensing and author details in an `AUTHORS` file; the
copies here were taken from the cached crates and checked byte-for-byte against the corresponding
upstream Git revisions.

A matching upstream license/notice filename does not exist for `dispatch` 0.2.0. Its `Cargo.toml`
and crates.io metadata declare MIT (author Steven Sheldon), but the packaged crate, the recorded
upstream revision `82d6c7a5b75dc0c71c3f46f87bb6c16a476f7748` and current upstream `master`
contain no license/notice/copyright file, the README has no license section, and the GitHub
license API returns no licence. No license text or copyright line can be collected legitimately,
and none was fabricated. A metadata notice record (`dispatch/0.2.0/README.md`) records the
finding; it is not a license text. The `dispatch` entry in `provenance.json` keeps `files: []` so
the provenance checker continues to report it as unresolved.

The local root-file inventory now also searches for `AUTHORS` and `CONTRIBUTORS`, which reduces
the packages without a matching root-level or declared file from 59 to 57. Such files may contain
only attribution and are not automatically complete license notices. The counts and collected
texts do not establish distribution compliance.

`provenance.json` consolidates the 59 packages identified by the original cache inventory and
records the revision and collected-file hashes for each. `harness/licenses/verify_upstream.py`
compares those records with locked Cargo metadata, cached crate VCS metadata and the retained
file bytes; it also explicitly reports the unresolved `dispatch` entry. This confirms consistency
of the recorded data, not the authenticity of each crate-to-revision mapping, completeness of
license texts, or legal compliance.
