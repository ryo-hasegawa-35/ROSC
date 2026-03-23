# ROSC

ROSC is a docs-first Rust project for a next-generation OSC routing bus and
message broker. The goal is to build a routing core that stays fast,
predictable, observable, recoverable, and backward-compatible with existing OSC
workflows even under heavy real-time pressure.

## Current Status

The repository is currently in the pre-implementation phase.

What exists now:

- a bilingual concept and planning stack
- a bilingual design specification set
- repository governance and delivery planning documents
- GitHub workflow and contribution rules for the next implementation phase
- an initial ADR set in English and Japanese
- pre-implementation conformance and benchmark fixture inventories
- cross-platform repository-sanity CI scaffolding

What does not exist yet:

- production Rust crates
- protocol adapters
- benchmark harness implementation
- native integrations

## Documentation Entry Points

- [Documentation Index](./docs/README.md)
- [Documentation Index (Japanese)](./docs/README.ja.md)
- [Repository README (Japanese)](./README.ja.md)
- [Concepts / Planning (English)](./docs/concepts/en/README.md)
- [Concepts / Planning (Japanese)](./docs/concepts/ja/README.md)
- [Design Specs (English)](./docs/design/en/README.md)
- [Design Specs (Japanese)](./docs/design/ja/README.md)

Recommended reading order:

1. [Design Reading Order](./docs/design/ja/reading-order.md)
2. [Glossary](./docs/design/ja/glossary.md)
3. [Implementation Readiness Checklist](./docs/design/ja/implementation-readiness-checklist.md)
4. [GitHub Foundation And Collaboration Plan](./docs/concepts/ja/github-foundation-and-collaboration-plan.md)

## Project Principles

- Preserve backward compatibility with existing OSC 1.0 traffic.
- Treat advanced behavior as additive overlays, not mandatory protocol changes.
- Keep the routing core deterministic and independently testable.
- Make observability and recovery first-class, not afterthoughts.
- Keep all core project documents available in both English and Japanese.

## Collaboration Rules

- `main` is for reviewed and approved changes only.
- Work should start from an Issue whenever practical.
- Significant changes should update the relevant design docs first.
- Pull requests should reference the affected issues and design documents.
- Final approval for changes landing on `main` is reserved for
  `@ryo-hasegawa-35`.

See also:

- [CONTRIBUTING.md](./CONTRIBUTING.md)
- [CONTRIBUTING.ja.md](./CONTRIBUTING.ja.md)
- [SECURITY.md](./SECURITY.md)
- [SECURITY.ja.md](./SECURITY.ja.md)
- [GitHub Foundation And Collaboration Plan](./docs/concepts/en/github-foundation-and-collaboration-plan.md)

## Source References

Primary OSC references are stored locally in:

- [OSC 1.0 Specification PDF](./docs/references/osc-1.0-specification.pdf)
- [OSC 1.1 NIME 2009 PDF](./docs/references/osc-1.1-nime-2009.pdf)

Additional online references:

- [OpenSoundControl.org](https://opensoundcontrol.stanford.edu/)
- [OSC 1.0 Spec Page](https://opensoundcontrol.stanford.edu/spec-1_0.html)
- [OSC 1.1 Spec Page](https://opensoundcontrol.stanford.edu/spec-1_1.html)

## License

This repository is licensed under the [MIT License](./LICENSE).

License rationale and contributor policy notes live in
[License And Contributor Policy](./docs/concepts/en/license-and-contributor-policy.md).
