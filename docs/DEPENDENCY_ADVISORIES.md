# Linux accessibility dependency advisories

Rechecked again on 1 November 2026 (second pass) using `cargo tree --offline --locked
--all-features -i ...`, cargo-deny 0.20.2 (`cargo deny --offline check`), the RustSec
advisory database, the offline crate/archive cache, the crates.io sparse index and API, and
the current upstream manifests and issues (local copies of the sources checked during the
investigation; access date 2026-11-01). `zbus 5.19.0`, `atspi 0.29.0`, `atspi-common 0.13.0`,
`atspi-proxies 0.13.0`, `accesskit_unix 0.21.1`, `accesskit_atspi_common 0.18.1`,
`zvariant 5.15.1`, `zvariant_derive 5.15.1` and `zbus_names 4.3.5` are reached only from the
optional native showcase (`demo` feature) via `eframe 0.36.2 -> egui-winit 0.36.2 ->
accesskit_winit 0.32.2 -> accesskit_unix -> atspi/zbus 5`, not from the library build or the
normal test suite. The advisories [RUSTSEC-2026-0370](https://rustsec.org/advisories/RUSTSEC-2026-0370.html),
[RUSTSEC-2026-0371](https://rustsec.org/advisories/RUSTSEC-2026-0371.html) and
[RUSTSEC-2026-0372](https://rustsec.org/advisories/RUSTSEC-2026-0372.html) are informational
"unmaintained" notices (reported 16 October 2026, issued 17 October 2026) with
`patched = []`. They record that the crates were merged into `zbus` 6.0.0 (`wire`, `names`
and re-exported derives) and identify no CVE, exploit or CVSS score. Because there are no
patched versions, every version of the three crates matches them; only removing those
crate identities from the graph (moving their dependents to zbus 6) resolves them.
`cargo deny --offline check` still fails on exactly these three findings; bans, licenses and
sources pass. The full all-features graph produced no other advisory findings in this run
(the upstream-mentioned `quick-xml` RUSTSEC-2026-0194 does not apply because the locked
`quick-xml` is the patched 0.41.0, and `crossbeam-epoch` RUSTSEC-2026-0204 is not in the
graph). This is not a vulnerability-free claim.

As checked on 1 November 2026, no released or `main`-branch version in the chain has migrated
to zbus 6. The newest published versions still require zbus 5: `accesskit_unix 0.24.1`
(published 25 October 2026) requires `atspi ^0.29` and `zbus ^5.19`;
`accesskit_atspi_common 0.21.1` requires `atspi-common ^0.13` and `zvariant ^5.15`;
`atspi 0.30.0` requires `atspi-common ^0.14.0` and (optionally) `zbus ^5.5`;
`atspi-common 0.14.0` requires `zbus ^5.5`, `zbus_names ^4.1.1` and `zvariant ^5.2`;
`accesskit_winit 0.34.2` requires `accesskit_unix ^0.24.1`, and `egui-winit 0.37.0` requires
`accesskit_winit ^0.34.0`. The AccessKit and AT-SPI `main` branches still declare zbus 5.19
and 5.5 respectively; AccessKit pull request #834 (merged 18 October 2026) explicitly defers
the migration ("zbus 6.0 has an MSRV of 1.89 so we can't upgrade quite yet. Ignores these
advisories for now") and zbus issue #2043 records that 5.15.1/5.15.1/4.3.5 were the final
docs-only releases from the `zbus-5` branch. A direct project-level dependency on zbus 6
would not replace the dependents' zbus 5 requirements; there is no compatible offline
upgrade, because the local crate-archive cache contains only the locked versions (no zbus 6
or newer chain archives), and even an online coordinated upgrade would not remove the chain
today. A scratch `cargo update --offline -p zbus --precise 6.0.0` fails on
`accesskit_unix`'s `zbus = "^5.5"` requirement; whole-graph and per-package offline updates
lock 0 packages (only an unrelated `windows-sys` lock-edge churn).

Replacement with a maintained alternative was reassessed on 1 November 2026 under the
permitted-replacement policy and found not suitable. The role of the chain is the standard
AccessKit-on-Linux path: `accesskit_unix` publishes the AccessKit tree to the AT-SPI bus so
screen readers such as Orca can drive the native showcase; `atspi`/`atspi-common`/
`atspi-proxies` are the only maintained pure-Rust AT-SPI2 implementation and they require
zbus 5, while `zvariant`/`zbus_names`/`zvariant_derive` are their D-Bus wire/name/derive
layer. AccessKit's `main` branch has no Linux adapter other than `accesskit_unix` plus
`accesskit_atspi_common`. No maintained drop-in alternative provides the same AT-SPI
functionality: `aad-atspi 0.0.3` and `perspicax-atspi 0.1.0` are brand-new, low-download
crates with different roles (and still use zbus 5), and `atspi-client 0.1.0` is stale.
The `dbus 0.9.12` binding is maintained (Apache-2.0/MIT) but provides only D-Bus transport,
not the AT-SPI protocol, so using it would require writing a new AT-SPI client and AccessKit
translation layer; it also adds the C `libdbus` library (AFL-2.1/GPL-2.0+ dual-licensed) and
is not in the offline cache. Disabling `accesskit`/accessibility, switching `all-features`
off, patching to hide registry sources, or faking versions would all weaken or hide the gate
and remain rejected. The chain therefore stays with accessibility enabled, the advisories
unsuppressed and the dependency gate failing; the smallest correct future change is the
coordinated upstream-led upgrade described below, which also needs a refreshed audited crate
cache and screen-reader/AT-SPI testing (unavailable on this host).

Reassess when AccessKit or AT-SPI publish versions requiring zbus 6 (watch
[AccessKit](https://github.com/AccessKit/accesskit),
[atspi](https://github.com/odilia-app/atspi) and
[zbus #2043](https://github.com/z-galaxy/zbus/issues/2043)). At that point the change is a
coordinated bump of `egui`/`egui_kittest`/`eframe`/`egui-winit`/`accesskit_winit` plus a
lockfile update, license-inventory regeneration, the integration profile, UI/visual checks,
a new CodeQL scan (a Cargo change invalidates the recorded snapshot) and real AT-SPI
screen-reader testing. This assessment does not establish that the dependency chain is
vulnerability-free.

License checks allow ISC and Boost-1.0 for applicable transitive dependencies; those remain
under their own terms and complete notice review remains open. The existing epaint font
license texts are retained separately. Full dependency and asset license compliance has not
yet been established. If a replacement were ever adopted, its license/notice picture would
need the same inventory and notice review; a `dbus`/`libdbus` route would add
AFL-2.1/GPL-2.0+ terms, while newer egui/AccessKit versions would change the collected
version set.
