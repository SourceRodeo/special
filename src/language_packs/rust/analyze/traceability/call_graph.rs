/**
@module SPECIAL.LANGUAGE_PACKS.RUST.ANALYZE.TRACEABILITY.CALL_GRAPH
Builds Rust traceability call edges from parser-visible calls, cargo binary entrypoints, import aliasing, and rust-analyzer reverse reachability.
*/
// @fileimplements SPECIAL.LANGUAGE_PACKS.RUST.ANALYZE.TRACEABILITY.CALL_GRAPH
use std::collections::{BTreeMap, BTreeSet};
use std::path::{Path, PathBuf};

use anyhow::Result;
use syn::{Expr, ExprStruct, Item, ItemConst};

use crate::syntax::{CallSyntaxKind, ParsedSourceGraph, SourceCall, SourceInvocation, SourceSpan};

use super::super::rust_analyzer::RustAnalyzerCallableItem;
use super::super::toolchain::RustToolchainProject;
use super::super::use_tree::collect_use_aliases;

#[derive(Debug, Clone)]
struct SourceCallableItem {
    stable_id: String,
    name: String,
    qualified_name: String,
    module_path: Vec<String>,
    container_path: Vec<String>,
    path: PathBuf,
    span: SourceSpan,
    calls: Vec<SourceCall>,
    invocations: Vec<SourceInvocation>,
}

#[derive(Debug, Clone, Default)]
struct CallableIndexes {
    global_name_counts: BTreeMap<String, usize>,
    same_file_name_counts: BTreeMap<(PathBuf, String), usize>,
    global_qualified_name_counts: BTreeMap<String, usize>,
    same_file_qualified_name_counts: BTreeMap<(PathBuf, String), usize>,
}

#[derive(Debug, Clone, Default)]
struct DescriptorDispatchTargets {
    language_pack_fields: BTreeMap<String, BTreeSet<String>>,
    traceability_scope_fields: BTreeMap<String, BTreeSet<String>>,
    traceability_graph_fields: BTreeMap<String, BTreeSet<String>>,
    language_pack_context_methods: BTreeMap<String, BTreeSet<String>>,
}

pub(super) fn build_parser_call_edges_with_toolchain(
    root: &Path,
    source_files: &[PathBuf],
    source_graphs: &BTreeMap<PathBuf, ParsedSourceGraph>,
    toolchain_project: Option<&RustToolchainProject>,
) -> BTreeMap<String, BTreeSet<String>> {
    let cargo_binary_entrypoints = toolchain_project
        .map(|project| collect_toolchain_binary_entrypoints(project, source_graphs))
        .unwrap_or_else(|| collect_cargo_binary_entrypoints(root, source_graphs));
    let crate_root_aliases = toolchain_project
        .map(RustToolchainProject::crate_root_aliases)
        .unwrap_or_else(|| collect_crate_root_aliases(root));

    build_parser_call_edges(
        root,
        source_files,
        source_graphs,
        cargo_binary_entrypoints,
        crate_root_aliases,
    )
}

pub(super) fn build_rust_analyzer_call_edges(
    root: &Path,
    source_graphs: &BTreeMap<PathBuf, ParsedSourceGraph>,
    cargo_binary_entrypoints: BTreeMap<String, BTreeSet<String>>,
    parser_call_edges: &BTreeMap<String, BTreeSet<String>>,
) -> Result<BTreeMap<String, BTreeSet<String>>> {
    let callable_items = collect_callable_items(source_graphs);
    let seed_ids = source_graphs
        .values()
        .flat_map(|graph| graph.items.iter().filter(|item| item.is_test))
        .map(|item| item.stable_id.clone())
        .collect::<BTreeSet<_>>();
    let items = callable_items
        .iter()
        .map(|item| RustAnalyzerCallableItem {
            stable_id: item.stable_id.clone(),
            name: item.name.clone(),
            path: root.join(&item.path),
            span: item.span,
            calls: item.calls.clone(),
            invocation_targets: item
                .invocations
                .iter()
                .flat_map(|invocation| {
                    resolve_invocation_targets(invocation, &cargo_binary_entrypoints)
                })
                .collect(),
        })
        .collect::<Vec<_>>();
    super::super::rust_analyzer::build_reachable_call_edges(root, &items, &seed_ids, parser_call_edges)
}

pub(super) fn collect_rust_analyzer_reference_items(
    source_graphs: &BTreeMap<PathBuf, ParsedSourceGraph>,
) -> Vec<RustAnalyzerCallableItem> {
    collect_callable_items(source_graphs)
        .into_iter()
        .map(|item| RustAnalyzerCallableItem {
            stable_id: item.stable_id,
            name: item.name,
            path: item.path,
            span: item.span,
            calls: item.calls,
            invocation_targets: BTreeSet::new(),
        })
        .collect()
}

