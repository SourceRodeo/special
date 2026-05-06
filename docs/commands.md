# Command Reference

Use `special --help` for the exact local help text. This page explains when to
use each command and which shapes matter most.

## `special`

```sh
special
```

Prints a compact repo overview with counts and suggested next commands.

## `special specs`

```sh
special specs
special specs --current
special specs --planned
special specs --deprecated
special specs --unsupported
special specs --unverified
special specs --metrics
special specs EXPORT.CSV.HEADERS --verbose
special specs --json
special specs --html
```

Use specs for product claims. `--verbose` shows attached proof bodies. `--metrics`
adds grouped counts. `--unverified` focuses review on current claims that do not
have direct verification or attestation.

## `special arch`

```sh
special arch
special arch --current
special arch --planned
special arch --unimplemented
special arch APP.EXPORT --verbose
special arch APP.EXPORT --metrics
special arch --json --metrics
special arch --html
```

Use arch for module ownership. `--metrics` adds implementation analysis when the
active language pack can derive it.

## `special patterns`

```sh
special patterns
special patterns CACHE.SINGLE_FLIGHT_FILL
special patterns CACHE.SINGLE_FLIGHT_FILL --verbose
special patterns --metrics
special patterns --metrics --target src/cache.ts
special patterns --metrics --target src/cache.ts --symbol loadExport
special patterns --json
```

Use patterns for implementation approaches the project intentionally repeats.
Pattern metrics are advisory; they can suggest missing applications or helper
extraction candidates, but they are not lint failures.

## `special health`

```sh
special health
special health --target src/export.ts
special health --target src/export.ts --symbol exportCsv
special health --within crates/app
special health --metrics
special health --metrics --verbose
special health --json
special health --html
```

Use health for repo-wide questions: duplicate items, unowned implementation,
unsupported implementation, traceability from tests to current specs, and
documentation coverage
for the declared product surface.

## `special docs`

```sh
special docs
special docs --target docs/src
special docs --metrics
special docs --metrics --json
special docs build
special docs build docs/src/install.md docs/install.md
```

By default, docs prints documentation relationships and writes nothing.
With `--metrics`, it reports
relationship inventory
for specs, groups, modules, areas, and patterns, plus generated docs
interconnectivity
from configured docs outputs. `special docs build` writes configured docs
outputs or the explicit source/output pair.

## `special mcp`

```sh
special mcp
```

Starts the stdio MCP server for controlled agent access to Special inspection and
validation surfaces.

## `special lint`

```sh
special lint
```

Checks malformed annotations, duplicate ids, unknown references, invalid
planned/deprecated state, and docs relationships.

## `special init`

```sh
special init
```

Creates a starter `special.toml` without overwriting an existing active config.

## `special skills`

```sh
special skills
special skills ship-product-change
special skills install
special skills install ship-product-change
special skills install --destination project
special skills install --destination global --force
```

Prints or installs bundled Codex-style workflow skills.
