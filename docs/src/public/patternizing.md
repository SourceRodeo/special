@applies DOCS.PATTERNIZING_GUIDE_PAGE
# Patternizing Code and Docs

Patterns are for repeated structures the project wants to recognize again.
They are not slogans, style rules, or names for isolated decisions. A good
pattern gives a maintainer enough shape to decide whether a new case follows
the pattern or is doing something different.

@implements SPECIAL.DOCUMENTATION.PUBLIC.PATTERNIZING.CODE
## Patternizing Code

Use a code pattern when the same implementation problem keeps producing the
same solution shape. The pattern should name the problem, the chosen structure,
the expected source shape, and the cases where the pattern should not be used.

A candidate is ready for `@pattern` when it has all of these:

- more than one real or imminent application
- a source shape that can be recognized without reading the author's mind
- a reason the repeated shape should stay intentional
- enough negative guidance to avoid matching unrelated code

Do not define a pattern for "clear code", "small functions", or "good docs".
Those may be good review principles, but
[`special patterns --metrics`](documents://spec/SPECIAL.PATTERNS.METRICS) cannot
compare them as implementation structures. Define the pattern around the
concrete shape instead:

```text
@pattern EXPORT.LABEL_VALUE_COLUMNS
Build export rows from an ordered label-to-value column map before serialization.
```

Apply it where the structure exists:

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

Review the pattern with:

```sh
special patterns EXPORT.LABEL_VALUE_COLUMNS --verbose
special patterns EXPORT.LABEL_VALUE_COLUMNS --metrics --verbose
special patterns --metrics
special health --metrics --target src
```

Use pattern metrics to review declared applications and the unannotated items
that may belong to a known pattern. Use health metrics for uncaptured raw
opportunities: a high-similarity unannotated item may mean a missing
application, a helper that should be extracted, or a pattern definition that is
too broad.

@implements SPECIAL.DOCUMENTATION.PUBLIC.PATTERNIZING.DOCUMENTATION
## Patternizing Documentation

Documentation patterns work the same way, but the repeated structure is a
reader-facing shape instead of an implementation shape. Use them when pages,
guides, reference entries, or examples should follow a recognizable form.

Special's own docs use page-scale and section-scale patterns such as
[`DOCS.SURFACE_GUIDE_PAGE`](documents://pattern/DOCS.SURFACE_GUIDE_PAGE),
[`DOCS.TASK_RECIPE_PAGE`](documents://pattern/DOCS.TASK_RECIPE_PAGE),
[`DOCS.REFERENCE_CATALOG_PAGE`](documents://pattern/DOCS.REFERENCE_CATALOG_PAGE),
and
[`DOCS.TRACEABLE_DOCS_EXAMPLE`](documents://pattern/DOCS.TRACEABLE_DOCS_EXAMPLE).
Those patterns are intentionally looser than code patterns because natural
language varies, but they still need a recognizable sequence of jobs for the
reader.

Prefer a page- or section-level `@applies` when the natural unit already has the
shape. Do not split prose into artificial fragments just to attach pattern
lines. Markdown headings are the addressable units for documentation patterns,
like functions or classes are in code.
[`@applies`](documents://spec/SPECIAL.PATTERNS.MARKDOWN_APPLICATIONS) attaches
to a heading-bounded section, not to an arbitrary paragraph, list item, table, or
code fence. For a whole page, place `@applies` immediately before the H1:

```markdown
@applies DOCS.TASK_RECIPE_PAGE
# Investigate a Failing Export
```

Use [`@fileapplies`](documents://spec/SPECIAL.PATTERNS.MARKDOWN_APPLICATIONS.FILE_SCOPE_BODY)
only when the entire markdown file is the natural pattern application. Special
uses the file body for verbose pattern output and metrics, but an H1-bounded
`@applies` is usually easier to review because it names the reader-facing unit
directly.

Use `documents://` links for factual claims inside the prose. Pattern
applications say which documentation shape a section follows; document links
say which product fact the prose is explaining.

@implements SPECIAL.DOCUMENTATION.PUBLIC.PATTERNIZING.DISPOSITION
## Disposition Before Writing

Do not turn every coverage gap into another paragraph. Start by deciding what
kind of gap it is:

| Disposition | Use when | Typical action |
| --- | --- | --- |
| public guide | a user needs the concept or workflow | add or revise tutorial, concept, or how-to docs |
| public reference | a user needs exact command, config, annotation, or output behavior | add a reference entry |
| contributor reference | a maintainer needs an internal contract to change Special safely | add contributor docs |
| source-local | the source declaration is already the right explanation | leave it in source |
| test scaffold | the target exists to support tests or fixtures | keep it out of public docs |
| merge or demote | the target is too narrow to be a durable contract | merge it upward or keep it as a test-only guard |

Run the docs and pattern checks together:

```sh
special docs --metrics --verbose
special health --metrics
special patterns --metrics
```

`special docs --metrics --verbose` tells you whether documented targets have
support. `special health --metrics` shows uncaptured prose and raw pattern
opportunities. Pattern metrics help decide whether declared docs patterns still
fit their known applications.
