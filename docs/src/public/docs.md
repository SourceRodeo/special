@applies DOCS.SURFACE_GUIDE_PAGE
@implements SPECIAL.DOCUMENTATION.PUBLIC.SURFACES.DOCS
@applies DOCS.SURFACE_OVERVIEW_PAGE
# Docs

Docs are Special's generated reader surface. Author markdown in docs source,
connect factual claims to Special ids, then build scrubbed output for readers.

Primary command:

```sh
special docs
```

@implements SPECIAL.DOCUMENTATION.PUBLIC.SURFACES.DOCS.TRACEABLE_EXAMPLE
@applies DOCS.TRACEABLE_DOCS_EXAMPLE
## Traceable Docs Example

Docs source can link to a [spec](documents://spec/SPECIAL.DOCS.LINKS):

```markdown
[CSV exports include headers](documents://spec/EXPORT.CSV.HEADERS).
```

`special docs build`
[writes generated markdown without the authoring URI](documents://spec/SPECIAL.DOCS.LINKS.OUTPUT):

```markdown
CSV exports include headers.
```

Check the relationship inventory and generated-docs graph:

```sh
special docs --metrics
```

Use `--verbose` when you need the
[relationship inventory](documents://spec/SPECIAL.DOCS_COMMAND.METRICS.RELATIONSHIPS).
It shows each documented target, where docs refer to it, and whether the target
has support such as verifies, attests, implementations, or pattern applications.
The audit reports
[planned specs and unsupported current specs](documents://spec/SPECIAL.DOCS_COMMAND.METRICS.TARGET_AUDIT)
directly in the docs metrics output.

Use [target coverage](documents://spec/SPECIAL.DOCS_COMMAND.METRICS.COVERAGE)
when the question is which specs, modules, areas, and patterns are documented
by generated docs, internal docs, or not at all.

## Output Mappings

Generated docs outputs come from [`[[docs.outputs]]`](documents://spec/SPECIAL.CONFIG.SPECIAL_TOML.DOCS_PATHS)
in `special.toml`:

```toml
[[docs.outputs]]
source = "docs/src/public"
output = "docs"
```

Directory mappings [preserve the tree](documents://spec/SPECIAL.DOCS_COMMAND.OUTPUT.DIRECTORY)
below the source directory and apply
[overwrite safety](documents://spec/SPECIAL.DOCS_COMMAND.OUTPUT.SAFETY).
