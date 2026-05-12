/**
@module SPECIAL.CACHE.FINGERPRINT
Builds stable invalidation fingerprints for parsed and analyzed cache entries in `SPECIAL.CACHE`.
*/
// @fileimplements SPECIAL.CACHE.FINGERPRINT
use std::collections::hash_map::DefaultHasher;
use std::fs;
use std::hash::{Hash, Hasher};
use std::path::Path;
use std::sync::OnceLock;
use std::time::UNIX_EPOCH;

use anyhow::{Context, Result};

use crate::config::SpecialVersion;
use crate::discovery::{DiscoveryConfig, discover_annotation_files};
use crate::model::{ModuleAnalysisOptions, ParsedRepo};
use crate::modules::analyze;

#[derive(Clone, Copy, Hash)]
pub(super) enum RepoAnalysisScopeKind {
    Target,
    Within,
}

pub(super) fn repo_fingerprint(
    root: &Path,
    ignore_patterns: &[String],
    version: SpecialVersion,
) -> Result<u64> {
    fingerprint_for_discovered_files(root, ignore_patterns, Some(version))
}

pub(super) fn architecture_fingerprint(root: &Path, ignore_patterns: &[String]) -> Result<u64> {
    fingerprint_for_discovered_files(root, ignore_patterns, None)
}

pub(super) fn repo_analysis_fingerprint(
    root: &Path,
    ignore_patterns: &[String],
    version: SpecialVersion,
    parsed_repo: &ParsedRepo,
) -> Result<u64> {
    repo_analysis_fingerprint_with_engine(
        root,
        ignore_patterns,
        version,
        parsed_repo,
        analysis_engine_fingerprint(),
    )
}

fn repo_analysis_fingerprint_with_engine(
    root: &Path,
    ignore_patterns: &[String],
    version: SpecialVersion,
    parsed_repo: &ParsedRepo,
    engine_fingerprint: u64,
) -> Result<u64> {
    let files = discover_annotation_files(DiscoveryConfig {
        root,
        ignore_patterns,
    })?
    .source_files;
    let mut hasher = DefaultHasher::new();
    repo_fingerprint(root, ignore_patterns, version)?.hash(&mut hasher);
    parsed_repo.verifies.len().hash(&mut hasher);
    parsed_repo.attests.len().hash(&mut hasher);
    engine_fingerprint.hash(&mut hasher);
    analyze::analysis_environment_fingerprint(root, &files).hash(&mut hasher);
    Ok(hasher.finish())
}

pub(super) fn scoped_repo_analysis_fingerprint(
    root: &Path,
    ignore_patterns: &[String],
    version: SpecialVersion,
    parsed_repo: &ParsedRepo,
    scope_kind: RepoAnalysisScopeKind,
    scoped_paths: &[std::path::PathBuf],
) -> Result<u64> {
    let mut hasher = DefaultHasher::new();
    repo_analysis_fingerprint(root, ignore_patterns, version, parsed_repo)?.hash(&mut hasher);
    scope_kind.hash(&mut hasher);
    for path in analyze::normalized_scope_paths(root, scoped_paths) {
        path.hash(&mut hasher);
    }
    Ok(hasher.finish())
}

pub(super) fn language_pack_scope_facts_fingerprint(
    root: &Path,
    language_id: &str,
    source_files: &[std::path::PathBuf],
    environment_fingerprint: &str,
) -> Result<u64> {
    language_pack_scope_facts_fingerprint_with_engine(
        root,
        language_id,
        source_files,
        environment_fingerprint,
        analysis_engine_fingerprint(),
    )
}

