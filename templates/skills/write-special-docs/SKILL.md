---
name: write-special-docs
description: 'Use this skill when authoring Special-generated public, contributor, or skill docs. Structure docs with modules and patterns, and use docs relationships only where the surrounding prose actually documents the linked target.'
---

# Write Special Docs

## When To Use

Use this when changing docs source that is built by `special docs build`, including
public docs, contributor docs, and generated skill markdown.

Do not use this for ordinary ungenerated notes unless the project is moving
those notes into the docs-as-code surface.

## Workflow

1. Find the docs source path, usually under `docs/src`.
2. Read the relevant docs architecture before editing. A docs page, section, or
   heading can implement a docs module with `@implements`.
3. Apply docs patterns such as page, recipe, reference, or skill structures with
   `@applies` on the heading or block that actually follows the pattern.
4. Use `documents://` links only when the surrounding text documents the linked spec, module, area, or pattern.
   Do not use them as generic "see also" links.
5. Use `@documents` only when a natural block documents one target, and `@filedocuments` only when the whole file does.
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

- If the prose documents a target, keep or add the docs relationship.
- If the prose merely mentions a target, use an ordinary markdown link or prose
  without `documents://`.
- If generated output still contains authoring annotations or `documents://`,
  fix docs build handling before treating the output as shippable.
- If a docs module is unimplemented, attach `@implements` at the page, section,
  heading, or block that owns the reader-visible responsibility.
- After editing, run `special docs --metrics`, `special trace docs` for the
  touched claim set, and `special lint`.
