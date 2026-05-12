---
name: find-planned-work
description: 'Use this skill when looking for future product work, release blockers, or requirements that are not current yet. Prefer MCP specs tools, with CLI fallback.'
---

# Find Planned Work

## When To Use

Use this when a task asks what is planned, what blocks a release, or whether
roadmap prose should become a tracked product claim.

## Workflow

1. Run `special_specs` with `planned: true`; fall back to `special specs --planned`.
2. Scope by id when the list is large.
3. Read release metadata as a planning label, not as a semantic version range.
4. Convert durable future behavior into `@spec ... @planned`.
5. Use `special_trace` only after a planned claim becomes current and needs support review.

## What To Do With Results

- Remove `@planned` only when the behavior is implemented and supported.
- Delete stale future claims instead of preserving empty roadmap structure.
- Track internal refactors outside product specs unless they change behavior.
- Run `special_lint` after changing planned/current state.
