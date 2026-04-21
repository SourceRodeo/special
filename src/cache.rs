/**
@module SPECIAL.CACHE
Persistent parsed and analysis cache for health and architecture surfaces. This module memoizes parsed repo annotations, parsed architecture declarations, and whole unscoped analysis summaries across command invocations using discovered-file metadata plus language-pack environment fingerprints as invalidation inputs. It should stay a reusable substrate underneath `special`, `special specs`, `special arch`, and `special health` rather than caching rendered output shapes.
*/
// @fileimplements SPECIAL.CACHE
use std::path::Path;

use anyhow::Result;

use crate::config::SpecialVersion;
use crate::model::{
    ArchitectureAnalysisSummary, ModuleAnalysisOptions, ParsedArchitecture, ParsedRepo,
};
use crate::modules::analyze::{self, ArchitectureAnalysis};
use crate::parser::{self, ParseDialect};

mod fingerprint;
mod lock;
mod stats;
mod storage;

#[cfg(test)]
pub use stats::CacheStats;

const CACHE_SCHEMA_VERSION: u32 = 3;
const CACHE_LOCK_STALE_AFTER: std::time::Duration = std::time::Duration::from_secs(300);
const CACHE_LOCK_POLL_INTERVAL: std::time::Duration = std::time::Duration::from_millis(100);
const CACHE_LOCK_REFRESH_INTERVAL: std::time::Duration = std::time::Duration::from_secs(30);

pub fn reset_cache_stats() {
    stats::reset_cache_stats();
}

#[cfg(test)]
pub fn snapshot_cache_stats() -> CacheStats {
    stats::snapshot_cache_stats()
}

pub fn format_cache_stats_summary() -> Option<String> {
    stats::format_cache_stats_summary()
}

pub fn with_cache_status_notifier<T>(
    notifier: impl Fn(&str) + 'static,
    f: impl FnOnce() -> T,
) -> T {
    stats::with_cache_status_notifier(notifier, f)
}

pub fn load_or_parse_repo(
    root: &Path,
    ignore_patterns: &[String],
    version: SpecialVersion,
) -> Result<ParsedRepo> {
    let fingerprint = fingerprint::repo_fingerprint(root, ignore_patterns, version)?;
    let cache_path =
        storage::cache_file_path(root, &format!("parsed-repo-v{CACHE_SCHEMA_VERSION}.json"));
    if let Some(parsed) = storage::read_repo_cache(&cache_path, fingerprint)? {
        stats::record_repo_hit();
        return Ok(parsed);
    }

    let _guard = lock::acquire_cache_fill_lock(&cache_path)?;
    if let Some(parsed) = storage::read_repo_cache(&cache_path, fingerprint)? {
        stats::record_repo_hit();
        return Ok(parsed);
    }

    let parsed = parser::parse_repo(root, ignore_patterns, parse_dialect(version))?;
    storage::write_repo_cache(&cache_path, fingerprint, &parsed)?;
    stats::record_repo_miss();
    Ok(parsed)
}

pub fn load_or_parse_architecture(
    root: &Path,
    ignore_patterns: &[String],
) -> Result<ParsedArchitecture> {
    let fingerprint = fingerprint::architecture_fingerprint(root, ignore_patterns)?;
    let cache_path = storage::cache_file_path(
        root,
        &format!("parsed-architecture-v{CACHE_SCHEMA_VERSION}.json"),
    );
    if let Some(parsed) = storage::read_architecture_cache(&cache_path, fingerprint)? {
        stats::record_architecture_hit();
        return Ok(parsed);
    }

    let _guard = lock::acquire_cache_fill_lock(&cache_path)?;
    if let Some(parsed) = storage::read_architecture_cache(&cache_path, fingerprint)? {
        stats::record_architecture_hit();
        return Ok(parsed);
    }

    let parsed = crate::modules::parse_architecture(root, ignore_patterns)?;
    storage::write_architecture_cache(&cache_path, fingerprint, &parsed)?;
    stats::record_architecture_miss();
    Ok(parsed)
}

pub fn load_or_build_repo_analysis_summary(
    root: &Path,
    ignore_patterns: &[String],
    version: SpecialVersion,
    parsed_architecture: &ParsedArchitecture,
    parsed_repo: &ParsedRepo,
) -> Result<ArchitectureAnalysisSummary> {
    let fingerprint =
        fingerprint::repo_analysis_fingerprint(root, ignore_patterns, version, parsed_repo)?;
    let cache_path =
        storage::cache_file_path(root, &format!("repo-analysis-v{CACHE_SCHEMA_VERSION}.json"));
    if let Some(summary) = storage::read_repo_analysis_cache(&cache_path, fingerprint)? {
        stats::record_repo_analysis_hit();
        return Ok(summary);
    }

    let _guard = lock::acquire_cache_fill_lock(&cache_path)?;
    if let Some(summary) = storage::read_repo_analysis_cache(&cache_path, fingerprint)? {
        stats::record_repo_analysis_hit();
        return Ok(summary);
    }

    let summary = analyze::build_repo_analysis_summary(
        root,
        ignore_patterns,
        parsed_architecture,
        parsed_repo,
        None,
    )?;
    storage::write_repo_analysis_cache(&cache_path, fingerprint, &summary)?;
    stats::record_repo_analysis_miss();
    Ok(summary)
}

pub fn load_or_build_architecture_analysis(
    root: &Path,
    ignore_patterns: &[String],
    version: SpecialVersion,
    parsed_architecture: &ParsedArchitecture,
    parsed_repo: Option<&ParsedRepo>,
    options: ModuleAnalysisOptions,
) -> Result<ArchitectureAnalysis> {
    let options = options.normalized();
    let fingerprint = fingerprint::architecture_analysis_fingerprint(
        root,
        ignore_patterns,
        version,
        parsed_repo.is_some(),
        options,
    )?;
    let cache_path = storage::cache_file_path(
        root,
        &format!("architecture-analysis-v{CACHE_SCHEMA_VERSION}.json"),
    );
    if let Some(analysis) = storage::read_architecture_analysis_cache(&cache_path, fingerprint)? {
        stats::record_architecture_analysis_hit();
        return Ok(analysis);
    }

    let _guard = lock::acquire_cache_fill_lock(&cache_path)?;
    if let Some(analysis) = storage::read_architecture_analysis_cache(&cache_path, fingerprint)? {
        stats::record_architecture_analysis_hit();
        return Ok(analysis);
    }

    let analysis = analyze::build_architecture_analysis(
        root,
        ignore_patterns,
        parsed_architecture,
        parsed_repo,
        options,
    )?;
    storage::write_architecture_analysis_cache(&cache_path, fingerprint, &analysis)?;
    stats::record_architecture_analysis_miss();
    Ok(analysis)
}

fn parse_dialect(version: SpecialVersion) -> ParseDialect {
    match version {
        SpecialVersion::V0 => ParseDialect::CompatibilityV0,
        SpecialVersion::V1 => ParseDialect::CurrentV1,
    }
}

#[cfg(test)]
mod tests;
