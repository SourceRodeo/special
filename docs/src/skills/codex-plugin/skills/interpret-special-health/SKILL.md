---
name: interpret-special-health
description: 'Use this skill when reading Special health output for an existing or unfamiliar repository. Treat health as broad analysis that suggests scoped follow-up work.'
---
@filedocuments spec SPECIAL.HEALTH_COMMAND

# Interpret Special Health
@implements SPECIAL.DOCUMENTATION.SKILLS.PLUGIN.INTERPRET_SPECIAL_HEALTH
@applies DOCS.SKILL_MAIN_ENTRY

## When To Use
@applies DOCS.SKILL_TRIGGER_BOUNDARY_SECTION

Use this for existing repos, unfamiliar code, or a touched path whose health
signals need triage.

## Workflow
@applies DOCS.SKILL_WORKFLOW_SECTION

1. Run [`special_health`](documents://spec/SPECIAL.HEALTH_COMMAND.METRICS) with metrics; fall back to `special health --metrics`.
2. Pick one cluster or path.
3. Rerun scoped health with item detail:

   ```sh
   special health --metrics --verbose --target src/billing
   ```

4. Move to the focused surface: specs for behavior support, arch for ownership,
   patterns for repeated shapes, docs for reader claims, trace for one explicit
   relationship.
5. Rerun the same scoped health command after one improvement.

## What To Do With Results
@applies DOCS.SKILL_RESULT_DISPOSITION_SECTION

- Do not force every count to zero with mechanical annotations.
- Prefer architectural fixes when business logic is only reachable through an
  I/O boundary.
- Prefer pattern or helper work when repeated source is the signal.
- Leave signals visible when the tradeoff is intentional and durable.
