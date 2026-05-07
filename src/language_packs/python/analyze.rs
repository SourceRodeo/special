/**
@module SPECIAL.LANGUAGE_PACKS.PYTHON.ANALYZE
Owns the built-in Python implementation analysis provider for parser-backed static Python semantics, reusing the shared traceability core and projected scoped-health kernel without Python runtime introspection.
*/
// @fileimplements SPECIAL.LANGUAGE_PACKS.PYTHON.ANALYZE
use std::collections::{BTreeMap, BTreeSet};
use std::path::{Path, PathBuf};

use anyhow::{Context, Result, anyhow, bail};

use crate::model::{
    ArchitectureTraceabilitySummary, ImplementRef, ModuleMetricsSummary, ParsedArchitecture,
    ParsedRepo,
};
use crate::modules::analyze::source_item_signals::summarize_source_item_signals_with_metrics;
use crate::modules::analyze::traceability_core::{
    TraceGraph, TraceabilityAnalysis, TraceabilityInputs, TraceabilityLanguagePack,
    TraceabilityOwnedItem, build_projected_traceability_reference_from_projected_items,
    build_root_supports, build_traceability_analysis, owned_module_ids_for_path,
    summarize_module_traceability, summarize_repo_traceability as summarize_shared_repo_traceability,
};
use crate::modules::analyze::{
    FileOwnership, ModuleCouplingInput, ProviderModuleAnalysis, build_dependency_summary,
    emit_analysis_status, read_owned_file_text, visit_owned_texts,
};
use crate::syntax::{ParsedSourceGraph, SourceCall, parse_source_graph};
use tree_sitter::{Node, Parser};

// @applies ADAPTER.FACTS_TO_MODEL
pub(crate) fn analyze_module(
    root: &Path,
    implementations: &[&ImplementRef],
    file_ownership: &BTreeMap<PathBuf, FileOwnership<'_>>,
    context: &PythonRepoAnalysisContext,
    include_traceability: bool,
) -> Result<ProviderModuleAnalysis> {
    let mut surface = PythonSurfaceSummary::default();
    let mut dependencies = PythonDependencySummary::default();
    let mut owned_items = Vec::new();
    let traceability_summary = include_traceability
        .then_some(context.traceability.as_ref())
        .flatten()
        .map(|traceability| {
            let owned_items = context.traceability_pack.owned_items_for_implementations(
                root,
                implementations,
                file_ownership,
            );
            summarize_module_traceability(&owned_items, traceability)
        });

    visit_owned_texts(root, implementations, file_ownership, |path, text| {
        if !is_python_path(path) {
            return Ok(());
        }
        let graph = parse_source_graph(path, text)
            .ok_or_else(|| anyhow!("failed to parse Python source graph for {}", path.display()))?;
        surface.observe(&graph.items);
        owned_items.extend(graph.items);
        let imports = parse_python_import_index(path, text)
            .with_context(|| format!("parsing Python import index for {}", path.display()))?;
        dependencies.observe(root, path, &imports);
        Ok(())
    })?;

    Ok(ProviderModuleAnalysis {
        metrics: ModuleMetricsSummary {
            public_items: surface.public_items,
            internal_items: surface.internal_items,
            ..ModuleMetricsSummary::default()
        },
        item_signals: Some(summarize_source_item_signals_with_metrics(
            &owned_items,
            &BTreeMap::new(),
        )),
        dependencies: Some(dependencies.summary()),
        coupling: Some(dependencies.coupling_input()),
        traceability: traceability_summary,
        traceability_unavailable_reason: include_traceability
            .then(|| context.traceability_unavailable_reason.clone())
            .flatten(),
        ..ProviderModuleAnalysis::default()
    })
}

pub(crate) struct PythonRepoAnalysisContext {
    traceability_pack: PythonTraceabilityPack,
    traceability: Option<TraceabilityAnalysis>,
    pub(crate) traceability_unavailable_reason: Option<String>,
}

