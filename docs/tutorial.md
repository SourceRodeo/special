# Tutorial

This tutorial starts with one claim, one test, one architecture boundary, and one
health pass. It is meant for an existing repository, not a new framework project.

## 1. Initialize the Repo

```sh
special init
```

Commit `special.toml` after reviewing the root and ignore settings.

## 2. Add a Product Claim

Create or edit a markdown file that belongs with the feature. Use a neutral
project id, not a Special-internal id:

```text
@group EXPORT
Export behavior.

@spec EXPORT.CSV.HEADERS
CSV exports include a header row with the selected column names.
```

Run:

```sh
special specs
```

The claim exists, but it is not supported yet.

## 3. Attach Verification

Attach the claim to the test that proves it:

```ts
// @verifies EXPORT.CSV.HEADERS
test("export writes headers", () => {
  expect(exportCsv([{ name: "Ava" }])).toContain("name");
});
```

Then inspect the exact support:

```sh
special specs EXPORT.CSV.HEADERS --verbose
special specs --unverified
special lint
```

## 4. Add Architecture Ownership

Declare the implementation boundary:

```text
@area APP
Application code.

@module APP.EXPORT
Owns export formatting and file writing.
```

Attach ownership from source:

```ts
// @fileimplements APP.EXPORT

export function exportCsv(rows: Array<Record<string, string>>): string {
  // ...
}
```

Inspect it:

```sh
special arch
special arch APP.EXPORT --verbose
special arch APP.EXPORT --metrics
```

## 5. Check Repo Health

Use health when the question is repo-wide rather than claim- or module-specific:

```sh
special health --metrics
special health --metrics --verbose
```

Health can show unsupported items, unowned code, duplicate source items, and
traceability from tests back to current specs when the language pack can derive
that evidence honestly.

## 6. Keep the Contract Small

When behavior changes, update the smallest affected claim and proof attachment.
Do not add a PRD or parallel planning document when the durable truth belongs in
Special annotations.
