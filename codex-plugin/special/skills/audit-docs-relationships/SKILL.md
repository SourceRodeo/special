---
name: audit-docs-relationships
description: 'Use this skill when checking whether docs claims are true. Build the relationship inventory with Special, then trace each important relationship through its target and support chain.'
---

# Audit Docs Relationships

## When To Use

Use this when reviewing docs before release, validating generated skill
guidance, or checking docs after product, architecture, pattern, or proof
changes.

## Workflow

1. Use `special_docs` with `metrics: true` and a narrow `target` to build the relationship inventory; fall back to `special docs --metrics --target PATH`.
2. Use `special_trace` with `surface: "docs"` for the same file or directory.
3. Review one packet at a time: docs prose, linked target declaration, and
   support evidence.
4. Decide whether the prose actually documents the target. A found relationship
   proves only that the relationship exists.
5. Fix either the docs prose, the target, or the support chain depending on
   which side is stale.

## What To Do With Results

- Keep relationships that align semantically.
- Use ordinary markdown links for see-also navigation.
- Use `@documents` when a whole paragraph documents a single target.
- Finish with `special_lint`.
