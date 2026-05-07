@applies DOCS.CONCEPTUAL_OVERVIEW_PAGE
# Concepts

@implements SPECIAL.DOCUMENTATION.PUBLIC.CONCEPTS.COGNITIVE_DEBT
@applies DOCS.SURFACE_OVERVIEW_PAGE
## Cognitive Debt

Special is about making repo work easier to understand before a change and
easier to review after a change. It does that with two complementary jobs:

1. It scans the repo for signals worth reviewing: source outside declared
   ownership, implementation with no visible proof path, repeated source shapes,
   possible missing pattern applications, long prose outside docs, exact prose
   assertions in tests, and changed relationships.
2. It lets you connect the important facts directly in source: product claims,
   tests and attestations, architecture ownership, adopted implementation
   patterns, and docs claims.

Cognitive debt shows up when a repo has behavior, tests, architecture, repeated
implementation structures, and docs, but nobody can quickly say how they fit
together. Special does not replace tests, docs, or review. It gives those
existing artifacts small source-level connections that can be inspected.

For example, a CSV export feature can have one spec for the header behavior, one
test that verifies it, one module that owns the export code, one pattern for
label-to-value column maps, one docs sentence linked to the spec, and one health
report showing whether anything around that slice is still unexplained. Each
piece stays small, but the repo can answer why the code exists, what proves it,
and what changed relationships need review.

The adoption path depends on what kind of repo you have. In a new project, write
specs and architecture as the behavior appears, then use docs and health to keep
the explanation connected. In an existing project, run health and pattern metrics
first, then annotate the small parts that are worth making durable.

@implements SPECIAL.DOCUMENTATION.PUBLIC.CONCEPTS.SPECS
## Specs

[`Specs`](documents://spec/SPECIAL.SPEC_COMMAND) are
durable product claims. A spec says what the repo currently promises, plans, or
deprecates. A claim becomes useful when it has direct evidence through
[`@verifies`](documents://spec/SPECIAL.PARSE.VERIFIES) or
[`@attests`](documents://spec/SPECIAL.PARSE.ATTESTS).

Question answered: what does this repo claim, and what supports that claim?

@implements SPECIAL.DOCUMENTATION.PUBLIC.CONCEPTS.ARCH
## Arch

[`Arch`](documents://spec/SPECIAL.MODULE_COMMAND) declares
implementation ownership. Areas organize the tree. Modules own real code or
planned architecture intent. Ownership attachments make the boundary inspectable
instead of relying on directory names alone.

Question answered: where should this implementation live, and what owns it?

@implements SPECIAL.DOCUMENTATION.PUBLIC.CONCEPTS.PATTERNS
## Patterns

[`Patterns`](documents://spec/SPECIAL.PATTERNS.COMMAND)
name repeated implementation structures that the project intentionally uses.
They are not writing principles or styleguide slogans. A useful pattern has a
recognizable implementation shape and concrete
[applications](documents://spec/SPECIAL.PATTERNS.SOURCE_APPLICATIONS).

Question answered: which repeated structures are intentional, and where are they
applied?

@implements SPECIAL.DOCUMENTATION.PUBLIC.CONCEPTS.DOCS
## Docs

[`Docs`](documents://spec/SPECIAL.DOCS_COMMAND) are
generated reader surfaces authored from traceable source markdown. A
[`documents://`](documents://spec/SPECIAL.DOCS.LINKS.POLYMORPHIC) link can
target a spec, group, module, area, or pattern. In generated output,
[`special docs build` strips the authoring link](documents://spec/SPECIAL.DOCS.LINKS.OUTPUT).

Question answered: where is this product surface explained to readers?

@implements SPECIAL.DOCUMENTATION.PUBLIC.CONCEPTS.HEALTH
## Health

[`Health`](documents://spec/SPECIAL.HEALTH_COMMAND) joins
the other surfaces. It reports raw inferred queues such as source outside
architecture, untraced implementation, duplicate source shapes, possible missing
pattern applications, and long prose outside docs.

Question answered: what part of the repo is still hard to explain?

Health is not a replacement for the other commands. It is the command that tells
you which surface to use next.
