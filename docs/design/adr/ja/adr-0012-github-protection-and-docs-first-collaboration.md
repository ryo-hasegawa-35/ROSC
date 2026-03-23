# ADR-0012: GitHub Protection And Docs-First Collaboration

- Status: accepted
- Date: 2026-03-23

## Context

project はまだ pre-implementation なので、
repository process も architecture foundation の一部です。

## Decision

- `main` は protected で PR-only に保つ
- `main` に入る前に maintainer approval を要求する
- risky semantic change には design / ADR 更新を前提にする
- 英語版と日本語版の project documentation を repository rule とする

## Consequences

- implementation work を planning / design decision と結びつけやすい
- repository automation も quality story の一部になる
- contributor には explicit な作業が求められるが、architecture は安全になる

## Rejected Alternatives

- major change の direct push
- documentation や ADR を伴わない code-first semantic change

## Affected Documents

- [GitHub Foundation And Collaboration Plan](../../../concepts/ja/github-foundation-and-collaboration-plan.md)
- [GitHub Backlog Map](../../../concepts/ja/github-backlog-map.md)
- [Implementation Readiness Checklist](../../ja/implementation-readiness-checklist.md)
