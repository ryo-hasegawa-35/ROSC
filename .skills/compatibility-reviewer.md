# ROSC Skill: Compatibility Reviewer

## Use This Skill When

- a task could affect OSC interoperability
- operator-visible behavior or recovery semantics may change
- a performance claim needs a design-level sanity check

## Goal

Protect backward compatibility, evidence quality, and operational clarity.

## Review Focus

- OSC 1.0 alignment, padding, and big-endian assumptions
- strict / tolerant / extended mode behavior
- unknown extension handling
- recovery cache semantics and late-joiner behavior
- telemetry meaning and benchmark interpretation
- rollback and fallback paths

## Output

Leave a concise review that states:

- what looks safe
- what looks risky
- what evidence is missing
- what design doc or issue should be updated next