fn build_parser_call_edges(
    root: &Path,
    source_files: &[PathBuf],
    source_graphs: &BTreeMap<PathBuf, ParsedSourceGraph>,
    cargo_binary_entrypoints: BTreeMap<String, BTreeSet<String>>,
    crate_root_aliases: BTreeSet<String>,
) -> BTreeMap<String, BTreeSet<String>> {
    let callable_items = collect_callable_items(source_graphs);
    let callable_indexes = build_callable_indexes(&callable_items);
    let imported_call_aliases = collect_imported_call_aliases(root, source_files);
    let descriptor_dispatch_targets =
        collect_language_pack_descriptor_dispatch_targets(root, source_files, &callable_items);
    build_call_edges(
        &callable_items,
        &callable_indexes,
        &cargo_binary_entrypoints,
        &crate_root_aliases,
        &imported_call_aliases,
        &descriptor_dispatch_targets,
    )
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
            module_path: item.module_path,
            container_path: item.container_path,
            path: path.clone(),
            span: item.span,
            calls: item.calls,
            invocations: item.invocations,
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
            .global_qualified_name_counts
            .entry(item.qualified_name.clone())
            .or_default() += 1;
        *indexes
            .same_file_qualified_name_counts
            .entry((item.path.clone(), item.qualified_name.clone()))
            .or_default() += 1;
    }
    indexes
}

fn build_call_edges(
    items: &[SourceCallableItem],
    indexes: &CallableIndexes,
    cargo_binary_entrypoints: &BTreeMap<String, BTreeSet<String>>,
    crate_root_aliases: &BTreeSet<String>,
    imported_call_aliases: &BTreeMap<PathBuf, BTreeMap<String, Vec<String>>>,
    descriptor_dispatch_targets: &DescriptorDispatchTargets,
) -> BTreeMap<String, BTreeSet<String>> {
    let mut edges = BTreeMap::new();
    for item in items {
        let mut callees = item
            .calls
            .iter()
            .filter_map(|call| {
                resolve_call_target(
                    item,
                    call,
                    items,
                    indexes,
                    crate_root_aliases,
                    imported_call_aliases,
                )
            })
            .collect::<BTreeSet<_>>();
        for target in descriptor_dispatch_targets_for_calls(item, descriptor_dispatch_targets) {
            callees.insert(target);
        }
        for invocation in &item.invocations {
            for entrypoint_id in resolve_invocation_targets(invocation, cargo_binary_entrypoints) {
                callees.insert(entrypoint_id);
            }
        }
        edges.insert(item.stable_id.clone(), callees);
    }
    edges
}

fn descriptor_dispatch_targets_for_calls(
    caller: &SourceCallableItem,
    descriptor_dispatch_targets: &DescriptorDispatchTargets,
) -> BTreeSet<String> {
    let mut targets = BTreeSet::new();
    for call in &caller.calls {
        if call.qualifier.as_deref() == Some("descriptor") {
            targets.extend(
                descriptor_dispatch_targets
                    .language_pack_fields
                    .get(&call.name)
                    .into_iter()
                    .flatten()
                    .cloned(),
            );
        }
        if call.qualifier.as_deref() == Some("scope_facts") {
            targets.extend(
                descriptor_dispatch_targets
                    .traceability_scope_fields
                    .get(&call.name)
                    .into_iter()
                    .flatten()
                    .cloned(),
            );
        }
        if call.qualifier.as_deref() == Some("graph_facts") {
            targets.extend(
                descriptor_dispatch_targets
                    .traceability_graph_fields
                    .get(&call.name)
                    .into_iter()
                    .flatten()
                    .cloned(),
            );
        }
        if is_language_pack_context_dispatch_call(call) {
            targets.extend(
                descriptor_dispatch_targets
                    .language_pack_context_methods
                    .get(&call.name)
                    .into_iter()
                    .flatten()
                    .cloned(),
            );
        }
    }
    targets
}

fn is_language_pack_context_dispatch_call(call: &SourceCall) -> bool {
    if call.syntax != CallSyntaxKind::Field {
        return false;
    }
    // This is a narrow bridge for the registry shapes Special uses today. The
    // durable fix is receiver/binding-aware dispatch facts, not more spellings.
    matches!(
        call.qualifier.as_deref(),
        Some("context") | Some("self.inner")
    ) || call
        .qualifier
        .as_deref()
        .is_some_and(|qualifier| qualifier.starts_with("contexts.get("))
}

