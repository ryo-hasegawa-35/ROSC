# Security Overlay Model

## Purpose

This document defines how security should be added without breaking raw OSC
compatibility.

Security in this project is an overlay and policy layer, not a replacement for
plain OSC.

## Core Principle

- raw OSC remains usable
- secure ingress is additive
- the broker terminates security before forwarding to legacy peers

## Security Objectives

- identify trusted senders
- scope traffic by project or environment
- authorize route access
- prevent spoofing in secure mode
- keep compatibility routes available where required

## Security Domains

### Ingress Identity

Possible identities:

- anonymous legacy sender
- verified operator client
- verified broker peer
- verified adapter or service

### Scope

Possible scopes:

- project
- venue
- workstation
- namespace

### Authorization

Possible permissions:

- send
- receive
- observe
- transform
- administer

## Secure Envelope Model

Secure routes may use an outer envelope or authenticated transport context that
contains:

- sender identity
- scope
- issued-at
- expiry
- signature or token proof

The raw OSC payload remains inside that secure envelope and is not modified for
legacy compatibility.

## Compatibility Bridge

The compatibility bridge works like this:

1. secure ingress arrives at broker
2. broker verifies identity and scope
3. broker authorizes route access
4. broker forwards plain compatible OSC downstream where allowed

This keeps legacy endpoints unchanged.

## Route Security Policy

Each route should be able to declare:

- `scope`
- `require_verified_source`
- `allowed_identities`
- `allow_legacy_bridge`
- `audit_level`

## Failure Behavior

Secure route failures should fail closed.

Examples:

- invalid signature
- expired token
- mismatched scope
- unauthorized sender

Compatibility routes may continue to operate according to their ordinary route
fault policy.

## Replay Protection

Secure routes should support:

- issued-at validation
- expiry validation
- nonce or sequence tracking where appropriate
- replay-session marking for operator-driven diagnostics

## Audit Model

The broker should record:

- who sent traffic
- what scope it claimed
- whether verification passed
- whether authorization passed
- what route decision resulted

## Recovery Interaction

Recovery must respect security scope.

Rules:

- secure cached state keeps its scope metadata
- resend across scope boundaries requires explicit authorization
- standby brokers must preserve security lineage if state is replicated

## Operator Controls

Useful controls:

- view active identities
- inspect denied events
- temporarily disable a secure route
- force safe mode
- audit config changes

## Non-Negotiable Invariants

- raw OSC payloads for legacy peers must not be polluted with mandatory security
  fields
- secure verification failure must not silently downgrade into trusted traffic
- security scope must remain visible in diagnostics and audit
- secure mode must be additive, not destructive to compatibility deployment
