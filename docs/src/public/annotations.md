@filedocuments spec SPECIAL.PARSE
# Annotation Reference

Special annotations are ordinary comment or markdown lines. Declarations can live
in markdown or supported source comments. Attachments usually live next to the
test, source item, or file they describe.

## Product Claims

```text
@group EXPORT
Export behavior.

@spec EXPORT.CSV.HEADERS
CSV exports include a header row with the selected column names.
```

Use `@group` for structure and `@spec` for claims. Child claims do not prove a
parent claim.

## Lifecycle

```text
@spec EXPORT.METADATA
@planned 1.4.0
Exports include provenance metadata.
```

```text
@spec EXPORT.LEGACY_FORMAT
@deprecated 2.0.0
Legacy exports keep the old field names until the retirement release.
```

`@planned` and `@deprecated` attach to the owning spec. A spec cannot be both.

## Verification

```ts
// @verifies EXPORT.CSV.HEADERS
test("export writes headers", () => {
  expect(exportCsv([{ name: "Ava" }])).toContain("name");
});
```

Use [`@fileverifies ID`](documents://spec/SPECIAL.PARSE.VERIFIES) when the whole
file is the proof artifact. One verify block targets one spec id.

## Attestation

```text
@attests EXPORT.SECURITY_REVIEW
artifact: docs/security-review.md
owner: security
last_reviewed: 2026-05-03
review_interval_days: 90
```

[Attestations](documents://spec/SPECIAL.PARSE.ATTESTS) are for manual or external
evidence. They should include the artifact, owner, review date, and review
interval when the claim depends on review freshness.

## Architecture

```text
@area APP
Application code.

@module APP.EXPORT
Owns export formatting and file writing.
```

```ts
// @fileimplements APP.EXPORT

export function exportCsv(rows: Array<Record<string, string>>): string {
  // ...
}
```

Use `@area` for structure and [`@module`](documents://spec/SPECIAL.MODULE_COMMAND)
for ownership. An area may be pure structure. A planned module may be
unattached architecture intent. A current module needs ownership: attach code
from source, or attach markdown docs with `@implements` or `@fileimplements`.
Without that attachment, `special arch --unimplemented` will keep reporting it.

## Patterns

```text
@pattern CACHE.SINGLE_FLIGHT_FILL
@strictness high
Use single-flight cache fills when concurrent callers may request the same
expensive cache entry and only one caller should rebuild it.
```

```ts
// @applies CACHE.SINGLE_FLIGHT_FILL
export async function loadOrBuildExportCache(key: string): Promise<ExportCache> {
  // ...
}
```

[Patterns](documents://spec/SPECIAL.PATTERNS.DEFINITIONS) capture intentional
implementation approaches. Applications may attach to source items, source
files, markdown sections, or markdown files.

## Documentation

[Docs source](documents://spec/SPECIAL.DOCS_COMMAND) can attach prose to Special
facts:

```markdown
@filedocuments spec APP.CONFIG

[Configuration is loaded from app.toml](documents://spec/APP.CONFIG).
```

`special docs build` removes `@documents` and `@filedocuments` lines and
rewrites `documents://kind/ID` links to normal text.
