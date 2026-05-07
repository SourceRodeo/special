---
name: special-workflow
description: 'Use this skill when working in a repository that uses Special. Prefer Special MCP tools to inspect repo claims, proof, ownership, patterns, docs, lint, and health before falling back to shell commands.'
---

# Special Workflow

Use Special as the repo-local contract surface for durable product behavior,
architecture ownership, adopted patterns, documentation relationships, and
traceability. Use the surfaces as one workflow, not as independent checkboxes.

## Workflow

1. Check project status with `special_status`.
2. If the repo is existing or unfamiliar, start with `special_health` and
   `special_patterns` metrics before adding annotations. Treat health as an
   investigation queue: source outside architecture suggests ownership work,
   untraced implementation suggests proof or facade work, duplicate source
   shapes suggest helper extraction or an adopted pattern, long prose outside
   docs suggests docs migration, and exact long-prose assertions suggest smaller
   semantic tests.
3. If the task is creating a new behavior or new repo slice, start with
   `special_specs` for the product claim and `special_arch` for ownership.
4. Use `special_patterns` when a repeated implementation shape appears; do not
   turn broad style advice into a pattern.
5. Use `special_docs` when public or contributor docs make claims about specs,
   modules, areas, or patterns. For docs relationship audits, use Special's
   parsed relationship view as the inventory: run `special_docs` with
   `metrics`, `verbose`, and a narrow `target` for the file or subtree being
   reviewed. Then check each linked target with the matching surface:
   `special_specs`, `special_arch`, or `special_patterns` with `verbose`.
6. Run `special_lint` after edits to catch broken ids, misplaced annotations,
   and graph errors.

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
- When changing reader-facing or contributor docs, prefer dense `documents://`
  links in docs source, then check docs metrics. Use `@documents` only when an
  entire natural block documents one target.
- When auditing docs claims, do not build the inventory with raw text search.
  Start from `special_docs` metrics plus verbose target detail so ignored files,
  docs-source rules, generated-output rules, and parsed relationship semantics
  match Special's product behavior.
- For existing repos, do not model the whole project on day one. Let health and
  pattern metrics identify one narrow slice worth making durable. Prefer a file
  or subsystem that appears in multiple queues, then rerun the same scoped
  command after one improvement.
- Do not treat every health count as a failure. Leave a signal visible when the
  local tradeoff is clear and the remaining work is intentionally deferred.
- After source or docs edits, run the narrow relevant Special check and then
  `special_lint` before calling the work done.
