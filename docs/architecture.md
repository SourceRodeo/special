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

Representative output:

```text
APP.EXPORT
  Owns export formatting and file writing.
  implements:
    src/export.ts
```

That output answers whether the architecture name owns real code. If
`special arch --unimplemented` lists `APP.EXPORT`, the module is still design
intent unless it is explicitly planned.

An `@area`
can be structural. A
current module needs ownership
through source or markdown
`@implements` /
`@fileimplements`;
otherwise
`special arch --unimplemented`
keeps it visible.
