/**
@module SPECIAL.LANGUAGE_PACKS.GO.ANALYZE
Owns the built-in Go implementation analysis provider, including pack-specific traceability setup and tool discovery while depending on shared analysis core only through protocolized helpers.
*/
// @fileimplements SPECIAL.LANGUAGE_PACKS.GO.ANALYZE
use std::collections::BTreeMap;
use std::path::{Path, PathBuf};

use anyhow::Result;

use crate::model::{
    ArchitectureTraceabilitySummary, ImplementRef, ModuleMetricsSummary, ParsedArchitecture,
    ParsedRepo,
};

use self::dependencies::GoDependencySummary;
use self::surface::{GoSurfaceSummary, is_go_path};
use self::traceability::GoTraceabilityPack;
use crate::modules::analyze::source_item_signals::summarize_source_item_signals;
use crate::modules::analyze::traceability_core::{
    TraceabilityAnalysis, TraceabilityLanguagePack,
};
use crate::modules::analyze::{FileOwnership, ProviderModuleAnalysis, visit_owned_texts};

#[path = "analyze/dependencies.rs"]
mod dependencies;
#[path = "analyze/surface.rs"]
mod surface;
#[path = "analyze/toolchain.rs"]
mod toolchain;
#[path = "analyze/traceability.rs"]
mod traceability;

pub(crate) fn analyze_module(
    root: &Path,
    implementations: &[&ImplementRef],
    file_ownership: &BTreeMap<PathBuf, FileOwnership<'_>>,
    context: &GoRepoAnalysisContext,
    include_traceability: bool,
) -> Result<ProviderModuleAnalysis> {
    let mut surface = GoSurfaceSummary::default();
    let mut owned_items = Vec::new();
    let mut dependencies = GoDependencySummary::default();
    let traceability_summary = include_traceability
        .then_some(context.traceability.as_ref())
        .flatten()
        .map(|traceability| {
            let owned_items = context.traceability_pack.owned_items_for_implementations(
                root,
                implementations,
                file_ownership,
            );
            crate::modules::analyze::traceability_core::summarize_module_traceability(
                &owned_items,
                traceability,
            )
        });

    visit_owned_texts(root, implementations, file_ownership, |path, text| {
        if !is_go_path(path) {
            return Ok(());
        }
        if let Some(graph) = crate::syntax::parse_source_graph(path, text) {
            surface.observe(&graph.items);
            owned_items.extend(graph.items);
        }
        dependencies.observe(root, text);
        Ok(())
    })?;

    Ok(ProviderModuleAnalysis {
        metrics: ModuleMetricsSummary {
            public_items: surface.public_items,
            internal_items: surface.internal_items,
            ..ModuleMetricsSummary::default()
        },
        item_signals: Some(summarize_source_item_signals(&owned_items)),
        traceability: traceability_summary,
        traceability_unavailable_reason: include_traceability
            .then(|| context.traceability_unavailable_reason.clone())
            .flatten(),
        coupling: Some(dependencies.coupling_input()),
        dependencies: Some(dependencies.summary()),
        ..ProviderModuleAnalysis::default()
    })
}

pub(crate) struct GoRepoAnalysisContext {
    traceability_pack: GoTraceabilityPack,
    traceability: Option<TraceabilityAnalysis>,
    pub(crate) traceability_unavailable_reason: Option<String>,
}

pub(crate) fn build_traceability_graph_facts(
    root: &Path,
    source_files: &[PathBuf],
) -> Result<Vec<u8>> {
    traceability::build_traceability_graph_facts(root, source_files)
}

#[allow(clippy::too_many_arguments)]
pub(crate) fn build_repo_analysis_context(
    root: &Path,
    source_files: &[PathBuf],
    scoped_source_files: Option<&[PathBuf]>,
    traceability_graph_facts: Option<&[u8]>,
    parsed_repo: &ParsedRepo,
    parsed_architecture: &ParsedArchitecture,
    file_ownership: &BTreeMap<PathBuf, FileOwnership<'_>>,
    include_traceability: bool,
) -> GoRepoAnalysisContext {
    let traceability_pack = GoTraceabilityPack;
    let traceability_unavailable_reason = traceability_pack
        .backward_trace_availability()
        .unavailable_reason()
        .map(ToString::to_string);
    let traceability = (include_traceability && traceability_unavailable_reason.is_none()).then(|| {
        if let Some(scoped_source_files) = scoped_source_files.filter(|files| !files.is_empty()) {
            traceability::build_scoped_traceability_analysis_from_cached_or_live_graph_facts(
                root,
                source_files,
                scoped_source_files,
                traceability_graph_facts,
                parsed_repo,
                file_ownership,
            )
        } else {
            traceability::build_traceability_analysis_from_cached_or_live_graph_facts(
                root,
                source_files,
                traceability_graph_facts,
                parsed_repo,
                parsed_architecture,
                file_ownership,
                &traceability_pack,
            )
        }
    });
    GoRepoAnalysisContext {
        traceability_pack,
        traceability,
        traceability_unavailable_reason,
    }
}

pub(crate) fn summarize_repo_traceability(
    root: &Path,
    context: &GoRepoAnalysisContext,
) -> Option<ArchitectureTraceabilitySummary> {
    context
        .traceability
        .as_ref()
        .map(|traceability| traceability::summarize_repo_traceability(root, traceability))
}

pub(crate) fn analysis_environment_fingerprint() -> String {
    toolchain::analysis_environment_fingerprint()
}
