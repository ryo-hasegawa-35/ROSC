# ROSC Agent Kit

This directory is a compatibility entry point for AI agents that look for
project guidance under `.agent/`.

The repository also ships a mirrored `.agents/` tree because some tools only
scan plural directory names. Keep the two trees aligned.

## Files

- `AGENT.md`
  - primary machine-readable agent brief
- `AGENTS.md`
  - compatibility alias for tools that prefer the plural filename
- `project-map.md`
  - condensed map of the repository and must-read documents
- `working-agreement.md`
  - mandatory workflow, safety boundaries, and quality gates
- `handoff-template.md`
  - expected structure for work summaries and baton passes between agents

## Maintenance Rule

- Update `.agent/` and `.agents/` in the same pull request.
- Keep file names and content mirrored across both trees.
- When agent instructions change, update the canonical project documents under
  `docs/` as needed so the short agent entry points do not drift away from the
  formal design.
