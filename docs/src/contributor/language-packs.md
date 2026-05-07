@applies DOCS.CONTRIBUTOR_RUNBOOK_PAGE
@implements SPECIAL.DOCUMENTATION.CONTRIBUTOR.LANGUAGE_PACKS
# Language Packs

Language packs are admitted only when they can satisfy the shared Special
interface. They are not just syntax highlighters. A pack must declare its
registration, parser surface, traceability behavior, and degradation behavior:
[registration](documents://spec/SPECIAL.LANGUAGE_PACKS.ADMISSION.REGISTRATION),
[parser surface](documents://spec/SPECIAL.LANGUAGE_PACKS.ADMISSION.PARSER_SURFACE),
[traceability](documents://spec/SPECIAL.LANGUAGE_PACKS.ADMISSION.TRACEABILITY),
and [degradation](documents://spec/SPECIAL.LANGUAGE_PACKS.ADMISSION.DEGRADATION).

## Built-In Pack Bar

Rust, TypeScript, Go, and Python share the same high-level bar: parse source
items, identify test/review surfaces, build traceability inputs where supported,
and report unavailable or degraded analysis honestly. Distribution tests keep
that bar visible through
[tooling or parser boundaries](documents://spec/SPECIAL.DISTRIBUTION.BUILD_SCRIPT.LANGUAGE_PACK_REGISTRY)
and the built-in language-pack admission checks.

## Scoped Analysis

Scoped health must not become a shallow path filter. For supported packs,
`special health --target` uses scoped graph discovery and compares scoped output
against full-then-filtered traceability:
[target traceability](documents://spec/SPECIAL.HEALTH_COMMAND.TARGET.TRACEABILITY),
[scoped graph discovery](documents://spec/SPECIAL.HEALTH_COMMAND.TARGET.TRACEABILITY.SCOPED_GRAPH_DISCOVERY),
[no eager fact blobs](documents://spec/SPECIAL.HEALTH_COMMAND.TARGET.TRACEABILITY.NO_EAGER_FACT_BLOBS),
and [language parity](documents://spec/SPECIAL.HEALTH_COMMAND.TARGET.TRACEABILITY.LANGUAGE_PARITY).

## Pack-Specific Edges

Maintainers should add edge support only when the language feature is real and
testable, not just because Special happens to need one case. The current
contracts include TypeScript [tool edges](documents://spec/SPECIAL.HEALTH_COMMAND.TRACEABILITY.TYPESCRIPT.TOOL_EDGES),
[reference edges](documents://spec/SPECIAL.HEALTH_COMMAND.TRACEABILITY.TYPESCRIPT.REFERENCE_EDGES),
[event callbacks](documents://spec/SPECIAL.HEALTH_COMMAND.TRACEABILITY.TYPESCRIPT.EVENT_CALLBACK_EDGES),
[forwarded callbacks](documents://spec/SPECIAL.HEALTH_COMMAND.TRACEABILITY.TYPESCRIPT.FORWARDED_CALLBACK_EDGES),
[hook callbacks](documents://spec/SPECIAL.HEALTH_COMMAND.TRACEABILITY.TYPESCRIPT.HOOK_CALLBACK_EDGES),
[effect callbacks](documents://spec/SPECIAL.HEALTH_COMMAND.TRACEABILITY.TYPESCRIPT.EFFECT_CALLBACK_EDGES),
and [context callbacks](documents://spec/SPECIAL.HEALTH_COMMAND.TRACEABILITY.TYPESCRIPT.CONTEXT_CALLBACK_EDGES).
Go carries [tool edges](documents://spec/SPECIAL.HEALTH_COMMAND.TRACEABILITY.GO.TOOL_EDGES)
and [reference edges](documents://spec/SPECIAL.HEALTH_COMMAND.TRACEABILITY.GO.REFERENCE_EDGES).
Python must surface [parse failures](documents://spec/SPECIAL.HEALTH_COMMAND.TRACEABILITY.PYTHON.PARSE_FAILURE)
instead of silently succeeding.

