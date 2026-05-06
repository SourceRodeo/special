# Configuration

`special.toml` anchors root discovery and optional project policy.

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

## Ignore Paths

```toml
ignore = [
  "README.md",
  "docs/commands.md",
]
```

Behavior enabled: matching paths are excluded from shared annotation discovery.
Use exact generated docs output paths so source docs remain visible.

## Docs Outputs

`[[docs.outputs]]`
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
