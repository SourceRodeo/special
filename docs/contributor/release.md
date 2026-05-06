# Release and Distribution

Special distributes source through GitHub and binaries through GitHub Releases,
Homebrew, and Cargo.

## Source Dependencies

The parser crate
lives in the `SourceRodeo/crates` monorepo at `parse-source-annotations`.
Special resolves that package through Cargo's Git dependency support during
local development and release builds.

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

## Homebrew

The Homebrew formula
lives in `sourcerodeo/homebrew-tap` at `Formula/special.rb`. Release validation
checks version, platform archive branches, release asset digests, and checksum
pairing against the GitHub release assets.

## Plugin Marketplace

The Special Codex plugin source lives under `codex-plugin/special/` in this
repository. The shared SourceRodeo marketplace entry points at that subdirectory.