fn collect_language_pack_descriptor_dispatch_targets(
    root: &Path,
    source_files: &[PathBuf],
    callable_items: &[SourceCallableItem],
) -> DescriptorDispatchTargets {
    let callable_ids_by_file_and_name = callable_items
        .iter()
        .map(|item| ((item.path.clone(), item.name.clone()), item.stable_id.clone()))
        .collect::<BTreeMap<_, _>>();
    let mut descriptors_by_name = BTreeMap::<(PathBuf, String), ParsedDescriptorConst>::new();
    let mut targets = DescriptorDispatchTargets::default();

    for path in source_files
        .iter()
        .filter(|path| path.extension().and_then(|ext| ext.to_str()) == Some("rs"))
    {
        let Ok(text) = crate::modules::analyze::read_owned_file_text(root, path) else {
            continue;
        };
        let Ok(file) = syn::parse_file(&text) else {
            continue;
        };
        for item in &file.items {
            let Item::Const(item_const) = item else {
                continue;
            };
            let Some(descriptor) =
                parse_descriptor_const(path, item_const, &callable_ids_by_file_and_name)
            else {
                continue;
            };
            descriptors_by_name.insert((descriptor.path.clone(), descriptor.name.clone()), descriptor);
        }
        collect_language_pack_context_method_targets(
            path,
            &file,
            &callable_ids_by_file_and_name,
            &mut targets.language_pack_context_methods,
        );
    }

    for descriptor in descriptors_by_name.values() {
        match descriptor.kind {
            DescriptorKind::TraceabilityScopeFacts => {
                merge_descriptor_function_fields(
                    &mut targets.traceability_scope_fields,
                    &descriptor.function_fields,
                );
            }
            DescriptorKind::TraceabilityGraphFacts => {
                merge_descriptor_function_fields(
                    &mut targets.traceability_graph_fields,
                    &descriptor.function_fields,
                );
            }
            DescriptorKind::LanguagePack => {}
        }
    }
    for descriptor in descriptors_by_name
        .values()
        .filter(|descriptor| descriptor.kind == DescriptorKind::LanguagePack)
    {
        merge_descriptor_function_fields(
            &mut targets.language_pack_fields,
            &descriptor.function_fields,
        );
        for (field, referenced_name) in &descriptor.referenced_descriptor_fields {
            let Some(referenced) =
                descriptors_by_name.get(&(descriptor.path.clone(), referenced_name.clone()))
            else {
                continue;
            };
            match (field.as_str(), referenced.kind) {
                ("traceability_scope_facts", DescriptorKind::TraceabilityScopeFacts) => {
                    merge_descriptor_function_fields(
                        &mut targets.traceability_scope_fields,
                        &referenced.function_fields,
                    );
                }
                ("traceability_graph_facts", DescriptorKind::TraceabilityGraphFacts) => {
                    merge_descriptor_function_fields(
                        &mut targets.traceability_graph_fields,
                        &referenced.function_fields,
                    );
                }
                _ => {}
            }
        }
    }
    targets
}

fn merge_descriptor_function_fields(
    targets: &mut BTreeMap<String, BTreeSet<String>>,
    fields: &BTreeMap<String, String>,
) {
    for (field, stable_id) in fields {
        targets
            .entry(field.clone())
            .or_default()
            .insert(stable_id.clone());
    }
}

fn collect_language_pack_context_method_targets(
    path: &Path,
    file: &syn::File,
    callable_ids_by_file_and_name: &BTreeMap<(PathBuf, String), String>,
    targets: &mut BTreeMap<String, BTreeSet<String>>,
) {
    for item in &file.items {
        let Item::Impl(item_impl) = item else {
            continue;
        };
        let Some((_, trait_path, _)) = &item_impl.trait_ else {
            continue;
        };
        if trait_path
            .segments
            .last()
            .is_none_or(|segment| segment.ident != "LanguagePackAnalysisContext")
        {
            continue;
        }
        for impl_item in &item_impl.items {
            let syn::ImplItem::Fn(method) = impl_item else {
                continue;
            };
            let method_name = method.sig.ident.to_string();
            if let Some(stable_id) =
                callable_ids_by_file_and_name.get(&(path.to_path_buf(), method_name.clone()))
            {
                targets
                    .entry(method_name)
                    .or_default()
                    .insert(stable_id.clone());
            }
        }
    }
}

