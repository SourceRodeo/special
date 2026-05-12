# Start a Fresh Project With Special

## Install and Initialize

Start in the repository where the code will live. Install the binary, initialize
config, then add one small contract that Special can inspect. In a fresh
project, you usually create the explicit connections first and use health as the
scan that checks what you missed.

```sh
brew install sourcerodeo/homebrew-tap/special
special init
```

`special init` creates
`special.toml` when no active
config already exists. Commit it after reviewing the root and ignore settings.

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
@pattern EXPORT.LABEL_VALUE_COLUMNS
Export tables should build columns as ordered label/value pairs.
```

Apply it where the implementation uses that structure:

```ts
// @applies EXPORT.LABEL_VALUE_COLUMNS
export function invoiceColumns(invoice: Invoice) {
  return [
    ["Invoice", invoice.number],
    ["Customer", invoice.customerName],
    ["Balance", formatCurrency(invoice.balanceCents)],
  ];
}
```

Inspect pattern usage:

```sh
special patterns EXPORT.LABEL_VALUE_COLUMNS --verbose
special patterns EXPORT.LABEL_VALUE_COLUMNS --metrics --verbose
special patterns --metrics
```

Use `special health --metrics` before naming many patterns; it keeps repeated
source shapes visible while you decide what deserves a durable name. Use
`special patterns --metrics` after you have declared patterns and want to review
their applications, fit, and possible missing applications.

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

Use docs links for claims a reader relies on. The source markdown carries the
review relationship; the generated markdown stays readable.

## Close the Loop With Health

Use health after the first spec, module, pattern, and docs link exist:

```sh
special health --metrics
```

Representative output:

```text
summary
  source outside architecture: 0
  untraced implementation: 1
  duplicate source shapes: 0
  possible pattern clusters: 0
  possible missing pattern applications: 0
  uncaptured prose outside docs: 0
  long prose test literals: 0
untraced implementation by file
  src/export.ts: 1
```

Use this output to choose the next review. In this example the architecture,
pattern, uncaptured-prose, and test-prose queues are clean, but one
implementation item is still not connected to proof. The next move is to
inspect `src/export.ts`, not to invent more annotations. Use
`special docs --metrics` for explicit documentation graph coverage.

That loop is the point of the fresh-project path: connect the claim, proof,
implementation, repeated structure, and reader-facing docs, then let health scan
for what still deserves attention.
