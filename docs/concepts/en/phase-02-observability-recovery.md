# Phase 02: Observability And Recovery

## Goal

Make the broker visible, debuggable, and recoverable in live environments.

## Deliverables

- Embedded dashboard backend and frontend
- Real-time traffic graph
- Route topology visualization
- Per-endpoint health and throughput views
- Stateful last-value cache
- Late joiner sync
- In-memory ring buffer for recent traffic
- Capture and replay tooling
- Time-travel debug workflow:
  - inspect packet history
  - filter by address / route / source
  - replay selected traffic safely
- Config snapshot history
- Structured logs with correlation IDs

## Product Decisions

- Decide retention strategy for the ring buffer
- Decide which values are safe to cache by default
- Decide how replay is isolated from live outputs
- Decide whether the dashboard is read-only or operationally active

## Operational Safeguards

- Replay must default to sandbox or dry-run output.
- Cache sync must be opt-in per route or namespace.
- Diagnostic features must not materially degrade the fast path.

## Non-Goals

- No general-purpose plugin marketplace yet
- No secure multi-tenant mode yet
- No shared memory IPC yet

## Exit Criteria

- Operators can identify bottlenecks and drops without packet sniffers.
- A restarted node can resync state through cache policies.
- A captured issue can be replayed in a controlled way for debugging.

## Rough Effort

120-180 hours

## Value

This phase turns the broker from a black box into an operational tool. That is
often the difference between "technically impressive" and "trusted on site."
