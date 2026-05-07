# Parser and Annotation Rules

The parser is the contract boundary for source-native Special claims. It must
recognize reserved annotations only when they use the exact directive shape,
while leaving unrelated `@` text alone. The base parser contract covers
reserved tag shape,
foreign tag boundaries,
and multi-file trees.

## Source Comments

Special extracts supported comment blocks before interpreting annotations.
Maintainers changing extractor behavior should preserve line comments,
block comments, shell comments, Python comments, Go comments, and TypeScript
comments as separate supported inputs:
line comments,
block comments,
Go line comments,
TypeScript line comments,
TypeScript block comments,
shell comments, and
Python line comments.

## Lifecycle Markers

`@planned` and `@deprecated` are lifecycle markers, not free-form prose. In
current `special.toml` version 1 parsing, adjacent `@planned` markers attach to
the declaration they sit next to, including the
inline form and
next-line form.
Duplicate, fuzzy, or backward marker forms should stay rejected through
duplicate marker rejection
and backward-form rejection.

## Evidence Attachments

`@verifies` and `@attests` are proof attachments. Keep the parser strict:
`@verifies` allows one reference per block,
supports file scope, and
only attached support counts toward current spec support through
attached-support semantics.
Attestations require structured metadata, including
required fields,
allowed fields,
ISO dates, and
positive review intervals.

## Markdown Architecture

Markdown docs can own architecture and apply patterns, but current modules are
not strawman declarations. Areas are structural, while current modules require
real ownership unless marked planned:
module declarations,
area declarations,
planned modules only,
current modules require implementation,
and areas remain structural.

Markdown ownership and pattern attachments use heading-bounded bodies:
markdown implements
and markdown pattern applications.