#[derive(Debug, Clone)]
struct ParsedDescriptorConst {
    path: PathBuf,
    name: String,
    kind: DescriptorKind,
    function_fields: BTreeMap<String, String>,
    referenced_descriptor_fields: BTreeMap<String, String>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum DescriptorKind {
    LanguagePack,
    TraceabilityScopeFacts,
    TraceabilityGraphFacts,
}

fn parse_descriptor_const(
    path: &Path,
    item_const: &ItemConst,
    callable_ids_by_file_and_name: &BTreeMap<(PathBuf, String), String>,
) -> Option<ParsedDescriptorConst> {
    let kind = descriptor_kind(&type_last_segment(&item_const.ty)?)?;
    let struct_expr = descriptor_struct_expr(&item_const.expr)?;
    let mut function_fields = BTreeMap::new();
    let mut referenced_descriptor_fields = BTreeMap::new();
    for field in &struct_expr.fields {
        let syn::Member::Named(field_ident) = &field.member else {
            continue;
        };
        let field_name = field_ident.to_string();
        if let Some(function_name) = function_path_name(&field.expr)
            && let Some(stable_id) =
                callable_ids_by_file_and_name.get(&(path.to_path_buf(), function_name))
        {
            function_fields.insert(field_name, stable_id.clone());
            continue;
        }
        if let Some(reference_name) = descriptor_reference_name(&field.expr) {
            referenced_descriptor_fields.insert(field_name, reference_name);
        }
    }
    Some(ParsedDescriptorConst {
        path: path.to_path_buf(),
        name: item_const.ident.to_string(),
        kind,
        function_fields,
        referenced_descriptor_fields,
    })
}

fn descriptor_kind(type_name: &str) -> Option<DescriptorKind> {
    match type_name {
        "LanguagePackDescriptor" => Some(DescriptorKind::LanguagePack),
        "TraceabilityScopeFactsDescriptor" => Some(DescriptorKind::TraceabilityScopeFacts),
        "TraceabilityGraphFactsDescriptor" => Some(DescriptorKind::TraceabilityGraphFacts),
        _ => None,
    }
}

fn type_last_segment(ty: &syn::Type) -> Option<String> {
    match ty {
        syn::Type::Path(type_path) => type_path
            .path
            .segments
            .last()
            .map(|segment| segment.ident.to_string()),
        _ => None,
    }
}

fn descriptor_struct_expr(expr: &Expr) -> Option<&ExprStruct> {
    match expr {
        Expr::Struct(struct_expr) => Some(struct_expr),
        _ => None,
    }
}

fn function_path_name(expr: &Expr) -> Option<String> {
    match expr {
        Expr::Path(path) => path.path.segments.last().map(|segment| segment.ident.to_string()),
        _ => None,
    }
}

fn descriptor_reference_name(expr: &Expr) -> Option<String> {
    let Expr::Call(call) = expr else {
        return None;
    };
    let Expr::Path(function_path) = call.func.as_ref() else {
        return None;
    };
    if function_path.path.segments.len() != 1
        || function_path.path.segments.first()?.ident != "Some"
        || call.args.len() != 1
    {
        return None;
    }
    let Expr::Reference(reference) = call.args.first()? else {
        return None;
    };
    let Expr::Path(reference_path) = reference.expr.as_ref() else {
        return None;
    };
    if reference_path.path.segments.len() != 1 {
        return None;
    }
    reference_path
        .path
        .segments
        .first()
        .map(|segment| segment.ident.to_string())
}

fn resolve_call_target(
    caller: &SourceCallableItem,
    call: &SourceCall,
    items: &[SourceCallableItem],
    indexes: &CallableIndexes,
    crate_root_aliases: &BTreeSet<String>,
    imported_call_aliases: &BTreeMap<PathBuf, BTreeMap<String, Vec<String>>>,
) -> Option<String> {
    if let Some(qualifier) = call.qualifier.as_ref() {
        for qualified_name in
            qualified_name_candidates(caller, qualifier, &call.name, crate_root_aliases)
        {
            if indexes
                .global_qualified_name_counts
                .get(&qualified_name)
                .copied()
                .unwrap_or(0)
                == 1
            {
                return items
                    .iter()
                    .find(|item| item.qualified_name == qualified_name)
                    .map(|item| item.stable_id.clone());
            }

            if indexes
                .same_file_qualified_name_counts
                .get(&(caller.path.clone(), qualified_name.clone()))
                .copied()
                .unwrap_or(0)
                == 1
            {
                return items
                    .iter()
                    .find(|item| item.path == caller.path && item.qualified_name == qualified_name)
                    .map(|item| item.stable_id.clone());
            }
        }
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

    if let Some(imports) = imported_call_aliases
        .get(&caller.path)
        .and_then(|aliases| aliases.get(&call.name))
    {
        for imported_path in imports {
            let Some((qualifier, imported_name)) = imported_path.rsplit_once("::") else {
                continue;
            };
            for qualified_name in
                qualified_name_candidates(caller, qualifier, imported_name, crate_root_aliases)
            {
                if indexes
                    .global_qualified_name_counts
                    .get(&qualified_name)
                    .copied()
                    .unwrap_or(0)
                    == 1
                {
                    return items
                        .iter()
                        .find(|item| item.qualified_name == qualified_name)
                        .map(|item| item.stable_id.clone());
                }

                if indexes
                    .same_file_qualified_name_counts
                    .get(&(caller.path.clone(), qualified_name.clone()))
                    .copied()
                    .unwrap_or(0)
                    == 1
                {
                    return items
                        .iter()
                        .find(|item| {
                            item.path == caller.path && item.qualified_name == qualified_name
                        })
                        .map(|item| item.stable_id.clone());
                }
            }
        }
    }

    None
}

fn resolve_invocation_targets(
    invocation: &SourceInvocation,
    cargo_binary_entrypoints: &BTreeMap<String, BTreeSet<String>>,
) -> BTreeSet<String> {
    match &invocation.kind {
        crate::syntax::SourceInvocationKind::LocalCargoBinary { binary_name } => {
            cargo_binary_entrypoints
                .get(binary_name)
                .cloned()
                .unwrap_or_default()
        }
    }
}

pub(super) fn collect_cargo_binary_entrypoints(
    root: &Path,
    source_graphs: &BTreeMap<PathBuf, ParsedSourceGraph>,
) -> BTreeMap<String, BTreeSet<String>> {
    let cargo_toml_path = root.join("Cargo.toml");
    let Ok(cargo_toml_text) = std::fs::read_to_string(cargo_toml_path) else {
        return BTreeMap::new();
    };
    let Ok(cargo_toml) = cargo_toml_text.parse::<toml::Value>() else {
        return BTreeMap::new();
    };
    let Some(bin_entries) = cargo_toml.get("bin").and_then(|value| value.as_array()) else {
        return BTreeMap::new();
    };

    let binary_sources = bin_entries.iter().filter_map(|bin_entry| {
        let bin_name = bin_entry.get("name").and_then(|value| value.as_str())?;
        let bin_path = bin_entry.get("path").and_then(|value| value.as_str())?;
        Some((bin_name.to_string(), PathBuf::from(bin_path)))
    });

    collect_binary_entrypoints_for_sources(source_graphs, binary_sources)
}

pub(super) fn collect_toolchain_binary_entrypoints(
    project: &RustToolchainProject,
    source_graphs: &BTreeMap<PathBuf, ParsedSourceGraph>,
) -> BTreeMap<String, BTreeSet<String>> {
    collect_binary_entrypoints_for_sources(
        source_graphs,
        project
            .binary_target_sources()
            .into_iter()
            .flat_map(|(bin_name, paths)| {
                paths
                    .into_iter()
                    .map(move |path| (bin_name.clone(), path))
                    .collect::<Vec<_>>()
            }),
    )
}

fn collect_binary_entrypoints_for_sources<I>(
    source_graphs: &BTreeMap<PathBuf, ParsedSourceGraph>,
    binary_sources: I,
) -> BTreeMap<String, BTreeSet<String>>
where
    I: IntoIterator<Item = (String, PathBuf)>,
{
    let mut entrypoints: BTreeMap<String, BTreeSet<String>> = BTreeMap::new();

    for (bin_name, bin_path) in binary_sources {
        let Some(graph) = find_source_graph_for_path(source_graphs, &bin_path) else {
            continue;
        };
        let main_ids = graph
            .items
            .iter()
            .filter(|item| item.name == "main")
            .map(|item| item.stable_id.clone())
            .collect::<BTreeSet<_>>();
        if !main_ids.is_empty() {
            entrypoints.entry(bin_name).or_default().extend(main_ids);
        }
    }

    entrypoints
}

fn find_source_graph_for_path<'a>(
    source_graphs: &'a BTreeMap<PathBuf, ParsedSourceGraph>,
    target_path: &Path,
) -> Option<&'a ParsedSourceGraph> {
    source_graphs
        .iter()
        .find(|(path, _)| *path == target_path || path.ends_with(target_path))
        .map(|(_, graph)| graph)
}

