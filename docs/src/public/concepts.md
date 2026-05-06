@applies DOCS.CONCEPTUAL_OVERVIEW_PAGE
# Concepts

@implements SPECIAL.DOCUMENTATION.PUBLIC.CONCEPTS.COGNITIVE_DEBT
@applies DOCS.SURFACE_OVERVIEW_PAGE
## Cognitive Debt

Special is about reducing cognitive debt inside a repository. Cognitive debt
shows up when the repo has behavior, tests, architecture, repeated
implementation structures, and docs, but nobody can quickly connect them.

Special does not replace tests, docs, or review. It adds a repo-native graph over
them so the important connections can be inspected.

@implements SPECIAL.DOCUMENTATION.PUBLIC.CONCEPTS.SPECS
## Specs

Specs are
durable product claims. A spec says what the repo currently promises, plans, or
deprecates. A claim becomes useful when it has direct evidence through
[`@verifies`](documents://spec/SPECIAL.PARSE.VERIFIES) or
[`@attests`](documents://spec/SPECIAL.PARSE.ATTESTS).

Question answered: what does this repo claim, and what supports that claim?

@implements SPECIAL.DOCUMENTATION.PUBLIC.CONCEPTS.ARCH
## Arch

Arch declares
implementation ownership. Areas organize the tree. Modules own real code or
planned architecture intent. Ownership attachments make the boundary inspectable
instead of relying on directory names alone.

Question answered: where should this implementation live, and what owns it?

@implements SPECIAL.DOCUMENTATION.PUBLIC.CONCEPTS.PATTERNS
## Patterns

Patterns
name repeated implementation structures that the project intentionally uses.
They are not writing principles or styleguide slogans. A useful pattern has a
recognizable implementation shape and concrete applications.

Question answered: which repeated structures are intentional, and where are they
applied?

@implements SPECIAL.DOCUMENTATION.PUBLIC.CONCEPTS.DOCS
## Docs

Docs are
generated reader surfaces authored from traceable source markdown. A
`documents://` link connects a docs claim to the smallest relevant Special id,
then `special docs build` strips the authoring link from generated output.

Question answered: where is this product surface explained to readers?

@implements SPECIAL.DOCUMENTATION.PUBLIC.CONCEPTS.HEALTH
## Health

Health joins
the other surfaces. It reports repo-wide signals such as unowned implementation,
unsupported implementation, traceability, duplication, and documentation
coverage.

Question answered: what part of the repo is still hard to explain?
