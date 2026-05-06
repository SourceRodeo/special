@applies DOCS.TASK_RECIPE_PAGE
# How-to

@implements SPECIAL.DOCUMENTATION.PUBLIC.HOW_TO.ADOPT_EXISTING_REPO
@applies DOCS.CROSS_SURFACE_WORKFLOW
## Adopt Special in an Existing Repo

Start with one narrow slice. Do not try to model the whole repository at once.

```sh
special init
special specs --unverified
special arch --unimplemented
special health --metrics
```

Add one spec and one proof. Add one module and one ownership attachment. Run
health again. Repeat around real work.

@implements SPECIAL.DOCUMENTATION.PUBLIC.HOW_TO.INVESTIGATE_HEALTH
@applies DOCS.CROSS_SURFACE_WORKFLOW
## Investigate Health Output

Use the signal to choose the next command:

| Health signal | Next command | Typical fix |
| --- | --- | --- |
| unowned implementation | `special arch --unimplemented` | Add or adjust module ownership. |
| unsupported implementation | `special specs --verbose` | Move behavior behind a tested module or add direct proof. |
| duplicate items | `special patterns --metrics` | Extract a helper or name a real repeated pattern. |
| undocumented targets | `special health --metrics` | Add generated docs links or confirm the target is internal. |

@implements SPECIAL.DOCUMENTATION.PUBLIC.HOW_TO.WRITE_TRACEABLE_DOCS
@applies DOCS.TRACEABLE_DOCS_EXAMPLE
## Write Traceable Docs

Link factual docs claims to the smallest relevant Special id:

```markdown
[Pattern metrics are advisory](documents://spec/SPECIAL.PATTERNS.METRICS).
```

Build and check:

```sh
special docs build
special docs --metrics
special health --metrics
```

Use `special docs --metrics` for docs graph and relationship inventory. Use
`special health --metrics` for cross-surface documentation coverage.

@implements SPECIAL.DOCUMENTATION.PUBLIC.HOW_TO.INTRODUCE_PATTERNS
@applies DOCS.CROSS_SURFACE_WORKFLOW
## Introduce a Pattern

Name a pattern only after you can point to a repeated implementation structure.

```text
@pattern API.IDEMPOTENT_RETRY
Retry idempotent requests with bounded backoff and a caller-visible final error.
```

Apply it where the structure appears:

```ts
// @applies API.IDEMPOTENT_RETRY
async function fetchWithRetry(request: Request): Promise<Response> {
  return retry(request, { attempts: 3 });
}
```

Inspect:

```sh
special patterns API.IDEMPOTENT_RETRY --verbose
special patterns --metrics
```
