/**
@module SPECIAL.LANGUAGE_PACKS.GO
Registers the built-in Go language pack with the shared compile-time pack registry, delegating parsing and implementation analysis to Go-owned provider code without reintroducing hardcoded dispatch into syntax or analysis cores.
*/
// @fileimplements SPECIAL.LANGUAGE_PACKS.GO
use std::collections::BTreeMap;
use std::path::{Path, PathBuf};

use anyhow::Result;

use super::{
    LanguagePackAnalysisContext, LanguagePackDescriptor, TraceabilityGraphFactsDescriptor,
};
use crate::model::{
    ArchitectureTraceabilitySummary, ImplementRef, ModuleAnalysisOptions, ParsedArchitecture,
    ParsedRepo,
};
use crate::modules::analyze::{FileOwnership, ProviderModuleAnalysis};
use crate::syntax::{ParsedSourceGraph, SourceLanguage};

#[path = "go/analyze.rs"]
pub(crate) mod analyze;

pub(crate) const DESCRIPTOR: LanguagePackDescriptor = LanguagePackDescriptor {
    language: SourceLanguage::new("go"),
    matches_path: is_go_path,
    parse_source_graph: parse_source_graph,
    build_repo_analysis_context: build_repo_analysis_context,
    analysis_environment_fingerprint: analysis_environment_fingerprint,
    traceability_scope_facts: None,
    traceability_graph_facts: Some(&TRACEABILITY_GRAPH_FACTS),
};

const TRACEABILITY_GRAPH_FACTS: TraceabilityGraphFactsDescriptor = TraceabilityGraphFactsDescriptor {
    build_facts: build_traceability_graph_facts,
};

impl LanguagePackAnalysisContext for analyze::GoRepoAnalysisContext {
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

fn build_repo_analysis_context(
    root: &Path,
    source_files: &[PathBuf],
    scoped_source_files: Option<&[PathBuf]>,
    traceability_graph_facts: Option<&[u8]>,
    parsed_repo: &ParsedRepo,
    parsed_architecture: &ParsedArchitecture,
    file_ownership: &BTreeMap<PathBuf, FileOwnership<'_>>,
    include_traceability: bool,
) -> Box<dyn LanguagePackAnalysisContext> {
    Box::new(analyze::build_repo_analysis_context(
        root,
        source_files,
        scoped_source_files,
        traceability_graph_facts,
        parsed_repo,
        parsed_architecture,
        file_ownership,
        include_traceability,
    ))
}

fn analysis_environment_fingerprint(_root: &Path) -> String {
    analyze::analysis_environment_fingerprint()
}

fn build_traceability_graph_facts(root: &Path, source_files: &[PathBuf]) -> Result<Vec<u8>> {
    analyze::build_traceability_graph_facts(root, source_files)
}

fn is_go_path(path: &Path) -> bool {
    path.extension().and_then(|ext| ext.to_str()) == Some("go")
}

fn parse_source_graph(path: &Path, text: &str) -> Option<ParsedSourceGraph> {
    crate::syntax::go::parse_source_graph(path, text)
}
