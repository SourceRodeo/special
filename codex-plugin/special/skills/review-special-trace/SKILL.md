---
name: review-special-trace
description: 'Use this skill when a task needs a focused Special relationship packet. Run trace through MCP or CLI, then compare claim, prose, code, and evidence yourself.'
---

# Review Special Trace

## When To Use

Use this for targeted relationship reviews across specs, docs, architecture, or
patterns. Use health first when there is no known explicit relationship.

## Workflow

1. Choose the surface: specs, docs, arch, or patterns.
2. Run `special_trace` with the surface plus the narrowest id or target. Fall back to:

   ```sh
   special trace specs --id EXPORT.CSV.HEADER
   special trace docs --target docs/src/public/docs.md
   special trace arch --id APP.EXPORT
   special trace patterns --id EXPORT.LABEL_VALUE_COLUMNS
   ```

3. Treat the packet as context. It is not a success marker by itself.
4. Compare the exact natural-language claim to the evidence in the packet.
5. Rerun with a narrower id or target when the packet is too large to review
   accurately.

## What To Do With Results

- Keep aligned relationships.
- Fix stale prose, claims, implementation, or proof where alignment fails.
- Do not count adjacency, naming, or mere link existence as support.
- Run `special_lint` after edits.
