/**
@module SPECIAL.RENDER.PROJECTION.REPO
Projects repo-wide health, signals, and traceability documents into backend-ready verbose or non-verbose shapes.
*/
// @fileimplements SPECIAL.RENDER.PROJECTION.REPO
use crate::model::{
    ArchitectureRepoSignalsSummary, GroupedCount, RepoDocument, RepoMetricsSummary,
};
use crate::modules::analyze::explain::{MetricExplanationKey, metric_explanation};

use super::ProjectedExplanation;

#[derive(Debug, Clone)]
pub(in crate::render) struct ProjectedRepoMetricSection {
    pub(in crate::render) title: &'static str,
    pub(in crate::render) guidance: Option<&'static str>,
    pub(in crate::render) counts: Vec<ProjectedRepoMetricCount>,
    pub(in crate::render) explanations: Vec<ProjectedExplanation>,
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
    verbose: bool,
) -> Vec<ProjectedRepoMetricSection> {
    let mut sections = vec![
        ProjectedRepoMetricSection {
            title: "architecture ownership",
            guidance: verbose.then_some(
                "Review source that exists outside declared modules before treating architecture metrics as complete.",
            ),
            counts: vec![metric_count(
                "source outside architecture",
                metrics.architecture.source_outside_architecture,
            )],
            explanations: repo_health_explanations(
                verbose,
                &[(
                    "source outside architecture",
                    MetricExplanationKey::UnownedItems,
                )],
            ),
        },
        ProjectedRepoMetricSection {
            title: "proof traceability",
            guidance: verbose.then_some(
                "Review implementation Special cannot connect to current spec support, without treating process or framework entrypoints as proof edges.",
            ),
            counts: vec![metric_count(
                "untraced implementation",
                metrics.specs.untraced_implementation,
            )],
            explanations: repo_health_explanations(
                verbose,
                &[(
                    "untraced implementation",
                    MetricExplanationKey::UntracedImplementation,
                )],
            ),
        },
        ProjectedRepoMetricSection {
            title: "repeated structure",
            guidance: verbose.then_some(
                "Review repeated implementation or documentation shapes before deciding whether they are duplication, named patterns, or acceptable parallelism.",
            ),
            counts: vec![
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
            ],
            explanations: repo_health_explanations(
                verbose,
                &[
                    ("duplicate source shapes", MetricExplanationKey::DuplicateItems),
                    (
                        "possible pattern clusters",
                        MetricExplanationKey::PossiblePatternClusters,
                    ),
                    (
                        "possible missing pattern applications",
                        MetricExplanationKey::PossibleMissingPatternApplications,
                    ),
                ],
            ),
        },
        ProjectedRepoMetricSection {
            title: "prose and tests",
            guidance: verbose.then_some(
                "Review substantial natural-language blocks and long prose literals so docs, comments, specs, and tests each carry the right kind of text.",
            ),
            counts: vec![
                metric_count(
                    "uncaptured prose outside docs",
                    metrics.docs.long_prose_outside_docs,
                ),
                metric_count(
                    "long prose test literals",
                    metrics.tests.exact_long_prose_assertions,
                ),
            ],
            explanations: repo_health_explanations(
                verbose,
                &[
                    (
                        "uncaptured prose outside docs",
                        MetricExplanationKey::LongProseOutsideDocs,
                    ),
                    (
                        "long prose test literals",
                        MetricExplanationKey::LongExactProseAssertions,
                    ),
                ],
            ),
        },
    ];

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
        "uncaptured prose outside docs by file",
        &metrics.docs.long_prose_outside_docs_by_file,
    );
    push_grouped_metric_section(
        &mut sections,
        "long prose test literals by file",
        &metrics.tests.exact_long_prose_assertions_by_file,
    );

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
        guidance: None,
        counts: counts
            .iter()
            .map(|count| ProjectedRepoMetricCount {
                label: count.value.clone(),
                value: count.count.to_string(),
            })
            .collect(),
        explanations: Vec::new(),
    });
}

fn metric_count(label: impl Into<String>, value: usize) -> ProjectedRepoMetricCount {
    ProjectedRepoMetricCount {
        label: label.into(),
        value: value.to_string(),
    }
}

fn repo_health_explanation(label: &'static str, key: MetricExplanationKey) -> ProjectedExplanation {
    let explanation = metric_explanation(key);
    ProjectedExplanation {
        label,
        plain: explanation.plain,
        precise: explanation.precise,
    }
}

fn repo_health_explanations(
    verbose: bool,
    items: &[(&'static str, MetricExplanationKey)],
) -> Vec<ProjectedExplanation> {
    if verbose {
        items
            .iter()
            .map(|(label, key)| repo_health_explanation(label, *key))
            .collect()
    } else {
        Vec::new()
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
