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

## Metric Interpretation

`unowned items` counts analyzable implementation outside declared module
ownership. It does not prove the code is wrong; it shows that the architecture
graph cannot explain that code yet.

`unsupported items` counts implementation that language-pack traceability cannot
connect back to current spec support. It does not mean the code is unused. It
means the proof path is missing or hidden behind a boundary Special deliberately
does not treat as the preferred proof path.

`documentation coverage` counts which declared specs, groups, modules, areas, and
patterns are connected to docs evidence. Use it to find public or internal docs
gaps, then inspect the specific target with `special specs`, `special arch`, or
`special patterns`.
