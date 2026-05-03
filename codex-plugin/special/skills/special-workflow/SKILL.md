---
name: special-workflow
description: Use this skill when working in a repository that uses Special. Prefer Special MCP tools for specs, architecture, patterns, docs, lint, and health before falling back to shell commands.
---

# Special Workflow

Use Special as the repo-local contract surface for durable product behavior,
architecture ownership, adopted patterns, documentation relationships, and
traceability.

## Workflow

1. Check project status with `special_status`.
2. Inspect product claims with `special_specs`.
3. Inspect architecture ownership with `special_arch`.
4. Inspect adopted implementation patterns with `special_patterns`.
5. Validate docs relationships with `special_docs`.
6. Run structural checks with `special_lint`.
7. Use `special_health` for code-health and traceability questions.

Use `special_docs_output` only when the task explicitly needs public docs output
written. Keep the same safety expectations as the CLI: explicit configured
outputs or an explicit target/output pair, no source overwrite, no output inside
the input tree, and no overwrite of files that still contain docs evidence.

## Editing Guidance

- Treat `specs/`, architecture declarations, pattern declarations, and docs
  relationships as the durable Special surfaces.
- Do not create parallel PRD, ADR, sprint, or scratch systems for behavior that
  belongs in Special.
- When changing product behavior, update the relevant `@spec` claims and proof
  attachments.
- When changing ownership boundaries, update `@module`, `@area`, and
  `@implements`.
- When changing recurring implementation shape, update `@pattern` and
  `@applies`.
- After source or docs edits, run the narrow relevant Special check and then
  `special_lint` before calling the work done.
