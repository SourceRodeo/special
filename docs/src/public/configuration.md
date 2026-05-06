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
  "docs/install.md",
  "docs/tutorial.md",
  "docs/contributor/release.md",
]
```

Use `ignore` for generated or intentionally out-of-scope paths that Special
should not parse as annotation source.

## [Docs Outputs](documents://spec/SPECIAL.CONFIG.SPECIAL_TOML.DOCS_PATHS)

```toml
[docs]
entrypoints = ["README.md"]

[[docs.outputs]]
source = "docs/src/public"
output = "docs"

[[docs.outputs]]
source = "docs/src/contributor"
output = "docs/contributor"

[[docs.outputs]]
source = "docs/src/README.md"
output = "README.md"
```

`special docs build` writes every configured mapping. Prefer directory mappings
for generated docs trees, with separate file mappings only for outputs that live
outside those trees, such as a root README. Directory mappings preserve paths
relative to the source directory. Keep generated output files ignored by exact
path so the ignore rules do not also hide docs source files.
[`entrypoints`](documents://spec/SPECIAL.CONFIG.SPECIAL_TOML.DOCS_ENTRYPOINTS)
selects the generated docs pages used for docs reachability metrics.

## [Health Ignore](documents://spec/SPECIAL.HEALTH_COMMAND.TRACEABILITY.IGNORE_UNEXPLAINED)

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
