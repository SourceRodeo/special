# Release and Distribution Notes

Special distributes source through GitHub and binaries through GitHub Releases,
Homebrew, and Cargo.

## Source Dependency Layout

Development expects sibling checkouts:

```text
workspace/
  special/
  crates/
    parse-source-annotations/
```

The `special-cli` crate depends on `../crates/parse-source-annotations`. Release
jobs recreate that sibling layout before Cargo runs.

## Release Workflow

Release automation uses the committed GitHub Actions workflow plus local release
scripts. The local release wrapper has three phases:

```sh
python3 scripts/tag-release.py X.Y.Z --prepare
python3 scripts/tag-release.py X.Y.Z --validate
python3 scripts/tag-release.py X.Y.Z --publish
```

The validation phase records evidence for the exact version and revision.

## Homebrew

The Homebrew formula lives in `sourcerodeo/homebrew-tap` at
`Formula/special.rb`. Release validation reads the tap formula and checks version,
release URL, archive selectors, release asset digests, and checksum pairing
against the GitHub release assets.

## Plugin Marketplace

The Special Codex plugin source lives under `codex-plugin/special/` in this
repository. The shared marketplace entry points at that subdirectory.
