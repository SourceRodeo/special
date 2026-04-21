/**
@module SPECIAL.MODULES.ANALYZE.REPO_SCOPE
Owns repo-scope path matching and summary filtering for repo-wide analysis views.
*/
// @fileimplements SPECIAL.MODULES.ANALYZE.REPO_SCOPE
use std::collections::BTreeSet;
use std::fs;
use std::path::{Path, PathBuf};

use anyhow::Result;

use crate::discovery::{DiscoveryConfig, discover_annotation_files};
use crate::model::{
    ArchitectureAnalysisSummary, ArchitectureRepoSignalsSummary, ArchitectureTraceabilityItem,
    ArchitectureTraceabilitySummary,
};

pub(crate) fn filter_repo_analysis_summary_to_scope(
    root: &Path,
    ignore_patterns: &[String],
    scoped_paths: &[PathBuf],
    summary: &mut ArchitectureAnalysisSummary,
) -> Result<()> {
    let all_files = discover_annotation_files(DiscoveryConfig {
        root,
        ignore_patterns,
    })?
    .source_files;
    let scoped_files = scope_source_files(root, &all_files, Some(scoped_paths))?;
    if let Some(repo_signals) = &mut summary.repo_signals {
        filter_repo_signals_to_scope(root, &scoped_files, repo_signals);
    }
    if let Some(traceability) = &mut summary.traceability {
        filter_traceability_to_scope(root, &scoped_files, traceability);
    }
    Ok(())
}

pub(crate) fn filter_repo_analysis_summary_to_symbol(
    symbol: &str,
    summary: &mut ArchitectureAnalysisSummary,
) {
    if let Some(repo_signals) = &mut summary.repo_signals {
        repo_signals
            .unowned_item_details
            .retain(|item| item.name == symbol);
        repo_signals.unowned_items = repo_signals.unowned_item_details.len();
        repo_signals
            .duplicate_item_details
            .retain(|item| item.name == symbol);
        repo_signals.duplicate_items = repo_signals.duplicate_item_details.len();
    }
    if let Some(traceability) = &mut summary.traceability {
        retain_traceability_items_by_symbol(symbol, &mut traceability.current_spec_items);
        retain_traceability_items_by_symbol(symbol, &mut traceability.planned_only_items);
        retain_traceability_items_by_symbol(symbol, &mut traceability.deprecated_only_items);
        retain_traceability_items_by_symbol(symbol, &mut traceability.file_scoped_only_items);
        retain_traceability_items_by_symbol(symbol, &mut traceability.unverified_test_items);
        retain_traceability_items_by_symbol(symbol, &mut traceability.statically_mediated_items);
        retain_traceability_items_by_symbol(symbol, &mut traceability.unexplained_items);
        traceability.analyzed_items = traceability.current_spec_items.len()
            + traceability.planned_only_items.len()
            + traceability.deprecated_only_items.len()
            + traceability.file_scoped_only_items.len()
            + traceability.unverified_test_items.len()
            + traceability.statically_mediated_items.len()
            + traceability.unexplained_items.len();
        traceability.sort_items();
    }
}

pub(super) fn scope_source_files(
    root: &Path,
    all_files: &[PathBuf],
    scoped_paths: Option<&[PathBuf]>,
) -> Result<Vec<PathBuf>> {
    let Some(scoped_paths) = scoped_paths else {
        return Ok(all_files.to_vec());
    };
    if scoped_paths.is_empty() {
        return Ok(all_files.to_vec());
    }

    let scope_roots = scoped_paths
        .iter()
        .map(|path| normalize_scope_path(root, path))
        .collect::<Vec<_>>();

    let scoped_files = all_files
        .iter()
        .filter(|path| {
            let candidate = normalize_scope_path(root, path);
            scope_roots
                .iter()
                .any(|scope| candidate == *scope || candidate.starts_with(scope))
        })
        .cloned()
        .collect::<Vec<_>>();

    if scoped_files.is_empty() {
        anyhow::bail!("repo scope did not match any analyzable source files");
    }

    Ok(scoped_files)
}

pub(super) fn filter_repo_signals_to_scope(
    root: &Path,
    scoped_files: &[PathBuf],
    summary: &mut ArchitectureRepoSignalsSummary,
) {
    let matcher = RepoScopeMatcher::new(root, scoped_files);
    summary
        .unowned_item_details
        .retain(|item| matcher.matches_display_path(&item.path));
    summary.unowned_items = summary.unowned_item_details.len();
    summary
        .duplicate_item_details
        .retain(|item| matcher.matches_display_path(&item.path));
    summary.duplicate_items = summary.duplicate_item_details.len();
}

pub(super) fn filter_traceability_to_scope(
    root: &Path,
    scoped_files: &[PathBuf],
    summary: &mut ArchitectureTraceabilitySummary,
) {
    let matcher = RepoScopeMatcher::new(root, scoped_files);
    retain_traceability_items(&matcher, &mut summary.current_spec_items);
    retain_traceability_items(&matcher, &mut summary.planned_only_items);
    retain_traceability_items(&matcher, &mut summary.deprecated_only_items);
    retain_traceability_items(&matcher, &mut summary.file_scoped_only_items);
    retain_traceability_items(&matcher, &mut summary.unverified_test_items);
    retain_traceability_items(&matcher, &mut summary.statically_mediated_items);
    retain_traceability_items(&matcher, &mut summary.unexplained_items);
    summary.analyzed_items = summary.current_spec_items.len()
        + summary.planned_only_items.len()
        + summary.deprecated_only_items.len()
        + summary.file_scoped_only_items.len()
        + summary.unverified_test_items.len()
        + summary.statically_mediated_items.len()
        + summary.unexplained_items.len();
    summary.sort_items();
}

fn normalize_scope_path(root: &Path, path: &Path) -> PathBuf {
    let joined = if path.is_absolute() {
        path.to_path_buf()
    } else {
        root.join(path)
    };
    fs::canonicalize(&joined).unwrap_or(joined)
}

fn retain_traceability_items(
    matcher: &RepoScopeMatcher,
    items: &mut Vec<ArchitectureTraceabilityItem>,
) {
    items.retain(|item| matcher.matches_display_path(&item.path));
}

fn retain_traceability_items_by_symbol(
    symbol: &str,
    items: &mut Vec<ArchitectureTraceabilityItem>,
) {
    items.retain(|item| item.name == symbol);
}

struct RepoScopeMatcher {
    root: PathBuf,
    scoped_files: BTreeSet<PathBuf>,
}

impl RepoScopeMatcher {
    fn new(root: &Path, scoped_files: &[PathBuf]) -> Self {
        Self {
            root: root.to_path_buf(),
            scoped_files: scoped_files
                .iter()
                .map(|path| normalize_scope_path(root, path))
                .collect(),
        }
    }

    fn matches_display_path(&self, display_path: &Path) -> bool {
        let candidate = normalize_scope_path(&self.root, display_path);
        self.scoped_files.contains(&candidate)
    }
}
