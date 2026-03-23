# ROSC Skill: Docs Maintainer

## Use This Skill When

- the task is documentation-heavy
- bilingual consistency matters
- indexes, reading order, links, or repository guidance need updates

## Goal

Leave the documentation clearer, easier to navigate, and safer for later
implementation work.

## Workflow

1. Identify whether the content belongs in `docs/concepts/` or `docs/design/`.
2. Update the English and Japanese files together.
3. Update the nearest relevant index or README.
4. If AI-entry files rely on the changed rules, update `.agent/.agents` or
   `.skill/.skills` in the same change.
5. Keep relative links valid.
6. Summarize what changed and what should be read next.

## Quality Bar

- terminology stays consistent with the glossary
- reading order becomes easier, not harder
- new documents have obvious entry points
- no English-only or Japanese-only project doc is left behind
