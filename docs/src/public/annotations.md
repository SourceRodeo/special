@applies DOCS.REFERENCE_CATALOG_PAGE
# Annotation Reference

Annotations are ordinary source comments or markdown lines. Put them where the
claim, proof, ownership, pattern, or docs relationship naturally belongs.

@implements SPECIAL.DOCUMENTATION.PUBLIC.REFERENCE.ANNOTATIONS.SPECS
@applies DOCS.ANNOTATION_REFERENCE_ENTRY
## Specs and Groups

Purpose: declare product claims and structural groups.

```text
@group EXPORT
Export behavior.

@spec EXPORT.CSV.HEADERS
CSV exports include a header row with the selected column names.
```

Validate with:

```sh
special specs
special lint
```

`@group` organizes.
[`@spec`](documents://spec/SPECIAL.PARSE.RESERVED_TAGS.REQUIRE_DIRECTIVE_SHAPE)
makes a claim. Child specs do not prove parent specs. Lifecycle markers can
mark specs as [planned](documents://spec/SPECIAL.PARSE.PLANNED) or
[deprecated](documents://spec/SPECIAL.PARSE.DEPRECATED).

@implements SPECIAL.DOCUMENTATION.PUBLIC.REFERENCE.ANNOTATIONS.EVIDENCE
@applies DOCS.ANNOTATION_REFERENCE_ENTRY
## Verifies and Attests

Purpose: attach direct evidence to specs.

```ts
// @verifies EXPORT.CSV.HEADERS
test("export writes headers", () => {
  expect(exportCsv([{ name: "Ava" }])).toContain("name");
});
```

Use [`@fileverifies`](documents://spec/SPECIAL.PARSE.VERIFIES.FILE_SCOPE) when
the whole file is the proof artifact. Use
[`@attests`](documents://spec/SPECIAL.PARSE.ATTESTS) for manual or external
evidence with review metadata. Special accepts
[file-scoped attests](documents://spec/SPECIAL.PARSE.ATTESTS.FILE_SCOPE) and
requires attestation
[metadata fields](documents://spec/SPECIAL.PARSE.ATTESTS.REQUIRED_FIELDS) in a
supported [date format](documents://spec/SPECIAL.PARSE.ATTESTS.DATE_FORMAT).

Validate with:

```sh
special specs EXPORT.CSV.HEADERS --verbose
special lint
```

@implements SPECIAL.DOCUMENTATION.PUBLIC.REFERENCE.ANNOTATIONS.ARCH
@applies DOCS.ANNOTATION_REFERENCE_ENTRY
## Areas, Modules, and Implements

Purpose: declare architecture ownership and attach implementation.

```text
@area APP
Application code.

@module APP.EXPORT
Owns export formatting and file writing.
```

```ts
// @fileimplements APP.EXPORT
export function exportCsv(rows: Array<Record<string, string>>): string {
  return rows.map(row => row.name).join("\n");
}
```

An [`@area`](documents://spec/SPECIAL.MODULE_PARSE.AREA_DECLARATIONS) can stay
structural. A [`@module`](documents://spec/SPECIAL.MODULE_PARSE.MODULE_DECLARATIONS)
declares an architecture owner. A
[current module needs implementation ownership](documents://spec/SPECIAL.MODULE_COMMAND.UNIMPLEMENTED)
unless it uses the module
[planned marker](documents://spec/SPECIAL.MODULE_PARSE.PLANNED.MODULE_ONLY).

In markdown, headings are the addressable units, like functions or classes in
code. [`@implements`](documents://spec/SPECIAL.MODULE_PARSE.IMPLEMENTS.MARKDOWN_SCOPE)
attaches to a heading-bounded section. Put it immediately before the heading you
want to own, or inside an existing section to attach that containing section. It
does not attach to an individual paragraph, list item, table row, or arbitrary
markdown element. Use
[`@fileimplements`](documents://spec/SPECIAL.MODULE_PARSE.IMPLEMENTS.FILE_SCOPE)
when the whole file is the ownership unit, and
[`@implements`](documents://spec/SPECIAL.MODULE_PARSE.IMPLEMENTS.ITEM_SCOPE) for
item-scoped ownership.

Validate with:

```sh
special arch --unimplemented
special lint
```

@implements SPECIAL.DOCUMENTATION.PUBLIC.REFERENCE.ANNOTATIONS.PATTERNS
@applies DOCS.ANNOTATION_REFERENCE_ENTRY
## Patterns and Applies

Purpose: declare a repeated implementation structure and attach concrete
applications.

```text
@pattern EXPORT.LABEL_VALUE_COLUMNS
Build export rows from an ordered label-to-value column map before serialization.
```

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

[`@pattern`](documents://spec/SPECIAL.PATTERNS.DEFINITIONS) declares the shape.
[`@applies`](documents://spec/SPECIAL.PATTERNS.SOURCE_APPLICATIONS) attaches
source applications. In markdown,
[`@applies`](documents://spec/SPECIAL.PATTERNS.MARKDOWN_APPLICATIONS)
uses the same heading-section rule as `@implements`: the applied body is the
matching heading section, not a single paragraph, list item, table, or code
fence. Use
[`@fileapplies`](documents://spec/SPECIAL.PATTERNS.MARKDOWN_APPLICATIONS.FILE_SCOPE_BODY)
when the entire markdown file is one pattern application.

Validate with:

```sh
special patterns EXPORT.LABEL_VALUE_COLUMNS --verbose
special patterns --metrics
special lint
```

@implements SPECIAL.DOCUMENTATION.PUBLIC.REFERENCE.ANNOTATIONS.DOCS
@applies DOCS.ANNOTATION_REFERENCE_ENTRY
## Docs Relationships

Purpose: connect docs prose to the smallest relevant Special id.

```markdown
[CSV exports include headers](documents://spec/EXPORT.CSV.HEADERS).
```

`special docs build`
[rewrites that link](documents://spec/SPECIAL.DOCS.LINKS.OUTPUT) to normal
reader text in generated output. Use
[`@documents`](documents://spec/SPECIAL.DOCS.DOCUMENTS_LINES) only when a
natural block really documents one target and an inline link would be awkward.

Validate with:

```sh
special docs --metrics
special health --metrics
```