fn collect_crate_root_aliases(root: &Path) -> BTreeSet<String> {
    let mut aliases = BTreeSet::new();
    let cargo_toml_path = root.join("Cargo.toml");
    let Ok(cargo_toml_text) = std::fs::read_to_string(cargo_toml_path) else {
        return aliases;
    };
    let Ok(cargo_toml) = cargo_toml_text.parse::<toml::Value>() else {
        return aliases;
    };

    if let Some(lib_name) = cargo_toml
        .get("lib")
        .and_then(|value| value.get("name"))
        .and_then(|value| value.as_str())
    {
        aliases.insert(lib_name.to_string());
    }

    if let Some(package_name) = cargo_toml
        .get("package")
        .and_then(|value| value.get("name"))
        .and_then(|value| value.as_str())
    {
        aliases.insert(package_name.replace('-', "_"));
    }

    aliases
}

fn collect_imported_call_aliases(
    root: &Path,
    source_files: &[PathBuf],
) -> BTreeMap<PathBuf, BTreeMap<String, Vec<String>>> {
    let mut aliases_by_file = BTreeMap::new();

    for path in source_files
        .iter()
        .filter(|path| path.extension().and_then(|ext| ext.to_str()) == Some("rs"))
    {
        let Ok(text) = crate::modules::analyze::read_owned_file_text(root, path) else {
            continue;
        };
        let Ok(file) = syn::parse_file(&text) else {
            continue;
        };
        let mut aliases = BTreeMap::<String, Vec<String>>::new();
        for item in &file.items {
            let syn::Item::Use(item_use) = item else {
                continue;
            };
            for (alias, targets) in collect_use_aliases(&item_use.tree) {
                aliases.entry(alias).or_default().extend(targets);
            }
        }
        if !aliases.is_empty() {
            aliases_by_file.insert(path.clone(), aliases);
        }
    }

    aliases_by_file
}

