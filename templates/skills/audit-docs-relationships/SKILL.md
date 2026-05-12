---
name: audit-docs-relationships
description: 'Use this skill when checking whether docs claims are true. Build the relationship inventory with Special, then trace each important docs relationship through its linked target and support evidence.'
---

# Audit Docs Relationships

## When To Use

Use this when reviewing docs before release, checking whether public docs match
implementation, or validating generated skill guidance.

Do not use raw text search as the source of truth for relationship inventory.
Special already applies ignore rules, docs-source rules, and parsed relationship
semantics.

## Workflow

1. Scope the audit to one file, directory, or feature slice.
2. Build the docs inventory with `special docs --metrics`:

   ```sh
   special docs --metrics --target docs/src/public/docs.md
   ```

3. Use `special trace docs` for the same target:

   ```sh
   special trace docs --target docs/src/public/docs.md
   ```

4. For each relationship packet, read the docs prose first, then the linked
   target declaration, then any proof, implementation, or pattern evidence
   included by the packet.
5. Decide whether the prose actually documents the linked target. A visible
   relationship proves only that Special found a relationship, not that the
   natural-language claim is correct.
6. When a docs claim is too broad, fix the prose or link it to a more exact
   target. When the target is stale, fix the spec, module, pattern, or proof
   instead of weakening the docs.

## What To Do With Results

- Keep relationships whose surrounding prose matches the linked target and
  support chain.
- Replace generic cross-references with ordinary markdown links or prose.
- Add `documents://` only where the sentence or paragraph contains a concrete
  claim about the target.
- Use `@documents` for a whole natural block when inline links would fragment
  otherwise readable prose.
- Finish with `special lint` to catch broken ids and graph errors.
