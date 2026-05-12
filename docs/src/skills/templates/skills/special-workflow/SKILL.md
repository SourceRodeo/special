---
name: special-workflow
description: 'Use this skill when working in a repository that uses Special. Use Special MCP tools when available, and use equivalent CLI commands when MCP is unavailable.'
---
@filedocuments spec SPECIAL.MCP_COMMAND.TOOLS

# Special Workflow
@implements SPECIAL.DOCUMENTATION.SKILLS.FALLBACK.SPECIAL_WORKFLOW
@applies DOCS.SKILL_MAIN_ENTRY

Use Special as the repo-local contract surface for durable product behavior,
architecture ownership, adopted patterns, documentation relationships, and
traceability. Use the surfaces as one workflow, not as independent checkboxes.

## When To Use
@applies DOCS.SKILL_TRIGGER_BOUNDARY_SECTION

Use this when the repository already uses Special and the task needs repo-local
claims, proof, ownership, patterns, docs, trace, health, or lint context.

## Workflow
@applies DOCS.SKILL_WORKFLOW_SECTION

1. Check project status with `special_status` when MCP is available, or run
   `special` and `special lint` through the CLI.
2. If the repo is existing or unfamiliar, start with [`special_health`](documents://spec/SPECIAL.HEALTH_COMMAND.METRICS) and
   [`special_patterns`](documents://spec/SPECIAL.PATTERNS.METRICS) metrics before adding annotations. Treat health as an
   investigation queue: source outside architecture suggests ownership work,
   untraced implementation suggests proof or facade work, duplicate source
   shapes suggest helper extraction or an adopted pattern, long prose outside
   docs suggests docs migration, and long prose test literals suggest smaller
   semantic tests, structured outputs, or fixtures.
3. If the task is creating a new behavior or new repo slice, start with
   [`special_specs`](documents://spec/SPECIAL.SPEC_COMMAND) for the product claim and [`special_arch`](documents://spec/SPECIAL.MODULE_COMMAND) for ownership.
4. Use [`special_patterns`](documents://spec/SPECIAL.PATTERNS.COMMAND) when a repeated implementation shape appears; do not
   turn broad style advice into a pattern.
5. Use [`special_docs`](documents://spec/SPECIAL.DOCS_COMMAND) when public or contributor docs make claims about specs,
   modules, areas, or patterns. For docs relationship audits, use Special's
   parsed relationship view as the inventory: run `special_docs` with
   `metrics`, `verbose`, and a narrow `target` for the file or subtree being
   reviewed. Then use [`special_trace`](documents://spec/SPECIAL.TRACE_COMMAND) on the relevant surface to inspect the
   current source text, linked target, and attached evidence in one packet.
6. Use [`special_trace`](documents://spec/SPECIAL.TRACE_COMMAND) when the task is an explicit relationship audit:
   docs-to-target, spec-to-proof, module-to-implementation, or
   pattern-to-application. Trace packets are deterministic context bundles, not
   truth judgments.
7. Run [`special_lint`](documents://spec/SPECIAL.LINT_COMMAND) after edits to catch broken ids, misplaced annotations,
   and graph errors.

Use `special_docs_output` only when the task explicitly needs [public docs output](documents://spec/SPECIAL.MCP_COMMAND.DOCS_OUTPUT)
written. Keep the same safety expectations as the CLI: explicit configured
outputs or an explicit target/output pair, no source overwrite, no output inside
the input tree, and no overwrite of files that still contain docs evidence.

If MCP tools are unavailable, use the equivalent CLI commands:

| MCP tool | CLI fallback |
| --- | --- |
| `special_status` | `special` and `special lint` |
| `special_specs` | `special specs ...` |
| `special_arch` | `special arch ...` |
| `special_patterns` | `special patterns ...` |
| `special_docs` | `special docs ...` |
| `special_docs_output` | `special docs build ...` |
| `special_trace` | `special trace ...` |
| `special_health` | `special health ...` |
| `special_lint` | `special lint` |

## Editing Guidance
@applies DOCS.SKILL_RESULT_DISPOSITION_SECTION

- Treat `specs/`, architecture declarations, pattern declarations, and docs
  relationships as the durable Special surfaces.
- Do not create parallel PRD, ADR, sprint, or scratch systems for behavior that
  belongs in Special.
- When changing product behavior, update the relevant [`@spec`](documents://spec/SPECIAL.SPEC_COMMAND.MARKDOWN_DECLARATIONS) claims and proof
  attachments.
- When changing ownership boundaries, update [`@module`](documents://spec/SPECIAL.MODULE_COMMAND.MARKDOWN_DECLARATIONS), [`@area`](documents://spec/SPECIAL.MODULE_COMMAND.AREA_NODES), and
  [`@implements`](documents://spec/SPECIAL.MODULE_PARSE.IMPLEMENTS.ITEM_SCOPE).
- When changing recurring implementation shape, update [`@pattern`](documents://spec/SPECIAL.PATTERNS.DEFINITIONS) and
  [`@applies`](documents://spec/SPECIAL.PATTERNS.SOURCE_APPLICATIONS).
- When changing reader-facing or contributor docs, use [`documents://`](documents://spec/SPECIAL.DOCS.LINKS.POLYMORPHIC)
  links only where the surrounding prose actually documents the linked target.
  Use ordinary markdown links for navigation or "see also" references. Use
  [`@documents`](documents://spec/SPECIAL.DOCS.DOCUMENTS_LINES) when an entire natural block documents one target.
- When auditing docs claims, do not build the inventory with raw text search.
  Start from `special_docs` metrics plus verbose target detail so ignored files,
  docs-source rules, generated-output rules, and parsed relationship semantics
  match Special's product behavior. Use `special_trace` for the review packets
  you hand to a human or model.
- For existing repos, do not model the whole project on day one. Let health and
  pattern metrics identify one narrow slice worth making durable. Prefer a file
  or subsystem that appears in multiple queues, then rerun the same scoped
  command after one improvement.
- Do not treat every health count as a failure. Leave a signal visible when the
  local tradeoff is clear and the remaining work is intentionally deferred.
- After source or docs edits, run the narrow relevant Special check and then
  `special_lint` before calling the work done.
