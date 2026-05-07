/**
@module SPECIAL.RENDER.PROJECTION.REPO
Projects repo-wide health, signals, and traceability documents into backend-ready verbose or non-verbose shapes.
*/
// @fileimplements SPECIAL.RENDER.PROJECTION.REPO
use crate::model::{ArchitectureRepoSignalsSummary, RepoDocument};

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