#[allow(clippy::too_many_arguments)]
pub(crate) fn build_repo_analysis_context(
    root: &Path,
    source_files: &[PathBuf],
    scoped_source_files: Option<&[PathBuf]>,
    parsed_repo: &ParsedRepo,
    parsed_architecture: &ParsedArchitecture,
    file_ownership: &BTreeMap<PathBuf, FileOwnership<'_>>,
    include_traceability: bool,
) -> PythonRepoAnalysisContext {
    let traceability_pack = PythonTraceabilityPack;
    let (traceability, traceability_unavailable_reason) = if include_traceability {
        match build_traceability_analysis_for_python(
            root,
            source_files,
            scoped_source_files,
            parsed_repo,
            parsed_architecture,
            file_ownership,
        ) {
            Ok(analysis) => (Some(analysis), None),
            Err(error) => (
                None,
                Some(format!(
                    "Python backward trace is unavailable because parser-backed graph construction failed: {error}"
                )),
            ),
        }
    } else {
        (None, None)
    };
    PythonRepoAnalysisContext {
        traceability_pack,
        traceability,
        traceability_unavailable_reason,
    }
}

pub(crate) fn summarize_repo_traceability(
    root: &Path,
    context: &PythonRepoAnalysisContext,
) -> Option<ArchitectureTraceabilitySummary> {
    context
        .traceability
        .as_ref()
        .map(|traceability| summarize_shared_repo_traceability(root, traceability))
}

#[derive(Debug, Clone, Copy)]
struct PythonTraceabilityPack;

impl TraceabilityLanguagePack for PythonTraceabilityPack {
    fn owned_items_for_implementations(
        &self,
        root: &Path,
        implementations: &[&ImplementRef],
        file_ownership: &BTreeMap<PathBuf, FileOwnership<'_>>,
    ) -> Vec<TraceabilityOwnedItem> {
        collect_owned_items(root, implementations, file_ownership)
    }
}

fn build_traceability_analysis_for_python(
    root: &Path,
    source_files: &[PathBuf],
    scoped_source_files: Option<&[PathBuf]>,
    parsed_repo: &ParsedRepo,
    parsed_architecture: &ParsedArchitecture,
    file_ownership: &BTreeMap<PathBuf, FileOwnership<'_>>,
) -> Result<TraceabilityAnalysis> {
    let source_graphs = parse_python_source_graphs(root, source_files)?;
    let metadata = parse_python_source_metadata(root, &source_graphs)?;
    let inputs = assemble_traceability_inputs_for_python(
        source_graphs,
        metadata,
        parsed_repo,
        parsed_architecture,
        file_ownership,
    );
    Ok(build_traceability_analysis(narrow_scoped_traceability_inputs_for_python(
        source_files,
        scoped_source_files,
        inputs,
    )?))
}

fn assemble_traceability_inputs_for_python(
    source_graphs: BTreeMap<PathBuf, ParsedSourceGraph>,
    metadata: PythonSourceMetadata,
    parsed_repo: &ParsedRepo,
    parsed_architecture: &ParsedArchitecture,
    file_ownership: &BTreeMap<PathBuf, FileOwnership<'_>>,
) -> TraceabilityInputs {
    let repo_items = collect_repo_items(&source_graphs, file_ownership);
    let context_items = collect_context_items(&source_graphs, file_ownership);
    let graph = TraceGraph {
        edges: build_parser_call_edges(&source_graphs, &metadata),
        root_supports: build_root_supports(parsed_repo, &source_graphs, |path, body| {
            parse_source_graph(path, body)
                .and_then(|graph| graph.items.first().map(|item| item.span.start_line))
        }),
    };
    let _ = parsed_architecture;
    TraceabilityInputs {
        repo_items,
        context_items,
        graph,
    }
}

