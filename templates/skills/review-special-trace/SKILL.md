---
name: review-special-trace
description: 'Use this skill when a task needs a focused Special relationship packet. Run trace on specs, docs, architecture, or patterns, then judge semantic alignment yourself instead of treating the packet as proof.'
---

# Review Special Trace

## When To Use

Use this for targeted reviews of one relationship chain:

- a spec and its verifies or attests
- docs prose and the target it documents
- a module and its implementation attachment
- a pattern and its applications

Use health first when the question is broad and no explicit relationship is
known yet.

## Workflow

1. Pick the smallest target that covers the review.
2. Run the appropriate `special trace` surface:

   ```sh
   special trace specs --id EXPORT.CSV.HEADER
   special trace docs --target docs/src/public/docs.md
   special trace arch --id APP.EXPORT
   special trace patterns --id EXPORT.LABEL_VALUE_COLUMNS
   ```

3. Read the packet as context, not as a verdict.
4. For specs, compare the exact claim to every proof attachment.
5. For docs, compare the prose to the linked target and the target's support
   chain.
6. For architecture, compare the module description to the attached
   implementation body.
7. For patterns, compare the pattern definition to each application and any fit
   metrics in `special patterns`.
8. If the packet is too large, rerun with a narrower id or target before making
   a judgment.

## What To Do With Results

- If the relationship is aligned, keep it and record the narrow validation you
  actually performed.
- If the target is correct but the prose or implementation is stale, update the
  prose or code.
- If the evidence proves a smaller claim, narrow or split the spec.
- If a relationship only exists by adjacency or naming convention, do not count
  it as support.
- Rerun `special trace` after edits and then run `special lint`.
