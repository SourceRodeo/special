---
name: write-special-docs
description: 'Use this skill when authoring Special-generated public, contributor, or skill docs. Structure docs with modules and patterns, and use docs relationships only where the surrounding prose actually documents the linked target.'
---
@filedocuments spec SPECIAL.DOCS_COMMAND

# Write Special Docs
@implements SPECIAL.DOCUMENTATION.SKILLS.FALLBACK.WRITE_SPECIAL_DOCS
@applies DOCS.SKILL_MAIN_ENTRY

## When To Use
@applies DOCS.SKILL_TRIGGER_BOUNDARY_SECTION

Use this when changing docs source that is built by [`special docs build`](documents://spec/SPECIAL.DOCS_COMMAND.OUTPUT.CONFIG), including
public docs, contributor docs, and generated skill markdown.

Do not use this for ordinary ungenerated notes unless the project is moving
those notes into the docs-as-code surface.

## Workflow
@applies DOCS.SKILL_WORKFLOW_SECTION

1. Find the docs source path, usually under `docs/src`.
2. Read the relevant docs architecture before editing. A docs page, section, or
   heading can implement a docs module with [`@implements`](documents://spec/SPECIAL.MODULE_PARSE.IMPLEMENTS.MARKDOWN_SCOPE).
3. Apply docs patterns such as page, recipe, reference, or skill structures with
   [`@applies`](documents://spec/SPECIAL.PATTERNS.MARKDOWN_APPLICATIONS) on the heading or block that actually follows the pattern.
4. Use [`documents://`](documents://spec/SPECIAL.DOCS.LINKS.POLYMORPHIC) links only when the surrounding text documents the linked spec, module, area, or pattern.
   Do not use them as generic "see also" links.
5. Use [`@documents`](documents://spec/SPECIAL.DOCS.DOCUMENTS_LINES) only when a natural block documents one target, and [`@filedocuments`](documents://spec/SPECIAL.DOCS.DOCUMENTS_LINES) only when the whole file does.
6. Build the generated output and inspect docs metrics:

   ```sh
   special docs build
   special docs --metrics --target docs/src
   ```

7. For important claims, run trace on the docs source and compare the prose to
   the linked target and support chain:

   ```sh
   special trace docs --target docs/src/public/docs.md
   ```

## What To Do With Results
@applies DOCS.SKILL_RESULT_DISPOSITION_SECTION

- If the prose documents a target, keep or add the docs relationship.
- If the prose merely mentions a target, use an ordinary markdown link or prose
  without `documents://`.
- If generated output still contains authoring annotations or `documents://`,
  fix docs build handling before treating the output as shippable.
- If a docs module is unimplemented, attach `@implements` at the page, section,
  heading, or block that owns the reader-visible responsibility.
- After editing, run `special docs --metrics`, `special trace docs` for the
  touched claim set, and `special lint`.
