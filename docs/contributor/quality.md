# Quality Gates

Release readiness is a local evidence trail. Do not claim a release or review
state until the command that owns the evidence has actually run.

## Core Gates

Use the broad suite before claiming readiness:

```sh
mise exec -- cargo fmt --check
mise exec -- cargo test
mise exec -- cargo run --quiet -- lint
mise exec -- cargo run --quiet -- docs build
mise exec -- cargo run --quiet -- docs --metrics
mise exec -- cargo test --test docs_self_check
```

Clippy is pinned through the quality test surface:
pinned clippy flags,
mise exec,
cargo clippy,
all targets,
all features, and
deny warnings.

## Release Review

The release-review wrapper is manual, local, and
warn-only.
It should capture durable structured output while staying code-focused:
spec-owned wrapper,
structured output,
DeepSeek swarm mode,
selective swarm context,
raw swarm findings,
code-only surface,
read-only sandbox,
no web,
durable output,
local only, and
manual only.

For a broad release pass, let the wrapper choose the swarm width:

```sh
python3 scripts/review-rust-release-style.py --swarm
```

That keeps each DeepSeek/OpenCode prompt selective. Pass an explicit count only
when you intentionally want a smaller or larger review swarm. The swarm writes
raw markdown findings so an imperfect model response is still reviewable.
Agents may use read-only repo inspection tools to check cross-file evidence,
but mutation, web access, shell execution, and external directories stay denied.

## Distribution Checks

Distribution tests protect the package, release, Homebrew, parser dependency,
Lean kernel embedding,
and plugin layout:
Cargo package name,
binary name,
GitHub release workflow,
parser source dependency,
no local parser checkout,
Homebrew formula, and
Codex plugin layout.
