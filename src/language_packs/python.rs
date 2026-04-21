/**
@module SPECIAL.LANGUAGE_PACKS.PYTHON
Registers the built-in Python language pack with the shared compile-time pack registry. This pack lives under `src/language_packs/python/`, uses Python's own `ast` as its parser baseline, and should only surface backward trace when the local `pyright-langserver` tool is available so Python traceability stays on the same union-graph contract as the other built-in packs.
*/
// @fileimplements SPECIAL.LANGUAGE_PACKS.PYTHON
use std::collections::{BTreeMap, BTreeSet};
use std::path::{Path, PathBuf};

use anyhow::Result;
use serde::{Deserialize, Serialize};

use super::{
    LanguagePackAnalysisContext, LanguagePackDescriptor, TraceabilityGraphFactsDescriptor,
};
use crate::model::{
    ArchitectureTraceabilitySummary, ImplementRef, ModuleAnalysisOptions, ModuleItemKind,
    ModuleMetricsSummary, ParsedArchitecture, ParsedRepo,
};
use crate::modules::analyze::{
    FileOwnership, ProviderModuleAnalysis, emit_analysis_status, read_owned_file_text,
    source_item_signals::summarize_source_item_signals,
    traceability_core::{
        BackwardTraceAvailability, TraceGraph, TraceabilityAnalysis, TraceabilityInputs,
        TraceabilityLanguagePack, TraceabilityOwnedItem, build_root_supports,
        build_traceability_analysis, merge_trace_graph_edges, owned_module_ids_for_path,
        summarize_module_traceability,
        summarize_repo_traceability as summarize_shared_repo_traceability,
    },
    visit_owned_texts,
};
use crate::syntax::{ParsedSourceGraph, SourceCall, SourceLanguage};

#[path = "python/ast_bridge.rs"]
mod ast_bridge;
#[path = "python/pyright_langserver.rs"]
mod pyright_langserver;

pub(crate) const DESCRIPTOR: LanguagePackDescriptor = LanguagePackDescriptor {
    language: SourceLanguage::new("python"),
    matches_path: is_python_path,
    parse_source_graph,
    build_repo_analysis_context,
    analysis_environment_fingerprint,
    traceability_scope_facts: None,
    traceability_graph_facts: Some(&TRACEABILITY_GRAPH_FACTS),
};

const TRACEABILITY_GRAPH_FACTS: TraceabilityGraphFactsDescriptor = TraceabilityGraphFactsDescriptor {
    build_facts: build_traceability_graph_facts,
};

pub(crate) struct PythonRepoAnalysisContext {
    traceability_pack: PythonTraceabilityPack,
    traceability: Option<TraceabilityAnalysis>,
    traceability_unavailable_reason: Option<String>,
}

fn build_traceability_graph_facts(root: &Path, source_files: &[PathBuf]) -> Result<Vec<u8>> {
    let source_graphs = parse_python_source_graphs_result(root, source_files)?;
    let facts = PythonTraceabilityGraphFacts {
        source_graphs: source_graphs
            .iter()
            .map(|(path, graph)| (path.clone(), CachedParsedSourceGraph::from_parsed(graph)))
            .collect(),
        parser_edges: build_parser_call_edges(&source_graphs),
    };
    Ok(serde_json::to_vec(&facts)?)
}

#[derive(Debug, Clone, Copy)]
struct PythonTraceabilityPack;

impl LanguagePackAnalysisContext for PythonRepoAnalysisContext {
    fn summarize_repo_traceability(
        &self,
        root: &Path,
    ) -> Option<ArchitectureTraceabilitySummary> {
        self.traceability
            .as_ref()
            .map(|analysis| summarize_shared_repo_traceability(root, analysis))
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
        analyze_module(
            root,
            implementations,
            file_ownership,
            self,
            options.traceability,
        )
    }
}

impl TraceabilityLanguagePack for PythonTraceabilityPack {
    fn backward_trace_availability(&self) -> BackwardTraceAvailability {
        if pyright_langserver_available() {
            BackwardTraceAvailability::default()
        } else {
            BackwardTraceAvailability::unavailable(
                "Python backward trace is unavailable because `pyright-langserver` is not installed",
            )
        }
    }

