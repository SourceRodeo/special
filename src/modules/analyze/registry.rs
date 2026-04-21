/**
@module SPECIAL.MODULES.ANALYZE.REGISTRY
Projects the shared `SPECIAL.LANGUAGE_PACKS` registry onto implementation analysis so shared analysis flow can build repo contexts, module analysis, and repo traceability without hardcoding one dispatch branch per language in the analysis core.
*/
// @fileimplements SPECIAL.MODULES.ANALYZE.REGISTRY
use std::collections::{BTreeMap, BTreeSet};
use std::path::{Path, PathBuf};

use anyhow::Result;

use crate::cache::load_or_build_language_pack_blob;
use crate::language_packs::{self, LanguagePackAnalysisContext};
use crate::model::{
    ArchitectureTraceabilitySummary, ImplementRef, ModuleAnalysisOptions, ParsedArchitecture,
    ParsedRepo,
};
use crate::syntax::SourceLanguage;

use super::{FileOwnership, ProviderModuleAnalysis, status};

pub(super) type RepoAnalysisContexts =
    BTreeMap<SourceLanguage, Box<dyn LanguagePackAnalysisContext>>;

pub(super) fn build_repo_analysis_contexts(
    root: &Path,
    source_files: &[PathBuf],
    scoped_source_files: Option<&[PathBuf]>,
    parsed_repo: &ParsedRepo,
    parsed_architecture: &ParsedArchitecture,
    file_ownership: &BTreeMap<PathBuf, FileOwnership<'_>>,
    include_traceability: bool,
) -> RepoAnalysisContexts {
    let available_languages = languages_in_files(source_files);
    let mut contexts = BTreeMap::new();
    for descriptor in language_packs::descriptors() {
        if !available_languages.contains(&descriptor.language) {
            continue;
        }
        let language_source_files = source_files
            .iter()
            .filter(|path| SourceLanguage::from_path(path) == Some(descriptor.language))
            .cloned()
            .collect::<Vec<_>>();
        let language_scoped_files = scoped_source_files.map(|files| {
            files
                .iter()
                .filter(|path| SourceLanguage::from_path(path) == Some(descriptor.language))
                .cloned()
                .collect::<Vec<_>>()
        });
        let traceability_source_files = resolve_traceability_source_files(
            root,
            descriptor,
            &language_source_files,
            language_scoped_files.as_deref(),
            file_ownership,
            include_traceability,
        )
        .unwrap_or_else(|_| language_source_files.clone());
        let traceability_graph_facts = resolve_traceability_graph_facts(
            root,
            descriptor,
            &traceability_source_files,
            include_traceability,
        )
        .unwrap_or(None);
        status::emit_analysis_status(&format!(
            "building {} analysis context for {} file(s){}",
            descriptor.language.id(),
            traceability_source_files.len(),
            if include_traceability {
                " with traceability"
            } else {
                ""
            }
        ));
        contexts.insert(
            descriptor.language,
            (descriptor.build_repo_analysis_context)(
                root,
                &traceability_source_files,
                language_scoped_files.as_deref(),
                traceability_graph_facts.as_deref(),
                parsed_repo,
                parsed_architecture,
                file_ownership,
                include_traceability,
            ),
        );
    }
    contexts
}

fn resolve_traceability_source_files(
    root: &Path,
    descriptor: &language_packs::LanguagePackDescriptor,
    source_files: &[PathBuf],
    scoped_source_files: Option<&[PathBuf]>,
    file_ownership: &BTreeMap<PathBuf, FileOwnership<'_>>,
    include_traceability: bool,
) -> Result<Vec<PathBuf>> {
    if !include_traceability {
        return Ok(source_files.to_vec());
    }
    let Some(scoped_source_files) = scoped_source_files else {
        return Ok(source_files.to_vec());
    };
    if scoped_source_files.is_empty() {
        return Ok(source_files.to_vec());
    }
    let Some(scope_facts) = descriptor.traceability_scope_facts else {
        return Ok(source_files.to_vec());
    };

    let environment_fingerprint = (descriptor.analysis_environment_fingerprint)(root);
    let facts = load_or_build_language_pack_blob(
        root,
        "scope-facts",
        descriptor.language.id(),
        source_files,
        &environment_fingerprint,
        || (scope_facts.build_facts)(root, source_files),
    )?;
    (scope_facts.expand_closure)(source_files, scoped_source_files, file_ownership, &facts)
}

fn resolve_traceability_graph_facts(
    root: &Path,
    descriptor: &language_packs::LanguagePackDescriptor,
    source_files: &[PathBuf],
    include_traceability: bool,
) -> Result<Option<Vec<u8>>> {
    if !include_traceability {
        return Ok(None);
    }
    let Some(graph_facts) = descriptor.traceability_graph_facts else {
        return Ok(None);
    };

    let environment_fingerprint = (descriptor.analysis_environment_fingerprint)(root);
    let facts = load_or_build_language_pack_blob(
        root,
        "traceability-graph-facts",
        descriptor.language.id(),
        source_files,
        &environment_fingerprint,
        || (graph_facts.build_facts)(root, source_files),
    )?;
    Ok(Some(facts))
}

pub(super) fn languages_in_files(files: &[PathBuf]) -> BTreeSet<SourceLanguage> {
    files
        .iter()
        .filter_map(|path| SourceLanguage::from_path(path))
        .collect()
}

pub(super) fn analysis_environment_fingerprint(root: &Path, files: &[PathBuf]) -> String {
    let languages = languages_in_files(files);
    let mut parts = Vec::new();
    for descriptor in language_packs::descriptors() {
        if !languages.contains(&descriptor.language) {
            continue;
        }
        parts.push(format!(
            "{}={}",
            descriptor.language.id(),
            (descriptor.analysis_environment_fingerprint)(root)
        ));
    }
    parts.join("|")
}

pub(super) fn summarize_repo_traceability(
    language: SourceLanguage,
    root: &Path,
    contexts: &RepoAnalysisContexts,
) -> Option<ArchitectureTraceabilitySummary> {
    status::emit_analysis_status(&format!("summarizing {} repo traceability", language.id()));
    contexts.get(&language)?.summarize_repo_traceability(root)
}

pub(super) fn traceability_unavailable_reason(
    language: SourceLanguage,
    contexts: &RepoAnalysisContexts,
) -> Option<String> {
    contexts.get(&language)?.traceability_unavailable_reason()
}

pub(super) fn analyze_module_language(
    language: SourceLanguage,
    root: &Path,
    implementations: &[&ImplementRef],
    file_ownership: &BTreeMap<PathBuf, FileOwnership<'_>>,
    contexts: &RepoAnalysisContexts,
    options: ModuleAnalysisOptions,
) -> Result<ProviderModuleAnalysis> {
    let context = contexts.get(&language).ok_or_else(|| {
        anyhow::anyhow!(
            "metrics analysis expected a {} repo context but none was prepared",
            language.id()
        )
    })?;
    context.analyze_module(root, implementations, file_ownership, options)
}
