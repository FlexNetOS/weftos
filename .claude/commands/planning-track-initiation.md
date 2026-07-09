---
name: planning-track-initiation
description: Workflow command scaffold for planning-track-initiation in weftos.
allowed_tools: ["Bash", "Read", "Write", "Grep", "Glob"]
---

# /planning-track-initiation

Use this workflow when working on **planning-track-initiation** in `weftos`.

## Goal

Initiate a new development track or phase by landing planning documents that define architecture, sequencing, or research base for upcoming work.

## Common Files

- `.planning/explorer/PROJECT-PLAN.md`
- `.planning/explorer/PHASE-2-PLAN.md`
- `.planning/ontology/ADOPTION.md`
- `.planning/ontology/palantir-foundry-research.md`
- `.planning/sensors/PIPELINE-PRIMITIVE-SPIKE.md`
- `.planning/sensors/PIPELINE-PRIMITIVE-JOURNAL.md`

## Suggested Sequence

1. Understand the current state and failure mode before editing.
2. Make the smallest coherent change that satisfies the workflow goal.
3. Run the most relevant verification for touched files.
4. Summarize what changed and what still needs review.

## Typical Commit Signals

- Create or update .planning/*/*.md files with detailed plans, research, or sequencing.
- Commit multiple related planning docs together (e.g., plan, adoption, spike, journal).
- Reference these docs in subsequent implementation commits.

## Notes

- Treat this as a scaffold, not a hard-coded script.
- Update the command if the workflow evolves materially.