    fn owned_items_for_implementations(
        &self,
        root: &Path,
        implementations: &[&ImplementRef],
        file_ownership: &BTreeMap<PathBuf, FileOwnership<'_>>,
    ) -> Vec<TraceabilityOwnedItem> {
        collect_owned_items(root, implementations, file_ownership)
    }
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
) -> Box<dyn LanguagePackAnalysisContext> {
    let traceability_pack = PythonTraceabilityPack;
    let traceability_unavailable_reason = traceability_pack
        .backward_trace_availability()
        .unavailable_reason()
        .map(ToString::to_string);
    let (traceability, traceability_unavailable_reason) =
        if include_traceability && traceability_unavailable_reason.is_none() {
            let build_result = if let Some(scoped_source_files) =
                scoped_source_files.filter(|files| !files.is_empty())
            {
                build_scoped_traceability_analysis_for_python(
                    root,
                    source_files,
                    scoped_source_files,
                    traceability_graph_facts,
                    parsed_repo,
                    file_ownership,
                )
            } else {
                build_traceability_analysis_for_python(
                    root,
                    source_files,
                    traceability_graph_facts,
                    parsed_repo,
                    parsed_architecture,
                    file_ownership,
                )
            };
            match build_result {
                Ok(analysis) => (Some(analysis), None),
                Err(error) => (
                    None,
                    Some(format!(
                        "Python backward trace is unavailable because syntax or tool-backed trace setup failed: {error}"
                    )),
                ),
            }
        } else {
            (None, traceability_unavailable_reason)
        };
    Box::new(PythonRepoAnalysisContext {
        traceability_pack,
        traceability,
        traceability_unavailable_reason,
    })
}

fn analyze_module(
    root: &Path,
    implementations: &[&ImplementRef],
    file_ownership: &BTreeMap<PathBuf, FileOwnership<'_>>,
    context: &PythonRepoAnalysisContext,
    include_traceability: bool,
) -> Result<ProviderModuleAnalysis> {
    let mut owned_items = Vec::new();
    let mut public_items = 0;
    let mut internal_items = 0;

    visit_owned_texts(root, implementations, file_ownership, |path, text| {
        if !is_python_path(path) {
            return Ok(());
        }
        if let Some(graph) = parse_source_graph(path, text) {
            for item in &graph.items {
                if item.public {
                    public_items += 1;
                } else {
                    internal_items += 1;
                }
            }
            owned_items.extend(graph.items);
        }
        Ok(())
    })?;

    let traceability_summary = include_traceability
        .then_some(context.traceability.as_ref())
        .flatten()
        .map(|analysis| {
            let owned_items = context.traceability_pack.owned_items_for_implementations(
                root,
                implementations,
                file_ownership,
            );
            summarize_module_traceability(&owned_items, analysis)
        });

    Ok(ProviderModuleAnalysis {
        metrics: ModuleMetricsSummary {
            public_items,
            internal_items,
            ..ModuleMetricsSummary::default()
        },
        item_signals: Some(summarize_source_item_signals(&owned_items)),
        traceability: traceability_summary,
        traceability_unavailable_reason: include_traceability
            .then(|| context.traceability_unavailable_reason.clone())
            .flatten(),
        ..ProviderModuleAnalysis::default()
    })
}

fn is_python_path(path: &Path) -> bool {
    path.extension().and_then(|ext| ext.to_str()) == Some("py")
}

fn parse_source_graph(path: &Path, text: &str) -> Option<ParsedSourceGraph> {
    ast_bridge::parse_source_graph(path, text)
}

fn pyright_langserver_available() -> bool {
    pyright_langserver::available()
}

fn analysis_environment_fingerprint(_root: &Path) -> String {
    pyright_langserver::environment_fingerprint()
}

#[derive(Debug, Clone, Default)]
struct CallableIndexes {
    global_name_counts: BTreeMap<String, usize>,
    same_file_name_counts: BTreeMap<(PathBuf, String), usize>,
    global_qualified_name_counts: BTreeMap<String, usize>,
}

