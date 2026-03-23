# License And Contributor Policy

## Purpose

This document records the pre-implementation decision for repository licensing
and contributor expectations.

## Decision

- the repository is licensed under the MIT License
- inbound contributions are accepted under the same MIT terms by default
- contributors should not submit code or assets they are not authorized to
  relicense under MIT

## Why MIT

MIT is the best fit for the current project stage because it:

- keeps adoption friction low for artists, studios, researchers, and toolmakers
- works cleanly with the intended Rust, UE5, TouchDesigner, and web-oriented
  ecosystem mix
- keeps the repository legally usable before the implementation grows
- avoids delaying collaboration while the project is still architecture-heavy

## Contributor Expectations

- substantial work should start from an issue
- normative design changes should reference an ADR or propose one
- documentation changes for `docs/concepts/` and `docs/design/` should keep the
  English and Japanese pair in sync
- security-sensitive concerns should be reported privately as described in
  `SECURITY.md`

## Non-Goals

This decision does not yet define:

- a separate CLA process
- trademark policy
- dual licensing
- commercial support terms

Those can be added later if the project matures in that direction.

## Repository Files Affected

- [LICENSE](../../../LICENSE)
- [README.md](../../../README.md)
- [README.ja.md](../../../README.ja.md)
- [CONTRIBUTING.md](../../../CONTRIBUTING.md)
- [CONTRIBUTING.ja.md](../../../CONTRIBUTING.ja.md)

## Review Rule

If the project later considers relicensing, contributor policy changes, or a
CLA, that should be handled as a new explicit architecture and governance
decision rather than a silent repository edit.