fn qualified_name_candidates(
    caller: &SourceCallableItem,
    qualifier: &str,
    call_name: &str,
    crate_root_aliases: &BTreeSet<String>,
) -> Vec<String> {
    let prefixes = qualifier_prefix_candidates(caller, qualifier, crate_root_aliases);
    let mut seen = BTreeSet::new();
    prefixes
        .into_iter()
        .filter_map(|mut segments| {
            segments.push(call_name.to_string());
            let qualified_name = segments.join("::");
            seen.insert(qualified_name.clone())
                .then_some(qualified_name)
        })
        .collect()
}

fn qualifier_prefix_candidates(
    caller: &SourceCallableItem,
    qualifier: &str,
    crate_root_aliases: &BTreeSet<String>,
) -> Vec<Vec<String>> {
    let segments = qualifier
        .split("::")
        .filter(|segment| !segment.is_empty())
        .map(|segment| segment.to_string())
        .collect::<Vec<_>>();
    if segments.is_empty() {
        return Vec::new();
    }

    if segments.first().map(String::as_str) == Some("crate") {
        return vec![segments.into_iter().skip(1).collect()];
    }

    if segments
        .first()
        .is_some_and(|segment| crate_root_aliases.contains(segment))
    {
        return vec![segments.into_iter().skip(1).collect()];
    }

    if segments.first().map(String::as_str) == Some("self") {
        if segments.len() == 1 && !caller.container_path.is_empty() {
            return vec![
                caller
                    .module_path
                    .iter()
                    .cloned()
                    .chain(caller.container_path.iter().cloned())
                    .collect(),
            ];
        }
        return vec![
            caller
                .module_path
                .iter()
                .cloned()
                .chain(segments.into_iter().skip(1))
                .collect(),
        ];
    }

    if segments.first().map(String::as_str) == Some("Self") {
        if caller.container_path.is_empty() {
            return Vec::new();
        }
        return vec![
            caller
                .module_path
                .iter()
                .cloned()
                .chain(caller.container_path.iter().cloned())
                .chain(segments.into_iter().skip(1))
                .collect(),
        ];
    }

    if segments.first().map(String::as_str) == Some("super") {
        let mut ancestor_depth = 0usize;
        while segments.get(ancestor_depth).map(String::as_str) == Some("super") {
            ancestor_depth += 1;
        }
        if ancestor_depth > caller.module_path.len() {
            return Vec::new();
        }
        let base_len = caller.module_path.len().saturating_sub(ancestor_depth);
        return vec![
            caller.module_path[..base_len]
                .iter()
                .cloned()
                .chain(segments.into_iter().skip(ancestor_depth))
                .collect(),
        ];
    }

    let mut candidates = Vec::new();
    for ancestor_len in (0..=caller.module_path.len()).rev() {
        candidates.push(
            caller.module_path[..ancestor_len]
                .iter()
                .cloned()
                .chain(segments.iter().cloned())
                .collect(),
        );
    }
    candidates
}

#[cfg(test)]
mod tests {
    use std::fs;

    use super::*;

