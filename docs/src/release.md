@filedocuments spec SPECIAL.DISTRIBUTION.GITHUB_RELEASES.WORKFLOW
# Release and Distribution Notes

Special distributes source through GitHub and binaries through GitHub Releases,
Homebrew, and Cargo.

## Source Dependency Layout

The [parser crate](special://spec/SPECIAL.DISTRIBUTION.SOURCE_DEPENDENCIES.PARSER_CRATE)
lives in the `SourceRodeo/crates` monorepo:

```text
crates/
  parse-source-annotations/
```

The `special-cli` crate depends on the `parse-source-annotations` package from
`https://github.com/SourceRodeo/crates`. Cargo resolves that package from the
monorepo workspace during local development and release builds; Special no
longer requires a local sibling parser checkout.

## Release Workflow

Release automation uses the committed GitHub Actions workflow plus local release
scripts. The local release wrapper has three phases:

```sh
python3 scripts/tag-release.py X.Y.Z --prepare
python3 scripts/tag-release.py X.Y.Z --validate
python3 scripts/tag-release.py X.Y.Z --publish
```

The validation phase records
[evidence for the exact version and revision](special://spec/SPECIAL.DISTRIBUTION.RELEASE_FLOW.VALIDATION_EVIDENCE).

## Homebrew

The [Homebrew formula](special://spec/SPECIAL.DISTRIBUTION.HOMEBREW.FORMULA.TAP_METADATA_CHECK)
lives in `sourcerodeo/homebrew-tap` at
`Formula/special.rb`. Release validation reads the tap formula and checks version,
release URL, archive selectors, release asset digests, and checksum pairing
against the GitHub release assets.

## Plugin Marketplace

The Special Codex plugin source lives under `codex-plugin/special/` in this
repository. The shared marketplace entry points at that subdirectory.
