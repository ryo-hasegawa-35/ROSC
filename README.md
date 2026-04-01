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
cargo run -p rosc-broker -- proxy-overview examples/phase-01-basic.toml --fail-on-warnings --require-fallback-ready
cargo run -p rosc-broker -- proxy-readiness examples/phase-01-basic.toml --fail-on-warnings --require-fallback-ready
cargo run -p rosc-broker -- proxy-assert-ready examples/phase-01-basic.toml --fail-on-warnings --require-fallback-ready
cargo run -p rosc-broker -- proxy-snapshot examples/phase-01-basic.toml --fail-on-warnings --require-fallback-ready --history-limit 10
cargo run -p rosc-broker -- proxy-diagnostics examples/phase-01-basic.toml --fail-on-warnings --require-fallback-ready --history-limit 10
cargo run -p rosc-broker -- proxy-attention examples/phase-01-basic.toml --fail-on-warnings --require-fallback-ready
cargo run -p rosc-broker -- proxy-incidents examples/phase-01-basic.toml --fail-on-warnings --require-fallback-ready --history-limit 10
cargo run -p rosc-broker -- proxy-handoff examples/phase-01-basic.toml --fail-on-warnings --require-fallback-ready --history-limit 10
cargo run -p rosc-broker -- proxy-timeline examples/phase-01-basic.toml --fail-on-warnings --require-fallback-ready --history-limit 10 --route-id camera
cargo run -p rosc-broker -- proxy-triage examples/phase-01-basic.toml --fail-on-warnings --require-fallback-ready --history-limit 10 --route-id camera
cargo run -p rosc-broker -- proxy-casebook examples/phase-01-basic.toml --fail-on-warnings --require-fallback-ready --history-limit 10 --route-id camera
cargo run -p rosc-broker -- proxy-board examples/phase-01-basic.toml --fail-on-warnings --require-fallback-ready --history-limit 10
cargo run -p rosc-broker -- proxy-focus examples/phase-01-basic.toml --fail-on-warnings --require-fallback-ready --history-limit 10 --route-id camera
cargo run -p rosc-broker -- proxy-lens examples/phase-01-basic.toml --fail-on-warnings --require-fallback-ready --history-limit 10 --route-id camera
cargo run -p rosc-broker -- proxy-brief examples/phase-01-basic.toml --fail-on-warnings --require-fallback-ready --history-limit 10 --route-id camera
cargo run -p rosc-broker -- proxy-dossier examples/phase-01-basic.toml --fail-on-warnings --require-fallback-ready --history-limit 10 --route-id camera
cargo run -p rosc-broker -- watch-config examples/phase-01-basic.toml --poll-ms 1000 --fail-on-warnings
cargo run -p rosc-broker -- watch-udp-proxy examples/phase-01-basic.toml --poll-ms 1000 --ingress-queue-depth 1024 --health-listen 127.0.0.1:19191 --control-listen 127.0.0.1:19292 --fail-on-warnings --require-fallback-ready --safe-mode
cargo run -p rosc-broker -- diff-config examples/phase-01-basic.toml examples/phase-01-basic-changed.toml
cargo run -p rosc-broker -- serve-health 127.0.0.1:19191 --config examples/phase-01-basic.toml
cargo run -p rosc-broker -- run-udp-proxy examples/phase-01-basic.toml --health-listen 127.0.0.1:19191 --control-listen 127.0.0.1:19292 --fail-on-warnings --require-fallback-ready --safe-mode
start http://127.0.0.1:19292/dashboard
curl -X POST http://127.0.0.1:19292/freeze
curl -X POST http://127.0.0.1:19292/routes/camera/isolate
curl -X POST http://127.0.0.1:19292/routes/restore-all
curl -X POST http://127.0.0.1:19292/destinations/udp_renderer/rehydrate
curl -X POST "http://127.0.0.1:19292/routes/camera/replay/sandbox_tap?limit=1"
curl http://127.0.0.1:19292/status
curl http://127.0.0.1:19292/report
curl http://127.0.0.1:19292/overview
curl http://127.0.0.1:19292/readiness
curl -i http://127.0.0.1:19292/readyz
curl -i "http://127.0.0.1:19292/readyz?allow_degraded=true"
curl http://127.0.0.1:19292/snapshot?limit=10
curl http://127.0.0.1:19292/diagnostics?limit=10
curl http://127.0.0.1:19292/attention
curl http://127.0.0.1:19292/incidents?limit=10
curl http://127.0.0.1:19292/handoff?limit=10
curl http://127.0.0.1:19292/triage?limit=10
curl http://127.0.0.1:19292/casebook?limit=10
curl http://127.0.0.1:19292/board?limit=10
curl http://127.0.0.1:19292/focus?limit=10
curl http://127.0.0.1:19292/brief?limit=10
curl http://127.0.0.1:19292/lens?limit=10
curl http://127.0.0.1:19292/timeline?limit=10
curl http://127.0.0.1:19292/trace?limit=10
curl http://127.0.0.1:19292/routes/camera/focus?limit=10
curl http://127.0.0.1:19292/routes/camera/lens?limit=10
curl http://127.0.0.1:19292/routes/camera/handoff?limit=10
curl http://127.0.0.1:19292/routes/camera/triage?limit=10
curl http://127.0.0.1:19292/routes/camera/casebook?limit=10
curl http://127.0.0.1:19292/routes/camera/board?limit=10
curl http://127.0.0.1:19292/routes/camera/timeline?limit=10
curl http://127.0.0.1:19292/routes/camera/trace?limit=10
curl http://127.0.0.1:19292/routes/camera/brief?limit=10
curl http://127.0.0.1:19292/routes/camera/dossier?limit=10
curl http://127.0.0.1:19292/destinations/udp_renderer/handoff?limit=10
curl http://127.0.0.1:19292/destinations/udp_renderer/triage?limit=10
curl http://127.0.0.1:19292/destinations/udp_renderer/casebook?limit=10
curl http://127.0.0.1:19292/destinations/udp_renderer/board?limit=10
curl http://127.0.0.1:19292/destinations/udp_renderer/focus?limit=10
curl http://127.0.0.1:19292/destinations/udp_renderer/lens?limit=10
curl http://127.0.0.1:19292/destinations/udp_renderer/brief?limit=10
curl http://127.0.0.1:19292/destinations/udp_renderer/dossier?limit=10
curl http://127.0.0.1:19292/destinations/udp_renderer/timeline?limit=10
curl http://127.0.0.1:19292/destinations/udp_renderer/trace?limit=10
curl http://127.0.0.1:19292/overrides
curl http://127.0.0.1:19292/signals
curl http://127.0.0.1:19292/signals?scope=problematic
curl http://127.0.0.1:19292/blockers
curl http://127.0.0.1:19292/history/operator-actions
curl http://127.0.0.1:19292/history/config-events
```

`--control-listen` is intentionally loopback-only. Bind it to `127.0.0.1`, `::1`, or another
local-only alias such as `localhost`; wildcard or externally reachable addresses are rejected.

`proxy-status`, `proxy-overview`, `proxy-readiness`, `proxy-assert-ready`, `proxy-snapshot`, `proxy-diagnostics`, `proxy-attention`, `proxy-incidents`, `proxy-handoff`, `proxy-timeline`, `proxy-triage`, `proxy-casebook`, `proxy-board`, `proxy-focus`, `proxy-lens`, `proxy-brief`, and `proxy-dossier`
intentionally write JSON only to stdout so they can be piped directly into tools such as `jq`
without stripping summary lines first.

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
- `proxy-overview` and control-plane `/overview` now expose a one-shot operator snapshot with report + current status + problematic signal view for dashboard/bootstrap workflows
- `proxy-readiness` and control-plane `/readiness` now expose a machine-readable readiness contract with `ready/degraded/blocked` level, operator-action reasons, and route/destination counts for automation and deployment gates
- `proxy-assert-ready` and control-plane `/readyz` now expose gate-style readiness checks that return non-zero / HTTP 503 when the current proxy state is not acceptable for startup or deployment automation
- `proxy-snapshot` and control-plane `/snapshot` now expose a full one-shot operator bundle with overview, readiness, diagnostics, attention, and incidents in one payload for dashboard/bootstrap and incident tooling
- `proxy-diagnostics` and control-plane `/diagnostics` now expose the same operator snapshot bundled with bounded recent operator/config history for incident triage
- `proxy-attention` and control-plane `/attention` now expose a compact triage view with active overrides, latest incident highlights, and only the route/destination ids that currently need attention
- `proxy-incidents` and control-plane `/incidents` now expose an incident-focused bundle with open blockers/warnings, filtered recent issue history, and the full problematic route/destination entries needed for recovery work
- control-plane `/dashboard` now serves a lightweight operator console that layers overview/readiness/traffic/config/timeline views and safe live actions over a single `/dashboard/data` localhost payload
- `/dashboard/data` now also includes route/destination drill-down detail models so the embedded dashboard can jump from incident lists into route/destination-specific recovery context without stitching extra requests
- snapshot and dashboard payloads now include a machine-readable operator worklist with recommended next actions such as thaw, restore-route, rehydrate-destination, and focus-only investigation jumps
- the embedded dashboard now keeps polling through transient control-plane failures, preserves the last successful snapshot as stale operator context, and marks isolated routes as isolated in the runtime table instead of silently healthy
- snapshot and dashboard payloads now also include an incident digest plus structured recovery candidates, so operators can move from grouped incident cards into concrete route/destination recovery actions without stitching extra control-plane calls
- snapshot and dashboard payloads now also include per-route and per-destination trace catalogs that connect current runtime pressure with related operator actions and config incidents
- control-plane `/trace`, `/routes/{id}/trace`, and `/destinations/{id}/trace` now expose those linked traces directly for external tooling, not only the embedded dashboard
- snapshot now also includes a machine-readable handoff catalog, and `proxy-handoff` plus control-plane `/handoff`, `/routes/{id}/handoff`, `/destinations/{id}/handoff` expose next-step guidance for route/destination recovery work
- the embedded dashboard now renders focused route/destination handoff panels so operators can move from trace history to concrete next actions without leaving the focus workflow
- snapshot-derived timeline catalogs now expose explicit global, route-linked, and destination-linked event history, and `proxy-timeline` plus control-plane `/timeline`, `/routes/{id}/timeline`, `/destinations/{id}/timeline` expose the same machine-readable slices for tooling
- the embedded dashboard now renders focused route/destination timeline panels alongside trace and handoff so triage can move from current pressure to exact recent events without stitching extra requests
- snapshot now also includes a triage catalog, and `proxy-triage` plus control-plane `/triage`, `/routes/{id}/triage`, `/destinations/{id}/triage` expose a merged global/focused recovery view with next steps, actions, and recorded timeline slices
- handoff and triage guidance now treat `traffic_frozen` as a first-class global override, so focused recovery guidance tells the operator to thaw traffic before trusting apparently stable routes or destinations
- snapshot now also includes a casebook catalog, and `proxy-casebook` plus control-plane `/casebook`, `/routes/{id}/casebook`, `/destinations/{id}/casebook` expose a focused route/destination recovery packet that bundles incident titles, next steps, recommended actions, recovery surface, recent trace, and recorded timeline in one machine-readable slice
- the embedded dashboard now renders focused route/destination casebook panels so an operator can move from focus selection straight into incident, recovery, and handoff context without jumping across multiple sections
- snapshot now also includes a board catalog, and `proxy-board` plus control-plane `/board`, `/routes/{id}/board`, `/destinations/{id}/board` expose blocked/degraded/watch lanes so operators and external tooling can sort what needs action before drilling into a casebook
- the embedded dashboard now renders a board section that groups the current operator workload into blocked, degraded, and watch lists with direct focus/recovery actions
- snapshot and dashboard payloads now also include a focus catalog, and `proxy-focus` plus control-plane `/focus`, `/routes/{id}/focus`, `/destinations/{id}/focus` expose a focused route/destination packet that bundles detail, trace, timeline, handoff, triage, casebook, and board lanes in one machine-readable slice
- the embedded dashboard now upgrades its existing focus drill-down cards into richer focus packets, so route/destination selection becomes a one-stop operator summary instead of a bare detail view
- snapshot and dashboard payloads now also include an operator lens catalog, and `proxy-lens` plus control-plane `/lens`, `/routes/{id}/lens`, `/destinations/{id}/lens` keep focused route/destination triage tied to global blockers, global overrides, work items, and board context
- snapshot and dashboard payloads now also include an operator brief catalog, and `proxy-brief` plus control-plane `/brief`, `/routes/{id}/brief`, `/destinations/{id}/brief` tighten focus and lens into a compact handoff packet with headline timeline, next steps, and recommended actions
- snapshot and dashboard payloads now also include an operator dossier catalog, and `proxy-dossier` plus control-plane `/dossier`, `/routes/{id}/dossier`, `/destinations/{id}/dossier` expose a fuller route/destination packet that separates global blockers from scoped blockers while bundling focus, brief, lens, work items, and recommended actions
- focused board slices now retain global blockers such as `traffic_frozen`, so route/destination-scoped investigation no longer hides the whole-system reason a path is still degraded
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
