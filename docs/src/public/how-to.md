@applies DOCS.TASK_RECIPE_PAGE
# How-to

@implements SPECIAL.DOCUMENTATION.PUBLIC.HOW_TO.ADOPT_EXISTING_REPO
@applies DOCS.CROSS_SURFACE_WORKFLOW
## Adopt Special in an Existing Repo

Start by reading the repository before adding annotations. Special can inspect a
plain source tree and give useful signals from `health` and `patterns`; deeper
traceability improves once the repo declares its toolchain and starts adding
durable Special ids.

```sh
special init
special health --metrics
special patterns --metrics
```

Use the first reports to choose one narrow slice:

| Signal | First durable move |
| --- | --- |
| repeated source shapes | decide whether to extract a helper or declare an adopted pattern |
| unowned implementation | declare a module and attach the code it owns |
| unsupported behavior | add a spec only when the behavior is a real product claim, then attach proof |
| undocumented public surface | add a docs link to the smallest relevant spec, module, area, or pattern |

Then run the loop around that slice:

```sh
special specs --unverified
special arch --unimplemented
special docs --metrics
special health --metrics
special lint
```

Do not model the whole repo on day one. Let the first `health` and `patterns`
reports tell you where the repo is already asking for a clearer boundary, proof,
pattern, or docs claim.

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

Health is the broad investigation command. Once it identifies a concrete
question, move to the surface command that owns the fix: `specs` for claims and
proof, `arch` for ownership, `patterns` for repeated structures, and `docs` for
reader-facing claims.

@implements SPECIAL.DOCUMENTATION.PUBLIC.HOW_TO.WRITE_TRACEABLE_DOCS
@applies DOCS.TRACEABLE_DOCS_EXAMPLE
## Write Traceable Docs

Link factual docs claims to the smallest relevant Special id:

```markdown
[Pattern similarity is advisory](documents://spec/SPECIAL.PATTERNS.METRICS.SIMILARITY).
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
