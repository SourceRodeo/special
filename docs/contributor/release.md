# Release and Distribution

Special distributes source through GitHub and binaries through
GitHub Releases,
Homebrew,
and Cargo.

## Source Dependencies

The parser crate
lives in the `SourceRodeo/crates` monorepo at `parse-source-annotations`.
Special resolves that package through Cargo's Git dependency support during
local development and release builds.
Release builds must not resolve it through a
local checkout.

The Cargo package declares its
minimum supported Rust version
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
evidence for the exact version and revision.
The publish phase
pushes `main`, a `release/vX.Y.Z` bookmark, and the Git tag for the same
revision.
The same flow supports
dry-run,
manifest-version checks,
GitHub release verification,
and Homebrew updates.

## Homebrew

The Homebrew formula
lives in `sourcerodeo/homebrew-tap` at `Formula/special.rb`. Release validation
checks version, platform archive branches, release asset digests, and checksum
pairing against the GitHub release assets.
The formula has a stable
path and
platform selection
contract.

## Plugin Marketplace

The Special Codex plugin source lives under
`codex-plugin/special/`
in this repository. The shared SourceRodeo marketplace entry points at that
subdirectory, and the plugin carries
version awareness.
