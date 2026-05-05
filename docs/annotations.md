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

Use `@fileverifies ID` when the whole
file is the proof artifact. One verify block targets one spec id.

## Attestation

```text
@attests EXPORT.SECURITY_REVIEW
artifact: docs/security-review.md
owner: security
last_reviewed: 2026-05-03
review_interval_days: 90
```

Attestations are for manual or external
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

Use `@area` for structure and `@module`
for concrete ownership. An area may be pure structure. A current module may be
declared in markdown, but it only becomes implemented when real source attaches
with `@implements` or `@fileimplements`; otherwise `special arch --unimplemented`
keeps it visible as architecture drift.

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

Patterns capture intentional
implementation approaches. Applications must attach to source, not markdown
declarations.

## Documentation

Docs source can attach prose to Special
facts:

```markdown
@filedocuments spec APP.CONFIG

[Configuration is loaded from app.toml](special://spec/APP.CONFIG).
```

`special docs build` removes `@documents` and `@filedocuments` lines and
rewrites `special://kind/ID` links to normal text.
