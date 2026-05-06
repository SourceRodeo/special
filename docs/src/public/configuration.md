# Configuration

`special.toml` anchors root discovery and optional project policy.

@implements SPECIAL.DOCUMENTATION.PUBLIC.REFERENCE.CONFIGURATION.MINIMAL
@applies DOCS.CONFIG_REFERENCE_BLOCK
## Minimal Config

```toml
version = "1"
root = "."
```

Behavior enabled: Special uses this file as an explicit project anchor instead
of relying on implicit VCS discovery.

Observe with:

```sh
special lint
```

@implements SPECIAL.DOCUMENTATION.PUBLIC.REFERENCE.CONFIGURATION.IGNORE
@applies DOCS.CONFIG_REFERENCE_BLOCK
## Ignore Paths

```toml
ignore = [
  "README.md",
  "docs/commands.md",
]
```

Behavior enabled: matching paths are excluded from shared annotation discovery.
Use exact generated docs output paths so source docs remain visible.

@implements SPECIAL.DOCUMENTATION.PUBLIC.REFERENCE.CONFIGURATION.DOCS_OUTPUTS
@applies DOCS.CONFIG_REFERENCE_BLOCK
## Docs Outputs

[`[[docs.outputs]]`](documents://spec/SPECIAL.CONFIG.SPECIAL_TOML.DOCS_PATHS)
maps docs source to generated docs output:

```toml
[docs]
entrypoints = ["README.md"]

[[docs.outputs]]
source = "docs/src/public"
output = "docs"
```

Behavior enabled: `special docs build` writes the configured output tree and
preserves paths below the source directory. Entrypoints feed generated-docs graph
reachability metrics.

Observe with:

```sh
special docs build
special docs --metrics
```

@implements SPECIAL.DOCUMENTATION.PUBLIC.REFERENCE.CONFIGURATION.HEALTH_IGNORE
@applies DOCS.CONFIG_REFERENCE_BLOCK
## Health Ignore

```toml
[health]
ignore-unexplained = [
  "generated/**",
]
```

Behavior enabled: matching paths stay out of the unsupported-implementation
review bucket without hiding them from all parsing or architecture analysis.

Observe with:

```sh
special health --metrics
```