fn parse_python_source_graphs_result(
    root: &Path,
    source_files: &[PathBuf],
) -> Result<BTreeMap<PathBuf, ParsedSourceGraph>> {
    let mut graphs = BTreeMap::new();
    for path in source_files.iter().filter(|path| is_python_path(path)) {
        let text = read_owned_file_text(root, path)?;
        let graph = ast_bridge::parse_source_graph_result(path, &text)?;
        graphs.insert(path.clone(), graph);
    }
    Ok(graphs)
}

fn collect_repo_items(
    source_graphs: &BTreeMap<PathBuf, ParsedSourceGraph>,
    file_ownership: &BTreeMap<PathBuf, FileOwnership<'_>>,
) -> Vec<TraceabilityOwnedItem> {
    let mut items = source_graphs
        .iter()
        .flat_map(|(path, graph)| {
            let module_ids = owned_module_ids_for_path(file_ownership, path);
            let test_file = is_test_file_path(path);
            graph
                .items
                .iter()
                .filter(|item| !item.is_test)
                .map(move |item| TraceabilityOwnedItem {
                    stable_id: item.stable_id.clone(),
                    name: item.name.clone(),
                    kind: source_item_kind(item.kind),
                    path: path.clone(),
                    public: item.public,
                    review_surface: item.public && !test_file,
                    test_file,
                    module_ids: module_ids.clone(),
                    mediated_reason: None,
                })
        })
        .collect::<Vec<_>>();
    items.sort_by(|left, right| {
        left.path
            .cmp(&right.path)
            .then_with(|| left.name.cmp(&right.name))
            .then_with(|| left.kind.cmp(&right.kind))
    });
    items
}

fn collect_owned_items(
    root: &Path,
    implementations: &[&ImplementRef],
    file_ownership: &BTreeMap<PathBuf, FileOwnership<'_>>,
) -> Vec<TraceabilityOwnedItem> {
    let mut items = Vec::new();
    let mut seen = BTreeSet::new();
    for implementation in implementations {
        if !is_python_path(&implementation.location.path) {
            continue;
        }
        let Some(graph) = parse_owned_implementation_graph(root, implementation, file_ownership)
        else {
            continue;
        };
        let test_file = is_test_file_path(&implementation.location.path);
        for item in graph.items.into_iter().filter(|item| !item.is_test) {
            if !seen.insert(item.stable_id.clone()) {
                continue;
            }
            items.push(TraceabilityOwnedItem {
                stable_id: item.stable_id,
                name: item.name,
                kind: source_item_kind(item.kind),
                path: implementation.location.path.clone(),
                public: item.public,
                review_surface: item.public && !test_file,
                test_file,
                module_ids: vec![implementation.module_id.clone()],
                mediated_reason: None,
            });
        }
    }
    items
}

fn parse_owned_implementation_graph(
    root: &Path,
    implementation: &ImplementRef,
    file_ownership: &BTreeMap<PathBuf, FileOwnership<'_>>,
) -> Option<ParsedSourceGraph> {
    if let Some(body) = &implementation.body {
        return parse_source_graph(&implementation.location.path, body);
    }

    let ownership = file_ownership.get(&implementation.location.path)?;
    if !ownership.item_scoped.is_empty() {
        return None;
    }

    let text = read_owned_file_text(root, &implementation.location.path).ok()?;
    parse_source_graph(&implementation.location.path, &text)
}

fn build_parser_call_edges(
    source_graphs: &BTreeMap<PathBuf, ParsedSourceGraph>,
) -> BTreeMap<String, BTreeSet<String>> {
    let callable_items = collect_callable_items(source_graphs);
    let indexes = build_callable_indexes(&callable_items);
    let mut edges = BTreeMap::new();

    for item in &callable_items {
        let callees = item
            .calls
            .iter()
            .flat_map(|call| resolve_call_targets(call, item, &callable_items, &indexes))
            .collect::<BTreeSet<_>>();
        if !callees.is_empty() {
            edges.insert(item.stable_id.clone(), callees);
        }
    }

    edges
}

fn build_tool_call_edges(
    root: &Path,
    source_graphs: &BTreeMap<PathBuf, ParsedSourceGraph>,
) -> Result<BTreeMap<String, BTreeSet<String>>> {
    let callable_items = collect_callable_items(source_graphs);
    if callable_items.is_empty() {
        return Ok(BTreeMap::new());
    }
    pyright_langserver::build_reachable_call_edges(root, &callable_items)
}

