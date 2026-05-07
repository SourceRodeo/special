# Annotation Reference

Annotations are ordinary source comments or markdown lines. Put them where the
claim, proof, ownership, pattern, or docs relationship naturally belongs.

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
`@spec`
makes a claim. Child specs do not prove parent specs. Lifecycle markers can
mark specs as planned or
deprecated.

## Verifies and Attests

Purpose: attach direct evidence to specs.

```ts
// @verifies EXPORT.CSV.HEADERS
test("export writes headers", () => {
  expect(exportCsv([{ name: "Ava" }])).toContain("name");
});
```

Use `@fileverifies` when
the whole file is the proof artifact. Use
`@attests` for manual or external
evidence with review metadata. Special accepts
file-scoped attests and
requires attestation
metadata fields in a
supported date format.

Validate with:

```sh
special specs EXPORT.CSV.HEADERS --verbose
special lint
```

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

An `@area` can stay
structural. A `@module`
declares an architecture owner. A
current module needs implementation ownership
unless it uses the module
planned marker.

In markdown, headings are the addressable units, like functions or classes in
code. `@implements`
attaches to a heading-bounded section. Put it immediately before the heading you
want to own, or inside an existing section to attach that containing section. It
does not attach to an individual paragraph, list item, table row, or arbitrary
markdown element. Use
`@fileimplements`
when the whole file is the ownership unit, and
`@implements` for
item-scoped ownership.

Validate with:

```sh
special arch --unimplemented
special lint
```

## Patterns and Applies

Purpose: declare a repeated implementation structure and attach concrete
applications.

```text
@pattern CACHE.SINGLE_FLIGHT_FILL
Use one in-flight fill per cache key.
```

```ts
// @applies CACHE.SINGLE_FLIGHT_FILL
async function loadOrFillCache(key: string): Promise<Value> {
  return fills.getOrCreate(key, () => rebuildValue(key));
}
```

`@pattern` declares the shape.
`@applies` attaches
source applications. In markdown,
`@applies`
uses the same heading-section rule as `@implements`: the applied body is the
matching heading section, not a single paragraph, list item, table, or code
fence. Use
`@fileapplies`
when the entire markdown file is one pattern application.

Validate with:

```sh
special patterns CACHE.SINGLE_FLIGHT_FILL --verbose
special patterns --metrics
special lint
```

## Docs Relationships

Purpose: connect docs prose to the smallest relevant Special id.

```markdown
[CSV exports include headers](documents://spec/EXPORT.CSV.HEADERS).
```

`special docs build`
rewrites that link to normal
reader text in generated output. Use
`@documents` only when a
natural block really documents one target and an inline link would be awkward.

Validate with:

```sh
special docs --metrics
special health --metrics
```
