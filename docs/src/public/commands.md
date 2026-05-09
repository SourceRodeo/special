@applies DOCS.REFERENCE_CATALOG_PAGE
# Command Reference

Use `special --help` for exact local help. This reference explains the common
command shapes and the decision each output supports.

Each focused resource command answers questions about one Special surface:
`specs`, `arch`, `patterns`, and `docs` inspect declared resources and their
direct evidence. [`special health`](documents://spec/SPECIAL.HEALTH_COMMAND)
is broader: it reports inferred repo signals, gaps, and cleanup queues, often
before a repository has many annotations. [`special trace`](documents://spec/SPECIAL.TRACE_COMMAND)
follows the relevant relationship chain when one claim, doc link, module, or
pattern needs detailed review.

@implements SPECIAL.DOCUMENTATION.PUBLIC.REFERENCE.COMMANDS.SPECS
@applies DOCS.COMMAND_REFERENCE_ENTRY
## `special specs`

Use [`special specs`](documents://spec/SPECIAL.SPEC_COMMAND) to inspect product
claims and proof attachments.

```sh
special specs
special specs --unverified
special specs EXPORT.CSV.HEADERS --verbose
special specs --metrics
```

Representative output shape for a small repo:

```text
EXPORT.CSV.HEADERS
  CSV exports include a header row with the selected column names.
  verifies: 1
  attests: 0
```

Decision supported: whether a claim exists, whether it is current or planned,
and whether direct support is attached.

Contract details: `special specs` supports
[current-only](documents://spec/SPECIAL.SPEC_COMMAND.CURRENT_ONLY),
[planned-only](documents://spec/SPECIAL.SPEC_COMMAND.PLANNED_ONLY),
[unverified](documents://spec/SPECIAL.SPEC_COMMAND.UNVERIFIED),
[id-scoped](documents://spec/SPECIAL.SPEC_COMMAND.ID_SCOPE),
[verbose](documents://spec/SPECIAL.SPEC_COMMAND.VERBOSE),
[metrics](documents://spec/SPECIAL.SPEC_COMMAND.METRICS),
[JSON](documents://spec/SPECIAL.SPEC_COMMAND.JSON), and
[HTML](documents://spec/SPECIAL.SPEC_COMMAND.HTML) views.

@implements SPECIAL.DOCUMENTATION.PUBLIC.REFERENCE.COMMANDS.ARCH
@applies DOCS.COMMAND_REFERENCE_ENTRY
## `special arch`

Use [`special arch`](documents://spec/SPECIAL.MODULE_COMMAND) to inspect declared
areas, modules, and implementation ownership.

```sh
special arch
special arch --unimplemented
special arch APP.EXPORT --verbose
special arch APP.EXPORT --metrics
```

Representative output:

```text
APP.EXPORT
  implements: src/export.ts
```

Decision supported: whether code has an explicit architecture owner and whether
declared modules are still only aspirational.

Source metrics currently cover [Rust](documents://group/SPECIAL.MODULE_COMMAND.METRICS.RUST), [TypeScript/TSX](documents://spec/SPECIAL.MODULE_COMMAND.METRICS.TYPESCRIPT), [Go](documents://spec/SPECIAL.MODULE_COMMAND.METRICS.GO), and [Python](documents://spec/SPECIAL.MODULE_COMMAND.METRICS.PYTHON). Python reports item-level signals; the other built-in source languages also expose complexity and quality metrics where the local parser/tool boundary supports them.

Contract details: `special arch` supports
[current-only](documents://spec/SPECIAL.MODULE_COMMAND.CURRENT_ONLY),
[planned-only](documents://spec/SPECIAL.MODULE_COMMAND.PLANNED_ONLY),
[unimplemented](documents://spec/SPECIAL.MODULE_COMMAND.UNIMPLEMENTED),
[id-scoped](documents://spec/SPECIAL.MODULE_COMMAND.ID_SCOPE),
[verbose](documents://spec/SPECIAL.MODULE_COMMAND.VERBOSE),
[metrics](documents://spec/SPECIAL.MODULE_COMMAND.METRICS),
[JSON](documents://spec/SPECIAL.MODULE_COMMAND.JSON), and
[HTML](documents://spec/SPECIAL.MODULE_COMMAND.HTML) views. Metrics include
[complexity explanations](documents://spec/SPECIAL.MODULE_COMMAND.METRICS.COMPLEXITY.EXPLANATIONS),
[coupling](documents://spec/SPECIAL.MODULE_COMMAND.METRICS.COUPLING),
[quality](documents://spec/SPECIAL.MODULE_COMMAND.METRICS.QUALITY), and
[item-signal explanations](documents://spec/SPECIAL.MODULE_COMMAND.METRICS.ITEM_SIGNALS.EXPLANATIONS).
Verbose JSON metrics also include dependency targets for language packs that
can report them.

@implements SPECIAL.DOCUMENTATION.PUBLIC.REFERENCE.COMMANDS.PATTERNS
@applies DOCS.COMMAND_REFERENCE_ENTRY
## `special patterns`

Use [`special patterns`](documents://spec/SPECIAL.PATTERNS.COMMAND) to inspect
declared repeated implementation structures and their known applications.

```sh
special patterns
special patterns EXPORT.LABEL_VALUE_COLUMNS --verbose
special patterns --metrics
special patterns --json
```

Representative output:

```text
EXPORT.LABEL_VALUE_COLUMNS
  Export tables should build columns as ordered label/value pairs.
  applications: 3
  modules: APP.EXPORT
```

Decision supported: whether a repeated structure is intentionally named,
where it is applied, and whether known applications still fit each other.

Contract details: `special patterns` supports
[id-scoped](documents://spec/SPECIAL.PATTERNS.ID_SCOPE),
[verbose](documents://spec/SPECIAL.PATTERNS.VERBOSE), and
[metrics](documents://spec/SPECIAL.PATTERNS.METRICS) views. Metrics report
[similarity](documents://spec/SPECIAL.PATTERNS.METRICS.SIMILARITY). Raw
missing-application and unannotated-cluster queues belong to
[`special health`](documents://spec/SPECIAL.HEALTH_COMMAND).

@implements SPECIAL.DOCUMENTATION.PUBLIC.REFERENCE.COMMANDS.DOCS
@applies DOCS.COMMAND_REFERENCE_ENTRY
## `special docs`

Use [`special docs`](documents://spec/SPECIAL.DOCS_COMMAND) to validate docs
relationships and build generated docs output.

```sh
special docs
special docs --metrics
special docs --json
special docs --target docs/src
special docs build
special docs build docs/src/public docs
```

Representative output:

```text
special docs metrics
  relationship inventory
    total references: 42
      link references: 42
      @documents references: 0
      @filedocuments references: 0
  generated docs graph
    generated pages: 7
    local doc links: 11
    broken local doc links: 0
    orphan pages: 0
    reachable from entrypoints: 7/7 page(s), 1 entrypoint(s)
```

Decision supported: whether docs links resolve, whether generated docs pages are
connected, and whether docs output can be built safely. Use
[`special health --metrics`](documents://spec/SPECIAL.HEALTH_COMMAND.DOCS.COVERAGE)
when the question is which specs, modules, or patterns lack public docs. Use
[`special trace`](documents://spec/SPECIAL.TRACE_COMMAND) when one docs
relationship needs its full resource chain.

Contract details: `special docs` supports
[target scoping](documents://spec/SPECIAL.DOCS_COMMAND.TARGET),
[metrics](documents://spec/SPECIAL.DOCS_COMMAND.METRICS),
[relationship metrics](documents://spec/SPECIAL.DOCS_COMMAND.METRICS.RELATIONSHIPS),
[generated docs graph metrics](documents://spec/SPECIAL.DOCS_COMMAND.METRICS.INTERCONNECTIVITY), and
[configured output builds](documents://spec/SPECIAL.DOCS_COMMAND.OUTPUT.CONFIG).
Generated output
[rewrites document links](documents://spec/SPECIAL.DOCS.LINKS.OUTPUT),
[removes authoring annotations](documents://spec/SPECIAL.DOCS_COMMAND.OUTPUT.AUTHORING_LINES),
and
[refuses to overwrite docs evidence-bearing sources](documents://spec/SPECIAL.DOCS_COMMAND.OUTPUT.SAFETY)
by accident.

@implements SPECIAL.DOCUMENTATION.PUBLIC.REFERENCE.COMMANDS.TRACE
@applies DOCS.COMMAND_REFERENCE_ENTRY
## `special trace`

Use [`special trace`](documents://spec/SPECIAL.TRACE_COMMAND) when a review
needs the explicit relationship packet, not an aggregate metric or a truth
judgment.

```sh
special trace specs --id EXPORT.CSV.HEADERS
special trace docs --target docs/src/public/commands.md
special trace arch --id APP.EXPORT
special trace patterns --id EXPORT.LABEL_VALUE_COLUMNS
special trace docs --json --output /tmp/docs-trace.json
```

Use the surface that owns the relationship you are reviewing:

| Surface | Packet focus |
| --- | --- |
| `trace specs` | current specs plus verifier and attestation evidence bodies |
| `trace docs` | docs relationships, surrounding prose, target declaration, and target evidence |
| `trace arch` | modules or areas with implementation attachments and pattern applications |
| `trace patterns` | pattern definitions with applications and module joins |

Representative output shape for a small repo:

```text
special trace specs
packets: 1

spec EXPORT.CSV.HEADERS @ specs/export.md:7
  text: CSV exports include a header row with selected column names.
  evidence:
    @verifies @ tests/export.test.ts:12
      body @ tests/export.test.ts:13
```

Decision supported: which exact source text, target declaration, and attached
evidence a human or agent should review for a docs, spec, architecture, or
pattern alignment audit.

For docs audits, `trace docs` follows the same documentation relationship graph
as [`special docs`](documents://spec/SPECIAL.DOCS_COMMAND): markdown
[`documents://`](documents://spec/SPECIAL.DOCS.LINKS.POLYMORPHIC) links,
[`@documents`](documents://spec/SPECIAL.DOCS.DOCUMENTS_LINES), and
[`@filedocuments`](documents://spec/SPECIAL.DOCS.DOCUMENTS_LINES).
That makes it useful for checking whether surrounding prose matches the linked
spec, module, area, group, or pattern. It does not run tests, prove claims, or
label evidence quality. It only gathers the current source packet so a reviewer
can make that judgment.

Contract details: `special trace` can packetize
[spec proof attachments](documents://spec/SPECIAL.TRACE_COMMAND.SPECS),
[docs relationships](documents://spec/SPECIAL.TRACE_COMMAND.DOCS),
[architecture ownership](documents://spec/SPECIAL.TRACE_COMMAND.ARCH), and
[pattern applications](documents://spec/SPECIAL.TRACE_COMMAND.PATTERNS).
Use [id and target filters](documents://spec/SPECIAL.TRACE_COMMAND.FILTERS) for
focused reviews: `--id` accepts an exact id or dotted subtree, and repeated
`--target PATH` scopes packets to source files or subtrees. Use `--json` for
machine-readable audit bundles and `--output PATH` to save packets without
mixing them into terminal output.

@implements SPECIAL.DOCUMENTATION.PUBLIC.REFERENCE.COMMANDS.DIFF
@applies DOCS.COMMAND_REFERENCE_ENTRY
## `special diff`

Use [`special diff`](documents://spec/SPECIAL.DIFF_COMMAND) after editing a
repo to review explicit Special relationships touched by the current VCS
changes.

```sh
special diff
special diff --metrics
special diff --target src/export.ts --verbose
special diff --id APP.EXPORT
```

Representative output shape for a changed repo:

```text
relationship diff
  changed paths: 2
  affected relationships: 6
  current relationships: 84
  @verifies spec APP.EXPORT.CSV at tests/export.test.ts:12 [affected by tests/export.test.ts]
```

Decision supported: which specs, modules, patterns, or docs relationships need
review because their source or target endpoint is inside the current VCS change.
Use your VCS for the old/new file diff; use Special to find the relationship
review queue.

Contract details: `special diff` uses the
[declared VCS backend](documents://spec/SPECIAL.CONFIG.SPECIAL_TOML.VCS) and
gracefully falls back to a full explicit relationship view when
[`vcs` is omitted or disabled](documents://spec/SPECIAL.DIFF_COMMAND.NO_VCS).
[`--metrics`](documents://spec/SPECIAL.DIFF_COMMAND.METRICS) reports affected
relationship counts by relationship kind, target kind, and source path.
[`--verbose`](documents://spec/SPECIAL.DIFF_COMMAND.VERBOSE) includes current
endpoint content for review.

@implements SPECIAL.DOCUMENTATION.PUBLIC.REFERENCE.COMMANDS.HEALTH
@applies DOCS.COMMAND_REFERENCE_ENTRY
## `special health`

Use [`special health`](documents://spec/SPECIAL.HEALTH_COMMAND) for repo-wide
signals that go deeper than explicit graph edges or sit outside the graph.

```sh
special health
special health --metrics
special health --metrics --verbose
special health --target src/export.ts --symbol exportCsv
special health --json
special health --html
```

Representative output shape for an existing repo:

```text
special health
summary
  source outside architecture: 12
  untraced implementation: 34
  duplicate source shapes: 7
  possible pattern clusters: 2
  possible missing pattern applications: 1
  uncaptured prose outside docs: 3
  long prose test literals: 0
duplicate source shapes by file
  src/billing/export.ts: 4
  src/billing/refunds.ts: 3
```

Decision supported: which raw inferred queues should be promoted into specs,
architecture, patterns, docs, or test changes.

Health uses built-in source analysis for Rust, TypeScript/TSX, Go, and Python.
Tool-backed traceability depends on the local project tools for the language;
when a tool is unavailable or a language is using parser-backed static edges,
the health output reports the boundary instead of treating missing evidence as
proof.

Contract details: `special health` supports
[target scoping](documents://spec/SPECIAL.HEALTH_COMMAND.TARGET),
[symbol scoping](documents://spec/SPECIAL.HEALTH_COMMAND.SYMBOL),
[within scoping](documents://spec/SPECIAL.HEALTH_COMMAND.WITHIN),
[verbose evidence](documents://spec/SPECIAL.HEALTH_COMMAND.VERBOSE),
[JSON](documents://spec/SPECIAL.HEALTH_COMMAND.JSON), and
[HTML](documents://spec/SPECIAL.HEALTH_COMMAND.HTML). Metrics cover
[source outside architecture](documents://spec/SPECIAL.HEALTH_COMMAND.UNOWNED_ITEMS),
[duplicate source shapes](documents://spec/SPECIAL.HEALTH_COMMAND.DUPLICATION),
[cleanup targets](documents://spec/SPECIAL.HEALTH_COMMAND.METRICS.CLEANUP_TARGETS),
[docs coverage](documents://spec/SPECIAL.HEALTH_COMMAND.DOCS.COVERAGE),
[untraced implementation](documents://spec/SPECIAL.HEALTH_COMMAND.TRACEABILITY),
[missing pattern applications](documents://spec/SPECIAL.HEALTH_COMMAND.PATTERNS.MISSING_APPLICATIONS),
[pattern clusters](documents://spec/SPECIAL.HEALTH_COMMAND.PATTERNS.CLUSTERS.INTERPRETATION),
[uncaptured prose outside docs](documents://spec/SPECIAL.HEALTH_COMMAND.DOCS.LONG_PROSE_OUTSIDE_DOCS),
and [long prose test literals](documents://spec/SPECIAL.HEALTH_COMMAND.TEST_QUALITY.LONG_PROSE_TEST_LITERALS).

@implements SPECIAL.DOCUMENTATION.PUBLIC.REFERENCE.COMMANDS.MCP
@applies DOCS.COMMAND_REFERENCE_ENTRY
## `special mcp`

Use [`special mcp`](documents://spec/SPECIAL.MCP_COMMAND) to run the stdio MCP
server for controlled agent access.

```sh
special mcp
```

Decision supported: whether an agent should access Special through bounded tools
instead of scraping arbitrary repo files first.

Contract details: `special mcp` exposes
[bounded Special tools](documents://spec/SPECIAL.MCP_COMMAND.TOOLS),
[docs output](documents://spec/SPECIAL.MCP_COMMAND.DOCS_OUTPUT), and a
[plugin version notice](documents://spec/SPECIAL.MCP_COMMAND.PLUGIN_VERSION_NOTICE)
when the plugin and binary versions differ.

@implements SPECIAL.DOCUMENTATION.PUBLIC.REFERENCE.COMMANDS.LINT
@applies DOCS.COMMAND_REFERENCE_ENTRY
## `special lint`

Use [`special lint`](documents://spec/SPECIAL.LINT_COMMAND) before committing
annotation changes.

```sh
special lint
```

Decision supported: whether ids, references, and lifecycle markers are
structurally valid.

Contract details: lint catches
[duplicate ids](documents://spec/SPECIAL.LINT_COMMAND.DUPLICATE_IDS),
[unknown verify refs](documents://spec/SPECIAL.LINT_COMMAND.UNKNOWN_VERIFY_REFS),
[unknown attest refs](documents://spec/SPECIAL.LINT_COMMAND.UNKNOWN_ATTEST_REFS),
[unknown implements refs](documents://spec/SPECIAL.LINT_COMMAND.UNKNOWN_IMPLEMENTS_REFS),
[orphan verifies](documents://spec/SPECIAL.LINT_COMMAND.ORPHAN_VERIFIES),
[intermediate specs](documents://spec/SPECIAL.LINT_COMMAND.INTERMEDIATE_SPECS),
[intermediate modules](documents://spec/SPECIAL.LINT_COMMAND.INTERMEDIATE_MODULES),
and [invalid planned-scope usage](documents://spec/SPECIAL.LINT_COMMAND.PLANNED_SCOPE).

@implements SPECIAL.DOCUMENTATION.PUBLIC.REFERENCE.COMMANDS.INIT
@applies DOCS.COMMAND_REFERENCE_ENTRY
## `special init`

Use `special init` to create a starter
[`special.toml`](documents://spec/SPECIAL.CONFIG.SPECIAL_TOML) in a repo without
an active config.

```sh
special init
```

Decision supported: whether the repo has an explicit Special root and generated
starter policy.

Contract details: `special init`
[creates starter config](documents://spec/SPECIAL.INIT.CREATES_SPECIAL_TOML),
[does not overwrite existing config](documents://spec/SPECIAL.INIT.DOES_NOT_OVERWRITE_SPECIAL_TOML),
[rejects nested active config](documents://spec/SPECIAL.INIT.REJECTS_NESTED_ACTIVE_CONFIG), and
[surfaces discovery errors](documents://spec/SPECIAL.INIT.SURFACES_DISCOVERY_ERRORS).

@implements SPECIAL.DOCUMENTATION.PUBLIC.REFERENCE.COMMANDS.SKILLS
@applies DOCS.COMMAND_REFERENCE_ENTRY
## `special skills`

Use [`special skills`](documents://spec/SPECIAL.SKILLS.COMMAND.HELP) to print or
install bundled workflow skills when a plugin path is not available.

```sh
special skills
special skills install
special skills install ship-product-change
special skills install --destination project
```

Decision supported: which local skill surface an agent should use for a
Special-aware workflow.

Contract details: `special skills` can
[print one bundled skill](documents://spec/SPECIAL.SKILLS.COMMAND.EMITS_SKILL_TO_STDOUT),
[install one skill](documents://spec/SPECIAL.SKILLS.COMMAND.INSTALL_SUBCOMMAND.ONE_SKILL),
[install all skills](documents://spec/SPECIAL.SKILLS.COMMAND.INSTALL_SUBCOMMAND.ALL_SKILLS_DEFAULT),
[install to a project destination](documents://spec/SPECIAL.SKILLS.COMMAND.INSTALL_SUBCOMMAND.PROJECT_DESTINATION),
[install to a global destination](documents://spec/SPECIAL.SKILLS.COMMAND.INSTALL_SUBCOMMAND.GLOBAL_DESTINATION),
and [preserve bundled references](documents://spec/SPECIAL.SKILLS.COMMAND.BUNDLES_REFERENCES_FOR_PROGRESSIVE_DISCLOSURE).
