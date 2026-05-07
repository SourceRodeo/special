# Specs

Specs are Special's product-claim surface. Use them for behavior the repo
promises, plans, deprecates, or intentionally does not support.

Primary command:

```sh
special specs
```

Primary annotations:

```text
@group EXPORT
Export behavior.

@spec EXPORT.CSV.HEADERS
CSV exports include a header row with the selected column names.
```

Attach proof:

```ts
// @verifies EXPORT.CSV.HEADERS
test("export writes headers", () => {
  expect(exportCsv([{ name: "Ava" }])).toContain("name");
});
```

Inspect support:

```sh
special specs EXPORT.CSV.HEADERS --verbose
special specs --unverified
```

Representative output:

```text
EXPORT.CSV.HEADERS
  CSV exports include a header row with the selected column names.
  verifies: 1
  attests: 0
```

That output answers a narrow question: the claim exists and has direct verifying
evidence. If `special specs --unverified` lists `EXPORT.CSV.HEADERS`, either the
claim is ahead of implementation or the proof attachment is missing.

Use specs when the question is: what does this repo claim, and what evidence is
attached to that claim? Use [health](health.md) when the question is whether the
implementation graph reaches those claims.
