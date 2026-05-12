---
name: ship-product-change
description: 'Use this skill when adding a feature, fixing a bug, or changing behavior. Keep specs, implementation, proof, architecture, patterns, docs, and lint aligned.'
---

# Ship Product Change

## When To Use

Use this for ordinary behavior work: a feature, bug fix, CLI/API change, output
change, or release-impacting product change.

## Workflow

1. Start with the behavior change in one sentence.
2. Use `special_specs` or `special specs` to find the existing claim.
3. Add or update the narrow `@spec` if no claim exists.
4. Implement the change.
5. Attach or update `@verifies` or `@attests`.
6. Update `@module` and `@implements` if ownership changed.
7. Update `@pattern` and `@applies` when the implementation follows an adopted approach.
8. Use `special_trace` for the touched spec, module, docs relationship, or pattern before calling the change complete.
9. Finish with `special_lint` and the relevant scoped command.

## What To Do With Results

- If health reports touched code, inspect whether it needs real architecture or
  proof work rather than suppressing it.
- If docs changed, trace the docs relationship and compare prose to the linked
  target.
- If the change is internal-only, keep product specs focused on behavior and use
  architecture or patterns for implementation intent.
