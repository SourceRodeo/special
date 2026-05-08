/**
@module SPECIAL.MODULES.ANALYZE.EXPLAIN
Defines shared plain-language and exact explanation text for architecture analysis metrics so renderers can present the same meaning across language providers.
*/
// @fileimplements SPECIAL.MODULES.ANALYZE.EXPLAIN
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub(crate) enum MetricExplanationKey {
    CyclomaticTotal,
    CyclomaticMax,
    CognitiveTotal,
    CognitiveMax,
    QualityPublicFunctions,
    QualityParameters,
    QualityBoolParameters,
    QualityRawStringParameters,
    QualityPanicSites,
    UnownedItems,
    ConnectedItem,
    OutboundHeavyItem,
    IsolatedItem,
    UnreachedItem,
    UnreachedItems,
    DuplicateItems,
    UntracedImplementation,
    PossiblePatternClusters,
    PossibleMissingPatternApplications,
    LongProseOutsideDocs,
    LongExactProseAssertions,
    HighestComplexityItem,
    ParameterHeavyItem,
    StringlyBoundaryItem,
    PanicHeavyItem,
    FanIn,
    FanOut,
    AfferentCoupling,
    EfferentCoupling,
    Instability,
}

#[derive(Debug, Clone, Copy)]
pub(crate) struct MetricExplanation {
    pub plain: &'static str,
    pub precise: &'static str,
}

