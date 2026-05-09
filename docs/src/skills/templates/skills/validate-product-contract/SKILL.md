---
name: validate-product-contract
description: 'Use this skill when reviewing whether a feature, bug fix, test, or release claim is honestly supported. Inspect the exact product claim, inspect the proof artifact, and decide whether to keep, tighten, split, plan, or remove the claim.'
---
@filedocuments spec SPECIAL.TRACE_COMMAND.SPECS

# Validate Product Contract
@implements SPECIAL.DOCUMENTATION.SKILLS.FALLBACK.VALIDATE_PRODUCT_CONTRACT
@applies DOCS.SKILL_MAIN_ENTRY

## When To Use
@applies DOCS.SKILL_TRIGGER_BOUNDARY_SECTION

Use this when a task asks:

- does this test really prove the behavior?
- is this release claim safe?
- is this spec too broad or too vague?
- are we overclaiming current behavior?
- should this requirement be planned instead of current?

This skill can introduce Special if the claim is currently untracked.

## How To Use
@applies DOCS.SKILL_WORKFLOW_SECTION

1. Start from one exact behavior claim.
2. If Special is present, run [`special specs SPEC.ID --verbose`](documents://spec/SPECIAL.SPEC_COMMAND.VERBOSE).
3. If you do not know the id, run [`special specs --current`](documents://spec/SPECIAL.SPEC_COMMAND.CURRENT_ONLY) or [`special specs --metrics`](documents://spec/SPECIAL.SPEC_COMMAND.METRICS) first.
4. Read the claim before reading the verify.
5. Read the attached `@verifies` or `@attests` body.
6. Use [`special trace specs --id SPEC.ID`](documents://spec/SPECIAL.TRACE_COMMAND.SPECS) when you need the current claim, proof attachment, and related evidence in one packet.
7. Ask whether the artifact proves that exact claim through observable behavior or durable evidence.

Useful commands:

```sh
special specs --current
special specs EXPORT.CSV.HEADER --verbose
special trace specs --id EXPORT.CSV.HEADER
special specs --metrics
special lint
```

## What To Do With Results
@applies DOCS.SKILL_RESULT_DISPOSITION_SECTION

- If the verify proves the claim, keep it.
- If the verify proves only part of the claim, split or narrow the spec.
- If the verify checks helper mechanics instead of behavior, replace or strengthen it.
- If the behavior is not ready, mark the claim `@planned`.
- If the claim is not product behavior, move it to architecture, pattern guidance, or ordinary docs.
- If the repo has no spec surface yet, add the smallest useful [`@spec`](documents://spec/SPECIAL.SPEC_COMMAND.MARKDOWN_DECLARATIONS) and proof attachment instead of leaving the claim implicit.

@applies DOCS.SKILL_REFERENCE_HANDOFF_SECTION

Read [references/validation-checklist.md](references/validation-checklist.md) for the review rubric and [references/trigger-evals.md](references/trigger-evals.md) for trigger examples.
