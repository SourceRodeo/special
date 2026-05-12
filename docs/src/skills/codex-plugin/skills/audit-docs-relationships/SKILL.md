---
name: audit-docs-relationships
description: 'Use this skill when checking whether docs claims are true. Build the relationship inventory with Special, then trace each important relationship through its target and support chain.'
---
@filedocuments spec SPECIAL.TRACE_COMMAND.DOCS

# Audit Docs Relationships
@implements SPECIAL.DOCUMENTATION.SKILLS.PLUGIN.AUDIT_DOCS_RELATIONSHIPS
@applies DOCS.SKILL_MAIN_ENTRY

## When To Use
@applies DOCS.SKILL_TRIGGER_BOUNDARY_SECTION

Use this when reviewing docs before release, validating generated skill
guidance, or checking docs after product, architecture, pattern, or proof
changes.

## Workflow
@applies DOCS.SKILL_WORKFLOW_SECTION

1. Use [`special_docs`](documents://spec/SPECIAL.DOCS_COMMAND.METRICS) with `metrics: true` and a narrow `target` to build the relationship inventory; fall back to `special docs --metrics --target PATH`.
2. Use [`special_trace`](documents://spec/SPECIAL.TRACE_COMMAND.DOCS) with `surface: "docs"` for the same file or directory.
3. Review one packet at a time: docs prose, linked target declaration, and
   support evidence.
4. Decide whether the prose actually documents the target. A found relationship
   proves only that the relationship exists.
5. Fix either the docs prose, the target, or the support chain depending on
   which side is stale.

## What To Do With Results
@applies DOCS.SKILL_RESULT_DISPOSITION_SECTION

- Keep relationships that align semantically.
- Use ordinary markdown links for see-also navigation.
- Use `@documents` when a whole paragraph documents a single target.
- Finish with `special_lint`.
