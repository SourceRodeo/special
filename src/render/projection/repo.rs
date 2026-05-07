/**
@module SPECIAL.RENDER.PROJECTION.REPO
Projects repo-wide health, signals, and traceability documents into backend-ready verbose or non-verbose shapes.
*/
// @fileimplements SPECIAL.RENDER.PROJECTION.REPO
use crate::model::{
    ArchitectureRepoSignalsSummary, GroupedCount, RepoDocument, RepoMetricsSummary,
    RepoTraceabilityMetrics,
};

#[derive(Debug, Clone)]
pub(in crate::render) struct ProjectedRepoMetricSection {
    pub(in crate::render) title: &'static str,
    pub(in crate::render) counts: Vec<ProjectedRepoMetricCount>,
}

#[derive(Debug, Clone)]
pub(in crate::render) struct ProjectedRepoMetricCount {
    pub(in crate::render) label: String,
    pub(in crate::render) value: String,
}

pub(in crate::render) fn project_repo_document(
    document: &RepoDocument,
    verbose: bool,
) -> RepoDocument {
    project_repo_document_with_policy(document, verbose)
}

pub(in crate::render) fn project_repo_document_json(
    document: &RepoDocument,
    verbose: bool,
) -> RepoDocument {
    project_repo_document_with_policy(document, verbose)
}

pub(in crate::render) fn project_repo_health_metric_sections(
    metrics: &RepoMetricsSummary,
) -> Vec<ProjectedRepoMetricSection> {
    let mut sections = vec![ProjectedRepoMetricSection {
        title: "summary",
        counts: vec![
            metric_count(
                "source outside architecture",
                metrics.architecture.source_outside_architecture,
            ),
            metric_count(
                "untraced implementation",
                metrics.specs.untraced_implementation,
            ),
            metric_count(
                "duplicate source shapes",
                metrics.patterns.duplicate_source_shapes,
            ),
            metric_count(
                "possible pattern clusters",
                metrics.patterns.possible_pattern_clusters,
            ),
            metric_count(
                "possible missing pattern applications",
                metrics.patterns.possible_missing_applications,
            ),
            metric_count(
                "long prose outside docs",
                metrics.docs.long_prose_outside_docs,
            ),
            metric_count(
                "exact long-prose test assertions",
                metrics.tests.exact_long_prose_assertions,
            ),
        ],
    }];

    push_grouped_metric_section(
        &mut sections,
        "source outside architecture by file",
        &metrics.architecture.source_outside_architecture_by_file,
    );
    push_grouped_metric_section(
        &mut sections,
        "untraced implementation by file",
        &metrics.specs.untraced_implementation_by_file,
    );
    push_grouped_metric_section(
        &mut sections,
        "untraced review-surface implementation by file",
        &metrics.specs.untraced_review_surface_by_file,
    );
    push_grouped_metric_section(
        &mut sections,
        "duplicate source shapes by file",
        &metrics.patterns.duplicate_source_shapes_by_file,
    );
    push_grouped_metric_section(
        &mut sections,
        "possible missing pattern applications by file",
        &metrics.patterns.possible_missing_applications_by_file,
    );
    push_grouped_metric_section(
        &mut sections,
        "long prose outside docs by file",
        &metrics.docs.long_prose_outside_docs_by_file,
    );
    push_grouped_metric_section(
        &mut sections,
        "exact long-prose test assertions by file",
        &metrics.tests.exact_long_prose_assertions_by_file,
    );
    if let Some(traceability) = &metrics.traceability {
        sections.push(project_repo_traceability_metric_section(traceability));
    }

    sections
}

fn push_grouped_metric_section(
    sections: &mut Vec<ProjectedRepoMetricSection>,
    title: &'static str,
    counts: &[GroupedCount],
) {
    if counts.is_empty() {
        return;
    }
    sections.push(ProjectedRepoMetricSection {
        title,
        counts: counts
            .iter()
            .map(|count| ProjectedRepoMetricCount {
                label: count.value.clone(),
                value: count.count.to_string(),
            })
            .collect(),
    });
}

fn project_repo_traceability_metric_section(
    metrics: &RepoTraceabilityMetrics,
) -> ProjectedRepoMetricSection {
    ProjectedRepoMetricSection {
        title: "context",
        counts: vec![
            metric_count("analyzed implementation items", metrics.analyzed_items),
            metric_count(
                "current-spec traced implementation",
                metrics.current_spec_items,
            ),
            metric_count(
                "statically mediated implementation",
                metrics.statically_mediated_items,
            ),
            metric_count(
                "test-covered unlinked implementation",
                metrics.unverified_test_items,
            ),
            metric_count("untraced implementation", metrics.unexplained_items),
            metric_count(
                "untraced review-surface implementation",
                metrics.unexplained_review_surface_items,
            ),
            metric_count(
                "untraced public implementation",
                metrics.unexplained_public_items,
            ),
            metric_count(
                "untraced internal implementation",
                metrics.unexplained_internal_items,
            ),
            metric_count(
                "untraced module-backed implementation",
                metrics.unexplained_module_backed_items,
            ),
            metric_count(
                "untraced module-connected implementation",
                metrics.unexplained_module_connected_items,
            ),
            metric_count(
                "untraced module-isolated implementation",
                metrics.unexplained_module_isolated_items,
            ),
        ],
    }
}

fn metric_count(label: impl Into<String>, value: usize) -> ProjectedRepoMetricCount {
    ProjectedRepoMetricCount {
        label: label.into(),
        value: value.to_string(),
    }
}

fn project_repo_document_with_policy(document: &RepoDocument, verbose: bool) -> RepoDocument {
    if verbose {
        document.clone()
    } else {
        RepoDocument {
            metrics: document.metrics.clone(),
            analysis: document
                .analysis
                .clone()
                .map(strip_repo_document_analysis_paths),
        }
    }
}

fn strip_repo_document_analysis_paths(
    mut analysis: crate::model::ArchitectureAnalysisSummary,
) -> crate::model::ArchitectureAnalysisSummary {
    if let Some(repo_signals) = &mut analysis.repo_signals {
        strip_repo_signal_paths(repo_signals);
    }
    if let Some(traceability) = &mut analysis.traceability {
        strip_repo_traceability_detail(traceability);
    }
    analysis
}

fn strip_repo_signal_paths(repo_signals: &mut ArchitectureRepoSignalsSummary) {
    repo_signals.unowned_item_details.clear();
    repo_signals.duplicate_item_details.clear();
    repo_signals
        .possible_missing_pattern_application_details
        .clear();
    repo_signals.possible_pattern_cluster_details.clear();
    repo_signals.long_prose_outside_docs_details.clear();
    repo_signals.long_exact_prose_assertion_details.clear();
}

fn strip_repo_traceability_detail(
    traceability: &mut crate::model::ArchitectureTraceabilitySummary,
) {
    traceability.current_spec_items.clear();
    traceability.planned_only_items.clear();
    traceability.deprecated_only_items.clear();
    traceability.file_scoped_only_items.clear();
    traceability.unverified_test_items.clear();
    traceability.statically_mediated_items.clear();
    traceability.unexplained_items.clear();
}
