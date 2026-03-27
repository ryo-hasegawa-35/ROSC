# ROSC

ROSC is a docs-first Rust project for a next-generation OSC routing bus and
message broker. The goal is to build a routing core that stays fast,
predictable, observable, recoverable, and backward-compatible with existing OSC
workflows even under heavy real-time pressure.

## Current Status

The repository has entered the first implementation phase.

What exists now:

- a bilingual concept and planning stack
- a bilingual design specification set
- repository governance and delivery planning documents
- GitHub workflow and contribution rules for the next implementation phase
- an initial ADR set in English and Japanese
- pre-implementation conformance and benchmark fixture inventories
- cross-platform repository-sanity CI scaffolding
- a Rust workspace bootstrap for the Phase 01 core crates
- an initial OSC parser/encoder core with conformance tests
- initial route, config, and bounded-queue primitives for the broker core

What does not exist yet:

- production-ready runtime behavior
- protocol adapters
- benchmark harness implementation
- native integrations

## Getting Started

Run the current workspace locally:

```bash
cargo test --workspace
cargo run -p rosc-broker -- check-config examples/phase-01-basic.toml
cargo run -p rosc-broker -- proxy-status examples/phase-01-basic.toml
cargo run -p rosc-broker -- proxy-status examples/phase-01-basic.toml --safe-mode
cargo run -p rosc-broker -- watch-config examples/phase-01-basic.toml --poll-ms 1000 --fail-on-warnings
cargo run -p rosc-broker -- watch-udp-proxy examples/phase-01-basic.toml --poll-ms 1000 --ingress-queue-depth 1024 --health-listen 127.0.0.1:19191 --control-listen 127.0.0.1:19292 --fail-on-warnings --require-fallback-ready --safe-mode
cargo run -p rosc-broker -- diff-config examples/phase-01-basic.toml examples/phase-01-basic-changed.toml
cargo run -p rosc-broker -- serve-health 127.0.0.1:19191 --config examples/phase-01-basic.toml
cargo run -p rosc-broker -- run-udp-proxy examples/phase-01-basic.toml --health-listen 127.0.0.1:19191 --control-listen 127.0.0.1:19292 --fail-on-warnings --require-fallback-ready --safe-mode
curl -X POST http://127.0.0.1:19292/freeze
curl -X POST http://127.0.0.1:19292/routes/camera/isolate
curl -X POST http://127.0.0.1:19292/routes/restore-all
curl -X POST http://127.0.0.1:19292/destinations/udp_renderer/rehydrate
curl -X POST "http://127.0.0.1:19292/routes/camera/replay/sandbox_tap?limit=1"
curl http://127.0.0.1:19292/status
curl http://127.0.0.1:19292/report
curl http://127.0.0.1:19292/overrides
curl http://127.0.0.1:19292/signals
curl http://127.0.0.1:19292/signals?scope=problematic
curl http://127.0.0.1:19292/blockers
curl http://127.0.0.1:19292/history/operator-actions
curl http://127.0.0.1:19292/history/config-events
```

`--control-listen` is intentionally loopback-only. Bind it to `127.0.0.1`, `::1`, or another
local-only alias such as `localhost`; wildcard or externally reachable addresses are rejected.

Run the same workspace inside Docker:

```bash
docker compose run --rm rosc-dev cargo test --workspace
```

Development container entry points:

- [Docker Compose](./compose.yaml)
- [Devcontainer](./.devcontainer/devcontainer.json)

Current Phase 01 runtime coverage:

- OSC parser/encoder for strict, legacy-tolerant, and extended modes
- route matching with static address rename transforms
- bounded ingress queue and UDP ingress binding
- bounded per-destination egress workers with breaker-based isolation
- in-memory health/metrics export rendered in Prometheus text format
- minimal HTTP `/healthz` and `/metrics` endpoint for early local troubleshooting
- safe config diffing and last-known-good config apply semantics
- top-level UDP ingress / destination config with end-to-end localhost proxy relay coverage
- first safe late-joiner recovery path with route-level cache policy and bounded rehydrate
- bounded capture, sandbox replay, and recovery audit primitives kept distinct from live routing
- configurable per-destination queue, drop, and breaker policy from TOML
- polling-based safe config watch flow that preserves the last-known-good revision
- startup-time proxy loop prevention for UDP destinations that point back into a bound ingress
- JSON proxy-status output that summarizes ingresses, destinations, routes, direct UDP fallback hints, and runtime queue health
- startup and reload safety gates that can block proxy activation when operator warnings or fallback gaps are present
- a minimal safe-mode launch profile that disables optional capture / replay / restart-rehydrate surfaces while keeping core UDP routing active
- clean proxy shutdown that releases ingress ports for controlled restart and future hot reload work
- managed proxy reload supervision with rollback to the previous live config when a replacement runtime fails
- optional co-hosted health and metrics endpoint while the live UDP proxy is running
- optional co-hosted control endpoint for freeze/thaw, route isolation, and live status inspection while the proxy is running
- control endpoint now also exposes destination rehydrate and sandbox replay actions for live operator workflows
- control endpoint also supports bulk route restore plus percent-decoded resource ids for safer operator recovery flows
- runtime status and control history endpoints now expose bounded recent operator actions and config transitions for post-incident tracing
- CLI reports and control-plane `/report` / `/blockers` now share the same structured operator safety evaluation
- control-plane `/report` now also exposes structured override/runtime-signal/route-signal/destination-signal sections, with `/overrides` and `/signals` endpoints for direct consumption
- `/signals?scope=problematic` can now trim route/destination signal payloads down to only the entries that currently need operator attention
- config rejection / block / reload-failure history now retains reason details instead of only counters

## Documentation Entry Points

- [Documentation Index](./docs/README.md)
- [Documentation Index (Japanese)](./docs/README.ja.md)
- [Repository README (Japanese)](./README.ja.md)
- [Concepts / Planning (English)](./docs/concepts/en/README.md)
- [Concepts / Planning (Japanese)](./docs/concepts/ja/README.md)
- [Design Specs (English)](./docs/design/en/README.md)
- [Design Specs (Japanese)](./docs/design/ja/README.md)
- [Changelog](./CHANGELOG.md)
- [Changelog (Japanese)](./CHANGELOG.ja.md)

## AI Collaboration Entry Points

- [Agent Brief](./.agent/AGENT.md)
- [Agent Brief (Plural Alias)](./.agents/AGENTS.md)
- [Skill Catalog](./.skill/SKILL.md)
- [Skill Catalog (Plural Alias)](./.skills/SKILLS.md)
- [AI Collaboration And Agent Interop Plan](./docs/concepts/en/ai-collaboration-and-agent-interop-plan.md)
- [AI Collaboration And Agent Interop Plan (Japanese)](./docs/concepts/ja/ai-collaboration-and-agent-interop-plan.md)
- [Gemini PR Review Assistant](./docs/concepts/en/gemini-pr-review-assistant.md)
- [Maintainer Approval And Merge Behavior](./docs/concepts/en/maintainer-approval-and-merge-behavior.md)
- [Release Note And Changelog Policy](./docs/concepts/en/release-note-and-changelog-policy.md)

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
- Keep AI entry-point trees mirrored so different tools receive the same rules.

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
