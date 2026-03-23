# Gemini PR Review Assistant

## Purpose

This document describes the repository bot that posts Japanese review-assist
comments on pull requests when the maintainer signals confusion.

## What The Bot Does

When the maintainer adds a `confused` reaction to a pull request body, the bot:

1. detects the reaction
2. adds a `+1` reaction to acknowledge it saw the request
3. gathers pull request metadata and changed files
4. asks Gemini for a Japanese review-assist summary
5. posts a PR comment with:
   - a rough overview
   - change summary
   - good points
   - weak points / concerns
   - risks
   - suggestions

## What The Bot Does Not Do

The bot does not:

- approve pull requests
- request changes as a formal GitHub review state
- merge pull requests
- modify code

It is strictly a comment assistant.

## Trigger Model

The ideal trigger would be the maintainer's `confused` reaction itself.

However, GitHub Actions currently exposes workflow triggers such as
`issue_comment`, `pull_request`, `pull_request_review`, and `schedule`, but not
a direct reaction workflow trigger. Because of that limitation, ROSC uses a
polling model:

- a scheduled workflow checks open pull requests every 5 minutes
- it looks for a `confused` reaction from the trigger user on the PR body
- once found, it processes that reaction exactly once

The workflow also supports `workflow_dispatch` for manual runs.

## Secret And Configuration

Required:

- repository secret `GEMINI_API_KEY`

Optional repository variables:

- `REVIEW_TRIGGER_USER`
  - defaults to the repository owner
- `REVIEW_MODEL`
  - defaults to `gemini-2.5-flash`
- `REVIEW_MAX_FILES`
  - defaults to `25`
- `REVIEW_MAX_PATCH_CHARS`
  - defaults to `60000`

## Anti-Spam Rule

The bot writes a hidden marker into its own PR comment so the same reaction is
not processed repeatedly on every scheduled run.

If the maintainer wants a fresh review, they can remove and re-add the
`confused` reaction, or use `workflow_dispatch` with `force`.

## Permissions Safety

The workflow intentionally uses minimal permissions:

- `contents: read`
- `pull-requests: read`
- `issues: write`

This means the bot can read PRs and post comments/reactions, but it is not
granted merge authority.

## Operational Note

This bot is designed to assist human review, not replace it. The maintainer
should still make the final decision after reading the pull request and the
bot's summary.