fn narrow_scoped_traceability_inputs_for_python(
    source_files: &[PathBuf],
    scoped_source_files: Option<&[PathBuf]>,
    inputs: TraceabilityInputs,
) -> Result<TraceabilityInputs> {
    let Some(scoped_source_files) = scoped_source_files else {
        return Ok(inputs);
    };
    if scoped_source_files.is_empty() {
        return Ok(inputs);
    }

    emit_analysis_status(&format!(
        "python scoped traceability is using parser-backed scoped graph discovery for {} scoped file(s)",
        scoped_source_files.len()
    ));
    let projected_files = scoped_source_files.iter().cloned().collect::<BTreeSet<_>>();
    let projected_item_ids = inputs
        .repo_items
        .iter()
        .filter(|item| projected_files.contains(&item.path))
        .map(|item| item.stable_id.clone())
        .collect::<BTreeSet<_>>();
    let reference =
        build_projected_traceability_reference_from_projected_items(projected_item_ids, &inputs.graph)
            .map_err(anyhow::Error::msg)?;
    let preserved_item_ids = reference
        .contract
        .projected_item_ids
        .iter()
        .cloned()
        .chain(reference.exact_reverse_closure.node_ids.iter().cloned())
        .collect::<BTreeSet<_>>();

    let repo_items = inputs
        .repo_items
        .into_iter()
        .filter(|item| reference.contract.projected_item_ids.contains(&item.stable_id))
        .collect::<Vec<_>>();
    let context_items = inputs
        .context_items
        .into_iter()
        .filter(|item| preserved_item_ids.contains(&item.stable_id))
        .collect::<Vec<_>>();
    let graph = TraceGraph {
        edges: inputs
            .graph
            .edges
            .into_iter()
            .filter(|(caller, _)| preserved_item_ids.contains(caller))
            .map(|(caller, callees)| {
                (
                    caller,
                    callees
                        .into_iter()
                        .filter(|callee| preserved_item_ids.contains(callee))
                        .collect(),
                )
            })
            .collect(),
        root_supports: inputs
            .graph
            .root_supports
            .into_iter()
            .filter(|(item_id, _)| preserved_item_ids.contains(item_id))
            .collect(),
    };

    let _ = source_files;
    Ok(TraceabilityInputs {
        repo_items,
        context_items,
        graph,
    })
}

fn parse_python_source_graphs(
    root: &Path,
    source_files: &[PathBuf],
) -> Result<BTreeMap<PathBuf, ParsedSourceGraph>> {
    let mut graphs = BTreeMap::new();
    for path in source_files.iter().filter(|path| is_python_path(path)) {
        let text = read_owned_file_text(root, path)
            .with_context(|| format!("reading Python source graph input {}", path.display()))?;
        let graph = parse_source_graph(path, &text)
            .ok_or_else(|| anyhow!("failed to parse Python source graph for {}", path.display()))?;
        graphs.insert(path.clone(), graph);
    }
    Ok(graphs)
}

fn parse_python_source_metadata(
    root: &Path,
    source_graphs: &BTreeMap<PathBuf, ParsedSourceGraph>,
) -> Result<PythonSourceMetadata> {
    let mut metadata = PythonSourceMetadata::default();
    for (path, graph) in source_graphs {
        let text = read_owned_file_text(root, path)
            .with_context(|| format!("reading Python source metadata input {}", path.display()))?;
        let mut parser = Parser::new();
        parser
            .set_language(&tree_sitter_python::LANGUAGE.into())
            .context("initializing Python tree-sitter parser")?;
        let tree = parser
            .parse(&text, None)
            .ok_or_else(|| anyhow!("failed to parse Python source metadata for {}", path.display()))?;
        if tree.root_node().has_error() {
            bail!("failed to parse Python source metadata for {}", path.display());
        }
        let source = text.as_bytes();
        let root = tree.root_node();
        metadata
            .imports
            .insert(path.clone(), collect_import_index(path, root, source));
        collect_function_metadata(root, source, graph, &mut metadata.functions);
    }
    Ok(metadata)
}

fn parse_python_import_index(path: &Path, text: &str) -> Result<PythonImportIndex> {
    let mut parser = Parser::new();
    parser
        .set_language(&tree_sitter_python::LANGUAGE.into())
        .context("initializing Python tree-sitter parser")?;
    let tree = parser
        .parse(text, None)
        .ok_or_else(|| anyhow!("failed to parse Python imports for {}", path.display()))?;
    if tree.root_node().has_error() {
        bail!("failed to parse Python imports for {}", path.display());
    }
    Ok(collect_import_index(
        path,
        tree.root_node(),
        text.as_bytes(),
    ))
}

fn collect_repo_items(
    source_graphs: &BTreeMap<PathBuf, ParsedSourceGraph>,
    file_ownership: &BTreeMap<PathBuf, FileOwnership<'_>>,
) -> Vec<TraceabilityOwnedItem> {
    collect_traceability_items(source_graphs, file_ownership, false)
}

fn collect_context_items(
    source_graphs: &BTreeMap<PathBuf, ParsedSourceGraph>,
    file_ownership: &BTreeMap<PathBuf, FileOwnership<'_>>,
) -> Vec<TraceabilityOwnedItem> {
    collect_traceability_items(source_graphs, file_ownership, true)
}

