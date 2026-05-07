@applies DOCS.REFERENCE_CATALOG_PAGE
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

Related contracts: Special can
[fall back to the current directory](documents://spec/SPECIAL.CONFIG.ROOT_DISCOVERY.CWD_FALLBACK),
[fall back to a VCS root](documents://spec/SPECIAL.CONFIG.ROOT_DISCOVERY.VCS_DEFAULT),
[warn on implicit roots](documents://spec/SPECIAL.CONFIG.ROOT_DISCOVERY.IMPLICIT_ROOT_WARNING),
[resolve explicit roots](documents://spec/SPECIAL.CONFIG.SPECIAL_TOML.EXPLICIT_ROOT),
and [find ancestor config](documents://spec/SPECIAL.CONFIG.SPECIAL_TOML.ANCESTOR_CONFIG).

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

Related contracts: ignore paths
[exclude shared discovery](documents://spec/SPECIAL.CONFIG.SPECIAL_TOML.IGNORE.SHARED_DISCOVERY).
Config parsing rejects
[unknown keys](documents://spec/SPECIAL.CONFIG.SPECIAL_TOML.UNKNOWN_KEYS),
[duplicate keys](documents://spec/SPECIAL.CONFIG.SPECIAL_TOML.DUPLICATE_KEYS_REJECTED),
[unquoted values](documents://spec/SPECIAL.CONFIG.SPECIAL_TOML.QUOTED_STRING_VALUES),
[bad key/value syntax](documents://spec/SPECIAL.CONFIG.SPECIAL_TOML.KEY_VALUE_SYNTAX),
and invalid roots such as
[empty roots](documents://spec/SPECIAL.CONFIG.SPECIAL_TOML.ROOT_MUST_NOT_BE_EMPTY),
[file roots](documents://spec/SPECIAL.CONFIG.SPECIAL_TOML.ROOT_MUST_BE_DIRECTORY), or
[missing roots](documents://spec/SPECIAL.CONFIG.SPECIAL_TOML.EXISTING_ROOT_REQUIRED).

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
reachability metrics. [`special health --metrics`](documents://spec/SPECIAL.HEALTH_COMMAND.METRICS.DOCUMENTATION_COVERAGE.DOCS_SOURCE_DECLARATIONS)
treats architecture and pattern targets implemented or applied by these sources
as docs structure, so docs modules do not recursively require docs links to
themselves.

Observe with:

```sh
special docs build
special docs --metrics
```

@implements SPECIAL.DOCUMENTATION.PUBLIC.REFERENCE.CONFIGURATION.VCS
@applies DOCS.CONFIG_REFERENCE_BLOCK
## VCS Backend

[`vcs`](documents://spec/SPECIAL.CONFIG.SPECIAL_TOML.VCS) declares the backend
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
[`git`, `jj`, and `none`](documents://spec/SPECIAL.CONFIG.SPECIAL_TOML.VCS), and
rejects [unsupported VCS values](documents://spec/SPECIAL.CONFIG.SPECIAL_TOML.VCS.UNKNOWN_REJECTED).

Observe with:

```sh
special diff --metrics
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

Related contracts: health ignore patterns
[exclude configured unsupported paths](documents://spec/SPECIAL.HEALTH_COMMAND.TRACEABILITY.IGNORE_UNEXPLAINED).

Observe with:

```sh
special health --metrics
```
