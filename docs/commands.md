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

## `special patterns`

Use `special patterns` to inspect
named repeated implementation structures.

```sh
special patterns
special patterns CACHE.SINGLE_FLIGHT_FILL --verbose
special patterns --metrics
special patterns --metrics --target src/cache.ts
```

Representative output:

```text
CACHE.SINGLE_FLIGHT_FILL
  applications: 3
  modules: APP.CACHE
```

Decision supported: whether a repeated structure is intentionally named,
where it is applied, and whether metrics suggest similar unannotated shapes.

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
  total references: ...
  generated pages: ...
  reachable from entrypoints: ...
```

Verbose metrics include the
documented target support audit:

```sh
special docs --metrics --verbose
```

Decision supported: whether docs links resolve, whether generated docs pages are
connected, whether documented targets have support, and whether docs output can
be built safely.

## `special health`

Use `special health` for repo-wide
signals that connect specs, arch, patterns, and docs.

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
documentation coverage
  specs: ...
traceability
  unsupported items: ...
```

Decision supported: what remains unowned, unsupported, duplicated, or
undocumented.

## `special mcp`

Use `special mcp` to run the stdio MCP
server for controlled agent access.

```sh
special mcp
```

Decision supported: whether an agent should access Special through bounded tools
instead of scraping arbitrary repo files first.

## `special lint`

Use `special lint` before committing
annotation changes.

```sh
special lint
```

Decision supported: whether ids, references, lifecycle markers, and docs links
are structurally valid.

## `special init`

Use `special init` to create a starter
`special.toml` in a repo without
an active config.

```sh
special init
```

Decision supported: whether the repo has an explicit Special root and generated
starter policy.

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
