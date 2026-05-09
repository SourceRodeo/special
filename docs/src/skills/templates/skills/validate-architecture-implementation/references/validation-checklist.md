@filedocuments spec SPECIAL.TRACE_COMMAND.ARCH

# Architecture Validation Checklist
@implements SPECIAL.DOCUMENTATION.SKILLS.FALLBACK.VALIDATE_ARCHITECTURE_IMPLEMENTATION
@applies DOCS.SKILL_SUPPORT_REFERENCE

- Read the exact [`@module`](documents://spec/SPECIAL.MODULE_COMMAND.MARKDOWN_DECLARATIONS) text before judging the code that claims to implement it.
- Compare attached [`@implements`](documents://spec/SPECIAL.MODULE_PARSE.IMPLEMENTS.ITEM_SCOPE) bodies to the module's stated responsibility, not to nearby files or names.
- Use [`special arch MODULE.ID --metrics --verbose`](documents://spec/SPECIAL.MODULE_COMMAND.METRICS) when you need architecture evidence beyond direct ownership tracing.
- If the module lists pattern applications, read [`special patterns PATTERN.ID --verbose`](documents://spec/SPECIAL.PATTERNS.VERBOSE) before judging whether the implementation approach is intentional.
- Treat uncovered files, weak coverage, isolated items, outbound-heavy items, and complexity hotspots as inspection cues, not automatic violations.
- Treat `@area` as structure-only. A structural node should not need direct implementation.
- Prefer direct ownership. A current `@module` without direct `@implements` is architecture drift unless it is explicitly `@planned`.
- Treat `@fileimplements` as honest but coarse ownership. Use item signals to decide whether the file likely hides functions that deserve narrower attachment.
- If multiple files implement one module, decide whether that split is intentional or a smell worth refactoring.
- Keep [`@applies`](documents://spec/SPECIAL.PATTERNS.SOURCE_APPLICATIONS) and [`@implements`](documents://spec/SPECIAL.MODULE_PARSE.IMPLEMENTS.ITEM_SCOPE) distinct: pattern application explains approach; module implementation explains ownership.
- Do not use architecture validation to prove product behavior. Switch to [`special specs`](documents://spec/SPECIAL.SPEC_COMMAND) when the question is what the product ships.
