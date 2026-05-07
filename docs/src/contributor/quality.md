@applies DOCS.CONTRIBUTOR_RUNBOOK_PAGE
@implements SPECIAL.DOCUMENTATION.CONTRIBUTOR.QUALITY
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
[pinned clippy flags](documents://spec/SPECIAL.QUALITY.RUST.CLIPPY.PINNED_FLAGS),
[mise exec](documents://spec/SPECIAL.QUALITY.RUST.CLIPPY.PINNED_FLAGS.MISE_EXEC),
[cargo clippy](documents://spec/SPECIAL.QUALITY.RUST.CLIPPY.PINNED_FLAGS.CARGO_CLIPPY),
[all targets](documents://spec/SPECIAL.QUALITY.RUST.CLIPPY.PINNED_FLAGS.ALL_TARGETS),
[all features](documents://spec/SPECIAL.QUALITY.RUST.CLIPPY.PINNED_FLAGS.ALL_FEATURES), and
[deny warnings](documents://spec/SPECIAL.QUALITY.RUST.CLIPPY.PINNED_FLAGS.DENY_WARNINGS).

## Release Review

The release-review wrapper is manual, local, and
[warn-only](documents://spec/SPECIAL.QUALITY.RUST.RELEASE_REVIEW.WARN_ONLY).
It should capture durable structured output while staying code-focused:
[spec-owned wrapper](documents://spec/SPECIAL.QUALITY.RUST.RELEASE_REVIEW.SPEC_OWNED),
[structured output](documents://spec/SPECIAL.QUALITY.RUST.RELEASE_REVIEW.STRUCTURED_OUTPUT),
[code-only surface](documents://spec/SPECIAL.QUALITY.RUST.RELEASE_REVIEW.CODE_ONLY_SURFACE),
[read-only sandbox](documents://spec/SPECIAL.QUALITY.RUST.RELEASE_REVIEW.READ_ONLY_SANDBOX),
[no web](documents://spec/SPECIAL.QUALITY.RUST.RELEASE_REVIEW.NO_WEB),
[durable output](documents://spec/SPECIAL.QUALITY.RUST.RELEASE_REVIEW.DURABLE_OUTPUT),
[local only](documents://spec/SPECIAL.QUALITY.RUST.RELEASE_REVIEW.LOCAL_ONLY), and
[manual only](documents://spec/SPECIAL.QUALITY.RUST.RELEASE_REVIEW.MANUAL_ONLY).

## Distribution Checks

Distribution tests protect the package, release, Homebrew, parser dependency,
[Lean kernel embedding](documents://spec/SPECIAL.DISTRIBUTION.GITHUB_RELEASES.LEAN_KERNEL),
and plugin layout:
[Cargo package name](documents://spec/SPECIAL.DISTRIBUTION.CRATES_IO.PACKAGE_NAME),
[binary name](documents://spec/SPECIAL.DISTRIBUTION.CRATES_IO.BINARY_NAME),
[GitHub release workflow](documents://spec/SPECIAL.DISTRIBUTION.GITHUB_RELEASES.WORKFLOW),
[parser source dependency](documents://spec/SPECIAL.DISTRIBUTION.SOURCE_DEPENDENCIES.PARSER_CRATE),
[no local parser checkout](documents://spec/SPECIAL.DISTRIBUTION.SOURCE_DEPENDENCIES.NO_LOCAL_CHECKOUT),
[Homebrew formula](documents://spec/SPECIAL.DISTRIBUTION.HOMEBREW.FORMULA), and
[Codex plugin layout](documents://spec/SPECIAL.DISTRIBUTION.CODEX_PLUGIN.SOURCE_LAYOUT).
