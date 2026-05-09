# Docs

Docs are Special's generated reader surface. Author markdown in docs source,
connect factual claims to Special ids, then build scrubbed output for readers.
The source stays connected to specs, modules, areas, and patterns; the generated
output stays readable.

Primary command:

```sh
special docs
```

## Traceable Docs Example

Docs source can link to a spec:

```markdown
[CSV exports include headers](documents://spec/EXPORT.CSV.HEADERS).
```

`special docs build`
writes generated markdown without the authoring URI:

```markdown
CSV exports include headers.
```

Check the relationship inventory and generated-docs graph:

```sh
special docs --metrics
```

Representative output shape for a small repo:

```text
special docs metrics
  relationship inventory
    total references: 42
      link references: 42
  generated docs graph
    generated pages: 7
    local doc links: 11
    broken local doc links: 0
    orphan pages: 0
    reachable from entrypoints: 7/7 page(s), 1 entrypoint(s)
```

That output answers whether docs links resolve and whether the generated docs
graph is connected.

Use `--verbose` when you need the
relationship inventory.
It shows relationship source counts and docs graph detail. It does not judge the
full resource chain behind a documented target.

Use `--json` when another tool needs the same docs relationship view in a
structured form.

Use `special health --metrics`
when the question is which current specs, modules, or patterns lack public docs.
Use `special trace` when a specific
docs link needs to be followed through the relevant spec, module, pattern, and
evidence chain.

## Output Mappings

Generated docs outputs come from `[[docs.outputs]]`
in `special.toml`:

```toml
[[docs.outputs]]
source = "docs/src/public"
output = "docs"
```

Directory mappings preserve the tree
below the source directory and apply
overwrite safety.
