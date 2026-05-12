---
name: write-special-docs
description: 'Use this skill when authoring Special-generated public, contributor, or skill docs. Structure docs with modules and patterns, and treat docs links as claims that require review.'
---
@filedocuments spec SPECIAL.DOCS_COMMAND

# Write Special Docs
@implements SPECIAL.DOCUMENTATION.SKILLS.PLUGIN.WRITE_SPECIAL_DOCS
@applies DOCS.SKILL_MAIN_ENTRY

## When To Use
@applies DOCS.SKILL_TRIGGER_BOUNDARY_SECTION

Use this when changing generated docs source, including public docs,
contributor docs, and generated plugin or fallback skills.

## Workflow
@applies DOCS.SKILL_WORKFLOW_SECTION

1. Inspect the docs path with [`special_docs`](documents://spec/SPECIAL.DOCS_COMMAND); fall back to `special docs --metrics --target PATH`.
2. Attach docs modules with [`@implements`](documents://spec/SPECIAL.MODULE_PARSE.IMPLEMENTS.MARKDOWN_SCOPE) at the page, section, heading, or block that owns the reader-visible responsibility.
3. Attach docs patterns with [`@applies`](documents://spec/SPECIAL.PATTERNS.MARKDOWN_APPLICATIONS) where the source actually follows the structure.
4. Use [`documents://`](documents://spec/SPECIAL.DOCS.LINKS.POLYMORPHIC) only where the surrounding prose documents the linked target. Do not use it as a generic cross-reference.
5. Use [`@documents`](documents://spec/SPECIAL.DOCS.DOCUMENTS_LINES) or `@filedocuments` when one natural block or file documents one target better than inline links.
6. Build generated docs with `special_docs_output`; fall back to `special docs build`.
7. Use `special_trace` with `surface: "docs"` for claims that need alignment review.

## What To Do With Results
@applies DOCS.SKILL_RESULT_DISPOSITION_SECTION

- Replace generic references with ordinary markdown links.
- Keep `documents://` links only when the claim and linked target match.
- Fix docs modules or patterns when metrics show the docs architecture is not
  implemented.
- Run `special_lint` after docs source edits.
