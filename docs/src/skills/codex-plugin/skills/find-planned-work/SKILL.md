---
name: find-planned-work
description: 'Use this skill when looking for future product work, release blockers, or requirements that are not current yet. Prefer MCP specs tools, with CLI fallback.'
---
@filedocuments spec SPECIAL.SPEC_COMMAND.PLANNED_ONLY

# Find Planned Work
@implements SPECIAL.DOCUMENTATION.SKILLS.PLUGIN.FIND_PLANNED_WORK
@applies DOCS.SKILL_MAIN_ENTRY

## When To Use
@applies DOCS.SKILL_TRIGGER_BOUNDARY_SECTION

Use this when a task asks what is planned, what blocks a release, or whether
roadmap prose should become a tracked product claim.

## Workflow
@applies DOCS.SKILL_WORKFLOW_SECTION

1. Run [`special_specs`](documents://spec/SPECIAL.SPEC_COMMAND.PLANNED_ONLY) with `planned: true`; fall back to `special specs --planned`.
2. Scope by id when the list is large.
3. Read release metadata as a planning label, not as a semantic version range.
4. Convert durable future behavior into [`@spec ... @planned`](documents://spec/SPECIAL.PARSE.PLANNED).
5. Use `special_trace` only after a planned claim becomes current and needs support review.

## What To Do With Results
@applies DOCS.SKILL_RESULT_DISPOSITION_SECTION

- Remove `@planned` only when the behavior is implemented and supported.
- Delete stale future claims instead of preserving empty roadmap structure.
- Track internal refactors outside product specs unless they change behavior.
- Run `special_lint` after changing planned/current state.