fn build_traceability_analysis_for_python(
    root: &Path,
    source_files: &[PathBuf],
    traceability_graph_facts: Option<&[u8]>,
    parsed_repo: &ParsedRepo,
    parsed_architecture: &ParsedArchitecture,
    file_ownership: &BTreeMap<PathBuf, FileOwnership<'_>>,
) -> Result<TraceabilityAnalysis> {
    let (source_graphs, parser_edges) =
        if let Some(decoded) = decode_traceability_graph_facts(traceability_graph_facts) {
            decoded
        } else {
            let source_graphs = parse_python_source_graphs_result(root, source_files)?;
            let parser_edges = build_parser_call_edges(&source_graphs);
            (source_graphs, parser_edges)
        };
    let repo_items = collect_repo_items(&source_graphs, file_ownership);
    let mut graph = TraceGraph {
        edges: parser_edges,
        root_supports: BTreeMap::new(),
    };
    merge_trace_graph_edges(
        &mut graph.edges,
        build_tool_call_edges(root, &source_graphs)?,
    );
    graph.root_supports = build_root_supports(parsed_repo, &source_graphs, |path, body| {
        parse_source_graph(path, body).and_then(|graph| graph.items.first().map(|item| item.span.start_line))
    });
    let _ = parsed_architecture;
    Ok(build_traceability_analysis(TraceabilityInputs { repo_items, graph }))
}

fn build_scoped_traceability_analysis_for_python(
    root: &Path,
    source_files: &[PathBuf],
    scoped_source_files: &[PathBuf],
    traceability_graph_facts: Option<&[u8]>,
    parsed_repo: &ParsedRepo,
    file_ownership: &BTreeMap<PathBuf, FileOwnership<'_>>,
) -> Result<TraceabilityAnalysis> {
    let (source_graphs, parser_edges) =
        if let Some(decoded) = decode_traceability_graph_facts(traceability_graph_facts) {
            decoded
        } else {
            let source_graphs = parse_python_source_graphs_result(root, source_files)?;
            let parser_edges = build_parser_call_edges(&source_graphs);
            (source_graphs, parser_edges)
        };
    let repo_items = select_scoped_repo_items(
        collect_repo_items(&source_graphs, file_ownership),
        scoped_source_files,
    );
    let scoped_seed_ids = repo_items
        .iter()
        .map(|item| item.stable_id.clone())
        .collect::<BTreeSet<_>>();
    emit_analysis_status(&format!(
        "python scoped traceability targets {} item(s) across {} file(s)",
        scoped_seed_ids.len(),
        repo_items
            .iter()
            .map(|item| item.path.clone())
            .collect::<BTreeSet<_>>()
            .len()
    ));
    let mut graph = TraceGraph {
        edges: parser_edges.clone(),
        root_supports: BTreeMap::new(),
    };
    merge_trace_graph_edges(
        &mut graph.edges,
        pyright_langserver::build_reverse_reachable_call_edges(
            root,
            &collect_callable_items(&source_graphs),
            &scoped_seed_ids,
            &parser_edges,
        )?,
    );
    graph.root_supports =
        build_root_supports(parsed_repo, &source_graphs, |path, body| {
            parse_source_graph(path, body)
                .and_then(|graph| graph.items.first().map(|item| item.span.start_line))
        });
    Ok(build_traceability_analysis(TraceabilityInputs { repo_items, graph }))
}

fn collect_callable_items(
    source_graphs: &BTreeMap<PathBuf, ParsedSourceGraph>,
) -> Vec<pyright_langserver::PyrightCallableItem> {
    let mut items = source_graphs
        .iter()
        .flat_map(|(path, graph)| {
            graph.items.iter().map(move |item| pyright_langserver::PyrightCallableItem {
                stable_id: item.stable_id.clone(),
                name: item.name.clone(),
                qualified_name: item.qualified_name.clone(),
                path: path.clone(),
                span: item.span,
                calls: item.calls.clone(),
                is_test: item.is_test,
            })
        })
        .collect::<Vec<_>>();
    items.sort_by(|left, right| {
        left.path
            .cmp(&right.path)
            .then_with(|| left.span.start_line.cmp(&right.span.start_line))
            .then_with(|| left.name.cmp(&right.name))
    });
    items
}

