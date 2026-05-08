# Health

Health is Special's broad scan of source, tests, docs, and repeated structures. Use
`special health` when explicit Special
relationships do not yet answer the practical question: which code, prose,
repeated shape, or proof path needs attention next?

Health is usually the best first command in an existing repository because it
can read plain source before the repo has many Special annotations.

```sh
special health --metrics
```

Representative output shape for a TypeScript service:

```text
special health
summary
  source outside architecture: 41
  untraced implementation: 128
  duplicate source shapes: 17
  possible pattern clusters: 6
  possible missing pattern applications: 3
  uncaptured prose outside docs: 9
  long prose test literals: 2
duplicate source shapes by file
  src/billing/invoices.ts: 5
  src/billing/refunds.ts: 4
  src/admin/export.ts: 3
possible missing pattern applications: 3
uncaptured prose outside docs by file
  src/billing/rules.ts: 4
  src/admin/export.ts: 2
```

Read this as a signal list, not as a failure list. The report says the
repo has repeated billing/export shapes, a few pattern candidates, prose worth
reviewing deliberately, and implementation paths Special cannot yet connect to
current proof.

When `--metrics` has enough detail to make the next review step concrete, it
also prints cleanup targets:
top files, representative item names, and the structural move to consider for
each signal.

## Source Outside Architecture

`source outside architecture`
counts analyzable implementation that is not inside declared module ownership.

That usually means one of three things:

- the repo has useful code that needs an `@module` and `@implements`
- a module boundary is too narrow
- generated or fixture-heavy paths should be excluded from the relevant review

It does not prove the code is wrong or unused. It says the architecture graph
cannot explain that code yet. Use `special arch --unimplemented` when the next
move is to add or repair ownership.

## Untraced Implementation

`untraced implementation`
counts implementation that language-pack traceability cannot connect back to
current spec support.

This is most useful when you are trying to separate proven behavior from code
that is only exercised indirectly, manually, or not at all. It does not mean the
code is dead. It means Special cannot see a preferred proof path from a current
spec through a verifying test to that implementation.

Special intentionally does not treat process, command, route, or framework
boundaries as proof edges. If a command handler owns the only call into important
business logic, the healthy move is usually to move that logic behind a module
facade and test the facade directly. The
boundary rule
keeps outside-in command execution visible as a design smell instead of hiding
it as proof.

Use `special specs --unverified`, `special specs --verbose`, and scoped health
to decide whether the next move is proof, refactoring, or a better module
boundary.

## Duplicate Source Shapes

`duplicate source shapes`
counts owned implementation items whose concrete parser shape or normalized
source projection is substantively similar.

Concrete shape catches near-copy code. Normalized shape catches repeated ideas
that survive small syntax differences, such as label-to-field mappings:

```ts
const invoiceColumns = {
  "Invoice ID": invoice.id,
  "Customer": invoice.customerName,
  "Total": invoice.totalCents,
};

const refundColumns = {
  "Refund ID": refund.id,
  "Customer": refund.customerName,
  "Total": refund.totalCents,
};
```

That signal might mean:

- extract a helper because two implementations are doing the same job
- declare a real `@pattern` because repeated structure is intentional
- leave it alone because two small shapes are clearer when kept separate

Do not silence this by adding annotations. First decide whether the repeated
shape is accidental duplication, an intentional pattern, or acceptable local
parallelism. Use `special health --metrics --verbose --target PATH` for evidence
and `special patterns --metrics` when you are reviewing declared patterns.

## Pattern Signals

`possible pattern clusters`
are candidate repeated structures Special found before you named a pattern.
`possible missing pattern applications`
are places that look similar to an existing pattern application but do not yet
carry `@applies`.

Use these as review queues:

- a cluster can become a helper extraction, a new `@pattern`, or no action
- a missing application can become `@applies` only after the surrounding code
  actually follows the pattern
- a weak cluster can remain visible until enough real examples exist

Patterns are for repeated implementation or documentation structures. They are
not style rules or broad principles.

## Long Prose and Test Assertions

`uncaptured prose outside docs`
is an advisory review queue for substantial natural-language blocks outside
configured docs sources when the block has no docs evidence link, docs
annotation, or Special declaration. It is not a ban on comments. It catches
policy prose, workflow explanations, and copied documentation that may deserve a
deliberate home.

The right move depends on the prose:

- product-facing explanation usually belongs in generated docs source with
  `documents://` links
- maintainer-only explanation may belong in contributor docs
- short local implementation context can stay near the code
- useful source-local contract text can stay in `@spec`, `@module`, `@pattern`,
  or other Special declaration bodies
- obsolete prose should be deleted

`long prose test literals`
reports tests that embed long human prose directly in source. Prefer checking
the smallest contractual pieces of human output, exposing a structured result,
or moving large prose samples into a fixture when the full prose is the test
subject.

## A Health-First Existing-Repo Pass

Start broad:

```sh
special init
special health --metrics
```

Choose one file cluster from the output. If `src/billing/invoices.ts` has five
duplicate shapes and a few untraced items, inspect just that slice:

```sh
special health --metrics --verbose --target src/billing
special patterns --metrics --target src/billing
special arch --unimplemented
```

Then make one durable improvement:

- add or adjust a module if billing code is outside architecture
- extract a shared billing export helper if duplicate shapes are accidental
- define `@pattern BILLING.TABLE_EXPORT` if the repeated export structure is
  intentional across invoices, refunds, and adjustments
- add a spec and direct test only when the behavior is a real product claim
- move reader-facing billing rules into generated docs source and link them to
  the relevant specs or modules

Run the same scoped health command again. The goal is not to make every number
zero. The goal is to connect the important facts and make the remaining signals
explainable.

## Scoping and Output Modes

Use scoping to keep health useful while you work:

```sh
special health --metrics --target src/billing
special health --metrics --within src/billing
special health --metrics --target src/billing/export.ts --symbol exportInvoices
special health --metrics --verbose --target src/billing
special health --json --metrics --target src/billing
special health --html --metrics --target src/billing
```

`--target` narrows the current
view. `--within` narrows the
analysis corpus. `--symbol`
inspects one item in one target file. Use
`--verbose` when you need item
names and evidence, `--json` when
a script or self-check needs stable data, and
`--html` when the review benefits
from a browsable report.

Docs coverage is explicit relationship accounting, so it belongs to
`special docs --metrics`.
Health can still report prose that is outside docs, because that prose is not
yet part of the docs graph.
