# ROSC Skill Catalog

This directory is a compatibility entry point for AI assistants that look for
project-local skills under `.skill/`.

The repository also ships a mirrored `.skills/` tree because some tools only
scan plural directory names. Keep the two trees aligned.

## Files

- `SKILL.md`
  - primary machine-readable skill catalog
- `SKILLS.md`
  - compatibility alias for tools that prefer the plural filename
- `docs-maintainer.md`
  - for bilingual documentation maintenance and index consistency
- `design-guardian.md`
  - for normative design changes, ADR alignment, and scope discipline
- `issue-curator.md`
  - for backlog shaping, issue quality, milestones, and acceptance criteria
- `implementation-planner.md`
  - for pre-code planning, slicing, and execution sequencing
- `compatibility-reviewer.md`
  - for compatibility, recovery, operator impact, and evidence-focused review

## Maintenance Rule

- Update `.skill/` and `.skills/` in the same pull request.
- Keep file names and content mirrored across both trees.
- If a skill changes the expected workflow, update the formal docs under
  `docs/` as well.