fn language_pack_scope_facts_fingerprint_with_engine(
    root: &Path,
    language_id: &str,
    source_files: &[std::path::PathBuf],
    environment_fingerprint: &str,
    engine_fingerprint: u64,
) -> Result<u64> {
    let mut hasher = DefaultHasher::new();
    super::CACHE_SCHEMA_VERSION.hash(&mut hasher);
    root.hash(&mut hasher);
    language_id.hash(&mut hasher);
    engine_fingerprint.hash(&mut hasher);
    environment_fingerprint.hash(&mut hasher);
    for path in source_files {
        path.hash(&mut hasher);
        if let Ok(metadata) = fs::metadata(path) {
            metadata.len().hash(&mut hasher);
            if let Ok(modified) = metadata.modified()
                && let Ok(duration) = modified.duration_since(UNIX_EPOCH)
            {
                duration.as_secs().hash(&mut hasher);
                duration.subsec_nanos().hash(&mut hasher);
            }
        }
        hash_file_contents(path, &mut hasher)?;
    }
    for path in language_pack_manifest_inputs(root, language_id) {
        path.hash(&mut hasher);
        if let Ok(metadata) = fs::metadata(&path) {
            metadata.len().hash(&mut hasher);
            if let Ok(modified) = metadata.modified()
                && let Ok(duration) = modified.duration_since(UNIX_EPOCH)
            {
                duration.as_secs().hash(&mut hasher);
                duration.subsec_nanos().hash(&mut hasher);
            }
        }
        hash_file_contents(&path, &mut hasher)?;
    }
    Ok(hasher.finish())
}

pub(super) fn parsed_repo_contract_fingerprint(parsed_repo: &ParsedRepo) -> u64 {
    let mut hasher = DefaultHasher::new();
    serde_json::to_vec(parsed_repo)
        .expect("parsed repo should serialize for cache fingerprinting")
        .hash(&mut hasher);
    hasher.finish()
}

pub(super) fn architecture_analysis_fingerprint(
    root: &Path,
    ignore_patterns: &[String],
    version: SpecialVersion,
    include_repo: bool,
    options: ModuleAnalysisOptions,
) -> Result<u64> {
    architecture_analysis_fingerprint_with_engine(
        root,
        ignore_patterns,
        version,
        include_repo,
        options,
        analysis_engine_fingerprint(),
    )
}

fn architecture_analysis_fingerprint_with_engine(
    root: &Path,
    ignore_patterns: &[String],
    version: SpecialVersion,
    include_repo: bool,
    options: ModuleAnalysisOptions,
    engine_fingerprint: u64,
) -> Result<u64> {
    let files = discover_annotation_files(DiscoveryConfig {
        root,
        ignore_patterns,
    })?
    .source_files;
    let mut hasher = DefaultHasher::new();
    architecture_fingerprint(root, ignore_patterns)?.hash(&mut hasher);
    if include_repo {
        repo_fingerprint(root, ignore_patterns, version)?.hash(&mut hasher);
    }
    include_repo.hash(&mut hasher);
    options.coverage.hash(&mut hasher);
    options.metrics.hash(&mut hasher);
    options.traceability.hash(&mut hasher);
    engine_fingerprint.hash(&mut hasher);
    analyze::analysis_environment_fingerprint(root, &files).hash(&mut hasher);
    Ok(hasher.finish())
}

fn fingerprint_for_discovered_files(
    root: &Path,
    ignore_patterns: &[String],
    version: Option<SpecialVersion>,
) -> Result<u64> {
    let discovered = discover_annotation_files(DiscoveryConfig {
        root,
        ignore_patterns,
    })?;
    let mut hasher = DefaultHasher::new();
    super::CACHE_SCHEMA_VERSION.hash(&mut hasher);
    root.hash(&mut hasher);
    ignore_patterns.hash(&mut hasher);
    version.map(SpecialVersion::as_str).hash(&mut hasher);

    for path in discovered
        .source_files
        .iter()
        .chain(discovered.markdown_files.iter())
    {
        path.hash(&mut hasher);
        if let Ok(metadata) = fs::metadata(path) {
            metadata.len().hash(&mut hasher);
            if let Ok(modified) = metadata.modified()
                && let Ok(duration) = modified.duration_since(UNIX_EPOCH)
            {
                duration.as_secs().hash(&mut hasher);
                duration.subsec_nanos().hash(&mut hasher);
            }
        }
        hash_file_contents(path, &mut hasher)?;
    }

    Ok(hasher.finish())
}

