---
name: inspect-current-spec-state
description: 'Use this skill when you need to know what behavior the project currently claims and supports. Prefer Special MCP specs tools, with CLI fallback.'
---
@filedocuments spec SPECIAL.SPEC_COMMAND.CURRENT_ONLY

# Inspect Current Spec State
@implements SPECIAL.DOCUMENTATION.SKILLS.PLUGIN.INSPECT_CURRENT_SPEC_STATE
@applies DOCS.SKILL_MAIN_ENTRY

## When To Use
@applies DOCS.SKILL_TRIGGER_BOUNDARY_SECTION

Use this before answering what the product currently does, preparing release
notes, checking support state, or changing behavior.

## Workflow
@applies DOCS.SKILL_WORKFLOW_SECTION

1. Check status with `special_status`.
2. Run [`special_specs`](documents://spec/SPECIAL.SPEC_COMMAND.CURRENT_ONLY) with `current: true`; fall back to `special specs --current`.
3. Include metrics when you need support counts; fall back to [`special specs --current --metrics`](documents://spec/SPECIAL.SPEC_COMMAND.METRICS).
4. Scope with an id when the tree is large.
5. Use `special_trace` with `surface: "specs"` when a current claim needs proof review.

## What To Do With Results
@applies DOCS.SKILL_RESULT_DISPOSITION_SECTION

- Cite a current claim only after reading its support.
- Add proof, narrow the claim, or mark it planned when support is missing.
- Move ownership questions to architecture and repeated-shape questions to
  patterns.
- Run `special_lint` after changing claims or support.
