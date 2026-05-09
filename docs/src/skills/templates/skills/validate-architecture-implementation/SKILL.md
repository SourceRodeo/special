---
name: validate-architecture-implementation
description: 'Use this skill when reviewing whether code matches an intended module or subsystem responsibility. Compare the module description to the attached implementation, then decide whether to keep, move, narrow, split, or add architecture annotations.'
---
@filedocuments spec SPECIAL.TRACE_COMMAND.ARCH

# Validate Architecture Implementation
@implements SPECIAL.DOCUMENTATION.SKILLS.FALLBACK.VALIDATE_ARCHITECTURE_IMPLEMENTATION
@applies DOCS.SKILL_MAIN_ENTRY

## When To Use
@applies DOCS.SKILL_TRIGGER_BOUNDARY_SECTION

Use this when a task asks:

- does this file belong to this module?
- is this module honestly implemented?
- did a refactor leave architecture annotations stale?
- is one file claiming too much ownership?
- should this code become a new module?

This can be used before or after Special is integrated.

## How To Use
@applies DOCS.SKILL_WORKFLOW_SECTION

1. Pick one intended boundary.
2. If a module id exists, run [`special arch MODULE.ID --verbose`](documents://spec/SPECIAL.MODULE_COMMAND.VERBOSE).
3. Read the module description.
4. Read the attached [`@implements`](documents://spec/SPECIAL.MODULE_PARSE.IMPLEMENTS.ITEM_SCOPE) or [`@fileimplements`](documents://spec/SPECIAL.MODULE_PARSE.IMPLEMENTS.FILE_SCOPE) body.
5. Decide whether the code actually performs that responsibility.
6. Use [`special arch MODULE.ID --metrics --verbose`](documents://spec/SPECIAL.MODULE_COMMAND.METRICS) when you need complexity, coupling, ownership, or hidden-item evidence.
7. Use [`special trace arch --id MODULE.ID`](documents://spec/SPECIAL.TRACE_COMMAND.ARCH) when you need one module-to-implementation review packet.
8. If pattern applications appear, inspect them with [`special patterns PATTERN.ID --verbose`](documents://spec/SPECIAL.PATTERNS.VERBOSE).

Useful commands:

```sh
special arch MODULE.ID --verbose
special arch MODULE.ID --metrics --verbose
special trace arch --id MODULE.ID
special arch --unimplemented
special lint
```

## What To Do With Results
@applies DOCS.SKILL_RESULT_DISPOSITION_SECTION

- If the code matches the module, keep the annotation.
- If the description is wrong, edit the [`@module`](documents://spec/SPECIAL.MODULE_COMMAND.MARKDOWN_DECLARATIONS) text where it is written.
- If the code belongs elsewhere, move [`@implements`](documents://spec/SPECIAL.MODULE_PARSE.IMPLEMENTS.ITEM_SCOPE).
- If a file-level attachment hides unrelated items, use narrower item-level attachments or split the module.
- If a current module has no implementation, add implementation or mark it planned.
- If the issue is behavior proof, switch to product specs.

@applies DOCS.SKILL_REFERENCE_HANDOFF_SECTION

Read [references/validation-checklist.md](references/validation-checklist.md) for the review rubric and [references/trigger-evals.md](references/trigger-evals.md) for trigger examples.
