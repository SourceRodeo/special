---
name: use-project-patterns
description: 'Use this skill when adding, reviewing, or refactoring code or docs that resembles a recurring project approach. Use pattern metrics as advisory evidence.'
---

# Use Project Patterns

## When To Use

Use this when code or docs resembles a repeated implementation structure,
template, or reviewable approach. Do not turn broad style advice into a pattern.

## Workflow

1. Inspect declared patterns with `special_patterns`; fall back to `special patterns --metrics`.
2. If a pattern exists, scope to it and include metrics.
3. If no pattern exists, define one only when it guides future work.
4. Attach `@applies` to the item, heading, or block that actually demonstrates the approach.
5. Use `@fileapplies` only when the whole file is the application.
6. Use `@strictness` when similarity expectations matter.
7. Use `special_trace` with `surface: "patterns"` when applications need review.

## What To Do With Results

- Apply a pattern when the source intentionally follows it.
- Extract a helper when examples are nearly identical and the repetition is only
  mechanics.
- Split or rewrite a pattern when applications drift.
- Treat missing-application metrics as review leads, not lint failures.