pub(crate) fn metric_explanation(key: MetricExplanationKey) -> MetricExplanation {
    // Keep this exhaustive instead of indexing a parallel table: metric labels
    // are user-facing enough that enum reordering must not remap explanations.
    match key {
        MetricExplanationKey::CyclomaticTotal => explanation(
            "this is the combined branchiness across owned functions and methods.",
            "sum of per-item cyclomatic complexity across analyzed owned implementation.",
        ),
        MetricExplanationKey::CyclomaticMax => explanation(
            "this is the branchiest single owned function or method.",
            "maximum per-item cyclomatic complexity across analyzed owned implementation.",
        ),
        MetricExplanationKey::CognitiveTotal => explanation(
            "this is the combined reading and control-flow burden across owned functions and methods.",
            "sum of per-item cognitive complexity across analyzed owned implementation.",
        ),
        MetricExplanationKey::CognitiveMax => explanation(
            "this is the hardest single owned function or method to follow structurally.",
            "maximum per-item cognitive complexity across analyzed owned implementation.",
        ),
        MetricExplanationKey::QualityPublicFunctions => explanation(
            "this counts public entrypoints whose shape affects how others use this module.",
            "count of analyzed owned public functions and methods.",
        ),
        MetricExplanationKey::QualityParameters => explanation(
            "this shows how much raw argument surface the module exposes through public APIs.",
            "sum of typed parameters across analyzed owned public functions and methods.",
        ),
        MetricExplanationKey::QualityBoolParameters => explanation(
            "this highlights public APIs that steer behavior with on or off flags.",
            "count of public parameters typed as bool across analyzed owned implementation.",
        ),
        MetricExplanationKey::QualityRawStringParameters => explanation(
            "this highlights public APIs that pass around raw string values instead of narrower named types.",
            "count of public parameters typed as String, str, or Cow<str> across analyzed owned implementation.",
        ),
        MetricExplanationKey::QualityPanicSites => explanation(
            "this highlights places where recoverable runtime paths can still crash or abort abruptly.",
            "count of panic-like macros and unwrap/expect method calls across analyzed owned implementation.",
        ),
        MetricExplanationKey::UnownedItems => explanation(
            "this counts analyzable code items outside declared modules, so the file or item lacks explicit architecture ownership.",
            "count of analyzable source items with no declared owning module ids.",
        ),
        MetricExplanationKey::ConnectedItem => explanation(
            "this item is meaningfully tied into the rest of the module's owned implementation.",
            "owned item with inbound or outbound references to other owned items in the same module.",
        ),
        MetricExplanationKey::OutboundHeavyItem => explanation(
            "this item reaches outward more than it talks to the rest of its own module.",
            "owned item whose external references exceed its outbound references to sibling owned items.",
        ),
        MetricExplanationKey::IsolatedItem => explanation(
            "this item talks outward without much visible relationship to the rest of its own module.",
            "owned item with zero inbound and outbound sibling references but at least one external reference.",
        ),
        MetricExplanationKey::UnreachedItem => explanation(
            "this private item has no observed path from public or test roots inside the analyzed implementation, so it may be trapped, incidental, or unused.",
            "non-public, non-test owned item not reachable from public or test items through observed sibling call edges.",
        ),
        MetricExplanationKey::UnreachedItems => explanation(
            "this counts private owned items with no observed path from public or test roots, so they may be trapped, incidental, framework-driven, or unused.",
            "count of non-public, non-test owned items not reachable from public or test items through observed sibling call edges.",
        ),
        MetricExplanationKey::DuplicateItems => explanation(
            "this counts owned items whose parser-normalized structure and substantive operation profile match another owned item, so they may indicate repeated implementation logic worth consolidating or reviewing.",
            "count of analyzed owned items whose structural fingerprint and substantive call/control-flow profile match at least one other analyzed owned item.",
        ),
        MetricExplanationKey::UntracedImplementation => explanation(
            "this counts implementation Special can see but cannot connect to a current spec, static mediation, or test-backed support path.",
            "count of analyzed implementation items in the unexplained traceability bucket.",
        ),
        MetricExplanationKey::PossiblePatternClusters => explanation(
            "this counts repeated structural groups that may deserve a named pattern or an intentional no-pattern decision.",
            "count of statistically similar source-shape clusters surfaced by pattern analysis.",
        ),
        MetricExplanationKey::PossibleMissingPatternApplications => explanation(
            "this counts places whose structure resembles a defined pattern but lacks an explicit pattern application.",
            "count of source or docs bodies above the configured pattern-similarity threshold without a matching applied pattern.",
        ),
        MetricExplanationKey::LongProseOutsideDocs => explanation(
            "this highlights substantial prose outside configured docs sources and without docs evidence, so explanatory text can be promoted, linked, or removed deliberately.",
            "count of long natural-language blocks outside configured docs source/output paths that pass the prose-shape filter and do not contain `documents://`, `@documents`, or `@filedocuments` evidence.",
        ),
        MetricExplanationKey::LongExactProseAssertions => explanation(
            "this highlights tests that pin long prose to exact string matching instead of checking smaller semantic pieces.",
            "count of long human-prose string literals used as exact assertion targets in recognized test or fixture source files.",
        ),
        MetricExplanationKey::HighestComplexityItem => explanation(
            "this is one of the most structurally complex owned items inside the module boundary.",
            "owned item ranked by cognitive complexity, then cyclomatic complexity.",
        ),
        MetricExplanationKey::ParameterHeavyItem => explanation(
            "this item carries a relatively wide argument surface compared with its peers.",
            "owned item ranked by parameter count, then raw string parameter count.",
        ),
        MetricExplanationKey::StringlyBoundaryItem => explanation(
            "this public item exposes raw string values at the module boundary.",
            "owned public item ranked by raw string parameter count, then total parameter count.",
        ),
        MetricExplanationKey::PanicHeavyItem => explanation(
            "this item contains more crash-prone runtime sites than its peers.",
            "owned item ranked by panic-like site count, then cognitive complexity.",
        ),
        MetricExplanationKey::FanIn => explanation(
            "other owned modules reach into this module.",
            "distinct inbound concrete-module dependencies resolved from owned code.",
        ),
        MetricExplanationKey::FanOut => explanation(
            "this module reaches into other owned modules.",
            "distinct outbound concrete-module dependencies resolved from owned code.",
        ),
        MetricExplanationKey::AfferentCoupling => explanation(
            "other owned modules depend on this module.",
            "count of inbound concrete-module dependencies.",
        ),
        MetricExplanationKey::EfferentCoupling => explanation(
            "this module depends on other owned modules.",
            "count of outbound concrete-module dependencies.",
        ),
        MetricExplanationKey::Instability => explanation(
            "this shows whether the module leans more on others than others lean on it.",
            "efferent coupling / (afferent coupling + efferent coupling).",
        ),
    }
}

fn explanation(plain: &'static str, precise: &'static str) -> MetricExplanation {
    MetricExplanation { plain, precise }
}
