---
name: inspect-current-spec-state
description: 'Use this skill when you need to know what behavior the project currently claims and supports. Prefer Special MCP specs tools, with CLI fallback.'
---

# Inspect Current Spec State

## When To Use

Use this before answering what the product currently does, preparing release
notes, checking support state, or changing behavior.

## Workflow

1. Check status with `special_status`.
2. Run `special_specs` with `current: true`; fall back to `special specs --current`.
3. Include metrics when you need support counts; fall back to `special specs --current --metrics`.
4. Scope with an id when the tree is large.
5. Use `special_trace` with `surface: "specs"` when a current claim needs proof review.

## What To Do With Results

- Cite a current claim only after reading its support.
- Add proof, narrow the claim, or mark it planned when support is missing.
- Move ownership questions to architecture and repeated-shape questions to
  patterns.
- Run `special_lint` after changing claims or support.
