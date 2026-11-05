# Security policy

Temivu is pre-1.0. Security fixes land on `main`.

## Private reports

Use GitHub’s [private vulnerability report form](https://github.com/kappyra/temivu/security/advisories/new)
if available, or contact the maintainer privately. Include the affected commit,
steps to reproduce, expected impact and any known mitigations.

## Scope

The library forbids `unsafe` code. Host applications are responsible for their
own input, permissions and media handling. The optional Linux demo opens local
media through a separate system Python and GStreamer process.

The [dependency advisory record](docs/DEPENDENCY_ADVISORIES.md) covers the
informational unmaintained advisories in the optional Linux accessibility stack.
GitHub Actions runs CodeQL and scans repository history for secrets.