// @applies ADAPTER.FACTS_TO_MODEL.TRACEABILITY_ITEMS
fn collect_traceability_items(
    source_graphs: &BTreeMap<PathBuf, ParsedSourceGraph>,
    file_ownership: &BTreeMap<PathBuf, FileOwnership<'_>>,
    include_tests: bool,
) -> Vec<TraceabilityOwnedItem> {
    let mut items = source_graphs
        .iter()
        .flat_map(|(path, graph)| {
            let module_ids = owned_module_ids_for_path(file_ownership, path);
            let test_file = crate::syntax::python::is_python_test_file(path);
            graph.items.iter().filter_map(move |item| {
                if item.is_test && !include_tests {
                    return None;
                }
                Some(TraceabilityOwnedItem {
                    stable_id: item.stable_id.clone(),
                    name: item.name.clone(),
                    kind: source_item_kind(item.kind),
                    path: path.clone(),
                    public: item.public,
                    review_surface: is_review_surface(item.public, &item.name, test_file),
                    test_file,
                    module_ids: module_ids.clone(),
                    mediated_reason: None,
                })
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
        let test_file = crate::syntax::python::is_python_test_file(&implementation.location.path);
        for item in graph.items.into_iter().filter(|item| !item.is_test) {
            if !seen.insert(item.stable_id.clone()) {
                continue;
            }
            let review_surface = is_review_surface(item.public, &item.name, test_file);
            items.push(TraceabilityOwnedItem {
                stable_id: item.stable_id,
                name: item.name,
                kind: source_item_kind(item.kind),
                path: implementation.location.path.clone(),
                public: item.public,
                review_surface,
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

#[derive(Debug, Clone)]
struct SourceCallableItem {
    stable_id: String,
    name: String,
    qualified_name: String,
    container_path: Vec<String>,
    path: PathBuf,
    is_test: bool,
    calls: Vec<SourceCall>,
}

#[derive(Debug, Clone, Default)]
struct CallableIndexes {
    global_name_counts: BTreeMap<String, usize>,
    same_file_name_counts: BTreeMap<(PathBuf, String), usize>,
    qualified_name_counts: BTreeMap<String, usize>,
    stable_id_by_qualified_name: BTreeMap<String, String>,
    stable_id_by_file_container_method: BTreeMap<(PathBuf, Vec<String>, String), String>,
}

#[derive(Debug, Clone, Default)]
struct PythonSourceMetadata {
    imports: BTreeMap<PathBuf, PythonImportIndex>,
    functions: BTreeMap<String, PythonFunctionMeta>,
}

#[derive(Debug, Clone, Default)]
struct PythonImportIndex {
    symbol_aliases: BTreeMap<String, Vec<String>>,
    module_aliases: BTreeMap<String, Vec<String>>,
    dependency_targets: Vec<Vec<String>>,
}

#[derive(Debug, Clone, Default)]
struct PythonFunctionMeta {
    parameters: Vec<String>,
    pytest_fixture: bool,
}

fn build_parser_call_edges(
    source_graphs: &BTreeMap<PathBuf, ParsedSourceGraph>,
    metadata: &PythonSourceMetadata,
) -> BTreeMap<String, BTreeSet<String>> {
    let callable_items = collect_callable_items(source_graphs);
    let indexes = build_callable_indexes(&callable_items);
    let mut edges = BTreeMap::new();
    for item in &callable_items {
        let mut callees = item
            .calls
            .iter()
            .filter_map(|call| {
                resolve_call_target(item, call, &callable_items, &indexes, metadata)
            })
            .collect::<BTreeSet<_>>();
        callees.extend(resolve_pytest_fixture_edges(item, &callable_items, &indexes, metadata));
        edges.insert(item.stable_id.clone(), callees);
    }
    edges
}

fn collect_callable_items(
    source_graphs: &BTreeMap<PathBuf, ParsedSourceGraph>,
) -> Vec<SourceCallableItem> {
    let mut items = Vec::new();
    for (path, graph) in source_graphs {
        items.extend(graph.items.iter().cloned().map(|item| SourceCallableItem {
            stable_id: item.stable_id,
            name: item.name,
            qualified_name: item.qualified_name,
            container_path: item.container_path,
            path: path.clone(),
            is_test: item.is_test,
            calls: item.calls,
        }));
    }
    items
}

fn build_callable_indexes(items: &[SourceCallableItem]) -> CallableIndexes {
    let mut indexes = CallableIndexes::default();
    for item in items {
        *indexes
            .global_name_counts
            .entry(item.name.clone())
            .or_default() += 1;
        *indexes
            .same_file_name_counts
            .entry((item.path.clone(), item.name.clone()))
            .or_default() += 1;
        *indexes
            .qualified_name_counts
            .entry(item.qualified_name.clone())
            .or_default() += 1;
        indexes
            .stable_id_by_qualified_name
            .entry(item.qualified_name.clone())
            .or_insert_with(|| item.stable_id.clone());
        indexes
            .stable_id_by_file_container_method
            .entry((
                item.path.clone(),
                item.container_path.clone(),
                item.name.clone(),
            ))
            .or_insert_with(|| item.stable_id.clone());
    }
    indexes
}

fn resolve_call_target(
    caller: &SourceCallableItem,
    call: &SourceCall,
    items: &[SourceCallableItem],
    indexes: &CallableIndexes,
    metadata: &PythonSourceMetadata,
) -> Option<String> {
    if let Some(target) = resolve_self_or_class_method_call(caller, call, indexes) {
        return Some(target);
    }

    if let Some(target) = resolve_imported_call(caller, call, indexes, metadata) {
        return Some(target);
    }

    if indexes
        .same_file_name_counts
        .get(&(caller.path.clone(), call.name.clone()))
        .copied()
        .unwrap_or(0)
        == 1
    {
        return items
            .iter()
            .find(|item| item.path == caller.path && item.name == call.name)
            .map(|item| item.stable_id.clone());
    }

    if indexes
        .global_name_counts
        .get(&call.name)
        .copied()
        .unwrap_or(0)
        == 1
    {
        return items
            .iter()
            .find(|item| item.name == call.name)
            .map(|item| item.stable_id.clone());
    }

    None
}

fn resolve_self_or_class_method_call(
    caller: &SourceCallableItem,
    call: &SourceCall,
    indexes: &CallableIndexes,
) -> Option<String> {
    let qualifier = call.qualifier.as_deref()?;
    if matches!(qualifier, "self" | "cls") && !caller.container_path.is_empty() {
        return indexes
            .stable_id_by_file_container_method
            .get(&(
                caller.path.clone(),
                caller.container_path.clone(),
                call.name.clone(),
            ))
            .cloned();
    }

    let class_name = qualifier.strip_suffix("()")?;
    if !is_identifier(class_name) {
        return None;
    }
    indexes
        .stable_id_by_file_container_method
        .get(&(
            caller.path.clone(),
            vec![class_name.to_string()],
            call.name.clone(),
        ))
        .cloned()
}

fn resolve_imported_call(
    caller: &SourceCallableItem,
    call: &SourceCall,
    indexes: &CallableIndexes,
    metadata: &PythonSourceMetadata,
) -> Option<String> {
    let imports = metadata.imports.get(&caller.path)?;
    if let Some(qualifier) = call.qualifier.as_deref() {
        let qualifier = qualifier.strip_suffix("()").unwrap_or(qualifier);
        if let Some(mut target_segments) = imports
            .module_aliases
            .get(qualifier)
            .or_else(|| imports.symbol_aliases.get(qualifier))
            .cloned()
        {
            target_segments.push(call.name.clone());
            return resolve_qualified_segments(&target_segments, indexes);
        }
    }

    let target_segments = imports.symbol_aliases.get(&call.name)?;
    resolve_qualified_segments(target_segments, indexes)
}

fn resolve_qualified_segments(
    target_segments: &[String],
    indexes: &CallableIndexes,
) -> Option<String> {
    let qualified_name = target_segments.join("::");
    if indexes
        .qualified_name_counts
        .get(&qualified_name)
        .copied()
        .unwrap_or(0)
        == 1
    {
        return indexes
            .stable_id_by_qualified_name
            .get(&qualified_name)
            .cloned();
    }
    None
}

fn resolve_pytest_fixture_edges(
    caller: &SourceCallableItem,
    items: &[SourceCallableItem],
    indexes: &CallableIndexes,
    metadata: &PythonSourceMetadata,
) -> Vec<String> {
    if !caller.is_test {
        return Vec::new();
    }
    let Some(meta) = metadata.functions.get(&caller.stable_id) else {
        return Vec::new();
    };
    meta.parameters
        .iter()
        .filter(|parameter| !matches!(parameter.as_str(), "self" | "cls"))
        .filter_map(|parameter| resolve_pytest_fixture_parameter(caller, parameter, items, indexes, metadata))
        .collect()
}

fn resolve_pytest_fixture_parameter(
    caller: &SourceCallableItem,
    parameter: &str,
    items: &[SourceCallableItem],
    indexes: &CallableIndexes,
    metadata: &PythonSourceMetadata,
) -> Option<String> {
    if indexes
        .same_file_name_counts
        .get(&(caller.path.clone(), parameter.to_string()))
        .copied()
        .unwrap_or(0)
        == 1
        && let Some(item) = items
            .iter()
            .find(|item| item.path == caller.path && item.name == parameter)
        && metadata
            .functions
            .get(&item.stable_id)
            .is_some_and(|meta| meta.pytest_fixture)
    {
        return Some(item.stable_id.clone());
    }

    let fixture_items = items
        .iter()
        .filter(|item| item.name == parameter)
        .filter(|item| {
            metadata
                .functions
                .get(&item.stable_id)
                .is_some_and(|meta| meta.pytest_fixture)
        })
        .collect::<Vec<_>>();
    (fixture_items.len() == 1).then(|| fixture_items[0].stable_id.clone())
}

fn collect_import_index(path: &Path, root: Node<'_>, source: &[u8]) -> PythonImportIndex {
    let mut index = PythonImportIndex::default();
    collect_import_index_inner(path, root, source, &mut index);
    index
}

fn collect_import_index_inner(
    path: &Path,
    node: Node<'_>,
    source: &[u8],
    index: &mut PythonImportIndex,
) {
    match node.kind() {
        "import_statement" => collect_import_statement(node, source, index),
        "import_from_statement" => collect_import_from_statement(path, node, source, index),
        _ => {}
    }

    let mut cursor = node.walk();
    for child in node.named_children(&mut cursor) {
        collect_import_index_inner(path, child, source, index);
    }
}

fn collect_import_statement(node: Node<'_>, source: &[u8], index: &mut PythonImportIndex) {
    for child in children_by_field_name(node, "name") {
        let (target_segments, alias) = import_name_and_alias(child, source);
        if target_segments.is_empty() {
            continue;
        }
        index.dependency_targets.push(target_segments.clone());
        if let Some(alias) = alias {
            index.module_aliases.insert(alias, target_segments);
            continue;
        }
        if let Some(first_segment) = target_segments.first() {
            index
                .module_aliases
                .insert(first_segment.clone(), vec![first_segment.clone()]);
        }
        index
            .module_aliases
            .insert(target_segments.join("."), target_segments);
    }
}

fn collect_import_from_statement(
    path: &Path,
    node: Node<'_>,
    source: &[u8],
    index: &mut PythonImportIndex,
) {
    let Some(module_node) = node.child_by_field_name("module_name") else {
        return;
    };
    let module_segments = import_from_module_segments(path, module_node, source);
    if !module_segments.is_empty() {
        index.dependency_targets.push(module_segments.clone());
    }
    for child in children_by_field_name(node, "name") {
        let (imported_segments, alias) = import_name_and_alias(child, source);
        if imported_segments.is_empty() {
            continue;
        }
        let local_name = alias.unwrap_or_else(|| {
            imported_segments
                .last()
                .cloned()
                .unwrap_or_else(|| imported_segments.join("_"))
        });
        let target_segments = module_segments
            .iter()
            .cloned()
            .chain(imported_segments.iter().cloned())
            .collect::<Vec<_>>();
        index
            .symbol_aliases
            .insert(local_name.clone(), target_segments.clone());
        index.module_aliases.insert(local_name, target_segments);
    }
}

fn import_name_and_alias(node: Node<'_>, source: &[u8]) -> (Vec<String>, Option<String>) {
    if node.kind() == "aliased_import" {
        let target = node
            .child_by_field_name("name")
            .map(|name| dotted_name_segments(name, source))
            .unwrap_or_default();
        let alias = node
            .child_by_field_name("alias")
            .and_then(|alias| alias.utf8_text(source).ok())
            .map(str::to_string);
        return (target, alias);
    }
    (dotted_name_segments(node, source), None)
}

fn import_from_module_segments(path: &Path, node: Node<'_>, source: &[u8]) -> Vec<String> {
    match node.kind() {
        "relative_import" => relative_import_segments(path, node, source),
        _ => dotted_name_segments(node, source),
    }
}

fn relative_import_segments(path: &Path, node: Node<'_>, source: &[u8]) -> Vec<String> {
    let text = node.utf8_text(source).unwrap_or_default();
    let leading_dots = text.chars().take_while(|value| *value == '.').count();
    let suffix = text.trim_start_matches('.');
    let mut base = python_package_segments_for_relative_import(path);
    for _ in 1..leading_dots {
        base.pop();
    }
    base.extend(
        suffix
            .split('.')
            .filter(|segment| !segment.is_empty())
            .map(str::to_string),
    );
    base
}

fn collect_function_metadata(
    node: Node<'_>,
    source: &[u8],
    graph: &ParsedSourceGraph,
    metadata: &mut BTreeMap<String, PythonFunctionMeta>,
) {
    if node.kind() == "function_definition"
        && let Some(name) = node
            .child_by_field_name("name")
            .and_then(|name| name.utf8_text(source).ok())
        && let Some(item) = graph
            .items
            .iter()
            .find(|item| item.name == name && item.span.start_line == node.start_position().row + 1)
    {
        metadata.insert(
            item.stable_id.clone(),
            PythonFunctionMeta {
                parameters: function_parameter_names(node, source),
                pytest_fixture: function_is_pytest_fixture(node, source),
            },
        );
    }

    let mut cursor = node.walk();
    for child in node.named_children(&mut cursor) {
        collect_function_metadata(child, source, graph, metadata);
    }
}

fn function_parameter_names(node: Node<'_>, source: &[u8]) -> Vec<String> {
    let Some(parameters) = node.child_by_field_name("parameters") else {
        return Vec::new();
    };
    let mut names = Vec::new();
    let mut cursor = parameters.walk();
    for child in parameters.named_children(&mut cursor) {
        if let Some(name) = parameter_name(child, source) {
            names.push(name);
        }
    }
    names
}

fn parameter_name(node: Node<'_>, source: &[u8]) -> Option<String> {
    if node.kind() == "identifier" {
        return node.utf8_text(source).ok().map(str::to_string);
    }
    if let Some(name) = node.child_by_field_name("name") {
        return name.utf8_text(source).ok().map(str::to_string);
    }
    first_identifier_child(node, source)
}

fn function_is_pytest_fixture(node: Node<'_>, source: &[u8]) -> bool {
    let Some(parent) = node.parent() else {
        return false;
    };
    if parent.kind() != "decorated_definition" {
        return false;
    }
    let mut cursor = parent.walk();
    parent.named_children(&mut cursor).any(|child| {
        child.kind() == "decorator"
            && child
                .utf8_text(source)
                .ok()
                .is_some_and(decorator_marks_pytest_fixture)
    })
}

fn decorator_marks_pytest_fixture(text: &str) -> bool {
    let decorator = text.trim().trim_start_matches('@');
    let head = decorator.split('(').next().unwrap_or(decorator).trim();
    matches!(head, "pytest.fixture" | "fixture")
}

fn children_by_field_name<'tree>(node: Node<'tree>, field_name: &str) -> Vec<Node<'tree>> {
    (0..node.child_count())
        .filter(|index| node.field_name_for_child(*index as u32) == Some(field_name))
        .filter_map(|index| node.child(index as u32))
        .collect()
}

fn dotted_name_segments(node: Node<'_>, source: &[u8]) -> Vec<String> {
    node.utf8_text(source)
        .unwrap_or_default()
        .split('.')
        .map(str::trim)
        .filter(|segment| !segment.is_empty())
        .map(str::to_string)
        .collect()
}

fn python_package_segments_for_relative_import(path: &Path) -> Vec<String> {
    let mut segments = python_module_segments_for_path(path);
    if path.file_stem().and_then(|stem| stem.to_str()) != Some("__init__") {
        segments.pop();
    }
    segments
}

fn python_module_segments_for_path(path: &Path) -> Vec<String> {
    let mut normalized = path.components().collect::<Vec<_>>();
    if let Some(index) = normalized
        .iter()
        .position(|component| component.as_os_str() == "src")
    {
        normalized.drain(..=index);
    }

    let mut segments = normalized
        .iter()
        .filter_map(|component| match component {
            std::path::Component::Normal(value) => Some(value.to_string_lossy().to_string()),
            _ => None,
        })
        .collect::<Vec<_>>();
    if segments.is_empty() {
        return segments;
    }

    let file_name = segments.pop().unwrap_or_default();
    let stem = file_name
        .rsplit_once('.')
        .map(|(stem, _)| stem.to_string())
        .unwrap_or(file_name);
    if stem != "__init__" {
        segments.push(stem);
    }
    segments
}

fn first_identifier_child(node: Node<'_>, source: &[u8]) -> Option<String> {
    let mut cursor = node.walk();
    node.named_children(&mut cursor).find_map(|child| {
        (child.kind() == "identifier")
            .then(|| child.utf8_text(source).ok().map(str::to_string))
            .flatten()
    })
}

fn is_identifier(value: &str) -> bool {
    let mut chars = value.chars();
    chars
        .next()
        .is_some_and(|first| first == '_' || first.is_ascii_alphabetic())
        && chars.all(|next| next == '_' || next.is_ascii_alphanumeric())
}

#[derive(Default)]
struct PythonDependencySummary {
    targets: BTreeMap<String, usize>,
    internal_files: BTreeSet<PathBuf>,
    external_targets: BTreeSet<String>,
}

impl PythonDependencySummary {
    fn observe(&mut self, root: &Path, source_path: &Path, imports: &PythonImportIndex) {
        for target_segments in &imports.dependency_targets {
            if target_segments.is_empty() {
                continue;
            }
            let target = target_segments.join(".");
            *self.targets.entry(target.clone()).or_default() += 1;
            if let Some(path) = resolve_internal_python_module(root, source_path, target_segments) {
                self.internal_files.insert(path);
            } else {
                self.external_targets.insert(target);
            }
        }
    }

    fn summary(&self) -> crate::model::ModuleDependencySummary {
        build_dependency_summary(&self.targets)
    }

    fn coupling_input(&self) -> ModuleCouplingInput {
        ModuleCouplingInput {
            internal_files: self.internal_files.clone(),
            external_targets: self.external_targets.clone(),
        }
    }
}

fn resolve_internal_python_module(
    root: &Path,
    source_path: &Path,
    target_segments: &[String],
) -> Option<PathBuf> {
    let mut relative = PathBuf::new();
    for segment in target_segments {
        relative.push(segment);
    }
    let source_dir = root
        .join(source_path)
        .parent()
        .map(Path::to_path_buf)
        .unwrap_or_else(|| root.to_path_buf());
    let candidate_bases = [
        root.join("src").join(&relative),
        root.join(&relative),
        source_dir.join(&relative),
    ];
    candidate_bases
        .into_iter()
        .flat_map(|base| [base.with_extension("py"), base.join("__init__.py")])
        .map(|candidate| normalize_path(&candidate))
        .find(|candidate| candidate.exists())
}

fn normalize_path(path: &Path) -> PathBuf {
    if let Ok(canonical) = path.canonicalize() {
        return canonical;
    }
    let mut normalized = PathBuf::new();
    for component in path.components() {
        match component {
            std::path::Component::CurDir => {}
            std::path::Component::ParentDir => {
                normalized.pop();
            }
            _ => normalized.push(component.as_os_str()),
        }
    }
    normalized
}

#[derive(Default)]
struct PythonSurfaceSummary {
    public_items: usize,
    internal_items: usize,
}

impl PythonSurfaceSummary {
    fn observe(&mut self, items: &[crate::syntax::SourceItem]) {
        for item in items {
            if item.public {
                self.public_items += 1;
            } else {
                self.internal_items += 1;
            }
        }
    }
}

fn is_python_path(path: &Path) -> bool {
    matches!(path.extension().and_then(|ext| ext.to_str()), Some("py"))
}

fn is_review_surface(public: bool, _name: &str, test_file: bool) -> bool {
    !test_file && public
}

fn source_item_kind(kind: crate::syntax::SourceItemKind) -> crate::model::ModuleItemKind {
    match kind {
        crate::syntax::SourceItemKind::Function => crate::model::ModuleItemKind::Function,
        crate::syntax::SourceItemKind::Method => crate::model::ModuleItemKind::Method,
    }
}
