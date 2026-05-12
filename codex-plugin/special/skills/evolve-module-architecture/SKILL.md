---
name: evolve-module-architecture
description: 'Use this skill when changing ownership boundaries, module responsibilities, subsystem splits, or command-surface design. Prefer MCP arch tools, with CLI fallback.'
---

# Evolve Module Architecture

## When To Use

Use this when a task changes code ownership: splitting modules, moving
responsibilities, adding a command boundary, or introducing architecture
annotations.

Do not use architecture to prove product behavior.

## Workflow

1. Inspect the current boundary with `special_arch`; fall back to `special arch --metrics`.
2. Add the smallest useful `@module` when no boundary exists.
3. Use `@area` only for structure.
4. Attach implementation with `@implements` or `@fileimplements`.
5. Use `special_trace` with `surface: "arch"` for a module review packet.
6. Run `special_lint`.

## What To Do With Results

- Move or narrow implementation annotations when ownership is too broad.
- Mark future architecture planned instead of leaving current modules
  unimplemented.
- Move behavior promises to specs and repeated implementation choices to
  patterns.
