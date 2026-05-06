@filedocuments module SPECIAL
# special

[Repo-native contracts, architecture ownership, adopted patterns, and traceability](documents://module/SPECIAL)
for codebases maintained by humans and agents.

`special` reads [lightweight annotations from normal source files and markdown](documents://spec/SPECIAL.PARSE),
then turns them into inspectable CLI views: what the repo claims, what supports
those claims, which code owns which architecture boundary, which implementation
patterns are intentional, and which code is still hard to explain.

## Install

[Homebrew is the primary install path](documents://spec/SPECIAL.DISTRIBUTION.HOMEBREW.INSTALLS_SPECIAL):

```sh
brew install sourcerodeo/homebrew-tap/special
```

Cargo is the secondary install path when you want to build from the Rust package
registry:

```sh
cargo install special-cli
```

Both install the [`special` binary](documents://spec/SPECIAL.DISTRIBUTION.CRATES_IO.BINARY_NAME).

## Start Here

New users should read:

- [Install and update](docs/install.md)
- [Tutorial](docs/tutorial.md)
- [Command reference](docs/commands.md)
- [Annotation reference](docs/annotations.md)
- [Configuration](docs/configuration.md)
- [Agent and MCP setup](docs/agents.md)
- [Release and distribution notes](docs/contributor/release.md)

## Quick Example

Declare a product claim:

```text
@spec EXPORT.CSV.HEADERS
CSV exports include a header row with the selected column names.
```

Attach support from a test:

```ts
// @verifies EXPORT.CSV.HEADERS
test("export writes headers", () => {
  expect(exportCsv([{ name: "Ava" }])).toContain("name");
});
```

Inspect it:

```sh
special specs EXPORT.CSV.HEADERS --verbose
special lint
```

For a repo overview, run:

```sh
special
```

## What Special Checks

`special` helps answer questions that are hard to answer with grep alone:

- Which product claims are current, planned, deprecated, unsupported, or backed
  only by weak-looking evidence?
- Which tests, source items, or reviewed artifacts are attached to each claim?
- Which files and source items implement each architecture module?
- Which repeated implementation approaches are intentional project patterns?
- Which code is duplicated, unowned, hard to trace to a current spec, or outside
  the declared architecture?
- Which annotation references are malformed, duplicated, or pointing at missing
  declarations?

The result is not a replacement for tests, docs, or review. It is a repo-local
index that keeps claims, evidence, and implementation boundaries connected.

## Command Map

| Command | Use it to |
| --- | --- |
| `special` | [See a compact repo overview and suggested next commands](documents://spec/SPECIAL.HELP.ROOT_OVERVIEW). |
| `special specs` | [Inspect current, planned, deprecated, and unsupported product claims](documents://spec/SPECIAL.SPEC_COMMAND). |
| `special arch` | [Inspect architecture declarations and implementation ownership](documents://spec/SPECIAL.MODULE_COMMAND). |
| `special patterns` | [Inspect adopted implementation patterns and their applications](documents://spec/SPECIAL.PATTERNS.COMMAND). |
| `special health` | [Inspect repo-wide quality, traceability, and documentation coverage signals](documents://spec/SPECIAL.HEALTH_COMMAND.METRICS.DOCUMENTATION_COVERAGE). |
| `special docs` | [Validate docs relationships](documents://spec/SPECIAL.DOCS_COMMAND), inspect [docs relationship metrics](documents://spec/SPECIAL.DOCS_COMMAND.METRICS), or write docs outputs. |
| `special mcp` | [Run the stdio MCP server for controlled agent access](documents://spec/SPECIAL.MCP_COMMAND). |
| `special lint` | [Check annotation and reference structure](documents://spec/SPECIAL.LINT_COMMAND). |
| `special init` | Add a starter `special.toml`. |
| `special skills` | [Print or install bundled workflow skills](documents://spec/SPECIAL.SKILLS.COMMAND.HELP). |

## Project Truth

Special is self-hosting: the canonical product truth for this repo lives in its
own `special` declarations, primarily colocated with the owning source and test
boundaries. Central markdown remains only for structural and planned contract
scaffolding.

If this README and the spec output disagree, the spec wins.
