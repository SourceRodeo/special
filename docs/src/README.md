@applies DOCS.PRODUCT_ENTRYPOINT_PAGE
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
## Choose a Starting Point

For a new project, start by writing the first durable claim and boundary as the
code appears:

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
[fresh-project tutorial](docs/quickstart.md) when you want to build with Special
from the start.

For an existing project, start by asking Special what it can see before adding
annotations:

```sh
special init
special health --metrics
special patterns --metrics
```

Use those reports to choose the first behavior, module, repeated structure, or
docs claim worth making durable. Continue with the
[existing-project tutorial](docs/how-to.md#adopt-special-in-an-existing-repo)
when the repository already has code, tests, and docs.

@implements SPECIAL.DOCUMENTATION.PUBLIC.README.SURFACE_MAP
## Surface Map

| Surface | Primary command | Use it when |
| --- | --- | --- |
| Specs | [`special specs`](documents://spec/SPECIAL.SPEC_COMMAND) | You need to inspect product claims, lifecycle state, and proof attachments. |
| Arch | [`special arch`](documents://spec/SPECIAL.MODULE_COMMAND) | You need to see module ownership and implementation boundaries. |
| Patterns | [`special patterns`](documents://spec/SPECIAL.PATTERNS.COMMAND) | You need to review intentional repeated implementation structures. |
| Docs | [`special docs`](documents://spec/SPECIAL.DOCS_COMMAND) | You need to validate docs links or build generated docs output. |
| Health | [`special health`](documents://spec/SPECIAL.HEALTH_COMMAND) | You need repo-wide signals that go beyond explicit graph edges. |

The commands are meant to be used together. `health` shows inferred signals and
off-graph gaps, `patterns` finds repeated source shapes, `specs` records
behavior and proof, `arch` records ownership, `docs` makes reader-facing claims
traceable, and `lint` checks that the graph still holds together.

@implements SPECIAL.DOCUMENTATION.PUBLIC.README.NEXT_STEPS
## Read Next

- [Concepts](docs/concepts.md): the mental model behind specs, arch, patterns, docs, and health.
- [Quickstart](docs/quickstart.md): start a fresh project with specs, arch, patterns, docs, and health.
- [How-to](docs/how-to.md#adopt-special-in-an-existing-repo): bring Special into an existing project by reading health and pattern signals first.
- [Specs](docs/specs.md), [Arch](docs/architecture.md), [Patterns](docs/patterns.md), and [Docs](docs/docs.md): first-class surface guides.
- [Patternizing code and docs](docs/patternizing.md): how to decide whether a repeated structure deserves a pattern.
- [Health](docs/health.md): how to read cross-surface signals.
- [Command reference](docs/commands.md), [Annotation reference](docs/annotations.md), and [Configuration](docs/configuration.md): lookup material.
- [Agents](docs/agents.md): MCP, plugin, and skill setup.
- [Contributor release notes](docs/contributor/release.md): release and distribution workflow.
