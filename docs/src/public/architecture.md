@applies DOCS.SURFACE_GUIDE_PAGE
@implements SPECIAL.DOCUMENTATION.PUBLIC.SURFACES.ARCH
@applies DOCS.SURFACE_OVERVIEW_PAGE
# Arch

Arch is Special's implementation-ownership surface. It keeps architecture tied
to code instead of letting module names become aspirational prose.

Primary command:

```sh
special arch
```

Primary annotations:

```text
@area APP
Application code.

@module APP.EXPORT
Owns export formatting and file writing.
```

Attach ownership from code:

```ts
// @fileimplements APP.EXPORT
export function exportCsv(rows: Array<Record<string, string>>): string {
  return rows.map(row => row.name).join("\n");
}
```

Inspect the boundary:

```sh
special arch APP.EXPORT --verbose
special arch APP.EXPORT --metrics
special arch --unimplemented
```

An [`@area`](documents://spec/SPECIAL.MODULE_COMMAND) can be structural. A
current module needs ownership through source or markdown `@implements` /
`@fileimplements`; otherwise `special arch --unimplemented` keeps it visible.
