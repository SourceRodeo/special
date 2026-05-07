# Implementation Patterns

This page is for maintainers changing Special's own pattern catalog. Public
docs explain how users should create patterns in their projects; this page
explains how this repository keeps its source patterns useful.

## Review Loop

Use the pattern commands before changing a pattern definition or adding a new
application:

```sh
special patterns --metrics
special patterns COMMAND.PROJECTION_PIPELINE --verbose
special health --metrics --target src
```

`special patterns --metrics` reviews declared applications. `special health`
shows raw duplicate structures and missing applications that are not yet part
of a declared pattern. A new `@applies` should make the existing implementation
shape easier to review; it should not be added only to make a queue disappear.

## Current Catalog

The root `PATTERNS.md` file owns the implementation-pattern definitions. These
are the patterns that currently have contributor-facing maintenance value:

| Pattern | Use it when |
| --- | --- |
| `ADAPTER.FACTS_TO_MODEL` | provider-specific facts must become shared Special model data |
| `ADAPTER.FACTS_TO_MODEL.TRACEABILITY_ITEMS` | a language pack projects parsed items into shared traceability items |
| `TRACEABILITY.SCOPED_PROJECTED_KERNEL` | scoped traceability must preserve full-then-filtered semantics |
| `TEST_FIXTURE.REPRESENTATIVE_PROJECT` | a test needs a compact real project instead of a source-string snippet |
| `COMMAND.PROJECTION_PIPELINE` | a command renders a read-only projection from parsed source state |
| `SINGLE_FLIGHT.CACHE_FILL` | expensive cache fills need one concurrent producer per cache key |
| `REGISTRY.PROVIDER_DESCRIPTOR` | built-in providers need one capability descriptor surface |

## Changing a Pattern

Before editing a pattern body, inspect its applications:

```sh
special patterns ADAPTER.FACTS_TO_MODEL --verbose
```

If the applications no longer share one recognizable structure, split the
pattern or remove the weaker applications. If a pattern reads like a principle,
rewrite it around concrete source shape: the provider boundary, the command
pipeline, the cache fill sequence, or the fixture project shape a reviewer can
actually compare.

Run the docs and pattern checks together after edits:

```sh
special docs --metrics
special patterns --metrics
special health --metrics --target src
```
