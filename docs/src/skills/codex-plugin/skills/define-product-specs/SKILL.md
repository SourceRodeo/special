---
name: define-product-specs
description: 'Use this skill when turning requirements, feature ideas, bug reports, roadmap notes, or vague behavior into Special product claims. Prefer MCP tools when available, with CLI fallback.'
---
@filedocuments spec SPECIAL.SPEC_COMMAND

# Define Product Specs
@implements SPECIAL.DOCUMENTATION.SKILLS.PLUGIN.DEFINE_PRODUCT_SPECS
@applies DOCS.SKILL_MAIN_ENTRY

## When To Use
@applies DOCS.SKILL_TRIGGER_BOUNDARY_SECTION

Use this when a task needs a durable behavior claim: new feature scope, bug
expectation, release behavior, or roadmap prose that should become a current or
planned contract.

Do not use specs for implementation ownership or repeated implementation shape.

## Workflow
@applies DOCS.SKILL_WORKFLOW_SECTION

1. Check status with `special_status`.
2. Inspect existing claims with [`special_specs`](documents://spec/SPECIAL.SPEC_COMMAND); fall back to `special specs --metrics`.
3. Use [`@group`](documents://spec/SPECIAL.GROUPS.STRUCTURAL_ONLY) for navigation and [`@spec`](documents://spec/SPECIAL.SPEC_COMMAND.MARKDOWN_DECLARATIONS) for real behavior claims.
4. Mark future claims with [`@planned`](documents://spec/SPECIAL.SPEC_COMMAND.PLANNED_ONLY).
5. Attach one honest [`@verifies`](documents://spec/SPECIAL.PARSE.VERIFIES) or [`@attests`](documents://spec/SPECIAL.PARSE.ATTESTS) artifact to each current claim.
6. For review, use `special_trace` with `surface: "specs"` and the exact id; fall back to `special trace specs --id SPEC.ID`.
7. Run `special_lint` or `special lint`.

## What To Do With Results
@applies DOCS.SKILL_RESULT_DISPOSITION_SECTION

- If the proof supports only part of the claim, split or narrow the spec.
- If the behavior is not current, keep it planned.
- If the claim is internal design, move it to architecture or pattern guidance.
- If the repo has no Special surface yet, add the smallest useful spec near the
  relevant tests or docs instead of building a parallel requirements document.
