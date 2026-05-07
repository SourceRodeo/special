@applies DOCS.SURFACE_GUIDE_PAGE
@implements SPECIAL.DOCUMENTATION.PUBLIC.SURFACES.HEALTH
@applies DOCS.SURFACE_OVERVIEW_PAGE
# Health

Health is Special's cross-surface analysis layer. Use it when the question is not
answered by explicit graph edges: "what is still hard to explain?"

Primary command:

```sh
special health --metrics
```

Representative output shape:

```text
summary
  source outside architecture: ...
  untraced implementation: ...
  duplicate source shapes: ...
  possible missing pattern applications: ...
  long prose outside docs: ...
```

@implements SPECIAL.DOCUMENTATION.PUBLIC.REFERENCE.METRICS
@applies DOCS.METRIC_REFERENCE_ENTRY
## Metric Interpretation

[`source outside architecture`](documents://spec/SPECIAL.HEALTH_COMMAND.UNOWNED_ITEMS) counts
analyzable implementation outside declared module
ownership. It does not prove the code is wrong; it shows that the architecture
graph cannot explain that code yet.

[`untraced implementation`](documents://spec/SPECIAL.HEALTH_COMMAND.TRACEABILITY) counts
implementation that language-pack traceability cannot connect back to current
spec support. It does not mean the code is unused. It means the proof path is
missing or hidden behind a
[boundary](documents://spec/SPECIAL.HEALTH_COMMAND.TRACEABILITY.BOUNDARY_NON_PENETRATION)
Special deliberately does not treat as the preferred proof path.

Docs coverage is explicit relationship accounting, so it belongs to
[`special docs --metrics`](documents://spec/SPECIAL.DOCS_COMMAND.METRICS.COVERAGE),
not health.
