# Language Packs

Language packs are admitted only when they can satisfy the shared Special
interface. They are not just syntax highlighters. A pack must declare its
registration, parser surface, traceability behavior, and degradation behavior:
registration,
parser surface,
traceability,
and degradation.

## Built-In Pack Bar

Rust, TypeScript, Go, and Python share the same high-level bar: parse source
items, identify test/review surfaces, build traceability inputs where supported,
and report unavailable or degraded analysis honestly. Distribution tests keep
that bar visible through
tooling or parser boundaries
and the built-in language-pack admission checks.

## Registry Layout

Top-level files in `src/language_packs` are built-in pack entries. Keep entry
files named after the language, such as `rust.rs` or `python.rs`, and give each
entry a `DESCRIPTOR`. Put helpers, fixtures, and implementation modules under a
language or shared subdirectory. The build script follows that
top-level entry layout
and rejects accidental top-level helper files with a layout error.

## Scoped Analysis

Scoped health must not become a shallow path filter. For supported packs,
`special health --target` uses scoped graph discovery and compares scoped output
against full-then-filtered traceability:
target traceability,
scoped graph discovery,
no eager fact blobs,
and language parity.

## Pack-Specific Edges

Maintainers should add edge support only when the language feature is real and
testable, not just because Special happens to need one case. The current
contracts include TypeScript tool edges,
reference edges,
event callbacks,
forwarded callbacks,
hook callbacks,
effect callbacks,
and context callbacks.
Go carries tool edges
and reference edges.
Python must surface parse failures
instead of silently succeeding.
