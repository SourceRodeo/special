/**
@module SPECIAL.LANGUAGE_PACKS.PYTHON
Registers the built-in Python language pack for parser-backed Python implementation traceability: functions, methods, imports, pytest roots, fixture injection, and clear direct call graphs without claiming dynamic import, monkeypatch, or runtime attribute resolution.
*/
// @fileimplements SPECIAL.LANGUAGE_PACKS.PYTHON
use std::collections::BTreeMap;
use std::path::{Path, PathBuf};

use anyhow::Result;

use super::{LanguagePackAnalysisContext, LanguagePackDescriptor, ScopedTraceabilityPreparation};
use crate::model::{
    ArchitectureTraceabilitySummary, ImplementRef, ModuleAnalysisOptions, ParsedArchitecture,
    ParsedRepo,
};
use crate::modules::analyze::{FileOwnership, ProviderModuleAnalysis};
use crate::source_paths::has_extension;
use crate::syntax::{ParsedSourceGraph, SourceLanguage};

#[path = "python/analyze.rs"]
mod analyze;

pub(crate) const DESCRIPTOR: LanguagePackDescriptor = LanguagePackDescriptor {
    language: SourceLanguage::new("python"),
    matches_path: is_python_path,
    parse_source_graph,
    build_repo_analysis_context,
    analysis_environment_fingerprint,
    project_tooling: None,
    traceability_scope_facts: None,
    traceability_graph_facts: None,
    scoped_traceability_preparation: ScopedTraceabilityPreparation::ScopedGraphDiscovery,
};

impl LanguagePackAnalysisContext for analyze::PythonRepoAnalysisContext {
    fn summarize_repo_traceability(&self, root: &Path) -> Option<ArchitectureTraceabilitySummary> {
        analyze::summarize_repo_traceability(root, self)
    }

    fn traceability_unavailable_reason(&self) -> Option<String> {
        self.traceability_unavailable_reason.clone()
    }

    fn analyze_module(
        &self,
        root: &Path,
        implementations: &[&ImplementRef],
        file_ownership: &BTreeMap<PathBuf, FileOwnership<'_>>,
        options: ModuleAnalysisOptions,
    ) -> Result<ProviderModuleAnalysis> {
        analyze::analyze_module(root, implementations, file_ownership, self, options.traceability)
    }
}

#[allow(clippy::too_many_arguments)]
fn build_repo_analysis_context(
    root: &Path,
    source_files: &[PathBuf],
    scoped_source_files: Option<&[PathBuf]>,
    _traceability_graph_facts: Option<&[u8]>,
    parsed_repo: &ParsedRepo,
    parsed_architecture: &ParsedArchitecture,
    file_ownership: &BTreeMap<PathBuf, FileOwnership<'_>>,
    include_traceability: bool,
) -> Box<dyn LanguagePackAnalysisContext> {
    Box::new(analyze::build_repo_analysis_context(
        root,
        source_files,
        scoped_source_files,
        parsed_repo,
        parsed_architecture,
        file_ownership,
        include_traceability,
    ))
}

fn analysis_environment_fingerprint(_root: &Path) -> String {
    "python-parser-backed-v1".to_string()
}

fn is_python_path(path: &Path) -> bool {
    has_extension(path, &["py"])
}

fn parse_source_graph(path: &Path, text: &str) -> Option<ParsedSourceGraph> {
    crate::syntax::python::parse_source_graph(path, text)
}
