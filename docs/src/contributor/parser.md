@applies DOCS.CONTRIBUTOR_RUNBOOK_PAGE
@implements SPECIAL.DOCUMENTATION.CONTRIBUTOR.PARSER
# Parser and Annotation Rules

The parser is the contract boundary for source-native Special claims. It must
recognize reserved annotations only when they use the exact directive shape,
while leaving unrelated `@` text alone. The base parser contract covers
[reserved tag shape](documents://spec/SPECIAL.PARSE.RESERVED_TAGS.REQUIRE_DIRECTIVE_SHAPE),
[foreign tag boundaries](documents://spec/SPECIAL.PARSE.FOREIGN_TAG_BOUNDARIES),
and [multi-file trees](documents://spec/SPECIAL.PARSE.MULTI_FILE_TREE).

## Source Comments

Special extracts supported comment blocks before interpreting annotations.
Maintainers changing extractor behavior should preserve line comments,
block comments, shell comments, Python comments, Go comments, and TypeScript
comments as separate supported inputs:
[line comments](documents://spec/SPECIAL.PARSE.LINE_COMMENTS),
[block comments](documents://spec/SPECIAL.PARSE.BLOCK_COMMENTS),
[Go line comments](documents://spec/SPECIAL.PARSE.GO_LINE_COMMENTS),
[TypeScript line comments](documents://spec/SPECIAL.PARSE.TYPESCRIPT_LINE_COMMENTS),
[TypeScript block comments](documents://spec/SPECIAL.PARSE.TYPESCRIPT_BLOCK_COMMENTS),
[shell comments](documents://spec/SPECIAL.PARSE.SHELL_COMMENTS), and
[Python line comments](documents://spec/SPECIAL.PARSE.PYTHON_LINE_COMMENTS).

## Lifecycle Markers

`@planned` and `@deprecated` are lifecycle markers, not free-form prose. In
current `special.toml` version 1 parsing, adjacent `@planned` markers attach to
the declaration they sit next to, including the
[inline form](documents://spec/SPECIAL.PARSE.PLANNED.ADJACENT_V1.INLINE) and
[next-line form](documents://spec/SPECIAL.PARSE.PLANNED.ADJACENT_V1.NEXT_LINE).
Duplicate, fuzzy, or backward marker forms should stay rejected through
[duplicate marker rejection](documents://spec/SPECIAL.PARSE.PLANNED.ADJACENT_V1.REJECTS_DUPLICATE_MARKERS)
and [backward-form rejection](documents://spec/SPECIAL.PARSE.PLANNED.ADJACENT_V1.REJECTS_BACKWARD_FORM).
Release metadata may follow the marker, but identifier-shaped suffixes are not
release metadata and are rejected by
[identifier-suffix rejection](documents://spec/SPECIAL.PARSE.PLANNED.ADJACENT_V1.REJECTS_IDENTIFIER_SUFFIX).

## Evidence Attachments

`@verifies` and `@attests` are proof attachments. Keep the parser strict:
`@verifies` allows [one reference per block](documents://spec/SPECIAL.PARSE.VERIFIES.ONE_PER_BLOCK),
supports [file scope](documents://spec/SPECIAL.PARSE.VERIFIES.FILE_SCOPE), and
only attached support counts toward current spec support through
[attached-support semantics](documents://spec/SPECIAL.PARSE.VERIFIES.ONLY_ATTACHED_SUPPORT_COUNTS).
Attestations require structured metadata, including
[required fields](documents://spec/SPECIAL.PARSE.ATTESTS.REQUIRED_FIELDS),
[allowed fields](documents://spec/SPECIAL.PARSE.ATTESTS.ALLOWED_FIELDS),
[ISO dates](documents://spec/SPECIAL.PARSE.ATTESTS.DATE_FORMAT), and
[positive review intervals](documents://spec/SPECIAL.PARSE.ATTESTS.REVIEW_INTERVAL_DAYS).

## Markdown Architecture

Markdown docs can own architecture and apply patterns, but current modules are
not strawman declarations. Areas are structural, while current modules require
real ownership unless marked planned:
[module declarations](documents://spec/SPECIAL.MODULE_PARSE.MODULE_DECLARATIONS),
[area declarations](documents://spec/SPECIAL.MODULE_PARSE.AREA_DECLARATIONS),
[planned modules only](documents://spec/SPECIAL.MODULE_PARSE.PLANNED.MODULE_ONLY),
[current modules require implementation](documents://spec/SPECIAL.MODULE_PARSE.CURRENT_MODULES_REQUIRE_IMPLEMENTATION),
and [areas remain structural](documents://spec/SPECIAL.MODULE_PARSE.AREAS_ARE_STRUCTURAL_ONLY).

Markdown ownership and pattern attachments use heading-bounded bodies:
[markdown implements](documents://spec/SPECIAL.MODULE_PARSE.IMPLEMENTS.MARKDOWN_SCOPE)
and [markdown pattern applications](documents://spec/SPECIAL.PATTERNS.MARKDOWN_APPLICATIONS).
