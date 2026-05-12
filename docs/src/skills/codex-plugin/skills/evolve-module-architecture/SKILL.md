---
name: evolve-module-architecture
description: 'Use this skill when changing ownership boundaries, module responsibilities, subsystem splits, or command-surface design. Prefer MCP arch tools, with CLI fallback.'
---
@filedocuments spec SPECIAL.MODULE_COMMAND

# Evolve Module Architecture
@implements SPECIAL.DOCUMENTATION.SKILLS.PLUGIN.EVOLVE_MODULE_ARCHITECTURE
@applies DOCS.SKILL_MAIN_ENTRY

## When To Use
@applies DOCS.SKILL_TRIGGER_BOUNDARY_SECTION

Use this when a task changes code ownership: splitting modules, moving
responsibilities, adding a command boundary, or introducing architecture
annotations.

Do not use architecture to prove product behavior.

## Workflow
@applies DOCS.SKILL_WORKFLOW_SECTION

1. Inspect the current boundary with [`special_arch`](documents://spec/SPECIAL.MODULE_COMMAND); fall back to `special arch --metrics`.
2. Add the smallest useful [`@module`](documents://spec/SPECIAL.MODULE_COMMAND.MARKDOWN_DECLARATIONS) when no boundary exists.
3. Use [`@area`](documents://spec/SPECIAL.MODULE_COMMAND.AREA_NODES) only for structure.
4. Attach implementation with [`@implements`](documents://spec/SPECIAL.MODULE_PARSE.IMPLEMENTS.ITEM_SCOPE) or [`@fileimplements`](documents://spec/SPECIAL.MODULE_PARSE.IMPLEMENTS.FILE_SCOPE).
5. Use `special_trace` with `surface: "arch"` for a module review packet.
6. Run `special_lint`.

## What To Do With Results
@applies DOCS.SKILL_RESULT_DISPOSITION_SECTION

- Move or narrow implementation annotations when ownership is too broad.
- Mark future architecture planned instead of leaving current modules
  unimplemented.
- Move behavior promises to specs and repeated implementation choices to
  patterns.
