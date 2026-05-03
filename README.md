# special

Repo-native contracts, architecture ownership, adopted patterns, and traceability
for codebases maintained by humans and agents.

`special` reads lightweight annotations from normal source files and markdown,
then turns them into inspectable views: what the repo claims, what supports those
claims, which code owns which architecture boundary, which implementation
patterns are intentional, and which code is still hard to explain.

[Install](#install) | [Quick Start](#quick-start) | [Commands](#commands) | [Annotations](#annotations) | [Development](#development)

## Install

Homebrew is the primary install path:

```sh
brew install sourcerodeo/homebrew-tap/special
```

Cargo is the secondary install path:

```sh
cargo install special-cli
```

Both install the `special` binary. Release archives and checksums are published
through GitHub Releases for `sourcerodeo/special`.

## Quick Start

Initialize a repo root:

```sh
special init
```

Add a product claim in markdown or a source comment:

```text
### @spec EXPORT.CSV.HEADERS
CSV exports include a header row with the selected column names.
```

Attach support from a test or implementation boundary:

```rust
/// @verifies EXPORT.CSV.HEADERS
#[test]
fn export_writes_headers() {
    // ...
}
```

Inspect the claim and its support:

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

## Commands

| Command | Use it to |
| --- | --- |
| `special` | See a compact repo overview and suggested next commands. |
| `special specs` | Inspect current, planned, deprecated, and unsupported product claims. |
| `special arch` | Inspect architecture declarations and implementation ownership. |
| `special patterns` | Inspect adopted implementation patterns and their source applications. |
| `special health` | Inspect repo-wide quality and traceability signals. |
| `special docs` | Validate or materialize docs relationships. |
| `special mcp` | Run the stdio MCP server for controlled agent access. |
| `special lint` | Check annotation and reference structure. |
| `special init` | Add a starter `special.toml`. |
| `special skills` | Print or install bundled workflow skills. |

Common command shapes:

```sh
special

special specs
special specs --current
special specs --planned
special specs EXPORT.CSV.HEADERS --verbose
special specs --unverified
special specs --metrics
special specs --json

special arch
special arch --current
special arch --planned
special arch APP.PARSER --verbose
special arch --metrics
special arch APP.PARSER --metrics
special arch --json --metrics

special patterns
special patterns CACHE.SINGLE_FLIGHT_FILL
special patterns CACHE.SINGLE_FLIGHT_FILL --verbose
special patterns --metrics
special patterns --metrics --target src/cache.rs
special patterns --json

special health
special health --target src/export.rs
special health --target src/export.rs --symbol writeCsv
special health --within crates/app
special health --metrics
special health --metrics --verbose
special health --json

special docs
special docs --target docs/src
special docs --output
special docs --target docs/src --output docs/dist

special mcp

special lint
special init
special skills
special skills ship-product-change
special skills install
```

Use `--verbose` when you want attached bodies, source evidence, or item-level
detail. Use `--metrics` when you want grouped counts and deeper analysis for the
current view. Use `--json` when another tool should consume the output.
`special docs` prints a documentation relationship view by default and writes
materialized public files only when `--output` is present. Projects can set
`[[docs.outputs]]` entries in `special.toml` so `special docs --output`
materializes every configured source/output mapping, including a root `README.md`
generated from `docs/src/README.md`.
For directory targets, materialization preserves the tree relative to the target
root inside the output path.

## Examples

### Product Claims

Declare current behavior:

```text
### @spec EXPORT.CSV.HEADERS
CSV exports include a header row with the selected column names.
```

Declare planned behavior:

```text
### @spec EXPORT.METADATA
@planned 1.4.0
Exports include provenance metadata.
```

Attach verification:

```rust
/// @verifies EXPORT.CSV.HEADERS
#[test]
fn export_writes_headers() {
    // ...
}
```

Then inspect:

```sh
special specs
special specs --unverified
special specs EXPORT.CSV.HEADERS --verbose
```

`special specs --unverified` surfaces current claims with no direct verification
or attestation. `special specs ID --verbose` shows the attached body so a reviewer
can judge whether the support really proves the claim.

### Architecture Ownership

Declare architecture:

```text
### @area APP
Application code.

### @module APP.PARSER
Parses reserved annotations from extracted source comments.
```

Attach source ownership:

```rust
// @fileimplements APP.PARSER
mod parser;
```

Then inspect:

```sh
special arch
special arch APP.PARSER --metrics
```

The architecture view shows declared boundaries and implementation evidence.
Metrics add ownership granularity, item counts, complexity, coupling, and
unreached-code indicators when the language pack can derive them.

### Adopted Patterns

Declare an intentional implementation approach:

```text
### @pattern CACHE.SINGLE_FLIGHT_FILL
@strictness high
Use single-flight cache fills when concurrent callers may request the same
expensive cache entry and only one caller should rebuild it.
```

Attach applications:

```rust
// @applies CACHE.SINGLE_FLIGHT_FILL
fn load_or_build_export_cache() {
    // ...
}
```

Then inspect:

```sh
special patterns
special patterns CACHE.SINGLE_FLIGHT_FILL --verbose
special patterns --metrics --target src/cache.rs
```

Pattern metrics are advisory. They help find possible missing applications,
possible new pattern clusters, and implementations that may be similar enough to
be a helper rather than a named pattern.

### Repo Health

Inspect repo-wide signals:

```sh
special health --metrics
special health --metrics --verbose
```

The health view is for questions that do not naturally belong to one module:
duplicate source items, unowned implementation, and traceability from tests and
source items back to current specs. When a language-specific trace route is
unavailable, `special` reports that instead of pretending weaker analysis proved
the same thing.

## Annotations

`special` currently recognizes these annotation families.

| Annotation | Meaning |
| --- | --- |
| `@group ID` | Structural spec container. Groups organize subtrees and do not carry direct support. |
| `@spec ID` | Product claim. |
| `@planned [RELEASE]` | Marks the owning spec as planned rather than current. |
| `@deprecated [RELEASE]` | Marks the owning current spec for retirement. |
| `@verifies ID` | Attaches one item-scoped verification artifact to one spec. |
| `@fileverifies ID` | Attaches one file-scoped verification artifact to one spec. |
| `@attests ID` | Attaches one item-scoped manual or external attestation to one spec. |
| `@fileattests ID` | Attaches one file-scoped attestation to one spec. |
| `@area ID` | Structural architecture container. |
| `@module ID` | Concrete architecture module. |
| `@implements ID` | Attaches item-scoped implementation ownership to a module. |
| `@fileimplements ID` | Attaches file-scoped implementation ownership to a module. |
| `@pattern ID` | Declares one adopted implementation pattern. |
| `@strictness high\|medium\|low` | Optional pattern metadata for similarity expectations. |
| `@applies ID` | Attaches an item-scoped application of a pattern. |
| `@fileapplies ID` | Attaches a file-scoped application of a pattern. |

Important constraints:

- `@group` and `@spec` are mutually exclusive for the same id.
- `@planned` and `@deprecated` are local to the owning `@spec`.
- a `@spec` may not be both `@planned` and `@deprecated`.
- child claims do not justify a parent `@spec`.
- one verify or attest block targets one spec id.
- `@verifies` only counts when it attaches to a supported owned item.
- current `@module` nodes require direct `@implements` or `@fileimplements`
  unless they are planned.
- `@area` is structural only and does not accept `@planned` or implementation
  ownership.
- each `@pattern ID` may have only one definition.
- `@applies` and `@fileapplies` must attach to source code, not markdown
  declarations.
- pattern metrics are advisory and do not create lint failures.

## Root Discovery

`special` prefers explicit root selection through `special.toml`:

```toml
version = "1"
root = "."
```

Current behavior:

- if `special.toml` is present, it anchors discovery
- `root` is resolved relative to the config file
- if no config exists, `special` prefers the nearest enclosing VCS root
- if no config or VCS marker exists, it falls back to the current directory
- implicit root selection emits a warning

`special init` exists to make root discovery explicit quickly.

## Supported Inputs

Current parser support covers:

- Rust line comments
- generic block comments
- Go line comments
- TypeScript line comments
- TypeScript block comments
- shell `#` comments
- Python `#` comments
- markdown annotation lines

Spec, architecture, and pattern trees may be spread across multiple files and
mixed supported file types.

Built-in implementation analysis currently surfaces code evidence for owned
Rust, TypeScript, and Go code. Depending on language and available tools,
`--metrics` can include:

- public and internal item counts
- function and cognitive complexity summaries
- quality evidence such as public API parameter shape, stringly typed
  boundaries, and recoverability signals
- unreached-code indicators
- language-native dependency evidence
- module coupling evidence derived from owned dependency targets
- per-item evidence for connected, outbound-heavy, isolated, unreached,
  high-complexity, parameter-heavy, stringly boundary, and panic-heavy source
  items

## Skills

`special skills` prints bundled workflow skills:

```sh
special skills
special skills ship-product-change
special skills install
special skills install ship-product-change
```

`special skills install` writes task-shaped skills
into `.agents/skills/` or
another selected destination for:

- shipping a product change without changing the contract by accident
- defining product specs
- validating whether a claim is honestly supported
- validating whether a concrete architecture module is honestly implemented
- inspecting the current spec state
- finding planned work
- following and reviewing adopted implementation patterns

Installed skill files are generated output and are typically ignored by the repo.

## Development

Source development currently expects sibling checkouts:

```text
workspace/
  special/
  crates/
    parse-source-annotations/
```

`special` consumes `parse-source-annotations` from
`../crates/parse-source-annotations`. Local development keeps that sibling
checkout in place. Release builds recreate the same source layout by cloning
`https://github.com/sourcerodeo/parse-source-annotations` into `../crates/`
before Cargo runs, using GitHub token authentication so the parser crate
repository may remain private.

For local repo development, use the tool-managed commands:

```sh
mise exec -- cargo test
mise exec -- cargo run -- lint
mise exec -- cargo run -- specs
mise exec -- cargo run -- patterns
mise exec -- cargo run -- health
mise exec -- cargo run -- arch --metrics
```

The repo is a small polyglot monorepo. The Rust CLI crate lives at the root with
Cargo's conventional `src/`, `tests/`, and Askama `templates/` layout. The Lean
scoped traceability kernel lives as a separate Lake project under `lean/`.
`build.rs` embeds the compiled Lean kernel for released host-native binaries.

## Release Automation

This repo carries its own release automation contract in `special` format.

Run the Rust code review separately when you want it:

```sh
python3 scripts/review-rust-release-style.py
```

Publish a release through the local wrapper as a three-step pipeline. The first
step makes you enter the exact release-visible changelog bullets and writes the
versioned `CHANGELOG.md` section:

```sh
python3 scripts/tag-release.py X.Y.Z --prepare
```

Then run the validation phase. It executes the release validation commands and
records ignored local evidence for the current release revision:

```sh
python3 scripts/tag-release.py X.Y.Z --validate
```

Publish only after the prepared changelog and validation evidence are attached
to the release revision:

```sh
python3 scripts/tag-release.py X.Y.Z --publish
```

The wrapper refuses missing or placeholder changelog notes, tracked private or
generated files, mismatched manifest versions, missing validation evidence, and
legacy checklist bypass flags.

The current distribution slice covers:

- crates.io package name and installed binary name
- GitHub repository metadata for release automation
- committed GitHub Actions release workflow
- published release archives and checksums for supported targets
- committed Homebrew formula in `sourcerodeo/homebrew-tap`

Actual published GitHub Releases are a separate claim from release automation
itself.

## Project Truth

Special is self-hosting: the canonical product truth for this repo lives in its
own `special` declarations, primarily colocated with the owning source and test
boundaries. Central markdown remains only for structural and planned contract
scaffolding.

If this README and the materialized spec disagree, the spec wins.

The repo root is explicitly anchored by [special.toml](special.toml).
