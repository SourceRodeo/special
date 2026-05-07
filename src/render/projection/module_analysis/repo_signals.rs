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
        explanation("unowned items", MetricExplanationKey::UnownedItems),
        explanation("duplicate items", MetricExplanationKey::DuplicateItems),
        explanation(
            "long exact prose assertions",
            MetricExplanationKey::LongExactProseAssertions,
        ),
    ];

    ProjectedRepoSignals {
        counts: vec![
            count("unowned items", coverage.unowned_items),
            count("duplicate items", coverage.duplicate_items),
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
