@applies DOCS.SURFACE_GUIDE_PAGE
@implements SPECIAL.DOCUMENTATION.PUBLIC.SURFACES.PATTERNS
@applies DOCS.SURFACE_OVERVIEW_PAGE
# Patterns

Patterns are Special's surface for connecting adopted repeated implementation
structures. Health can scan for repeated shapes before you name them; patterns
record the shapes the project intentionally wants to recognize and review.

Primary command:

```sh
special patterns
```

Primary annotations:

```text
@pattern EXPORT.LABEL_VALUE_COLUMNS
Build export rows from an ordered label-to-value column map before serialization.
```

Apply the pattern where the structure appears:

```ts
// @applies EXPORT.LABEL_VALUE_COLUMNS
function invoiceColumns(invoice: Invoice): Record<string, string> {
  return {
    "Invoice ID": invoice.id,
    "Customer": invoice.customerName,
    "Total": formatCents(invoice.totalCents),
  };
}
```

Inspect usage:

```sh
special patterns EXPORT.LABEL_VALUE_COLUMNS --verbose
special patterns --metrics
```

Representative output shape for a small repo:

```text
EXPORT.LABEL_VALUE_COLUMNS
  Build export rows from an ordered label-to-value column map before serialization.
  applications: 3
  modules: APP.EXPORT
```

That output answers whether the repeated structure has been intentionally named
and where it appears. If health reports a similar repeated source shape but
`special patterns` has no pattern for it, decide whether the signal deserves a
helper extraction, an adopted pattern, or no action.

Pattern metrics are advisory fit checks for declared applications. Scoped
pattern metrics also show source items that may be missing `@applies` for that
known pattern. Raw repeated source shapes still appear in
[`special health --metrics`](documents://spec/SPECIAL.HEALTH_COMMAND), because
health owns uncaptured analysis queues. A good pattern is identifiable by
structure; a principle like "write clear docs" is not a Special pattern.

For the opinionated admission bar, see [Patternizing Code and Docs](patternizing.md).
