---
name: validate-architecture-implementation
description: 'Use this skill when reviewing whether code matches an intended module or subsystem responsibility. Compare the module description to the attached implementation.'
---
@filedocuments spec SPECIAL.TRACE_COMMAND.ARCH

# Validate Architecture Implementation
@implements SPECIAL.DOCUMENTATION.SKILLS.PLUGIN.VALIDATE_ARCHITECTURE_IMPLEMENTATION
@applies DOCS.SKILL_MAIN_ENTRY

## When To Use
@applies DOCS.SKILL_TRIGGER_BOUNDARY_SECTION

Use this when deciding whether a file belongs to a module, whether a module is
honestly implemented, or whether a refactor left stale architecture annotations.

## Workflow
@applies DOCS.SKILL_WORKFLOW_SECTION

1. Pick one module or area.
2. Run [`special_trace`](documents://spec/SPECIAL.TRACE_COMMAND.ARCH) with `surface: "arch"` and the id; fall back to `special trace arch --id MODULE.ID`.
3. Read the module description.
4. Read each [`@implements`](documents://spec/SPECIAL.MODULE_PARSE.IMPLEMENTS.ITEM_SCOPE) or [`@fileimplements`](documents://spec/SPECIAL.MODULE_PARSE.IMPLEMENTS.FILE_SCOPE) body.
5. Decide whether the code actually performs the module responsibility.
6. Use `special_arch` with metrics when complexity, coupling, or ownership
   evidence matters.

## What To Do With Results
@applies DOCS.SKILL_RESULT_DISPOSITION_SECTION

- Keep aligned module and implementation attachments.
- Edit the module text when the responsibility is wrong.
- Move the implementation annotation when the code belongs elsewhere.
- Split broad file-level attachments when they hide unrelated items.
- Switch to specs when the issue is product behavior.
