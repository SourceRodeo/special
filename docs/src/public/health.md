@applies DOCS.SURFACE_GUIDE_PAGE
@implements SPECIAL.DOCUMENTATION.PUBLIC.SURFACES.HEALTH
@applies DOCS.SURFACE_OVERVIEW_PAGE
# Health

Health is Special's cross-surface analysis layer. Use it when the question is not
just "what did we declare?" but "what is still hard to explain?"

Primary command:

```sh
special health --metrics
```

Representative output shape:

```text
special health metrics
  duplicate items: ...
  unowned items: ...
  documentation coverage
    specs: ...
  traceability
    unsupported items: ...
```

@implements SPECIAL.DOCUMENTATION.PUBLIC.REFERENCE.METRICS
@applies DOCS.METRIC_REFERENCE_ENTRY
## Metric Interpretation

`unowned items` counts analyzable implementation outside declared module
ownership. It does not prove the code is wrong; it shows that the architecture
graph cannot explain that code yet.

`unsupported items` counts implementation that language-pack traceability cannot
connect back to current spec support. It does not mean the code is unused. It
means the proof path is missing or hidden behind a boundary Special deliberately
does not treat as the preferred proof path.

[`documentation coverage`](documents://spec/SPECIAL.HEALTH_COMMAND.METRICS.DOCUMENTATION_COVERAGE)
counts which declared specs, groups, modules, areas, and patterns are connected
to docs evidence. Architecture and pattern targets delivered by configured docs
output sources are treated as docs structure, not as separate targets that must
be documented again. Use coverage gaps to inspect the specific target with
`special specs`, `special arch`, or `special patterns`.
