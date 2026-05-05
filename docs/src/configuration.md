@filedocuments spec SPECIAL.CONFIG.SPECIAL_TOML
# Configuration

`special.toml` anchors root discovery and optional project policy.

## Minimal Config

```toml
version = "1"
root = "."
```

When present, the config anchors the project root. Without it, Special falls back
to VCS root discovery or the current directory and warns about implicit root
selection.

## Ignore Paths

```toml
ignore = [
  "CHANGELOG.md",
  "README.md",
  "docs/*.md",
]
```

Use `ignore` for generated or intentionally out-of-scope paths that Special
should not parse as annotation source.

## [Docs Outputs](special://spec/SPECIAL.CONFIG.SPECIAL_TOML.DOCS_PATHS)

```toml
[[docs.outputs]]
source = "docs/src/install.md"
output = "docs/install.md"

[[docs.outputs]]
source = "docs/src/README.md"
output = "README.md"
```

`special docs build` writes every configured mapping. File mappings write one
file. Directory mappings preserve the tree relative to the source directory.

## [Health Ignore](special://spec/SPECIAL.HEALTH_COMMAND.TRACEABILITY.IGNORE_UNEXPLAINED)

```toml
[health]
ignore-unexplained = [
  "generated/**",
]
```

This keeps matching paths out of the unsupported-implementation review bucket
without hiding them from all parsing or architecture analysis.

## Toolchain Contract

Special can use the project tool manager when language-pack analysis needs local
tools:

```toml
[toolchain]
manager = "mise"
```

Supported project contracts include `mise.toml` and `.tool-versions`.
