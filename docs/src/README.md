# special

@implements SPECIAL.DOCUMENTATION.PUBLIC.README.POSITIONING
@applies DOCS.SURFACE_OVERVIEW_PAGE
## What Special Is

[Special](documents://module/SPECIAL) keeps repo knowledge close to the code that
depends on it. It is for teams and agents who need to know what a repository
claims, what proves those claims, where the implementation belongs, which
implementation structures are intentional, and whether generated docs still
cover the product surface.

The first-class surfaces are:

- Specs: product claims and proof attachments.
- Arch: areas, modules, and implementation ownership.
- Patterns: named repeated implementation structures.
- Docs: generated reader docs tied back to repo truth.
- Health: cross-surface analysis that shows what is still hard to explain.

@implements SPECIAL.DOCUMENTATION.PUBLIC.README.INSTALL
## Install It

[Homebrew installs the `special` binary](documents://spec/SPECIAL.DISTRIBUTION.HOMEBREW.INSTALLS_SPECIAL):

```sh
brew install sourcerodeo/homebrew-tap/special
special --version
```

Cargo can install the same binary from the
[`special-cli` package](documents://spec/SPECIAL.DISTRIBUTION.CRATES_IO.BINARY_NAME):

```sh
cargo install special-cli
```

@implements SPECIAL.DOCUMENTATION.PUBLIC.README.QUICKSTART
@applies DOCS.GETTING_STARTED_SEQUENCE
## Quick Start

Initialize a repo, add one claim, attach one proof, and inspect the result:

```sh
special init
special specs EXPORT.CSV.HEADERS --verbose
special lint
```

The quickstart uses TypeScript examples, but the same annotation model works
across supported source languages and markdown:

```ts
// @verifies EXPORT.CSV.HEADERS
test("export writes headers", () => {
  expect(exportCsv([{ name: "Ava" }])).toContain("name");
});
```

Representative output shape:

```text
EXPORT.CSV.HEADERS
  CSV exports include a header row with the selected column names.
  verifies: 1
```

Use that output to decide whether a claim has direct support. Continue with the
[quickstart](docs/quickstart.md) when adopting Special for the first time.

@implements SPECIAL.DOCUMENTATION.PUBLIC.README.SURFACE_MAP
## Surface Map

| Surface | Primary command | Use it when |
| --- | --- | --- |
| Specs | [`special specs`](documents://spec/SPECIAL.SPEC_COMMAND) | You need to inspect product claims, lifecycle state, and proof attachments. |
| Arch | [`special arch`](documents://spec/SPECIAL.MODULE_COMMAND) | You need to see module ownership and implementation boundaries. |
| Patterns | [`special patterns`](documents://spec/SPECIAL.PATTERNS.COMMAND) | You need to review intentional repeated implementation structures. |
| Docs | [`special docs`](documents://spec/SPECIAL.DOCS_COMMAND) | You need to validate docs links or build generated docs output. |
| Health | [`special health`](documents://spec/SPECIAL.HEALTH_COMMAND) | You need repo-wide traceability, ownership, duplication, and documentation coverage signals. |

@implements SPECIAL.DOCUMENTATION.PUBLIC.README.NEXT_STEPS
## Read Next

- [Concepts](docs/concepts.md): the mental model behind specs, arch, patterns, docs, and health.
- [Quickstart](docs/quickstart.md): one end-to-end adoption path.
- [Specs](docs/specs.md), [Arch](docs/architecture.md), [Patterns](docs/patterns.md), and [Docs](docs/docs.md): first-class surface guides.
- [Health](docs/health.md): how to read cross-surface signals.
- [How-to](docs/how-to.md): task recipes for adoption, health investigation, traceable docs, and patterns.
- [Command reference](docs/commands.md), [Annotation reference](docs/annotations.md), and [Configuration](docs/configuration.md): lookup material.
- [Agents](docs/agents.md): MCP, plugin, and skill setup.
- [Contributor release notes](docs/contributor/release.md): release and distribution workflow.
