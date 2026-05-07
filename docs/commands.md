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
special patterns CACHE.SINGLE_FLIGHT_FILL --verbose
special patterns --metrics
special patterns --metrics
```

Representative output:

```text
CACHE.SINGLE_FLIGHT_FILL
  applications: 3
  modules: APP.CACHE
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
    total references: ...
  target coverage
    specs: ...
  generated docs graph
  generated pages: ...
  reachable from entrypoints: ...
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
summary
  source outside architecture: ...
  untraced implementation: ...
  duplicate source shapes: ...
  possible missing pattern applications: ...
  long prose outside docs: ...
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
and long exact prose assertions.

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