    #[test]
    fn language_pack_descriptor_dispatch_edges_cover_registered_pack_functions() {
        let root = temp_root("special-rust-descriptor-dispatch");
        let sources = [
            (
                "src/language_packs/go.rs",
                pack_source("go", "is_go_path"),
            ),
            (
                "src/language_packs/rust.rs",
                pack_source("rust", "is_rust_path"),
            ),
            (
                "src/language_packs/typescript.rs",
                pack_source("typescript", "is_typescript_path"),
            ),
            (
                "src/syntax/registry.rs",
                r#"
fn parse_source_graph_at_path() {
    (descriptor.matches_path)();
    (descriptor.parse_source_graph)();
}
"#
                .to_string(),
            ),
            (
                "src/modules/analyze/registry.rs",
                r#"
fn build_repo_analysis_contexts() {
    (descriptor.build_repo_analysis_context)();
}

fn resolve_traceability_scope_facts() {
    (descriptor.analysis_environment_fingerprint)();
    (scope_facts.build_facts)();
    (scope_facts.expand_closure)();
}

fn resolve_traceability_graph_facts() {
    (descriptor.analysis_environment_fingerprint)();
    (graph_facts.build_facts)();
}

fn summarize_repo_traceability() {
    contexts.get(&language).unwrap().summarize_repo_traceability(root);
}

fn traceability_unavailable_reason() {
    contexts.get(&language).unwrap().traceability_unavailable_reason();
}

fn analyze_module_language() {
    context.analyze_module(root, implementations, file_ownership, options);
}
"#
                .to_string(),
            ),
        ];
        let mut source_files = Vec::new();
        let mut source_graphs = BTreeMap::new();
        for (path_text, source) in sources {
            let path = PathBuf::from(path_text);
            fs::create_dir_all(root.join(path.parent().expect("source path should have parent")))
                .expect("source dir should exist");
            fs::write(root.join(&path), &source).expect("source should be written");
            let graph = crate::syntax::parse_source_graph(&path, &source)
                .expect("source graph should parse");
            source_files.push(path.clone());
            source_graphs.insert(path, graph);
        }

        let edges = build_parser_call_edges(
            &root,
            &source_files,
            &source_graphs,
            BTreeMap::new(),
            BTreeSet::new(),
        );

        let syntax_registry = source_graphs
            .get(&PathBuf::from("src/syntax/registry.rs"))
            .and_then(|graph| graph.items.first())
            .expect("syntax registry item should exist");
        let syntax_callees = edges
            .get(&syntax_registry.stable_id)
            .expect("syntax registry should have edges");
        assert_contains_all(
            syntax_callees,
            [
                stable_id(&source_graphs, "src/language_packs/go.rs", "is_go_path"),
                stable_id(&source_graphs, "src/language_packs/rust.rs", "is_rust_path"),
                stable_id(
                    &source_graphs,
                    "src/language_packs/typescript.rs",
                    "is_typescript_path",
                ),
                stable_id(&source_graphs, "src/language_packs/go.rs", "parse_source_graph"),
                stable_id(&source_graphs, "src/language_packs/rust.rs", "parse_source_graph"),
                stable_id(
                    &source_graphs,
                    "src/language_packs/typescript.rs",
                    "parse_source_graph",
                ),
            ],
        );

        let analysis_registry = source_graphs
            .get(&PathBuf::from("src/modules/analyze/registry.rs"))
            .expect("analysis registry source graph should exist");
        let context_callees =
            item_callees(&edges, analysis_registry, "build_repo_analysis_contexts");
        assert_contains_all(
            context_callees,
            [
                stable_id(
                    &source_graphs,
                    "src/language_packs/go.rs",
                    "build_repo_analysis_context",
                ),
                stable_id(
                    &source_graphs,
                    "src/language_packs/rust.rs",
                    "build_repo_analysis_context",
                ),
                stable_id(
                    &source_graphs,
                    "src/language_packs/typescript.rs",
                    "build_repo_analysis_context",
                ),
            ],
        );
        let scope_callees =
            item_callees(&edges, analysis_registry, "resolve_traceability_scope_facts");
        assert_contains_all(
            scope_callees,
            [
                stable_id(
                    &source_graphs,
                    "src/language_packs/go.rs",
                    "analysis_environment_fingerprint",
                ),
                stable_id(
                    &source_graphs,
                    "src/language_packs/rust.rs",
                    "analysis_environment_fingerprint",
                ),
                stable_id(
                    &source_graphs,
                    "src/language_packs/typescript.rs",
                    "analysis_environment_fingerprint",
                ),
                stable_id(
                    &source_graphs,
                    "src/language_packs/go.rs",
                    "build_traceability_scope_facts",
                ),
                stable_id(
                    &source_graphs,
                    "src/language_packs/rust.rs",
                    "build_traceability_scope_facts",
                ),
                stable_id(
                    &source_graphs,
                    "src/language_packs/typescript.rs",
                    "build_traceability_scope_facts",
                ),
                stable_id(
                    &source_graphs,
                    "src/language_packs/go.rs",
                    "expand_traceability_closure_from_facts",
                ),
                stable_id(
                    &source_graphs,
                    "src/language_packs/rust.rs",
                    "expand_traceability_closure_from_facts",
                ),
                stable_id(
                    &source_graphs,
                    "src/language_packs/typescript.rs",
                    "expand_traceability_closure_from_facts",
                ),
            ],
        );
        let graph_callees =
            item_callees(&edges, analysis_registry, "resolve_traceability_graph_facts");
        assert_contains_all(
            graph_callees,
            [
                stable_id(
                    &source_graphs,
                    "src/language_packs/go.rs",
                    "build_traceability_graph_facts",
                ),
                stable_id(
                    &source_graphs,
                    "src/language_packs/rust.rs",
                    "build_traceability_graph_facts",
                ),
                stable_id(
                    &source_graphs,
                    "src/language_packs/typescript.rs",
                    "build_traceability_graph_facts",
                ),
            ],
        );
        let summary_callees = item_callees(&edges, analysis_registry, "summarize_repo_traceability");
        assert_contains_all(
            summary_callees,
            [
                stable_id(
                    &source_graphs,
                    "src/language_packs/go.rs",
                    "summarize_repo_traceability",
                ),
                stable_id(
                    &source_graphs,
                    "src/language_packs/rust.rs",
                    "summarize_repo_traceability",
                ),
                stable_id(
                    &source_graphs,
                    "src/language_packs/typescript.rs",
                    "summarize_repo_traceability",
                ),
            ],
        );
        let unavailable_callees =
            item_callees(&edges, analysis_registry, "traceability_unavailable_reason");
        assert_contains_all(
            unavailable_callees,
            [
                stable_id(
                    &source_graphs,
                    "src/language_packs/go.rs",
                    "traceability_unavailable_reason",
                ),
                stable_id(
                    &source_graphs,
                    "src/language_packs/rust.rs",
                    "traceability_unavailable_reason",
                ),
                stable_id(
                    &source_graphs,
                    "src/language_packs/typescript.rs",
                    "traceability_unavailable_reason",
                ),
            ],
        );
        let module_callees = item_callees(&edges, analysis_registry, "analyze_module_language");
        assert_contains_all(
            module_callees,
            [
                stable_id(&source_graphs, "src/language_packs/go.rs", "analyze_module"),
                stable_id(&source_graphs, "src/language_packs/rust.rs", "analyze_module"),
                stable_id(
                    &source_graphs,
                    "src/language_packs/typescript.rs",
                    "analyze_module",
                ),
            ],
        );

        fs::remove_dir_all(root).expect("temp root should be removed");
    }

