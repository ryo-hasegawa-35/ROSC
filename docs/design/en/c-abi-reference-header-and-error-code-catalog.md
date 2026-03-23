# C ABI Reference Header And Error-Code Catalog

## Purpose

This document defines the intended shape of the public C ABI surface used by
host-native integrations and external tooling.

This is a design reference, not an implementation header.

## Design Goals

- stable ABI boundary
- minimal surface area
- opaque ownership
- clear error reporting
- explicit version negotiation

## Naming Conventions

Recommended symbol prefixes:

- `rosc_` for public C ABI functions
- `ROSC_` for constants and error codes

Recommended handle names:

- `rosc_broker_t`
- `rosc_route_t`
- `rosc_endpoint_t`
- `rosc_channel_t`

## Fundamental ABI Rules

- all public structs with stable layout must include explicit size or version
- opaque handles should be preferred over exposed structs
- strings should use explicit pointer-plus-length or defined ownership rules
- caller / callee ownership must be stated for every buffer

## ABI Version Negotiation

The ABI surface should expose:

- library ABI version
- minimum compatible caller version
- feature bitset

Version mismatch should return an explicit error, never undefined behavior.

## Function Families

The reference surface should include families for:

- version and feature query
- broker lifecycle
- configuration load / validate / apply
- endpoint and route inspection
- channel / IPC lifecycle
- diagnostics and health query
- last-error retrieval

## Result Model

Prefer a compact explicit result model:

- success code
- stable error code
- optional detail retrieval function

Do not rely on:

- exceptions
- hidden global mutable state without thread rules
- undocumented error side channels

## Error Code Catalog

Suggested stable codes:

| Code | Meaning |
| --- | --- |
| `ROSC_OK` | success |
| `ROSC_ERR_UNKNOWN` | unspecified failure |
| `ROSC_ERR_INVALID_ARGUMENT` | invalid argument passed by caller |
| `ROSC_ERR_UNSUPPORTED_VERSION` | ABI or feature version mismatch |
| `ROSC_ERR_NOT_INITIALIZED` | operation attempted before init |
| `ROSC_ERR_ALREADY_INITIALIZED` | duplicate init or start |
| `ROSC_ERR_INVALID_HANDLE` | stale or invalid opaque handle |
| `ROSC_ERR_BUFFER_TOO_SMALL` | caller-provided buffer insufficient |
| `ROSC_ERR_CONFIG_INVALID` | config failed validation |
| `ROSC_ERR_CONFIG_APPLY_FAILED` | config could not be applied |
| `ROSC_ERR_IO_FAILURE` | I/O or OS-level failure |
| `ROSC_ERR_TIMEOUT` | operation timed out |
| `ROSC_ERR_BACKPRESSURE` | downstream or channel pressure blocked operation |
| `ROSC_ERR_CHANNEL_UNAVAILABLE` | IPC channel unavailable |
| `ROSC_ERR_SECURITY_DENIED` | security verification or authorization failed |
| `ROSC_ERR_UNSUPPORTED_FEATURE` | requested feature not compiled or enabled |
| `ROSC_ERR_STATE_CONFLICT` | operation incompatible with current state |

## Error Detail Model

If richer detail is needed, expose:

- last error code
- last error message
- optional structured diagnostic snapshot

But the stable contract should rely on the error code first.

## Threading Expectations

The ABI should document:

- whether the broker handle is thread-safe
- whether multiple readers are allowed
- whether configuration apply requires exclusive access
- whether callback registration is allowed and under what rules

## Buffer Conventions

Recommended pattern:

- caller provides buffer and size
- callee returns required size when insufficient
- ownership remains explicit

## State Machine Expectations

The ABI should define stable lifecycle states such as:

- created
- configured
- running
- degraded
- stopped
- destroyed

Operations should fail clearly if called in the wrong state.

## Non-Negotiable Invariants

- public ABI must remain smaller and simpler than internal Rust APIs
- all ownership rules must be explicit
- error handling must be machine-readable
- version mismatch must never degrade into undefined behavior
- fallback-friendly design takes priority over surface convenience
