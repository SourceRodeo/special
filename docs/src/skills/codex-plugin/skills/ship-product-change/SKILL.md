---
name: ship-product-change
description: 'Use this skill when adding a feature, fixing a bug, or changing behavior. Keep specs, implementation, proof, architecture, patterns, docs, and lint aligned.'
---
@filedocuments spec SPECIAL.TRACE_COMMAND.SPECS

# Ship Product Change
@implements SPECIAL.DOCUMENTATION.SKILLS.PLUGIN.SHIP_PRODUCT_CHANGE
@applies DOCS.SKILL_MAIN_ENTRY

## When To Use
@applies DOCS.SKILL_TRIGGER_BOUNDARY_SECTION

Use this for ordinary behavior work: a feature, bug fix, CLI/API change, output
change, or release-impacting product change.

## Workflow
@applies DOCS.SKILL_WORKFLOW_SECTION

1. Start with the behavior change in one sentence.
2. Use `special_specs` or [`special specs`](documents://spec/SPECIAL.SPEC_COMMAND) to find the existing claim.
3. Add or update the narrow [`@spec`](documents://spec/SPECIAL.SPEC_COMMAND.MARKDOWN_DECLARATIONS) if no claim exists.
4. Implement the change.
5. Attach or update [`@verifies`](documents://spec/SPECIAL.PARSE.VERIFIES) or [`@attests`](documents://spec/SPECIAL.PARSE.ATTESTS).
6. Update [`@module` and `@implements`](documents://spec/SPECIAL.MODULE_PARSE.IMPLEMENTS.ITEM_SCOPE) if ownership changed.
7. Update [`@pattern` and `@applies`](documents://spec/SPECIAL.PATTERNS.SOURCE_APPLICATIONS) when the implementation follows an adopted approach.
8. Use `special_trace` for the touched spec, module, docs relationship, or pattern before calling the change complete.
9. Finish with `special_lint` and the relevant scoped command.

## What To Do With Results
@applies DOCS.SKILL_RESULT_DISPOSITION_SECTION

- If health reports touched code, inspect whether it needs real architecture or
  proof work rather than suppressing it.
- If docs changed, trace the docs relationship and compare prose to the linked
  target.
- If the change is internal-only, keep product specs focused on behavior and use
  architecture or patterns for implementation intent.