fn hash_file_contents(path: &Path, hasher: &mut DefaultHasher) -> Result<()> {
    let contents = fs::read(path)
        .with_context(|| format!("reading cache fingerprint input {}", path.display()))?;
    contents.hash(hasher);
    Ok(())
}

fn analysis_engine_fingerprint() -> u64 {
    static FINGERPRINT: OnceLock<u64> = OnceLock::new();
    *FINGERPRINT.get_or_init(compute_analysis_engine_fingerprint)
}

fn compute_analysis_engine_fingerprint() -> u64 {
    let mut hasher = DefaultHasher::new();
    super::CACHE_SCHEMA_VERSION.hash(&mut hasher);
    env!("CARGO_PKG_NAME").hash(&mut hasher);
    env!("CARGO_PKG_VERSION").hash(&mut hasher);

    if let Ok(path) = std::env::current_exe() {
        if let Ok(bytes) = fs::read(&path) {
            "current-exe-bytes".hash(&mut hasher);
            bytes.hash(&mut hasher);
            return hasher.finish();
        }
        if let Ok(metadata) = fs::metadata(&path) {
            "current-exe-metadata".hash(&mut hasher);
            metadata.len().hash(&mut hasher);
            if let Ok(modified) = metadata.modified()
                && let Ok(duration) = modified.duration_since(UNIX_EPOCH)
            {
                duration.as_secs().hash(&mut hasher);
                duration.subsec_nanos().hash(&mut hasher);
            }
            return hasher.finish();
        }
    }

    "package-only-engine-fingerprint".hash(&mut hasher);
    hasher.finish()
}

#[cfg(test)]
pub(super) fn repo_analysis_fingerprint_for_engine(
    root: &Path,
    ignore_patterns: &[String],
    version: SpecialVersion,
    parsed_repo: &ParsedRepo,
    engine_fingerprint: u64,
) -> Result<u64> {
    repo_analysis_fingerprint_with_engine(
        root,
        ignore_patterns,
        version,
        parsed_repo,
        engine_fingerprint,
    )
}

#[cfg(test)]
pub(super) fn architecture_analysis_fingerprint_for_engine(
    root: &Path,
    ignore_patterns: &[String],
    version: SpecialVersion,
    include_repo: bool,
    options: ModuleAnalysisOptions,
    engine_fingerprint: u64,
) -> Result<u64> {
    architecture_analysis_fingerprint_with_engine(
        root,
        ignore_patterns,
        version,
        include_repo,
        options,
        engine_fingerprint,
    )
}

#[cfg(test)]
pub(super) fn language_pack_scope_facts_fingerprint_for_engine(
    root: &Path,
    language_id: &str,
    source_files: &[std::path::PathBuf],
    environment_fingerprint: &str,
    engine_fingerprint: u64,
) -> Result<u64> {
    language_pack_scope_facts_fingerprint_with_engine(
        root,
        language_id,
        source_files,
        environment_fingerprint,
        engine_fingerprint,
    )
}

fn language_pack_manifest_inputs(root: &Path, language_id: &str) -> Vec<std::path::PathBuf> {
    let candidates: &[&str] = match language_id {
        "rust" => &[
            "Cargo.toml",
            "Cargo.lock",
            "rust-toolchain",
            "rust-toolchain.toml",
        ],
        "go" => &["go.mod", "go.sum", "vendor/modules.txt"],
        "typescript" => &[
            "package.json",
            "package-lock.json",
            "pnpm-lock.yaml",
            "yarn.lock",
            "tsconfig.json",
            "jsconfig.json",
        ],
        "python" => &[
            "pyproject.toml",
            "poetry.lock",
            "requirements.txt",
            "requirements-dev.txt",
            "requirements.lock",
            "uv.lock",
            "Pipfile",
            "Pipfile.lock",
        ],
        _ => &[],
    };

    candidates
        .iter()
        .map(|relative| root.join(relative))
        .filter(|path| path.is_file())
        .collect()
}
