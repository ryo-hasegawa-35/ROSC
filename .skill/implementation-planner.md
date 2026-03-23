# ROSC Skill: Implementation Planner

## Use This Skill When

- the work is still pre-code
- design docs need to be translated into milestones or execution slices
- you need to define what must happen before implementation begins

## Goal

Turn the current specification set into a safe, staged plan for engineering
work.

## Workflow

1. Start from the approved design docs and ADRs.
2. Break work into slices that preserve compatibility and rollback safety.
3. Define acceptance criteria and evidence requirements for each slice.
4. Separate core requirements from optional accelerators or integrations.
5. Keep cross-platform expectations explicit.
6. Recommend the next smallest high-leverage step.

## Avoid

- writing speculative runtime code
- collapsing planning and implementation into one step
- mixing long-term stretch goals into the first milestone without boundaries