    #[test]
    fn descriptor_references_require_unqualified_some_around_local_const_reference() {
        let valid = syn::parse_str::<Expr>("Some(&TRACEABILITY_SCOPE_FACTS)")
            .expect("valid descriptor reference expression should parse");
        assert_eq!(
            descriptor_reference_name(&valid).as_deref(),
            Some("TRACEABILITY_SCOPE_FACTS")
        );

        for source in [
            "option::Some(&TRACEABILITY_SCOPE_FACTS)",
            "Some(self.traceability_scope_facts)",
            "Some(&nested::TRACEABILITY_SCOPE_FACTS)",
            "Some(&TRACEABILITY_SCOPE_FACTS, &OTHER)",
        ] {
            let expr = syn::parse_str::<Expr>(source)
                .expect("non-matching descriptor reference expression should parse");
            assert_eq!(
                descriptor_reference_name(&expr),
                None,
                "{source} should not be accepted as a descriptor reference"
            );
        }
    }

    fn pack_source(language: &str, matches_path: &str) -> String {
        format!(
            r#"
const DESCRIPTOR: LanguagePackDescriptor = LanguagePackDescriptor {{
    language: SourceLanguage::new("{language}"),
    matches_path: {matches_path},
    parse_source_graph,
    build_repo_analysis_context,
    analysis_environment_fingerprint,
    project_tooling: None,
    traceability_scope_facts: Some(&TRACEABILITY_SCOPE_FACTS),
    traceability_graph_facts: Some(&TRACEABILITY_GRAPH_FACTS),
    scoped_traceability_preparation: ScopedTraceabilityPreparation::ScopedGraphDiscovery,
}};

const TRACEABILITY_SCOPE_FACTS: TraceabilityScopeFactsDescriptor =
    TraceabilityScopeFactsDescriptor {{
        build_facts: build_traceability_scope_facts,
        expand_closure: expand_traceability_closure_from_facts,
    }};

const TRACEABILITY_GRAPH_FACTS: TraceabilityGraphFactsDescriptor =
    TraceabilityGraphFactsDescriptor {{
        build_facts: build_traceability_graph_facts,
    }};

struct RepoAnalysisContext;

impl LanguagePackAnalysisContext for RepoAnalysisContext {{
    fn summarize_repo_traceability() {{}}
    fn traceability_unavailable_reason() {{}}
    fn analyze_module() {{}}
}}

fn {matches_path}() {{}}
fn parse_source_graph() {{}}
fn build_repo_analysis_context() {{}}
fn analysis_environment_fingerprint() {{}}
fn build_traceability_scope_facts() {{}}
fn expand_traceability_closure_from_facts() {{}}
fn build_traceability_graph_facts() {{}}
"#
        )
    }

    fn item_callees<'a>(
        edges: &'a BTreeMap<String, BTreeSet<String>>,
        graph: &ParsedSourceGraph,
        item_name: &str,
    ) -> &'a BTreeSet<String> {
        let item = graph
            .items
            .iter()
            .find(|item| item.name == item_name)
            .expect("item should exist");
        edges
            .get(&item.stable_id)
            .expect("item should have call edges")
    }

    fn stable_id(
        source_graphs: &BTreeMap<PathBuf, ParsedSourceGraph>,
        path: &str,
        item_name: &str,
    ) -> String {
        source_graphs
            .get(&PathBuf::from(path))
            .and_then(|graph| graph.items.iter().find(|item| item.name == item_name))
            .map(|item| item.stable_id.clone())
            .expect("target item should exist")
    }

    fn assert_contains_all<const N: usize>(actual: &BTreeSet<String>, expected: [String; N]) {
        for item in expected {
            assert!(
                actual.contains(&item),
                "missing expected item {item}; actual: {actual:#?}"
            );
        }
    }

    fn temp_root(prefix: &str) -> PathBuf {
        let unique = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .expect("system clock should be valid")
            .as_nanos();
        let path = std::env::temp_dir().join(format!("{prefix}-{}-{unique}", std::process::id()));
        fs::create_dir_all(&path).expect("temp root should exist");
        path
    }
}
