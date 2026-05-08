@applies DOCS.CONTRIBUTOR_RUNBOOK_PAGE
@implements SPECIAL.DOCUMENTATION.CONTRIBUTOR.RELEASE
# Release and Distribution

Special distributes source through GitHub and binaries through
[GitHub Releases](documents://spec/SPECIAL.DISTRIBUTION.GITHUB_RELEASES.PUBLISHED),
[Homebrew](documents://spec/SPECIAL.DISTRIBUTION.HOMEBREW.INSTALLS_SPECIAL),
and [Cargo](documents://spec/SPECIAL.DISTRIBUTION.CRATES_IO.PACKAGE_NAME).

## Source Dependencies

The [parser crate](documents://spec/SPECIAL.DISTRIBUTION.SOURCE_DEPENDENCIES.PARSER_CRATE)
lives in the `SourceRodeo/crates` monorepo at `parse-source-annotations`.
Special resolves that package through Cargo's Git dependency support during
local development and release builds.
Release builds must not resolve it through a
[local checkout](documents://spec/SPECIAL.DISTRIBUTION.SOURCE_DEPENDENCIES.NO_LOCAL_CHECKOUT).

The Cargo package declares its
[minimum supported Rust version](documents://spec/SPECIAL.DISTRIBUTION.CRATES_IO.RUST_VERSION)
in `Cargo.toml`. Keep that value aligned with the Rust edition and source syntax
used by the release. The repo-local `mise.toml` can use a newer stable Rust for
development; it is not the MSRV contract.

## Release Workflow

The local release script has explicit phases:

```sh
python3 scripts/tag-release.py X.Y.Z --prepare
python3 scripts/tag-release.py X.Y.Z --validate
python3 scripts/tag-release.py X.Y.Z --publish
```

The validation phase records
[evidence for the exact version and revision](documents://spec/SPECIAL.DISTRIBUTION.RELEASE_FLOW.VALIDATION_EVIDENCE).
The [publish phase](documents://spec/SPECIAL.DISTRIBUTION.RELEASE_FLOW.PUSHES_MAIN_RELEASE_BOOKMARK_AND_TAG)
pushes `main`, a `release/vX.Y.Z` bookmark, and the Git tag for the same
revision.
The same flow supports
[dry-run](documents://spec/SPECIAL.DISTRIBUTION.RELEASE_FLOW.DRY_RUN),
[manifest-version checks](documents://spec/SPECIAL.DISTRIBUTION.RELEASE_FLOW.MATCHES_MANIFEST_VERSION),
[GitHub release verification](documents://spec/SPECIAL.DISTRIBUTION.RELEASE_FLOW.VERIFIES_GITHUB_RELEASE),
and [Homebrew updates](documents://spec/SPECIAL.DISTRIBUTION.RELEASE_FLOW.UPDATES_HOMEBREW).

## Homebrew

The [Homebrew formula](documents://spec/SPECIAL.DISTRIBUTION.HOMEBREW.FORMULA.TAP_METADATA_CHECK)
lives in `sourcerodeo/homebrew-tap` at `Formula/special.rb`. Release validation
checks version, platform archive branches, release asset digests, and checksum
pairing against the GitHub release assets.
The formula has a stable
[path](documents://spec/SPECIAL.DISTRIBUTION.HOMEBREW.FORMULA.PATH) and
[platform selection](documents://spec/SPECIAL.DISTRIBUTION.HOMEBREW.FORMULA.PLATFORM_SELECTION)
contract.

## Plugin Marketplace

The Special Codex plugin source lives under
[`codex-plugin/special/`](documents://spec/SPECIAL.DISTRIBUTION.CODEX_PLUGIN.SOURCE_LAYOUT)
in this repository. The shared SourceRodeo marketplace entry points at that
subdirectory, and the plugin carries
[version awareness](documents://spec/SPECIAL.DISTRIBUTION.CODEX_PLUGIN.VERSION_AWARENESS).
