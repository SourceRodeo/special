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

An [`@area`](documents://spec/SPECIAL.MODULE_PARSE.AREAS_ARE_STRUCTURAL_ONLY)
can be structural. A
[current module needs ownership](documents://spec/SPECIAL.MODULE_COMMAND.UNIMPLEMENTED)
through source or markdown
[`@implements`](documents://spec/SPECIAL.MODULE_PARSE.IMPLEMENTS.ITEM_SCOPE) /
[`@fileimplements`](documents://spec/SPECIAL.MODULE_PARSE.IMPLEMENTS.FILE_SCOPE);
otherwise
[`special arch --unimplemented`](documents://spec/SPECIAL.MODULE_COMMAND.UNIMPLEMENTED)
keeps it visible.
