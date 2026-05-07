@applies DOCS.CONTRIBUTOR_RUNBOOK_PAGE
@implements SPECIAL.DOCUMENTATION.CONTRIBUTOR.CACHE
# Cache Behavior

Special caches expensive parse, architecture, health, and language-pack analysis
work so repeated command runs stay usable. Cache behavior is part of command UX:
status output must make waiting visible, cache locks must recover cleanly, and
invalid cached graph facts must not silently become trusted analysis.

## What To Preserve

Before changing cache keys or invalidation, run the cache behavior tests and at
least one command that exercises shared discovery:

```sh
mise exec -- cargo test cache::tests
mise exec -- cargo run --quiet -- health --metrics --target docs/src --within docs/src
```

The release-sensitive contracts are currently held by tests rather than public
spec ids. Treat these behaviors as maintainer-facing:

- content edits invalidate parsed repo and architecture cache entries
- scoped repo analysis cache entries are separate from full-repo analysis
- concurrent fills single-flight instead of corrupting cache files
- stale locks are recovered before a new fill proceeds
- wait status is emitted when another process owns the cache fill
- invalid language-pack graph fact blobs are rejected rather than trusted

## When To Re-key

Re-key a cache only when the input boundary changes: parser dialect, source
content, configured ignore rules, language-pack fact schema, toolchain contract,
or the shape of the serialized analysis output. Do not re-key just to hide a
determinism issue; fix the unstable input or output ordering first.

