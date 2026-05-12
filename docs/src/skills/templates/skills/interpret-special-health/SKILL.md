---
name: interpret-special-health
description: 'Use this skill when reading Special health output for an existing or unfamiliar repository. Treat health as broad analysis that suggests follow-up work, not as a mandatory zero-count checklist.'
---
@filedocuments spec SPECIAL.HEALTH_COMMAND

# Interpret Special Health
@implements SPECIAL.DOCUMENTATION.SKILLS.FALLBACK.INTERPRET_SPECIAL_HEALTH
@applies DOCS.SKILL_MAIN_ENTRY

## When To Use
@applies DOCS.SKILL_TRIGGER_BOUNDARY_SECTION

Use this when starting in an existing repo, after a change touches an unfamiliar
area, or when health output is large enough that the next step is unclear.

Do not use this to prove one explicit relationship. Use `special trace` for a
known spec, docs claim, module, or pattern chain.

## Workflow
@applies DOCS.SKILL_WORKFLOW_SECTION

1. Start broad with [`special health --metrics`](documents://spec/SPECIAL.HEALTH_COMMAND.METRICS):

   ```sh
   special health --metrics
   ```

2. Pick one cluster or path before editing:

   ```sh
   special health --metrics --verbose --target src/billing
   ```

3. Interpret signals by surface:
   source outside architecture suggests a missing or stale module boundary;
   untraced implementation suggests product behavior that lacks direct proof or
   code that is too coupled to an I/O boundary; duplicate shapes and pattern
   clusters suggest helper extraction or an adopted pattern; uncaptured prose
   suggests docs migration; long prose test literals suggest smaller semantic
   assertions, structured output checks, or fixtures.
4. Move to the focused command for the chosen surface: `special specs`,
   `special arch`, `special patterns`, `special docs`, or `special trace`.
5. Rerun the same scoped health command after one improvement.

## What To Do With Results
@applies DOCS.SKILL_RESULT_DISPOSITION_SECTION

- Do not make every number zero by adding annotations mechanically.
- Do not hide a signal unless the path is generated, fixture-heavy, or otherwise
  intentionally outside that health queue.
- Prefer architectural fixes when business logic is only reachable through a
  command, route, script, or framework boundary.
- Prefer pattern work when repeated source has an identifiable implementation
  shape.
- Leave a visible signal when the tradeoff is clear and no durable Special
  relationship is appropriate.
