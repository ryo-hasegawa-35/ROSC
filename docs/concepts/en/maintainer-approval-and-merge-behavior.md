# Maintainer Approval And Merge Behavior

## Purpose

This document explains the practical review and merge behavior maintainers will
see on GitHub, especially while the repository is still operated by a single
owner account.

## Current ROSC Setup

At the time this document was written, `main` is protected with:

- pull request required
- required status checks
- required conversation resolution
- one approving review required
- code owner review required
- stale reviews dismissed on new pushes
- admin enforcement disabled

This setup is meant to protect `main` while still letting the repository owner
complete urgent maintenance if needed.

## Why GitHub Shows A Bypass Option

The bypass option is normal in the current ROSC setup for two reasons.

### 1. The maintainer account is an admin

Repository admins can bypass branch rules when admin enforcement is disabled.

### 2. Self-authored pull requests do not gain a qualifying approval from the same account

If the pull request is authored by the same GitHub account that is trying to
approve it, GitHub does not treat that as an independent required approval.

This is especially common here because local automation and Codex-driven PRs
currently use the maintainer's GitHub credentials, so the PR author appears as
the maintainer account.

## Is This Normal

Yes. In a single-owner repository, it is normal to see:

- required review rules configured
- no qualifying independent approval available
- GitHub offering an admin bypass path to merge anyway

That does not mean the branch protection is broken. It means GitHub is
correctly distinguishing between:

- a true independent review
- the repository owner's administrative override

## Recommended Workflow For ROSC Right Now

Until a second reviewer identity exists, the practical maintainer workflow is:

1. let all required checks pass
2. review the PR contents carefully
3. leave comments or request changes when needed
4. merge only when satisfied that the PR is safe

In this setup, the merge decision itself becomes the maintainer's final
approval action.

## If A Stricter Review Model Is Desired Later

To make GitHub's approval requirement behave like a true non-bypass gate, one
of these must change:

- add a second human reviewer account or team
- have PRs authored from a separate bot or service account
- enable admin enforcement only after an independent approval path exists

Without one of those changes, a single-owner repository cannot require an
independent approval from the same account that opened the PR.

## What ROSC Should Not Pretend

ROSC should not pretend that:

- self-approval on a self-authored PR is equivalent to independent review
- "bypass and merge" means GitHub is malfunctioning
- required reviews alone can create separation of duties in a one-account setup

## Maintenance Rule

If branch protection settings change materially, update this document and the
related repository-governance docs in the same PR.
