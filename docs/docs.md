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
  target coverage
    specs: 18 total, 14 documented, 14 generated, 0 internal-only, 4 undocumented
  generated docs graph
    generated pages: 7
    broken local doc links: 0
    reachable from entrypoints: 7/7 page(s), 1 entrypoint(s)
```

That output answers whether docs links resolve, whether the generated docs graph
is connected, and which Special targets still lack docs evidence.

Use `--verbose` when you need the
relationship inventory.
It shows each documented target, where docs refer to it, and whether the target
has support such as verifies, attests, implementations, or pattern applications.
The audit reports
planned specs and unsupported current specs
directly in the docs metrics output.

Use `--json` when another tool needs the same docs relationship view in a
structured form.

Use target coverage
when the question is which specs, modules, areas, and patterns are documented
by generated docs, internal docs, or not at all.

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
