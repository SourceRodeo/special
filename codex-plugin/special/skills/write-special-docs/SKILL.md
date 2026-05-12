---
name: write-special-docs
description: 'Use this skill when authoring Special-generated public, contributor, or skill docs. Structure docs with modules and patterns, and treat docs links as claims that require review.'
---

# Write Special Docs

## When To Use

Use this when changing generated docs source, including public docs,
contributor docs, and generated plugin or fallback skills.

## Workflow

1. Inspect the docs path with `special_docs`; fall back to `special docs --metrics --target PATH`.
2. Attach docs modules with `@implements` at the page, section, heading, or block that owns the reader-visible responsibility.
3. Attach docs patterns with `@applies` where the source actually follows the structure.
4. Use `documents://` only where the surrounding prose documents the linked target. Do not use it as a generic cross-reference.
5. Use `@documents` or `@filedocuments` when one natural block or file documents one target better than inline links.
6. Build generated docs with `special_docs_output`; fall back to `special docs build`.
7. Use `special_trace` with `surface: "docs"` for claims that need alignment review.

## What To Do With Results

- Replace generic references with ordinary markdown links.
- Keep `documents://` links only when the claim and linked target match.
- Fix docs modules or patterns when metrics show the docs architecture is not
  implemented.
- Run `special_lint` after docs source edits.
