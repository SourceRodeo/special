# Command Reference

Use `special --help` for exact local help. This reference explains the common
command shapes and the decision each output supports.

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

Representative output:

```text
EXPORT.CSV.HEADERS
  verifies: 1
  attests: 0
```

Decision supported: whether a claim exists, whether it is current or planned,
and whether direct support is attached.

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

@implements SPECIAL.DOCUMENTATION.PUBLIC.REFERENCE.COMMANDS.PATTERNS
@applies DOCS.COMMAND_REFERENCE_ENTRY
## `special patterns`

Use [`special patterns`](documents://spec/SPECIAL.PATTERNS.COMMAND) to inspect
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

@implements SPECIAL.DOCUMENTATION.PUBLIC.REFERENCE.COMMANDS.DOCS
@applies DOCS.COMMAND_REFERENCE_ENTRY
## `special docs`

Use [`special docs`](documents://spec/SPECIAL.DOCS_COMMAND) to validate docs
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
[documented target support audit](documents://spec/SPECIAL.DOCS_COMMAND.METRICS.TARGET_AUDIT):

```sh
special docs --metrics --verbose
```

Decision supported: whether docs links resolve, whether generated docs pages are
connected, whether documented targets have support, and whether docs output can
be built safely.

@implements SPECIAL.DOCUMENTATION.PUBLIC.REFERENCE.COMMANDS.HEALTH
@applies DOCS.COMMAND_REFERENCE_ENTRY
## `special health`

Use [`special health`](documents://spec/SPECIAL.HEALTH_COMMAND) for repo-wide
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

@implements SPECIAL.DOCUMENTATION.PUBLIC.REFERENCE.COMMANDS.LINT
@applies DOCS.COMMAND_REFERENCE_ENTRY
## `special lint`

Use [`special lint`](documents://spec/SPECIAL.LINT_COMMAND) before committing
annotation changes.

```sh
special lint
```

Decision supported: whether ids, references, lifecycle markers, and docs links
are structurally valid.

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

@implements SPECIAL.DOCUMENTATION.PUBLIC.REFERENCE.COMMANDS.SKILLS
@applies DOCS.COMMAND_REFERENCE_ENTRY
## `special skills`

Use [`special skills`](documents://spec/SPECIAL.SKILLS.COMMAND.HELP) to print or
install bundled workflow skills when a plugin path is not available.

```sh
special skills
special skills install
special skills install ship-product-change --destination project
```

Decision supported: which local skill surface an agent should use for a
Special-aware workflow.
