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
                "uncaptured prose outside docs",
                metrics.docs.long_prose_outside_docs,
            ),
            metric_count(
                "long prose test literals",
                metrics.tests.exact_long_prose_assertions,
            ),
        ],
        explanations: verbose
            .then(repo_health_summary_explanations)
            .unwrap_or_default(),
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
        "uncaptured prose outside docs by file",
        &metrics.docs.long_prose_outside_docs_by_file,
    );
    push_grouped_metric_section(
        &mut sections,
        "long prose test literals by file",
        &metrics.tests.exact_long_prose_assertions_by_file,
    );
    push_docs_coverage_section(&mut sections, metrics, verbose);

    sections
}

fn push_docs_coverage_section(
    sections: &mut Vec<ProjectedRepoMetricSection>,
    metrics: &RepoMetricsSummary,
    verbose: bool,
) {
    let docs = &metrics.docs;
    let counts = [
        (
            "undocumented current specs",
            docs.undocumented_current_specs,
        ),
        ("undocumented modules", docs.undocumented_modules),
        ("undocumented patterns", docs.undocumented_patterns),
        (
            "internal-only documented targets",
            docs.internal_only_documented_targets,
        ),
    ];
    if !verbose && counts.iter().all(|(_, value)| *value == 0) {
        return;
    }
    sections.push(ProjectedRepoMetricSection {
        title: "docs coverage",
        counts: counts
            .into_iter()
            .filter(|(_, value)| verbose || *value > 0)
            .map(|(label, value)| metric_count(label, value))
            .collect(),
        explanations: Vec::new(),
    });
    if !verbose {
        return;
    }
    push_id_list_metric_section(
        sections,
        "undocumented current spec ids",
        &docs.undocumented_current_spec_ids,
    );
    push_id_list_metric_section(
        sections,
        "undocumented module ids",
        &docs.undocumented_module_ids,
    );
    push_id_list_metric_section(
        sections,
        "undocumented pattern ids",
        &docs.undocumented_pattern_ids,
    );
    push_id_list_metric_section(
        sections,
        "internal-only documented target ids",
        &docs.internal_only_documented_target_ids,
    );
}

fn push_id_list_metric_section(
    sections: &mut Vec<ProjectedRepoMetricSection>,
    title: &'static str,
    ids: &[String],
) {
    if ids.is_empty() {
        return;
    }
    let counts = ids
        .iter()
        .map(|id| ProjectedRepoMetricCount {
            label: id.clone(),
            value: String::new(),
        })
        .collect::<Vec<_>>();
    sections.push(ProjectedRepoMetricSection {
        title,
        counts,
        explanations: Vec::new(),
    });
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
        explanations: Vec::new(),
    });
}

fn metric_count(label: impl Into<String>, value: usize) -> ProjectedRepoMetricCount {
    ProjectedRepoMetricCount {
        label: label.into(),
        value: value.to_string(),
    }
}

fn repo_health_summary_explanations() -> Vec<ProjectedExplanation> {
    vec![
        repo_health_explanation(
            "source outside architecture",
            MetricExplanationKey::UnownedItems,
        ),
        repo_health_explanation(
            "untraced implementation",
            MetricExplanationKey::UntracedImplementation,
        ),
        repo_health_explanation(
            "duplicate source shapes",
            MetricExplanationKey::DuplicateItems,
        ),
        repo_health_explanation(
            "possible pattern clusters",
            MetricExplanationKey::PossiblePatternClusters,
        ),
        repo_health_explanation(
            "possible missing pattern applications",
            MetricExplanationKey::PossibleMissingPatternApplications,
        ),
        repo_health_explanation(
            "uncaptured prose outside docs",
            MetricExplanationKey::LongProseOutsideDocs,
        ),
        repo_health_explanation(
            "long prose test literals",
            MetricExplanationKey::LongExactProseAssertions,
        ),
    ]
}

fn repo_health_explanation(label: &'static str, key: MetricExplanationKey) -> ProjectedExplanation {
    let explanation = metric_explanation(key);
    ProjectedExplanation {
        label,
        plain: explanation.plain,
        precise: explanation.precise,
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
