---
name: find-planned-work
description: 'Use this skill when looking for future product work, release blockers, or requirements that are not current yet. Use `special specs --planned` when available, or convert untracked roadmap/backlog prose into planned specs.'
---
@filedocuments spec SPECIAL.SPEC_COMMAND.PLANNED_ONLY

# Find Planned Work
@implements SPECIAL.DOCUMENTATION.SKILLS.FALLBACK.FIND_PLANNED_WORK
@applies DOCS.SKILL_MAIN_ENTRY

## When To Use
@applies DOCS.SKILL_TRIGGER_BOUNDARY_SECTION

Use this when a task asks:

- what is still planned?
- what blocks this release?
- what work is promised but not current?
- where did we leave future requirements?
- should this backlog item become a tracked planned spec?

## How To Use
@applies DOCS.SKILL_WORKFLOW_SECTION

1. If Special is present, run [`special specs --planned`](documents://spec/SPECIAL.SPEC_COMMAND.PLANNED_ONLY).
2. Scope with [`special specs --planned SPEC.ID`](documents://spec/SPECIAL.SPEC_COMMAND.ID_SCOPE) when the tree is large.
3. Read release metadata as a label for the intended release, not as a version range.
4. If Special is not present, inspect roadmap/backlog/release notes and convert durable product claims into `@spec ... @planned`.

Example:

```md
@spec EXPORT.PDF @planned 0.9.0
Users can export the current report as a PDF.
```

## What To Do With Results
@applies DOCS.SKILL_RESULT_DISPOSITION_SECTION

- If a planned claim is now implemented and verified, remove `@planned`.
- If planned work is stale, update or delete the claim.
- If the item is only an internal refactor, track it outside product specs or use architecture annotations.
- If future behavior is buried in prose, move the stable claim into [`@spec`](documents://spec/SPECIAL.SPEC_COMMAND.MARKDOWN_DECLARATIONS).

@applies DOCS.SKILL_REFERENCE_HANDOFF_SECTION

Read [references/planned-workflow.md](references/planned-workflow.md) for the walkthrough and [references/trigger-evals.md](references/trigger-evals.md) for trigger examples.
