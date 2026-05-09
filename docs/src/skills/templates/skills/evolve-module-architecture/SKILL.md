---
name: evolve-module-architecture
description: 'Use this skill when changing ownership boundaries, module responsibilities, subsystem splits, or command-surface design. Capture the intended architecture with `@module`/`@area`, attach implementation with `@implements`, and keep architecture separate from product specs and patterns.'
---
@filedocuments spec SPECIAL.MODULE_COMMAND

# Evolve Module Architecture
@implements SPECIAL.DOCUMENTATION.SKILLS.FALLBACK.EVOLVE_MODULE_ARCHITECTURE
@applies DOCS.SKILL_MAIN_ENTRY

## When To Use
@applies DOCS.SKILL_TRIGGER_BOUNDARY_SECTION

Use this when a developer task changes how the codebase is organized:

- splitting or merging modules
- moving responsibilities between subsystems
- adding a new command or major internal boundary
- documenting who owns a file or component
- introducing Special architecture annotations to a repo

Do not use this to prove product behavior. Use specs for behavior and patterns for repeated implementation approaches.

## How To Use
@applies DOCS.SKILL_WORKFLOW_SECTION

1. Identify the boundary you are changing.
2. If Special is present, run [`special arch MODULE.ID --verbose`](documents://spec/SPECIAL.MODULE_COMMAND.VERBOSE) or [`special arch --metrics`](documents://spec/SPECIAL.MODULE_COMMAND.METRICS).
3. If there is no module yet, add the smallest useful [`@module ID`](documents://spec/SPECIAL.MODULE_COMMAND.MARKDOWN_DECLARATIONS) declaration.
4. Use [`@area ID`](documents://spec/SPECIAL.MODULE_COMMAND.AREA_NODES) only for structure.
5. Attach code with [`@implements ID`](documents://spec/SPECIAL.MODULE_PARSE.IMPLEMENTS.ITEM_SCOPE) or [`@fileimplements ID`](documents://spec/SPECIAL.MODULE_PARSE.IMPLEMENTS.FILE_SCOPE).
6. Edit the annotation where it is written when the module description is wrong.
7. Move or narrow implementation annotations when ownership is too broad or attached to the wrong code.
8. If the change also introduces a repeated approach, use `use-project-patterns`.

Example:

```rust
/**
@module EXPORT.CSV
Builds CSV export output from selected records and columns.
*/
// @fileimplements EXPORT.CSV
```

Useful commands:

```sh
special arch
special arch EXPORT.CSV --verbose
special arch EXPORT.CSV --metrics --verbose
special lint
```

## What To Do With Results
@applies DOCS.SKILL_RESULT_DISPOSITION_SECTION

- If a current module has no implementation, add [`@implements`](documents://spec/SPECIAL.MODULE_PARSE.IMPLEMENTS.ITEM_SCOPE) or mark the module planned.
- If code implements the wrong module, move the annotation.
- If one file owns too much, split or narrow the module/implementation boundary.
- If a module description is really a product claim, move it to `@spec`.
- After editing, run [`special arch`](documents://spec/SPECIAL.MODULE_COMMAND) and [`special lint`](documents://spec/SPECIAL.LINT_COMMAND) when available.
