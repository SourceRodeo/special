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

Groups organize. Specs make claims. Child specs do not prove parent specs.

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

Use [`@fileverifies`](documents://spec/SPECIAL.PARSE.VERIFIES) when the whole
file is the proof artifact. Use
[`@attests`](documents://spec/SPECIAL.PARSE.ATTESTS) for manual or external
evidence with review metadata.

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

An area can stay structural. A current module needs implementation ownership
unless it is explicitly planned.

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
@pattern CACHE.SINGLE_FLIGHT_FILL
Use one in-flight fill per cache key.
```

```ts
// @applies CACHE.SINGLE_FLIGHT_FILL
async function loadOrFillCache(key: string): Promise<Value> {
  return fills.getOrCreate(key, () => rebuildValue(key));
}
```

Validate with:

```sh
special patterns CACHE.SINGLE_FLIGHT_FILL --verbose
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

`special docs build` rewrites that link to normal reader text in generated
output. Use `@documents` only when a natural block really documents one target
and an inline link would be awkward.

Validate with:

```sh
special docs --metrics
special health --metrics
```
