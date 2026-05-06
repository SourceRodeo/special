# Concepts

## Cognitive Debt

Special is about reducing cognitive debt inside a repository. Cognitive debt
shows up when the repo has behavior, tests, architecture, repeated
implementation structures, and docs, but nobody can quickly connect them.

Special does not replace tests, docs, or review. It adds a repo-native graph over
them so the important connections can be inspected.

The adoption path depends on what kind of repo you have. In a new project, write
specs and architecture as the behavior appears, then use docs and health to keep
the explanation connected. In an existing project, run health and pattern metrics
first, then annotate the small parts that are worth making durable.

## Specs

Specs are
durable product claims. A spec says what the repo currently promises, plans, or
deprecates. A claim becomes useful when it has direct evidence through
`@verifies` or
`@attests`.

Question answered: what does this repo claim, and what supports that claim?

## Arch

Arch declares
implementation ownership. Areas organize the tree. Modules own real code or
planned architecture intent. Ownership attachments make the boundary inspectable
instead of relying on directory names alone.

Question answered: where should this implementation live, and what owns it?

## Patterns

Patterns
name repeated implementation structures that the project intentionally uses.
They are not writing principles or styleguide slogans. A useful pattern has a
recognizable implementation shape and concrete applications.

Question answered: which repeated structures are intentional, and where are they
applied?

## Docs

Docs are
generated reader surfaces authored from traceable source markdown. A
`documents://` link connects a docs claim to the smallest relevant Special id,
then `special docs build` strips the authoring link from generated output.

Question answered: where is this product surface explained to readers?

## Health

Health joins
the other surfaces. It reports repo-wide signals such as unowned implementation,
unsupported implementation, traceability, duplication, and documentation
coverage.

Question answered: what part of the repo is still hard to explain?

Health is not a replacement for the other commands. It is the command that tells
you which surface to use next.
