---
name: use-project-patterns
description: 'Use this skill when adding, reviewing, or refactoring code or docs that resembles a recurring project approach. Use pattern metrics as advisory evidence.'
---
@filedocuments spec SPECIAL.PATTERNS.COMMAND

# Use Project Patterns
@implements SPECIAL.DOCUMENTATION.SKILLS.PLUGIN.USE_PROJECT_PATTERNS
@applies DOCS.SKILL_MAIN_ENTRY

## When To Use
@applies DOCS.SKILL_TRIGGER_BOUNDARY_SECTION

Use this when code or docs resembles a repeated implementation structure,
template, or reviewable approach. Do not turn broad style advice into a pattern.

## Workflow
@applies DOCS.SKILL_WORKFLOW_SECTION

1. Inspect declared patterns with [`special_patterns`](documents://spec/SPECIAL.PATTERNS.COMMAND); fall back to `special patterns --metrics`.
2. If a pattern exists, scope to it and include metrics.
3. If no pattern exists, define one only when it guides future work.
4. Attach [`@applies`](documents://spec/SPECIAL.PATTERNS.SOURCE_APPLICATIONS) to the item, heading, or block that actually demonstrates the approach.
5. Use [`@fileapplies`](documents://spec/SPECIAL.PATTERNS.FILE_APPLICATIONS) only when the whole file is the application.
6. Use [`@strictness`](documents://spec/SPECIAL.PATTERNS.STRICTNESS) when similarity expectations matter.
7. Use `special_trace` with `surface: "patterns"` when applications need review.

## What To Do With Results
@applies DOCS.SKILL_RESULT_DISPOSITION_SECTION

- Apply a pattern when the source intentionally follows it.
- Extract a helper when examples are nearly identical and the repetition is only
  mechanics.
- Split or rewrite a pattern when applications drift.
- Treat missing-application metrics as review leads, not lint failures.
