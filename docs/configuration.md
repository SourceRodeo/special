# Configuration

`special.toml` anchors root discovery and optional project policy.

## Minimal Config

```toml
version = "1"
root = "."
```

Behavior enabled: Special uses this file as an explicit project anchor instead
of relying on implicit VCS discovery.

Related contracts: Special can
fall back to the current directory,
fall back to a VCS root,
warn on implicit roots,
resolve explicit roots,
and find ancestor config.

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

Related contracts: ignore paths
exclude shared discovery.
Config parsing rejects
unknown keys,
duplicate keys,
unquoted values,
bad key/value syntax,
and invalid roots such as
empty roots,
file roots, or
missing roots.

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
reachability metrics. `special docs --metrics`
treats architecture and pattern targets implemented or applied by these sources
as docs structure, so docs modules do not recursively require docs links to
themselves.

Observe with:

```sh
special docs build
special docs --metrics
```

## VCS Backend

`vcs` declares the backend
used by VCS-aware review commands:

```toml
vcs = "git"
```

Use `"jj"` for Jujutsu repositories and `"none"` when a project should not ask a
VCS for changed paths.

Behavior enabled: `special diff` can ask the declared backend for changed paths,
then show the explicit Special relationships whose source or target endpoint is
inside that change. With `vcs = "none"` or no `vcs` setting, `special diff`
shows the full explicit relationship view instead.

Related contracts: config parsing accepts
`git`, `jj`, and `none`, and
rejects unsupported VCS values.

Observe with:

```sh
special diff --metrics
```

## Health Ignore

```toml
[health]
ignore-unexplained = [
  "generated/**",
]
```

Behavior enabled: matching paths stay out of the untraced-implementation review
queue without hiding them from all parsing or architecture analysis.

Related contracts: health ignore patterns
exclude configured unsupported paths.

Observe with:

```sh
special health --metrics
```
