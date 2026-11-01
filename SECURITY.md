# Security policy

Temivu is an experimental project with no supported release yet. I still take
security reports seriously.

## Reporting a vulnerability

Please **do not** open a public issue for a suspected vulnerability. Report it
privately through GitHub's **Security → Advisories → Report a vulnerability** on
this repository, or contact the maintainer listed on the GitHub profile. Include:

- affected version or commit;
- a minimal reproduction or description;
- impact and any known mitigations;
- whether you would like to be credited.

I will acknowledge reports as soon as I can, investigate, and publish an advisory
and fix when appropriate. As this is a volunteer project, please allow reasonable
time for a response.

## Scope and known dependency advisories

- The crate forbids `unsafe` code.
- The optional Linux accessibility stack pulls in `zbus`/`zvariant`. `cargo-deny`
  reports three *informational unmaintained* advisories
  (`RUSTSEC-2026-0370/0371/0372`). They are not suppressed, and no compatible
  upgrade or maintained replacement exists as of November 2026. Accessibility
  remains enabled. See [docs/DEPENDENCY_ADVISORIES.md](docs/DEPENDENCY_ADVISORIES.md).
- This repository does not store credentials or secrets. History is scanned with
  Gitleaks in CI.
- The demo is a native application; it does not sandbox plugins, load remote
  content, or execute user-provided code.
- No sandbox is provided for application code using the library. Temivu is a UI
  library; security of the host application remains the host's responsibility.

## Supported versions

There is no stable release yet; security fixes currently land on `main`. A
supported-version policy will be published with the first release.