fn build_callable_indexes(items: &[pyright_langserver::PyrightCallableItem]) -> CallableIndexes {
    let mut indexes = CallableIndexes::default();
    for item in items {
        *indexes.global_name_counts.entry(item.name.clone()).or_default() += 1;
        *indexes
            .same_file_name_counts
            .entry((item.path.clone(), item.name.clone()))
            .or_default() += 1;
        *indexes
            .global_qualified_name_counts
            .entry(item.qualified_name.clone())
            .or_default() += 1;
    }
    indexes
}

fn resolve_call_targets(
    call: &SourceCall,
    caller: &pyright_langserver::PyrightCallableItem,
    items: &[pyright_langserver::PyrightCallableItem],
    indexes: &CallableIndexes,
) -> Vec<String> {
    if let Some(target) = resolve_exact_qualified_target(call, items, indexes) {
        return vec![target];
    }

    let same_file_matches = items
        .iter()
        .filter(|item| {
            item.path == caller.path
                && item.name == call.name
                && item.stable_id != caller.stable_id
                && indexes
                    .same_file_name_counts
                    .get(&(item.path.clone(), item.name.clone()))
                    .copied()
                    == Some(1)
        })
        .map(|item| item.stable_id.clone())
        .collect::<Vec<_>>();
    if same_file_matches.len() == 1 {
        return same_file_matches;
    }

    let global_matches = items
        .iter()
        .filter(|item| {
            item.name == call.name
                && item.stable_id != caller.stable_id
                && indexes.global_name_counts.get(&item.name).copied() == Some(1)
        })
        .map(|item| item.stable_id.clone())
        .collect::<Vec<_>>();
    if global_matches.len() == 1 {
        return global_matches;
    }

    Vec::new()
}

fn resolve_exact_qualified_target(
    call: &SourceCall,
    items: &[pyright_langserver::PyrightCallableItem],
    indexes: &CallableIndexes,
) -> Option<String> {
    let qualifier = call.qualifier.as_deref()?;
    let qualified = normalize_qualified_target(qualifier, &call.name);
    if indexes.global_qualified_name_counts.get(&qualified).copied() != Some(1) {
        return None;
    }
    items.iter()
        .find(|item| item.qualified_name == qualified)
        .map(|item| item.stable_id.clone())
}

fn select_scoped_repo_items(
    repo_items: Vec<TraceabilityOwnedItem>,
    scoped_source_files: &[PathBuf],
) -> Vec<TraceabilityOwnedItem> {
    let scoped_file_set = scoped_source_files.iter().cloned().collect::<BTreeSet<_>>();
    let scoped_module_ids = repo_items
        .iter()
        .filter(|item| scoped_file_set.contains(&item.path))
        .flat_map(|item| item.module_ids.iter().cloned())
        .collect::<BTreeSet<_>>();

    repo_items
        .into_iter()
        .filter(|item| {
            scoped_file_set.contains(&item.path)
                || item
                    .module_ids
                    .iter()
                    .any(|module_id| scoped_module_ids.contains(module_id))
        })
        .collect()
}

#[derive(Serialize, Deserialize)]
struct PythonTraceabilityGraphFacts {
    source_graphs: BTreeMap<PathBuf, CachedParsedSourceGraph>,
    parser_edges: BTreeMap<String, BTreeSet<String>>,
}

type PythonGraphFactsDecoded = (
    BTreeMap<PathBuf, ParsedSourceGraph>,
    BTreeMap<String, BTreeSet<String>>,
);

fn decode_traceability_graph_facts(
    facts: Option<&[u8]>,
) -> Option<PythonGraphFactsDecoded> {
    let facts = facts?;
    let facts = serde_json::from_slice::<PythonTraceabilityGraphFacts>(facts).ok()?;
    Some((
        facts
            .source_graphs
            .into_iter()
            .map(|(path, graph)| (path, graph.into_parsed()))
            .collect(),
        facts.parser_edges,
    ))
}

