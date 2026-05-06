# Quickstart

## Install and Initialize

Start in an existing repository. Install the binary, initialize config, then add
one small contract that Special can inspect.

```sh
brew install sourcerodeo/homebrew-tap/special
special init
```

`special init` creates `special.toml`
when no active config already exists. Commit it after reviewing the root and
ignore settings.

## Add One Spec

Create a small product claim:

```text
@group EXPORT
Export behavior.

@spec EXPORT.CSV.HEADERS
CSV exports include a header row with the selected column names.
```

Inspect it:

```sh
special specs EXPORT.CSV.HEADERS
```

Add direct proof from a test:

```ts
// @verifies EXPORT.CSV.HEADERS
test("export writes headers", () => {
  expect(exportCsv([{ name: "Ava" }])).toContain("name");
});
```

Then run:

```sh
special specs EXPORT.CSV.HEADERS --verbose
special lint
```

## Add One Module

Declare one architecture boundary:

```text
@area APP
Application code.

@module APP.EXPORT
Owns export formatting and file writing.
```

Attach code ownership:

```ts
// @fileimplements APP.EXPORT
export function exportCsv(rows: Array<Record<string, string>>): string {
  return rows.map(row => row.name).join("\n");
}
```

Inspect it:

```sh
special arch APP.EXPORT --verbose
```

## Add One Pattern

Name a repeated implementation structure only when the structure is real enough
to recognize in multiple places:

```text
@pattern EXPORT.ROW_NORMALIZER
Normalize external row shapes before formatting output.
```

Apply it where the implementation uses that structure:

```ts
// @applies EXPORT.ROW_NORMALIZER
function normalizeExportRow(row: Record<string, string>): ExportRow {
  return { name: row.name.trim() };
}
```

Inspect pattern usage:

```sh
special patterns EXPORT.ROW_NORMALIZER --verbose
special patterns --metrics
```

## Add One Docs Link

Author docs source with a traceable relationship:

```markdown
[CSV exports include headers](documents://spec/EXPORT.CSV.HEADERS).
```

Build generated docs:

```sh
special docs build
```

Generated markdown keeps the reader text and removes the authoring URI:

```markdown
CSV exports include headers.
```

Check docs relationships:

```sh
special docs --metrics
```

## Run Health

Use health after the first spec, module, pattern, and docs link exist:

```sh
special health --metrics
```

Representative output shape:

```text
special health metrics
  unowned items: ...
  documentation coverage
    specs: ...
  traceability
    unsupported items: ...
```

Use this output to choose the next cleanup: add missing ownership, move behavior
behind a clearer implementation module, add proof to a current spec, document an
exposed surface, or name a repeated implementation structure as a pattern.
