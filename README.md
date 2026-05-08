# special

## What Special Is

Special does two things: it scans a repository for
signals worth reviewing, and it lets you connect important claims, tests, code,
patterns, and docs directly in source.

Those connections let teams and agents answer practical questions: what the
repository claims, what proves those claims, where the implementation belongs,
which repeated structures are intentional, which docs depend on repo facts, and
which changed relationships need review.

The first-class surfaces are:

- Specs: product claims and proof attachments.
- Arch: areas, modules, and implementation ownership.
- Patterns: named repeated implementation structures.
- Docs: generated reader docs tied back to repo truth.
- Health: repo analysis that shows which source, docs, tests, and repeated
  structures deserve attention.
- Trace: deterministic relationship packets for audits that need the current
  source text, linked target, and attached evidence in one view.
- Diff: VCS-scoped relationship review for changed source and docs.

## Install It

Homebrew installs the `special` binary:

```sh
brew install sourcerodeo/homebrew-tap/special
special --version
```

Cargo can install the same binary from the
`special-cli` package:

```sh
cargo install special-cli
```

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

Representative output shape for a small repo:

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
special diff --metrics
```

An early health report might show billing export code in several queues:

```text
summary
  source outside architecture: 12
  untraced implementation: 34
  duplicate source shapes: 7
  possible pattern clusters: 2
  uncaptured prose outside docs: 3
duplicate source shapes by file
  src/billing/export.ts: 4
  src/billing/refunds.ts: 3
```

That output supports one concrete next step: inspect billing export code before
trying to model the whole repository. Continue with the
[existing-project tutorial](docs/how-to.md#adopt-special-in-an-existing-repo)
when the repository already has code, tests, and docs.

## Surface Map

| Surface | Primary command | Use it when |
| --- | --- | --- |
| Specs | `special specs` | You need to inspect product claims, lifecycle state, and proof attachments. |
| Arch | `special arch` | You need to see module ownership and implementation boundaries. |
| Patterns | `special patterns` | You need to review intentional repeated implementation structures. |
| Docs | `special docs` | You need to validate docs links or build generated docs output. |
| Health | `special health` | You need repo-wide signals that go beyond explicit graph edges. |
| Trace | `special trace` | You need deterministic packets for a docs, spec, architecture, or pattern audit. |
| Diff | `special diff` | You changed files and need the relationship review queue affected by that VCS diff. |

The commands are meant to be used together. `health` shows inferred signals and
off-graph gaps, `patterns` reviews repeated source shapes, `specs` records
behavior and proof, `arch` records ownership, `docs` makes reader-facing claims
traceable, `trace` builds explicit audit packets, `diff` focuses review on
changed relationships, and `lint` checks that the explicit graph still holds
together.

## Read Next

- [Concepts](docs/concepts.md): the mental model behind specs, arch, patterns, docs, and health.
- [Quickstart](docs/quickstart.md): start a fresh project with specs, arch, patterns, docs, and health.
- [How-to](docs/how-to.md#adopt-special-in-an-existing-repo): bring Special into an existing project by reading health and pattern signals first.
- [Specs](docs/specs.md), [Arch](docs/architecture.md), [Patterns](docs/patterns.md), and [Docs](docs/docs.md): first-class surface guides.
- [Patternizing code and docs](docs/patternizing.md): how to decide whether a repeated structure deserves a pattern.
- [Health](docs/health.md): how to read cross-surface signals.
- [Command reference](docs/commands.md), [Annotation reference](docs/annotations.md), and [Configuration](docs/configuration.md): lookup material.
- [Agents](docs/agents.md): MCP, plugin, and skill setup.
- [Contributor reference](docs/contributor/index.md): maintainer release, parser, language-pack, traceability, rendering, cache, and quality workflow.
