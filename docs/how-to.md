# How-to

## Adopt Special in an Existing Repo

Start with the scanning side of Special before adding annotations. Health and
pattern metrics can inspect plain source and show useful signals before the repo
has many explicit connections.

```sh
special init
special health --metrics
special patterns --metrics
```

The first report should choose a small slice, not a modeling campaign. Use the
signals to find one place where an explicit connection would clarify the repo:

| Signal | First durable move |
| --- | --- |
| `source outside architecture` in `src/billing` | declare a billing module and attach the implementation it owns |
| `duplicate source shapes` in export code | extract a helper or name an adopted export pattern |
| `untraced implementation` near business logic | move behavior behind a tested module facade or add direct proof for a real product claim |
| `long prose outside docs` in policy code | move reader-facing explanation into generated docs source and link it to the relevant claim |

Then run the loop around that slice:

```sh
special health --metrics --verbose --target src/billing
special arch --unimplemented
special lint
```

Make one improvement, then rerun the same scoped health command. The goal is to
connect the important fact or make the remaining signal explainable. A duplicate
shape can stay if the local parallelism is clearer than an abstraction. An
untraced command handler can stay visible if it is only an I/O boundary and the
underlying behavior is directly tested.

## Investigate Health Output

Use the signal to choose the next command. Keep the first investigation scoped;
`--target` is usually the difference between a useful report and an intimidating
repo-wide list.

| Health signal | Next command | Typical fix |
| --- | --- | --- |
| source outside architecture | `special arch --unimplemented` | Add or adjust module ownership. |
| untraced implementation | `special specs --verbose` | Move behavior behind a tested module or add direct proof. |
| duplicate source shapes | `special health --metrics --verbose --target PATH` | Decide whether to extract a helper, name a pattern, or leave local parallelism visible. |
| possible pattern clusters | `special patterns --metrics` | Promote a real repeated structure into `@pattern`, or do no action. |
| possible missing pattern applications | `special health --metrics --target PATH --verbose` | Add `@applies` where the pattern is really present. |
| long prose outside docs | `special docs --metrics` | Promote, link, or remove the prose deliberately. |
| long prose test literals | focused test review | Assert smaller contractual pieces, test a structured representation, or move large samples into fixtures. |

Health is the broad scanning command. Once it identifies a concrete question,
move to the surface command that owns the connection: `specs` for claims and
proof, `arch` for ownership, `patterns` for declared structures, and `docs` for
reader-facing claims.

Example scoped pass:

```text
special health
summary
  source outside architecture: 8
  untraced implementation: 23
  duplicate source shapes: 6
  possible pattern clusters: 2
  possible missing pattern applications: 1
  long prose outside docs: 3
duplicate source shapes by file
  src/billing/export.ts: 4
  src/billing/refunds.ts: 2
```

That output supports a concrete decision: inspect billing export code first. It
does not say to annotate everything. It says billing export shape, ownership,
and proof are the current review queue.

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
`special health --metrics` for source and prose signals that are not yet part of
an explicit connection.

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
