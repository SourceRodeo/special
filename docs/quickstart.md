# Start a Fresh Project With Special

## Install and Initialize

Start in the repository where the code will live. Install the binary, initialize
config, then add one small contract that Special can inspect.

```sh
brew install sourcerodeo/homebrew-tap/special
special init
```

`special init` creates `special.toml`
when no active config already exists. Commit it after reviewing the root and
ignore settings.

## Write the First Behavior Claim

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

Use `special specs` to check whether the claim has direct proof. Use
`special lint` to catch broken ids,
misplaced attachments, and graph errors before the claim spreads.

## Name the First Implementation Boundary

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

Use `special arch` when the question is ownership: where the implementation
belongs, whether the declared module has code, and whether a planned boundary is
still only intent.

## Name a Pattern Only After It Repeats

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

Use `special patterns --metrics` even before a project has many adopted
patterns. In a new project, it keeps repeated structures visible before they
turn into copy-paste architecture.

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

Use docs links for claims a reader relies on. The source markdown stays dense
and traceable; the generated markdown stays readable.

## Close the Loop With Health

Use health after the first spec, module, pattern, and docs link exist:

```sh
special health --metrics
```

Representative output shape:

```text
summary
  source outside architecture: ...
  untraced implementation: ...
  possible missing pattern applications: ...
  long prose outside docs: ...
```

Use this output to choose the next cleanup: add missing ownership, move behavior
behind a clearer implementation module, add proof to a current spec, or name a
repeated implementation structure as a pattern. Use `special docs --metrics` for
explicit documentation graph coverage.

That loop is the point of the fresh-project path: write the claim, attach proof,
own the implementation, name repeated structures, document reader-facing facts,
then let health show what is still weak.
