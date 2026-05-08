# Command Reference

Use `special --help` for exact local help. This reference explains the common
command shapes and the decision each output supports.

## `special specs`

Use `special specs` to inspect product
claims and proof attachments.

```sh
special specs
special specs --unverified
special specs EXPORT.CSV.HEADERS --verbose
special specs --metrics
```

Representative output:

```text
EXPORT.CSV.HEADERS
  CSV exports include a header row with the selected column names.
  verifies: 1
  attests: 0
```

Decision supported: whether a claim exists, whether it is current or planned,
and whether direct support is attached.

Contract details: `special specs` supports
current-only,
planned-only,
unverified,
id-scoped,
verbose,
metrics,
JSON, and
HTML views.

## `special arch`

Use `special arch` to inspect declared
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

Contract details: `special arch` supports
current-only,
planned-only,
unimplemented,
id-scoped,
verbose,
metrics,
JSON, and
HTML views. Metrics include
complexity explanations,
coupling,
quality, and
item-signal explanations.

## `special patterns`

Use `special patterns` to inspect
declared repeated implementation structures and their known applications.

```sh
special patterns
special patterns EXPORT.LABEL_VALUE_COLUMNS --verbose
special patterns --metrics
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
id-scoped,
verbose, and
metrics views. Metrics report
similarity. Raw
missing-application and unannotated-cluster queues belong to
`special health`.

## `special docs`

Use `special docs` to validate docs
relationships and build generated docs output.

```sh
special docs
special docs --metrics
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
  target coverage
    specs: 18 total, 14 documented, 14 generated, 0 internal-only, 4 undocumented
    modules: 6 total, 3 documented, 3 generated, 0 internal-only, 3 undocumented
  generated docs graph
    generated pages: 7
    local doc links: 11
    broken local doc links: 0
    orphan pages: 0
    reachable from entrypoints: 7/7 page(s), 1 entrypoint(s)
```

Verbose metrics include the
documented target support audit:

```sh
special docs --metrics --verbose
```

Decision supported: whether docs links resolve, whether generated docs pages are
connected, which declared targets have docs evidence, whether documented targets
have support, and whether docs output can be built safely.

Contract details: `special docs` supports
target scoping,
metrics,
relationship metrics,
target coverage, and
configured output builds.
Generated output
rewrites document links,
removes authoring annotations,
and refuses to overwrite docs evidence-bearing sources by accident.

## `special diff`

Use `special diff` after editing a
repo to review explicit Special relationships touched by the current VCS
changes.

```sh
special diff
special diff --metrics
special diff --target src/export.ts --verbose
special diff --id APP.EXPORT
```

Representative output:

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
declared VCS backend and
gracefully falls back to a full explicit relationship view when
`vcs` is omitted or disabled.
`--metrics` reports affected
relationship counts by relationship kind, target kind, and source path.
`--verbose` includes current
endpoint content for review.

## `special health`

Use `special health` for repo-wide
signals that go deeper than explicit graph edges or sit outside the graph.

```sh
special health
special health --metrics
special health --metrics --verbose
special health --target src/export.ts --symbol exportCsv
special health --json
special health --html
```

Representative output:

```text
special health
summary
  source outside architecture: 12
  untraced implementation: 34
  duplicate source shapes: 7
  possible pattern clusters: 2
  possible missing pattern applications: 1
  long prose outside docs: 3
  long prose test literals: 0
duplicate source shapes by file
  src/billing/export.ts: 4
  src/billing/refunds.ts: 3
```

Decision supported: which raw inferred queues should be promoted into specs,
architecture, patterns, docs, or test changes.

Contract details: `special health` supports
target scoping,
symbol scoping,
within scoping,
verbose evidence,
JSON, and
HTML. Metrics cover
source outside architecture,
duplicate source shapes,
untraced implementation,
missing pattern applications,
pattern clusters,
long prose outside docs,
and long prose test literals.

## `special mcp`

Use `special mcp` to run the stdio MCP
server for controlled agent access.

```sh
special mcp
```

Decision supported: whether an agent should access Special through bounded tools
instead of scraping arbitrary repo files first.

Contract details: `special mcp` exposes
bounded Special tools,
docs output, and a
plugin version notice
when the plugin and binary versions differ.

## `special lint`

Use `special lint` before committing
annotation changes.

```sh
special lint
```

Decision supported: whether ids, references, and lifecycle markers are
structurally valid.

Contract details: lint catches
duplicate ids,
unknown verify refs,
unknown attest refs,
unknown implements refs,
orphan verifies,
intermediate specs,
intermediate modules,
and invalid planned-scope usage.

## `special init`

Use `special init` to create a starter
`special.toml` in a repo without
an active config.

```sh
special init
```

Decision supported: whether the repo has an explicit Special root and generated
starter policy.

Contract details: `special init`
creates starter config,
does not overwrite existing config,
rejects nested active config, and
surfaces discovery errors.

## `special skills`

Use `special skills` to print or
install bundled workflow skills when a plugin path is not available.

```sh
special skills
special skills install
special skills install ship-product-change --destination project
```

Decision supported: which local skill surface an agent should use for a
Special-aware workflow.

Contract details: `special skills` can
print one bundled skill,
install one skill,
install all skills,
install to a project destination,
install to a global destination,
and preserve bundled references.