#[derive(Serialize, Deserialize)]
struct CachedParsedSourceGraph {
    items: Vec<CachedSourceItem>,
}

impl CachedParsedSourceGraph {
    fn from_parsed(graph: &ParsedSourceGraph) -> Self {
        Self {
            items: graph.items.iter().map(CachedSourceItem::from_parsed).collect(),
        }
    }

    fn into_parsed(self) -> ParsedSourceGraph {
        ParsedSourceGraph {
            language: SourceLanguage::new("python"),
            items: self
                .items
                .into_iter()
                .map(CachedSourceItem::into_parsed)
                .collect(),
        }
    }
}

#[derive(Serialize, Deserialize)]
struct CachedSourceItem {
    source_path: String,
    stable_id: String,
    name: String,
    qualified_name: String,
    module_path: Vec<String>,
    container_path: Vec<String>,
    shape_fingerprint: String,
    shape_node_count: usize,
    kind: CachedSourceItemKind,
    span: CachedSourceSpan,
    public: bool,
    root_visible: bool,
    is_test: bool,
    calls: Vec<CachedSourceCall>,
    invocations: Vec<CachedSourceInvocation>,
}

impl CachedSourceItem {
    fn from_parsed(item: &crate::syntax::SourceItem) -> Self {
        Self {
            source_path: item.source_path.clone(),
            stable_id: item.stable_id.clone(),
            name: item.name.clone(),
            qualified_name: item.qualified_name.clone(),
            module_path: item.module_path.clone(),
            container_path: item.container_path.clone(),
            shape_fingerprint: item.shape_fingerprint.clone(),
            shape_node_count: item.shape_node_count,
            kind: CachedSourceItemKind::from_parsed(item.kind),
            span: CachedSourceSpan::from_parsed(item.span),
            public: item.public,
            root_visible: item.root_visible,
            is_test: item.is_test,
            calls: item.calls.iter().map(CachedSourceCall::from_parsed).collect(),
            invocations: item
                .invocations
                .iter()
                .map(CachedSourceInvocation::from_parsed)
                .collect(),
        }
    }

    fn into_parsed(self) -> crate::syntax::SourceItem {
        crate::syntax::SourceItem {
            source_path: self.source_path,
            stable_id: self.stable_id,
            name: self.name,
            qualified_name: self.qualified_name,
            module_path: self.module_path,
            container_path: self.container_path,
            shape_fingerprint: self.shape_fingerprint,
            shape_node_count: self.shape_node_count,
            kind: self.kind.into_parsed(),
            span: self.span.into_parsed(),
            public: self.public,
            root_visible: self.root_visible,
            is_test: self.is_test,
            calls: self
                .calls
                .into_iter()
                .map(CachedSourceCall::into_parsed)
                .collect(),
            invocations: self
                .invocations
                .into_iter()
                .map(CachedSourceInvocation::into_parsed)
                .collect(),
        }
    }
}

#[derive(Serialize, Deserialize)]
enum CachedSourceItemKind {
    Function,
    Method,
}

impl CachedSourceItemKind {
    fn from_parsed(kind: crate::syntax::SourceItemKind) -> Self {
        match kind {
            crate::syntax::SourceItemKind::Function => Self::Function,
            crate::syntax::SourceItemKind::Method => Self::Method,
        }
    }

    fn into_parsed(self) -> crate::syntax::SourceItemKind {
        match self {
            Self::Function => crate::syntax::SourceItemKind::Function,
            Self::Method => crate::syntax::SourceItemKind::Method,
        }
    }
}

#[derive(Serialize, Deserialize)]
struct CachedSourceSpan {
    start_line: usize,
    end_line: usize,
    start_column: usize,
    end_column: usize,
    start_byte: usize,
    end_byte: usize,
}

impl CachedSourceSpan {
    fn from_parsed(span: crate::syntax::SourceSpan) -> Self {
        Self {
            start_line: span.start_line,
            end_line: span.end_line,
            start_column: span.start_column,
            end_column: span.end_column,
            start_byte: span.start_byte,
            end_byte: span.end_byte,
        }
    }

