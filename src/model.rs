/**
@module SPECIAL.MODEL
Canonical Rust domain types grouped by core primitives, spec modeling, architecture modeling, analysis summaries, and rendered report documents.
*/
// @fileimplements SPECIAL.MODEL
mod analysis;
mod architecture;
mod core;
mod overview;
mod spec;

pub use analysis::{
    ArchitectureAnalysisSummary, ArchitectureDuplicateItem, ArchitectureLongExactProseAssertion,
    ArchitectureLongProseBlock, ArchitectureRepoSignalsSummary, ArchitectureTraceabilityItem,
    ArchitectureTraceabilitySummary, ArchitectureUnownedItem, ModuleAnalysisOptions,
    ModuleAnalysisSummary, ModuleComplexitySummary, ModuleCouplingSummary, ModuleCoverageSummary,
    ModuleDependencySummary, ModuleDependencyTargetSummary, ModuleItemKind, ModuleItemSignal,
    ModuleItemSignalsSummary, ModuleMetricsSummary, ModuleQualitySummary, ModuleTraceabilityItem,
    ModuleTraceabilitySummary,
};
pub use architecture::{
    ImplementRef, ModuleDecl, ModuleFilter, ModuleNode, ParsedArchitecture, PatternApplication,
    PatternApplicationNode, PatternBenchmarkEstimate, PatternCandidateConfidence,
    PatternClusterCandidate, PatternClusterInterpretation, PatternClusterItem, PatternDefinition,
    PatternDocument, PatternFilter, PatternMetricsSummary, PatternMissingApplicationCandidate,
    PatternModuleRef, PatternNode, PatternSimilarityMetrics, PatternStrictness,
};
pub use core::{
    ArchitectureKind, BlockLine, CommentBlock, DeclaredStateFilter, DeprecatedRelease,
    DiagnosticSeverity, ModelInvariantError, NodeKind, OwnedItem, PlanState, PlannedRelease,
    SourceLocation,
};
pub use overview::{
    ArchitectureMetricsSummary, DocumentationCoverageSummary, DocumentationTargetCoverage,
    GroupedCount, LintReport, ModuleDocument, OVERVIEW_LOOK_NEXT_COMMANDS, OverviewArchSummary,
    OverviewDocument, OverviewHealthSummary, OverviewLintSummary, OverviewSpecsSummary,
    RepoArchitectureHealthMetrics, RepoDocsHealthMetrics, RepoDocument, RepoMetricsSummary,
    RepoPatternHealthMetrics, RepoSpecHealthMetrics, RepoTestHealthMetrics,
    RepoTraceabilityMetrics, SpecDocument, SpecMetricsSummary, grouped_count_map, grouped_counts,
};
pub use spec::{
    AttestRef, AttestScope, Diagnostic, ParsedRepo, SpecDecl, SpecFilter, SpecNode, VerifyRef,
};
