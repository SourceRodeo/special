---
name: validate-product-contract
description: 'Use this skill when reviewing whether a feature, bug fix, test, or release claim is honestly supported. Use trace packets for context, then judge the claim against the proof.'
---
@filedocuments spec SPECIAL.TRACE_COMMAND.SPECS

# Validate Product Contract
@implements SPECIAL.DOCUMENTATION.SKILLS.PLUGIN.VALIDATE_PRODUCT_CONTRACT
@applies DOCS.SKILL_MAIN_ENTRY

## When To Use
@applies DOCS.SKILL_TRIGGER_BOUNDARY_SECTION

Use this when a task asks whether a product claim is supported, too broad, stale,
or safe to release.

## Workflow
@applies DOCS.SKILL_WORKFLOW_SECTION

1. Find the exact claim with `special_specs`; fall back to `special specs --current --metrics`.
2. Run [`special_trace`](documents://spec/SPECIAL.TRACE_COMMAND.SPECS) with `surface: "specs"` and the exact id; fall back to `special trace specs --id SPEC.ID`.
3. Read the spec text before reading the proof.
4. Read every `@verifies` or `@attests` body in the packet.
5. Decide whether the proof demonstrates the exact observable behavior claimed.
6. Run `special_lint` after edits.

## What To Do With Results
@applies DOCS.SKILL_RESULT_DISPOSITION_SECTION

- Keep aligned claims and proof attachments.
- Split or narrow claims when the proof covers only part of the behavior.
- Replace helper-only tests when the claim needs observable behavior proof.
- Mark unfinished claims planned.
- Do not treat a trace packet's existence as proof; it is the context you must
  review.
