---
name: define-product-specs
description: 'Use this skill when turning requirements, feature ideas, bug reports, roadmap notes, or vague behavior into Special product claims. Prefer MCP tools when available, with CLI fallback.'
---

# Define Product Specs

## When To Use

Use this when a task needs a durable behavior claim: new feature scope, bug
expectation, release behavior, or roadmap prose that should become a current or
planned contract.

Do not use specs for implementation ownership or repeated implementation shape.

## Workflow

1. Check status with `special_status`.
2. Inspect existing claims with `special_specs`; fall back to `special specs --metrics`.
3. Use `@group` for navigation and `@spec` for real behavior claims.
4. Mark future claims with `@planned`.
5. Attach one honest `@verifies` or `@attests` artifact to each current claim.
6. For review, use `special_trace` with `surface: "specs"` and the exact id; fall back to `special trace specs --id SPEC.ID`.
7. Run `special_lint` or `special lint`.

## What To Do With Results

- If the proof supports only part of the claim, split or narrow the spec.
- If the behavior is not current, keep it planned.
- If the claim is internal design, move it to architecture or pattern guidance.
- If the repo has no Special surface yet, add the smallest useful spec near the
  relevant tests or docs instead of building a parallel requirements document.
