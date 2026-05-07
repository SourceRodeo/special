/**
@module SPECIAL.RENDER.PROJECTION.MODULE_ANALYSIS.REPO_SIGNALS
Projects repo-level ownership and duplicate-item signal summaries into the shared render view.
*/
// @fileimplements SPECIAL.RENDER.PROJECTION.MODULE_ANALYSIS.REPO_SIGNALS
use crate::model::{ArchitectureRepoSignalsSummary, ModuleItemKind};
use crate::modules::analyze::explain::MetricExplanationKey;

use super::{ProjectedRepoSignals, count, explanation};

pub(in crate::render) fn project_repo_signals_view(
    coverage: &ArchitectureRepoSignalsSummary,
    verbose: bool,
) -> ProjectedRepoSignals {
    let explanations = vec![
        explanation(
            "source outside architecture",
            MetricExplanationKey::UnownedItems,
        ),
        explanation(
            "duplicate source shapes",
            MetricExplanationKey::DuplicateItems,
        ),
        explanation(
            "long prose outside docs",
            MetricExplanationKey::LongProseOutsideDocs,
        ),
        explanation(
            "long exact prose assertions",
            MetricExplanationKey::LongExactProseAssertions,
        ),
    ];

    ProjectedRepoSignals {
        counts: vec![
            count("source outside architecture", coverage.unowned_items),
            count("duplicate source shapes", coverage.duplicate_items),
            count(
                "possible missing pattern applications",
                coverage.possible_missing_pattern_applications,
            ),
            count(
                "possible pattern clusters",
                coverage.possible_pattern_clusters,
            ),
            count("long prose outside docs", coverage.long_prose_outside_docs),
            count(
                "long exact prose assertions",
                coverage.long_exact_prose_assertions,
            ),
        ],
        explanations,
        unowned_items: if verbose {
            coverage
                .unowned_item_details
                .iter()
                .map(|item| {
                    format!(
                        "{}:{} [{}]",
                        item.path.display(),
                        item.name,
                        match item.kind {
                            ModuleItemKind::Function => "function",
                            ModuleItemKind::Method => "method",
                        }
                    )
                })
                .collect()
        } else {
            Vec::new()
        },
        duplicate_items: if verbose {
            coverage
                .duplicate_item_details
                .iter()
                .map(|item| {
                    format!(
                        "{}:{}:{} [{}; duplicate peers {}]",
                        item.module_id,
                        item.path.display(),
                        item.name,
                        match item.kind {
                            ModuleItemKind::Function => "function",
                            ModuleItemKind::Method => "method",
                        },
                        item.duplicate_peer_count,
                    )
                })
                .collect()
        } else {
            Vec::new()
        },
        possible_missing_pattern_applications: if verbose {
            coverage
                .possible_missing_pattern_application_details
                .iter()
                .map(|item| {
                    format!(
                        "{} {} {:.3} at {}:{} ({})",
                        item.confidence.label(),
                        item.pattern_id,
                        item.score,
                        item.location.path.display(),
                        item.location.line,
                        item.item_name
                    )
                })
                .collect()
        } else {
            Vec::new()
        },
        possible_pattern_clusters: if verbose {
            coverage
                .possible_pattern_cluster_details
                .iter()
                .map(|cluster| {
                    let first = cluster
                        .items
                        .first()
                        .map(|item| {
                            format!(
                                "{} at {}:{}",
                                item.item_name,
                                item.location.path.display(),
                                item.location.line
                            )
                        })
                        .unwrap_or_else(|| "no representative item".to_string());
                    format!(
                        "{} item(s), score {:.3}, suggested strictness {}, {}; {first}",
                        cluster.item_count,
                        cluster.score,
                        cluster.suggested_strictness.as_str(),
                        cluster.interpretation.label()
                    )
                })
                .collect()
        } else {
            Vec::new()
        },
        long_prose_outside_docs: if verbose {
            coverage
                .long_prose_outside_docs_details
                .iter()
                .map(|item| {
                    format!(
                        "{}:{} [{} words, {} sentence(s), score {:.3}] {}",
                        item.path.display(),
                        item.line,
                        item.word_count,
                        item.sentence_count,
                        item.prose_score,
                        item.preview.replace('\n', "\\n")
                    )
                })
                .collect()
        } else {
            Vec::new()
        },
        long_exact_prose_assertions: if verbose {
            coverage
                .long_exact_prose_assertion_details
                .iter()
                .map(|item| {
                    format!(
                        "{}:{} [{} {}; {} words] {}",
                        item.path.display(),
                        item.line,
                        item.language,
                        item.callee,
                        item.word_count,
                        item.literal.replace('\n', "\\n")
                    )
                })
                .collect()
        } else {
            Vec::new()
        },
    }
}