    fn into_parsed(self) -> crate::syntax::SourceSpan {
        crate::syntax::SourceSpan {
            start_line: self.start_line,
            end_line: self.end_line,
            start_column: self.start_column,
            end_column: self.end_column,
            start_byte: self.start_byte,
            end_byte: self.end_byte,
        }
    }
}

#[derive(Serialize, Deserialize)]
struct CachedSourceCall {
    name: String,
    qualifier: Option<String>,
    syntax: CachedCallSyntaxKind,
    span: CachedSourceSpan,
}

impl CachedSourceCall {
    fn from_parsed(call: &SourceCall) -> Self {
        Self {
            name: call.name.clone(),
            qualifier: call.qualifier.clone(),
            syntax: CachedCallSyntaxKind::from_parsed(call.syntax.clone()),
            span: CachedSourceSpan::from_parsed(call.span),
        }
    }

    fn into_parsed(self) -> SourceCall {
        SourceCall {
            name: self.name,
            qualifier: self.qualifier,
            syntax: self.syntax.into_parsed(),
            span: self.span.into_parsed(),
        }
    }
}

#[derive(Serialize, Deserialize)]
enum CachedCallSyntaxKind {
    Identifier,
    ScopedIdentifier,
    Field,
}

impl CachedCallSyntaxKind {
    fn from_parsed(kind: crate::syntax::CallSyntaxKind) -> Self {
        match kind {
            crate::syntax::CallSyntaxKind::Identifier => Self::Identifier,
            crate::syntax::CallSyntaxKind::ScopedIdentifier => Self::ScopedIdentifier,
            crate::syntax::CallSyntaxKind::Field => Self::Field,
        }
    }

    fn into_parsed(self) -> crate::syntax::CallSyntaxKind {
        match self {
            Self::Identifier => crate::syntax::CallSyntaxKind::Identifier,
            Self::ScopedIdentifier => crate::syntax::CallSyntaxKind::ScopedIdentifier,
            Self::Field => crate::syntax::CallSyntaxKind::Field,
        }
    }
}

#[derive(Serialize, Deserialize)]
struct CachedSourceInvocation {
    span: CachedSourceSpan,
    kind: CachedSourceInvocationKind,
}

impl CachedSourceInvocation {
    fn from_parsed(invocation: &crate::syntax::SourceInvocation) -> Self {
        Self {
            span: CachedSourceSpan::from_parsed(invocation.span),
            kind: CachedSourceInvocationKind::from_parsed(&invocation.kind),
        }
    }

    fn into_parsed(self) -> crate::syntax::SourceInvocation {
        crate::syntax::SourceInvocation {
            span: self.span.into_parsed(),
            kind: self.kind.into_parsed(),
        }
    }
}

#[derive(Serialize, Deserialize)]
enum CachedSourceInvocationKind {
    LocalCargoBinary { binary_name: String },
}

impl CachedSourceInvocationKind {
    fn from_parsed(kind: &crate::syntax::SourceInvocationKind) -> Self {
        match kind {
            crate::syntax::SourceInvocationKind::LocalCargoBinary { binary_name } => {
                Self::LocalCargoBinary {
                    binary_name: binary_name.clone(),
                }
            }
        }
    }

    fn into_parsed(self) -> crate::syntax::SourceInvocationKind {
        match self {
            Self::LocalCargoBinary { binary_name } => {
                crate::syntax::SourceInvocationKind::LocalCargoBinary { binary_name }
            }
        }
    }
}

fn normalize_qualified_target(qualifier: &str, name: &str) -> String {
    qualifier
        .split('.')
        .chain(std::iter::once(name))
        .collect::<Vec<_>>()
        .join("::")
}

fn is_test_file_path(path: &Path) -> bool {
    path.file_name()
        .and_then(|name| name.to_str())
        .is_some_and(|name| name.starts_with("test_") || name.ends_with("_test.py"))
}

fn source_item_kind(kind: crate::syntax::SourceItemKind) -> ModuleItemKind {
    match kind {
        crate::syntax::SourceItemKind::Function => ModuleItemKind::Function,
        crate::syntax::SourceItemKind::Method => ModuleItemKind::Method,
    }
}
