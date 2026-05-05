/**
@module SPECIAL.DOCS
Validates documentation-to-Special relationships and renders public markdown from docs sources.

@group SPECIAL.DOCS
Documentation relationship and docs-output behavior.

@spec SPECIAL.DOCS.LINKS
Markdown links whose destination is `special://KIND/ID` attach the linked label text as documentation evidence for the targeted Special identifier.

@spec SPECIAL.DOCS.LINKS.POLYMORPHIC
Documentation links accept `spec`, `group`, `module`, `area`, and `pattern` targets.

@spec SPECIAL.DOCS.LINKS.OUTPUT
special docs build SOURCE OUTPUT rewrites markdown `special://KIND/ID` links to their label text in the emitted artifact.

@spec SPECIAL.DOCS.DOCUMENTS_LINES
Documentation relationship lines `@documents KIND ID` and `@filedocuments KIND ID` attach one documentation relationship per line, are removed from docs output, and may not appear as adjacent stacked relationship lines.

@spec SPECIAL.DOCS_COMMAND
special docs validates documentation links and prints a documentation relationship view without writing files.

@spec SPECIAL.DOCS_COMMAND.TARGET
special docs --target PATH validates and prints only documentation relationships under the target file or subtree without writing files.

@spec SPECIAL.DOCS_COMMAND.PATH_SCOPE_SYNTAX
special docs rejects hidden positional path scopes and requires path scopes to use --target PATH.

@spec SPECIAL.DOCS_COMMAND.OUTPUT
special docs build validates documentation links and writes public docs.

@spec SPECIAL.DOCS_COMMAND.OUTPUT.DIRECTORY
special docs build SOURCE OUTPUT accepts an input directory and output directory, then mirrors the input tree relative to the source root while writing markdown output files.

@spec SPECIAL.DOCS_COMMAND.OUTPUT.SAFETY
special docs build SOURCE OUTPUT refuses to write docs output over the input path, into an input directory, or over an existing file that still contains docs evidence.

@spec SPECIAL.DOCS_COMMAND.OUTPUT.CONFIG
special docs build uses `[[docs.outputs]]` mappings from special.toml to write configured public docs without repeating paths on the command line.

@spec SPECIAL.DOCS_COMMAND.METRICS
special docs --metrics reports documentation coverage and public docs graph metrics without writing files.

@spec SPECIAL.DOCS_COMMAND.METRICS.COVERAGE
special docs --metrics classifies specs, groups, modules, areas, and patterns as publicly documented, internally documented only, or undocumented.

@spec SPECIAL.DOCS_COMMAND.METRICS.INTERCONNECTIVITY
special docs --metrics reports configured public docs pages, markdown links among those pages, broken local docs links, orphan pages, and configured-entrypoint reachability.
*/
// @fileimplements SPECIAL.DOCS
use std::collections::{BTreeMap, BTreeSet, VecDeque};
use std::fs;
use std::path::{Path, PathBuf};

use anyhow::{Context, Result, bail};
use ignore::WalkBuilder;
use serde::{Deserialize, Serialize};

use crate::annotation_syntax::{
    ReservedSpecialAnnotation, normalize_markdown_declaration_line,
    reserved_special_annotation_rest,
};
use crate::cache::{load_or_parse_architecture, load_or_parse_repo};
use crate::config::{DocsOutputConfig, SpecialVersion};
use crate::discovery::{DiscoveryConfig, discover_annotation_files};
use crate::extractor::collect_comment_blocks;
use crate::model::{
    ArchitectureKind, Diagnostic, DiagnosticSeverity, LintReport, NodeKind, SourceLocation,
};
use crate::parser::starts_markdown_fence;

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub(crate) enum DocumentTargetKind {
    Spec,
    Group,
    Module,
    Area,
    Pattern,
}

impl DocumentTargetKind {
    fn all() -> [Self; 5] {
        [
            Self::Spec,
            Self::Group,
            Self::Module,
            Self::Area,
            Self::Pattern,
        ]
    }

    fn parse(value: &str) -> Option<Self> {
        match value {
            "spec" => Some(Self::Spec),
            "group" => Some(Self::Group),
            "module" => Some(Self::Module),
            "area" => Some(Self::Area),
            "pattern" => Some(Self::Pattern),
            _ => None,
        }
    }

    fn as_str(self) -> &'static str {
        match self {
            Self::Spec => "spec",
            Self::Group => "group",
            Self::Module => "module",
            Self::Area => "area",
            Self::Pattern => "pattern",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub(crate) enum DocumentRefSource {
    Link,
    DocumentsLine,
    FileDocumentsLine,
}

impl DocumentRefSource {
    fn as_str(self) -> &'static str {
        match self {
            Self::Link => "link",
            Self::DocumentsLine => "@documents",
            Self::FileDocumentsLine => "@filedocuments",
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub(crate) struct DocumentRef {
    pub target_kind: DocumentTargetKind,
    pub target_id: String,
    pub location: SourceLocation,
    pub source: DocumentRefSource,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub text: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub(crate) struct DocsDocument {
    pub references: Vec<DocumentRef>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub(crate) struct DocsMetricsDocument {
    pub metrics: DocsMetricsSummary,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub(crate) struct DocsMetricsSummary {
    pub total_references: usize,
    pub link_references: usize,
    pub documents_line_references: usize,
    pub file_documents_line_references: usize,
    pub public_pages: usize,
    pub local_doc_links: usize,
    pub broken_local_doc_links: usize,
    pub orphan_pages: usize,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub reachable_pages_from_entrypoints: Option<usize>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub entrypoint_pages: Option<usize>,
    pub target_kinds: Vec<DocsTargetKindMetrics>,
    pub broken_local_doc_link_details: Vec<DocsLocalLinkIssue>,
    pub orphan_page_paths: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub(crate) struct DocsTargetKindMetrics {
    pub kind: DocumentTargetKind,
    pub total: usize,
    pub documented: usize,
    pub public: usize,
    pub internal_only: usize,
    pub undocumented: usize,
    pub undocumented_ids: Vec<String>,
}

impl DocsTargetKindMetrics {
    fn plural_label(&self) -> &'static str {
        match self.kind {
            DocumentTargetKind::Spec => "specs",
            DocumentTargetKind::Group => "groups",
            DocumentTargetKind::Module => "modules",
            DocumentTargetKind::Area => "areas",
            DocumentTargetKind::Pattern => "patterns",
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub(crate) struct DocsLocalLinkIssue {
    pub source: String,
    pub line: usize,
    pub target: String,
}

pub(crate) fn build_docs_lint_report(
    root: &Path,
    ignore_patterns: &[String],
    version: SpecialVersion,
) -> Result<LintReport> {
    let (_, report) = build_docs_document(root, ignore_patterns, version, &[])?;
    Ok(report)
}

pub(crate) fn build_docs_document(
    root: &Path,
    ignore_patterns: &[String],
    version: SpecialVersion,
    scope_paths: &[PathBuf],
) -> Result<(DocsDocument, LintReport)> {
    let (refs, mut diagnostics) = collect_repo_document_refs(root, ignore_patterns)?;
    let refs = scoped_refs(refs, scope_paths);
    retain_scoped_diagnostics(&mut diagnostics, scope_paths);
    diagnostics.extend(validate_document_refs(
        root,
        ignore_patterns,
        version,
        refs.clone(),
    )?);
    sort_diagnostics(&mut diagnostics);
    Ok((
        DocsDocument { references: refs },
        LintReport { diagnostics },
    ))
}

pub(crate) fn build_docs_metrics_document(
    root: &Path,
    ignore_patterns: &[String],
    version: SpecialVersion,
    scope_paths: &[PathBuf],
    outputs: &[DocsOutputConfig],
    entrypoints: &[PathBuf],
) -> Result<(DocsMetricsDocument, LintReport)> {
    let (document, report) = build_docs_document(root, ignore_patterns, version, scope_paths)?;
    let targets = DocumentTargets::load(root, ignore_patterns, version)?;
    let public_sources = configured_output_sources(root, outputs);
    let public_graph = build_public_docs_graph(root, outputs, entrypoints)?;
    let metrics = docs_metrics_summary(root, &document, &targets, &public_sources, public_graph);
    Ok((DocsMetricsDocument { metrics }, report))
}

pub(crate) fn write_docs_path(
    root: &Path,
    ignore_patterns: &[String],
    version: SpecialVersion,
    input: &Path,
    output: &Path,
) -> Result<LintReport> {
    write_docs_paths(
        root,
        ignore_patterns,
        version,
        &[(input.to_path_buf(), output.to_path_buf())],
    )
}

pub(crate) fn write_docs_paths(
    root: &Path,
    ignore_patterns: &[String],
    version: SpecialVersion,
    mappings: &[(PathBuf, PathBuf)],
) -> Result<LintReport> {
    let mut plan = Vec::new();
    for (input, output) in mappings {
        expand_output_mapping(input, output, &mut plan)?;
    }
    validate_output_plan(&plan)?;
    write_docs_files(root, ignore_patterns, version, &plan)
}

fn expand_output_mapping(
    input: &Path,
    output: &Path,
    plan: &mut Vec<(PathBuf, PathBuf)>,
) -> Result<()> {
    ensure_distinct_paths(
        input,
        output,
        "docs output path must not equal the input path",
    )?;
    if input.is_dir() {
        if path_is_inside(output, input) {
            bail!("docs output directory must not be inside the input directory");
        }
        for entry in WalkBuilder::new(input).hidden(false).build() {
            let entry = entry?;
            let path = entry.path();
            if !entry.file_type().is_some_and(|kind| kind.is_file()) {
                continue;
            }
            let relative = path
                .strip_prefix(input)
                .with_context(|| format!("building relative path for {}", path.display()))?;
            let output_path = output.join(relative);
            if path_is_inside(&output_path, input) {
                bail!("docs output file must not be inside the input directory");
            }
            plan.push((path.to_path_buf(), output_path));
        }
    } else {
        plan.push((input.to_path_buf(), output.to_path_buf()));
    }
    Ok(())
}

fn write_docs_files(
    root: &Path,
    ignore_patterns: &[String],
    version: SpecialVersion,
    files: &[(PathBuf, PathBuf)],
) -> Result<LintReport> {
    let mut outputs = Vec::new();
    let mut refs = Vec::new();
    let mut diagnostics = Vec::new();

    for (input, output) in files {
        let content =
            fs::read_to_string(input).with_context(|| format!("reading {}", input.display()))?;
        if is_markdown_path(input) {
            let output_content =
                write_markdown_output(input, &content, &mut refs, &mut diagnostics);
            outputs.push((output.clone(), output_content.into_bytes()));
        } else {
            outputs.push((output.clone(), content.into_bytes()));
        }
    }

    diagnostics.extend(validate_document_refs(
        root,
        ignore_patterns,
        version,
        refs,
    )?);
    sort_diagnostics(&mut diagnostics);
    let report = LintReport { diagnostics };
    if report.has_errors() {
        return Ok(report);
    }

    for (output, content) in outputs {
        if let Some(parent) = output.parent() {
            fs::create_dir_all(parent)
                .with_context(|| format!("creating output directory {}", parent.display()))?;
        }
        fs::write(&output, content).with_context(|| format!("writing {}", output.display()))?;
    }

    Ok(report)
}

pub(crate) fn render_docs_text(root: &Path, document: &DocsDocument) -> String {
    if document.references.is_empty() {
        return "No docs relationships found.".to_string();
    }

    let mut output = String::new();
    output.push_str("Docs\n\n");

    let mut grouped: BTreeMap<(DocumentTargetKind, String), Vec<&DocumentRef>> = BTreeMap::new();
    for reference in &document.references {
        grouped
            .entry((reference.target_kind, reference.target_id.clone()))
            .or_default()
            .push(reference);
    }

    for ((kind, id), references) in grouped {
        output.push_str(&format!("{} {}\n", kind.as_str(), id));
        for reference in references {
            output.push_str(&format!(
                "  {}:{} {}",
                display_path(root, &reference.location.path),
                reference.location.line,
                reference.source.as_str()
            ));
            if let Some(text) = reference.text.as_deref() {
                output.push_str(&format!(": {}", text.trim()));
            }
            output.push('\n');
        }
        output.push('\n');
    }

    output.trim_end().to_string()
}

pub(crate) fn render_docs_json(document: &DocsDocument) -> Result<String> {
    Ok(serde_json::to_string_pretty(document)?)
}

pub(crate) fn render_docs_metrics_text(document: &DocsMetricsDocument, verbose: bool) -> String {
    let metrics = &document.metrics;
    let mut output = String::from("special docs metrics\n");
    output.push_str(&format!(
        "  total references: {}\n",
        metrics.total_references
    ));
    output.push_str(&format!(
        "    link references: {}\n",
        metrics.link_references
    ));
    output.push_str(&format!(
        "    @documents references: {}\n",
        metrics.documents_line_references
    ));
    output.push_str(&format!(
        "    @filedocuments references: {}\n",
        metrics.file_documents_line_references
    ));
    for kind in &metrics.target_kinds {
        output.push_str(&format!(
            "  {}: {} total, {} documented, {} public, {} internal-only, {} undocumented\n",
            kind.plural_label(),
            kind.total,
            kind.documented,
            kind.public,
            kind.internal_only,
            kind.undocumented
        ));
        if verbose && !kind.undocumented_ids.is_empty() {
            output.push_str(&format!(
                "    undocumented {}: {}\n",
                kind.plural_label(),
                kind.undocumented_ids.join(", ")
            ));
        }
    }
    output.push_str(&format!("  public pages: {}\n", metrics.public_pages));
    output.push_str(&format!("  local doc links: {}\n", metrics.local_doc_links));
    output.push_str(&format!(
        "  broken local doc links: {}\n",
        metrics.broken_local_doc_links
    ));
    for issue in metrics
        .broken_local_doc_link_details
        .iter()
        .take(if verbose { usize::MAX } else { 10 })
    {
        output.push_str(&format!(
            "    {}:{} -> {}\n",
            issue.source, issue.line, issue.target
        ));
    }
    output.push_str(&format!("  orphan pages: {}\n", metrics.orphan_pages));
    for path in metrics
        .orphan_page_paths
        .iter()
        .take(if verbose { usize::MAX } else { 10 })
    {
        output.push_str(&format!("    {path}\n"));
    }
    match (
        metrics.reachable_pages_from_entrypoints,
        metrics.entrypoint_pages,
    ) {
        (Some(reachable), Some(entrypoints)) => output.push_str(&format!(
            "  reachable from entrypoints: {}/{} page(s), {} entrypoint(s)\n",
            reachable, metrics.public_pages, entrypoints
        )),
        _ => output.push_str("  reachable from entrypoints: not configured\n"),
    }
    output.trim_end().to_string()
}

pub(crate) fn render_docs_metrics_json(document: &DocsMetricsDocument) -> Result<String> {
    Ok(serde_json::to_string_pretty(document)?)
}

struct PublicDocsGraph {
    pages: BTreeSet<PathBuf>,
    local_links: Vec<PublicDocsLink>,
    broken_links: Vec<DocsLocalLinkIssue>,
    orphan_pages: Vec<String>,
    reachable_pages_from_entrypoints: Option<usize>,
    entrypoint_pages: Option<usize>,
}

struct PublicDocsLink {
    source: PathBuf,
    target: PathBuf,
}

fn docs_metrics_summary(
    _root: &Path,
    document: &DocsDocument,
    targets: &DocumentTargets,
    public_sources: &[PathBuf],
    public_graph: PublicDocsGraph,
) -> DocsMetricsSummary {
    DocsMetricsSummary {
        total_references: document.references.len(),
        link_references: document
            .references
            .iter()
            .filter(|reference| reference.source == DocumentRefSource::Link)
            .count(),
        documents_line_references: document
            .references
            .iter()
            .filter(|reference| reference.source == DocumentRefSource::DocumentsLine)
            .count(),
        file_documents_line_references: document
            .references
            .iter()
            .filter(|reference| reference.source == DocumentRefSource::FileDocumentsLine)
            .count(),
        public_pages: public_graph.pages.len(),
        local_doc_links: public_graph.local_links.len(),
        broken_local_doc_links: public_graph.broken_links.len(),
        orphan_pages: public_graph.orphan_pages.len(),
        reachable_pages_from_entrypoints: public_graph.reachable_pages_from_entrypoints,
        entrypoint_pages: public_graph.entrypoint_pages,
        target_kinds: DocumentTargetKind::all()
            .into_iter()
            .map(|kind| target_kind_metrics(kind, document, targets, public_sources))
            .collect(),
        broken_local_doc_link_details: public_graph.broken_links,
        orphan_page_paths: public_graph.orphan_pages,
    }
}

fn target_kind_metrics(
    kind: DocumentTargetKind,
    document: &DocsDocument,
    targets: &DocumentTargets,
    public_sources: &[PathBuf],
) -> DocsTargetKindMetrics {
    let target_ids = targets.ids(kind);
    let mut documented = BTreeSet::new();
    let mut public = BTreeSet::new();
    let mut internal = BTreeSet::new();

    for reference in &document.references {
        if reference.target_kind != kind {
            continue;
        }
        documented.insert(reference.target_id.clone());
        if is_public_doc_source(&reference.location.path, public_sources) {
            public.insert(reference.target_id.clone());
        } else {
            internal.insert(reference.target_id.clone());
        }
    }

    let undocumented_ids = target_ids
        .iter()
        .filter(|id| !documented.contains(*id))
        .cloned()
        .collect::<Vec<_>>();
    let internal_only = target_ids
        .iter()
        .filter(|id| internal.contains(*id) && !public.contains(*id))
        .count();

    DocsTargetKindMetrics {
        kind,
        total: target_ids.len(),
        documented: target_ids
            .iter()
            .filter(|id| documented.contains(*id))
            .count(),
        public: target_ids.iter().filter(|id| public.contains(*id)).count(),
        internal_only,
        undocumented: undocumented_ids.len(),
        undocumented_ids,
    }
}

fn configured_output_sources(root: &Path, outputs: &[DocsOutputConfig]) -> Vec<PathBuf> {
    outputs
        .iter()
        .map(|output| configured_docs_path(root, &output.source))
        .collect()
}

fn configured_docs_path(root: &Path, path: &Path) -> PathBuf {
    if path.is_absolute() {
        path.to_path_buf()
    } else {
        root.join(path)
    }
}

fn is_public_doc_source(path: &Path, public_sources: &[PathBuf]) -> bool {
    public_sources.iter().any(|source| {
        if source.is_file() || is_markdown_path(source) {
            path == source
        } else {
            path.starts_with(source)
        }
    })
}

fn build_public_docs_graph(
    root: &Path,
    outputs: &[DocsOutputConfig],
    entrypoints: &[PathBuf],
) -> Result<PublicDocsGraph> {
    let mut plan = Vec::new();
    for output in outputs {
        expand_output_mapping(
            &configured_docs_path(root, &output.source),
            &configured_docs_path(root, &output.output),
            &mut plan,
        )?;
    }
    let markdown_plan = plan
        .into_iter()
        .filter(|(source, output)| is_markdown_path(source) && is_markdown_path(output))
        .collect::<Vec<_>>();
    let pages = markdown_plan
        .iter()
        .map(|(_, output)| output.clone())
        .collect::<BTreeSet<_>>();
    let page_lookup = pages.iter().cloned().collect::<BTreeSet<_>>();
    let mut local_links = Vec::new();
    let mut broken_links = Vec::new();

    for (source, output) in &markdown_plan {
        let content =
            fs::read_to_string(source).with_context(|| format!("reading {}", source.display()))?;
        for link in collect_local_markdown_links(&content) {
            let target = resolve_local_doc_link(output, &link.target);
            let display_target = display_path(root, &target);
            if page_lookup.contains(&target) {
                local_links.push(PublicDocsLink {
                    source: output.clone(),
                    target,
                });
            } else {
                broken_links.push(DocsLocalLinkIssue {
                    source: display_path(root, output),
                    line: link.line,
                    target: display_target,
                });
            }
        }
    }

    let incoming = incoming_link_counts(&pages, &local_links);
    let entrypoint_pages = entrypoints
        .iter()
        .map(|entrypoint| configured_docs_path(root, entrypoint))
        .filter(|entrypoint| pages.contains(entrypoint))
        .collect::<BTreeSet<_>>();
    let orphan_pages = pages
        .iter()
        .filter(|page| !entrypoint_pages.contains(*page))
        .filter(|page| incoming.get(*page).copied().unwrap_or_default() == 0)
        .map(|page| display_path(root, page))
        .collect::<Vec<_>>();
    let reachable_pages_from_entrypoints =
        (!entrypoints.is_empty()).then(|| reachable_pages(&entrypoint_pages, &local_links).len());
    let entrypoint_pages_count = (!entrypoints.is_empty()).then_some(entrypoint_pages.len());

    Ok(PublicDocsGraph {
        pages,
        local_links,
        broken_links,
        orphan_pages,
        reachable_pages_from_entrypoints,
        entrypoint_pages: entrypoint_pages_count,
    })
}

#[derive(Debug)]
struct LocalMarkdownLink {
    line: usize,
    target: String,
}

fn collect_local_markdown_links(content: &str) -> Vec<LocalMarkdownLink> {
    let mut links = Vec::new();
    let mut in_code_fence = false;
    for (index, line) in content.lines().enumerate() {
        if starts_markdown_fence(line) {
            in_code_fence = !in_code_fence;
            continue;
        }
        if in_code_fence {
            continue;
        }
        collect_local_markdown_links_from_line(line, index + 1, &mut links);
    }
    links
}

fn collect_local_markdown_links_from_line(
    line: &str,
    line_number: usize,
    links: &mut Vec<LocalMarkdownLink>,
) {
    let bytes = line.as_bytes();
    let mut cursor = 0;
    while let Some(label_start_offset) = line[cursor..].find('[') {
        let label_start = cursor + label_start_offset;
        if label_start > 0 && bytes[label_start - 1] == b'!' {
            cursor = label_start + 1;
            continue;
        }
        let Some(label_end_offset) = line[label_start + 1..].find(']') else {
            break;
        };
        let label_end = label_start + 1 + label_end_offset;
        if !line[label_end + 1..].starts_with('(') {
            cursor = label_end + 1;
            continue;
        }
        let target_start = label_end + 2;
        let Some(target_end_offset) = line[target_start..].find(')') else {
            break;
        };
        let target_end = target_start + target_end_offset;
        let target = line[target_start..target_end].trim();
        if is_local_docs_link_target(target) {
            links.push(LocalMarkdownLink {
                line: line_number,
                target: target.to_string(),
            });
        }
        cursor = target_end + 1;
    }
}

fn is_local_docs_link_target(target: &str) -> bool {
    !target.is_empty()
        && !target.starts_with('#')
        && !target.starts_with("special://")
        && !target.starts_with("http://")
        && !target.starts_with("https://")
        && !target.starts_with("mailto:")
}

fn resolve_local_doc_link(source_output: &Path, target: &str) -> PathBuf {
    let without_fragment = target.split('#').next().unwrap_or(target);
    let target_path = Path::new(without_fragment);
    let resolved = if target_path.is_absolute() {
        target_path.to_path_buf()
    } else {
        source_output
            .parent()
            .unwrap_or_else(|| Path::new(""))
            .join(target_path)
    };
    normalize_lexical_path(&resolved)
}

fn normalize_lexical_path(path: &Path) -> PathBuf {
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

fn incoming_link_counts(
    pages: &BTreeSet<PathBuf>,
    links: &[PublicDocsLink],
) -> BTreeMap<PathBuf, usize> {
    let mut incoming = pages
        .iter()
        .map(|page| (page.clone(), 0))
        .collect::<BTreeMap<_, _>>();
    for link in links {
        if let Some(count) = incoming.get_mut(&link.target) {
            *count += 1;
        }
    }
    incoming
}

fn reachable_pages(entrypoints: &BTreeSet<PathBuf>, links: &[PublicDocsLink]) -> BTreeSet<PathBuf> {
    let mut adjacency: BTreeMap<PathBuf, Vec<PathBuf>> = BTreeMap::new();
    for link in links {
        adjacency
            .entry(link.source.clone())
            .or_default()
            .push(link.target.clone());
    }
    let mut visited = BTreeSet::new();
    let mut pending = entrypoints.iter().cloned().collect::<VecDeque<_>>();
    while let Some(page) = pending.pop_front() {
        if !visited.insert(page.clone()) {
            continue;
        }
        if let Some(targets) = adjacency.get(&page) {
            pending.extend(targets.iter().cloned());
        }
    }
    visited
}

fn collect_repo_document_refs(
    root: &Path,
    ignore_patterns: &[String],
) -> Result<(Vec<DocumentRef>, Vec<Diagnostic>)> {
    let mut refs = Vec::new();
    let mut diagnostics = Vec::new();
    collect_source_document_refs(root, ignore_patterns, &mut refs, &mut diagnostics)?;
    collect_markdown_document_refs(root, ignore_patterns, &mut refs, &mut diagnostics)?;
    Ok((refs, diagnostics))
}

fn scoped_refs(refs: Vec<DocumentRef>, scope_paths: &[PathBuf]) -> Vec<DocumentRef> {
    if scope_paths.is_empty() {
        return refs;
    }
    refs.into_iter()
        .filter(|reference| scope_matches(&reference.location.path, scope_paths))
        .collect()
}

fn retain_scoped_diagnostics(diagnostics: &mut Vec<Diagnostic>, scope_paths: &[PathBuf]) {
    if scope_paths.is_empty() {
        return;
    }
    diagnostics.retain(|diagnostic| scope_matches(&diagnostic.path, scope_paths));
}

fn scope_matches(path: &Path, scope_paths: &[PathBuf]) -> bool {
    scope_paths.iter().any(|scope| {
        if scope.is_dir() {
            path.starts_with(scope)
        } else {
            path == scope || path.starts_with(scope)
        }
    })
}

fn collect_source_document_refs(
    root: &Path,
    ignore_patterns: &[String],
    refs: &mut Vec<DocumentRef>,
    diagnostics: &mut Vec<Diagnostic>,
) -> Result<()> {
    for block in collect_comment_blocks(root, ignore_patterns)? {
        let mut previous_docs_line = false;
        for entry in &block.lines {
            let parsed = parse_documents_annotation_line(
                entry.text.trim(),
                &block.path,
                entry.line,
                refs,
                diagnostics,
            );
            if parsed && previous_docs_line {
                push_stacked_document_line_diagnostic(&block.path, entry.line, diagnostics);
            }
            previous_docs_line = parsed;
        }
    }
    Ok(())
}

fn collect_markdown_document_refs(
    root: &Path,
    ignore_patterns: &[String],
    refs: &mut Vec<DocumentRef>,
    diagnostics: &mut Vec<Diagnostic>,
) -> Result<()> {
    for path in discover_annotation_files(DiscoveryConfig {
        root,
        ignore_patterns,
    })?
    .markdown_files
    {
        let content =
            fs::read_to_string(&path).with_context(|| format!("reading {}", path.display()))?;
        collect_markdown_refs(&path, &content, refs, diagnostics);
    }
    Ok(())
}

fn collect_markdown_refs(
    path: &Path,
    content: &str,
    refs: &mut Vec<DocumentRef>,
    diagnostics: &mut Vec<Diagnostic>,
) {
    let mut in_code_fence = false;
    let mut previous_docs_line = false;
    for (index, line) in content.lines().enumerate() {
        let line_number = index + 1;
        if starts_markdown_fence(line) {
            in_code_fence = !in_code_fence;
            previous_docs_line = false;
            continue;
        }
        if in_code_fence {
            continue;
        }
        let parsed = parse_markdown_documents_line(line, path, line_number, refs, diagnostics);
        if parsed && previous_docs_line {
            push_stacked_document_line_diagnostic(path, line_number, diagnostics);
        }
        previous_docs_line = parsed;
        collect_special_link_refs(path, line, line_number, refs, diagnostics);
    }
}

fn write_markdown_output(
    path: &Path,
    content: &str,
    refs: &mut Vec<DocumentRef>,
    diagnostics: &mut Vec<Diagnostic>,
) -> String {
    let mut output = String::new();
    let mut in_code_fence = false;
    let mut previous_docs_line = false;

    for (index, line) in content.split_inclusive('\n').enumerate() {
        let line_number = index + 1;
        let raw = line.trim_end_matches('\n').trim_end_matches('\r');
        if starts_markdown_fence(raw) {
            in_code_fence = !in_code_fence;
            previous_docs_line = false;
            output.push_str(line);
            continue;
        }
        if in_code_fence {
            output.push_str(line);
            continue;
        }

        if parse_markdown_documents_line(raw, path, line_number, refs, diagnostics) {
            if previous_docs_line {
                push_stacked_document_line_diagnostic(path, line_number, diagnostics);
            }
            previous_docs_line = true;
            continue;
        }
        previous_docs_line = false;

        output.push_str(&write_special_link_output(
            path,
            line,
            line_number,
            refs,
            diagnostics,
        ));
    }

    output
}

fn parse_markdown_documents_line(
    line: &str,
    path: &Path,
    line_number: usize,
    refs: &mut Vec<DocumentRef>,
    diagnostics: &mut Vec<Diagnostic>,
) -> bool {
    if parse_source_annotations::is_whole_line_code_span(line) {
        return false;
    }
    let Some(trimmed) = normalize_markdown_declaration_line(line) else {
        return false;
    };
    parse_documents_annotation_line(trimmed, path, line_number, refs, diagnostics)
}

fn push_stacked_document_line_diagnostic(
    path: &Path,
    line_number: usize,
    diagnostics: &mut Vec<Diagnostic>,
) {
    diagnostics.push(Diagnostic {
        severity: DiagnosticSeverity::Error,
        path: path.to_path_buf(),
        line: line_number,
        message: "documentation relationship lines may not be stacked; use local special:// links"
            .to_string(),
    });
}

fn write_special_link_output(
    path: &Path,
    line: &str,
    line_number: usize,
    refs: &mut Vec<DocumentRef>,
    diagnostics: &mut Vec<Diagnostic>,
) -> String {
    let mut output = String::new();
    let mut cursor = 0;

    for link in parse_source_annotations::special_markdown_link_candidates(line) {
        output.push_str(&line[cursor..link.span.start]);
        output.push_str(link.label);
        push_special_link_ref(path, line_number, link, refs, diagnostics);
        cursor = link.span.end;
    }

    output.push_str(&line[cursor..]);
    output
}

fn collect_special_link_refs(
    path: &Path,
    line: &str,
    line_number: usize,
    refs: &mut Vec<DocumentRef>,
    diagnostics: &mut Vec<Diagnostic>,
) {
    for link in parse_source_annotations::special_markdown_link_candidates(line) {
        push_special_link_ref(path, line_number, link, refs, diagnostics);
    }
}

fn push_special_link_ref(
    path: &Path,
    line_number: usize,
    link: parse_source_annotations::SpecialMarkdownLinkCandidate<'_>,
    refs: &mut Vec<DocumentRef>,
    diagnostics: &mut Vec<Diagnostic>,
) {
    let Some(target) = link.target else {
        diagnostics.push(Diagnostic {
            severity: DiagnosticSeverity::Error,
            path: path.to_path_buf(),
            line: line_number,
            message: format!("malformed Special docs URI `{}`", link.uri),
        });
        return;
    };

    match DocumentTargetKind::parse(target.kind) {
        Some(target_kind) => refs.push(DocumentRef {
            target_kind,
            target_id: target.id.to_string(),
            location: SourceLocation {
                path: path.to_path_buf(),
                line: line_number,
            },
            source: DocumentRefSource::Link,
            text: Some(link.label.to_string()),
        }),
        None => diagnostics.push(Diagnostic {
            severity: DiagnosticSeverity::Error,
            path: path.to_path_buf(),
            line: line_number,
            message: format!("unknown Special docs target kind `{}`", target.kind),
        }),
    }
}

fn parse_documents_annotation_line(
    line: &str,
    path: &Path,
    line_number: usize,
    refs: &mut Vec<DocumentRef>,
    diagnostics: &mut Vec<Diagnostic>,
) -> bool {
    parse_documents_line(
        line,
        path,
        line_number,
        DocumentRefSource::DocumentsLine,
        refs,
        diagnostics,
    ) || parse_filedocuments_line(
        line,
        path,
        line_number,
        DocumentRefSource::FileDocumentsLine,
        refs,
        diagnostics,
    )
}

fn parse_documents_line(
    line: &str,
    path: &Path,
    line_number: usize,
    source: DocumentRefSource,
    refs: &mut Vec<DocumentRef>,
    diagnostics: &mut Vec<Diagnostic>,
) -> bool {
    let Some(rest) = reserved_special_annotation_rest(line, ReservedSpecialAnnotation::Documents)
    else {
        return false;
    };
    parse_documents_rest(rest, path, line_number, source, refs, diagnostics);
    true
}

fn parse_filedocuments_line(
    line: &str,
    path: &Path,
    line_number: usize,
    source: DocumentRefSource,
    refs: &mut Vec<DocumentRef>,
    diagnostics: &mut Vec<Diagnostic>,
) -> bool {
    let Some(rest) =
        reserved_special_annotation_rest(line, ReservedSpecialAnnotation::FileDocuments)
    else {
        return false;
    };
    parse_documents_rest(rest, path, line_number, source, refs, diagnostics);
    true
}

fn parse_documents_rest(
    rest: &str,
    path: &Path,
    line_number: usize,
    source: DocumentRefSource,
    refs: &mut Vec<DocumentRef>,
    diagnostics: &mut Vec<Diagnostic>,
) {
    let mut parts = rest.split_whitespace();
    let Some(kind) = parts.next() else {
        diagnostics.push(Diagnostic {
            severity: DiagnosticSeverity::Error,
            path: path.to_path_buf(),
            line: line_number,
            message: "missing docs target kind after documentation annotation".to_string(),
        });
        return;
    };
    let Some(target_kind) = DocumentTargetKind::parse(kind) else {
        diagnostics.push(Diagnostic {
            severity: DiagnosticSeverity::Error,
            path: path.to_path_buf(),
            line: line_number,
            message: format!("unknown docs target kind `{kind}`"),
        });
        return;
    };
    let Some(id) = parts.next() else {
        diagnostics.push(Diagnostic {
            severity: DiagnosticSeverity::Error,
            path: path.to_path_buf(),
            line: line_number,
            message: "missing docs target id after documentation annotation".to_string(),
        });
        return;
    };
    if parts.next().is_some() {
        diagnostics.push(Diagnostic {
            severity: DiagnosticSeverity::Error,
            path: path.to_path_buf(),
            line: line_number,
            message: "unexpected trailing content after documentation target id".to_string(),
        });
        return;
    }
    refs.push(DocumentRef {
        target_kind,
        target_id: id.to_string(),
        location: SourceLocation {
            path: path.to_path_buf(),
            line: line_number,
        },
        source,
        text: None,
    });
}

fn validate_document_refs(
    root: &Path,
    ignore_patterns: &[String],
    version: SpecialVersion,
    refs: Vec<DocumentRef>,
) -> Result<Vec<Diagnostic>> {
    let targets = DocumentTargets::load(root, ignore_patterns, version)?;
    Ok(refs
        .into_iter()
        .filter_map(|reference| {
            if targets.contains(reference.target_kind, &reference.target_id) {
                None
            } else {
                Some(Diagnostic {
                    severity: DiagnosticSeverity::Error,
                    path: reference.location.path,
                    line: reference.location.line,
                    message: format!(
                        "unknown documentation target {} `{}`",
                        reference.target_kind.as_str(),
                        reference.target_id
                    ),
                })
            }
        })
        .collect())
}

struct DocumentTargets {
    specs: BTreeSet<String>,
    groups: BTreeSet<String>,
    modules: BTreeSet<String>,
    areas: BTreeSet<String>,
    patterns: BTreeSet<String>,
}

impl DocumentTargets {
    fn load(root: &Path, ignore_patterns: &[String], version: SpecialVersion) -> Result<Self> {
        let parsed_repo = load_or_parse_repo(root, ignore_patterns, version)?;
        let parsed_architecture = load_or_parse_architecture(root, ignore_patterns)?;
        Ok(Self {
            specs: parsed_repo
                .specs
                .iter()
                .filter(|decl| decl.kind() == NodeKind::Spec)
                .map(|decl| decl.id.clone())
                .collect(),
            groups: parsed_repo
                .specs
                .iter()
                .filter(|decl| decl.kind() == NodeKind::Group)
                .map(|decl| decl.id.clone())
                .collect(),
            modules: parsed_architecture
                .modules
                .iter()
                .filter(|decl| decl.kind() == ArchitectureKind::Module)
                .map(|decl| decl.id.clone())
                .collect(),
            areas: parsed_architecture
                .modules
                .iter()
                .filter(|decl| decl.kind() == ArchitectureKind::Area)
                .map(|decl| decl.id.clone())
                .collect(),
            patterns: parsed_architecture
                .patterns
                .iter()
                .map(|definition| definition.pattern_id.clone())
                .collect(),
        })
    }

    fn contains(&self, kind: DocumentTargetKind, id: &str) -> bool {
        match kind {
            DocumentTargetKind::Spec => self.specs.contains(id),
            DocumentTargetKind::Group => self.groups.contains(id),
            DocumentTargetKind::Module => self.modules.contains(id),
            DocumentTargetKind::Area => self.areas.contains(id),
            DocumentTargetKind::Pattern => self.patterns.contains(id),
        }
    }

    fn ids(&self, kind: DocumentTargetKind) -> &BTreeSet<String> {
        match kind {
            DocumentTargetKind::Spec => &self.specs,
            DocumentTargetKind::Group => &self.groups,
            DocumentTargetKind::Module => &self.modules,
            DocumentTargetKind::Area => &self.areas,
            DocumentTargetKind::Pattern => &self.patterns,
        }
    }
}

fn is_markdown_path(path: &Path) -> bool {
    matches!(path.extension().and_then(|ext| ext.to_str()), Some("md"))
}

fn validate_output_plan(files: &[(PathBuf, PathBuf)]) -> Result<()> {
    let mut outputs = BTreeSet::new();
    for (input, output) in files {
        ensure_distinct_paths(
            input,
            output,
            "docs output path must not equal an input path",
        )?;
        let normalized_output = normalize_existing_path(output);
        if !outputs.insert(normalized_output) {
            bail!("docs output maps multiple inputs to {}", output.display());
        }
        if output.exists() && existing_file_contains_docs_evidence(output)? {
            bail!(
                "refusing to overwrite docs evidence in {}; choose a separate output path",
                output.display()
            );
        }
    }
    Ok(())
}

fn existing_file_contains_docs_evidence(path: &Path) -> Result<bool> {
    let content =
        fs::read_to_string(path).with_context(|| format!("reading {}", path.display()))?;
    let mut in_fenced_code = false;
    for line in content.lines() {
        if line.trim_start().starts_with("```") {
            in_fenced_code = !in_fenced_code;
            continue;
        }
        if in_fenced_code {
            continue;
        }
        let evidence_line = remove_inline_code_spans(line);
        if evidence_line.contains("special://")
            || evidence_line.contains("@documents")
            || evidence_line.contains("@filedocuments")
        {
            return Ok(true);
        }
    }
    Ok(false)
}

fn remove_inline_code_spans(line: &str) -> String {
    let mut output = String::new();
    let mut in_code = false;
    for character in line.chars() {
        if character == '`' {
            in_code = !in_code;
            continue;
        }
        if !in_code {
            output.push(character);
        }
    }
    output
}

fn ensure_distinct_paths(left: &Path, right: &Path, message: &str) -> Result<()> {
    if normalize_existing_path(left) == normalize_existing_path(right) {
        bail!("{message}");
    }
    Ok(())
}

fn path_is_inside(path: &Path, parent: &Path) -> bool {
    let path = normalize_existing_path(path);
    let parent = normalize_existing_path(parent);
    path != parent && path.starts_with(parent)
}

fn normalize_existing_path(path: &Path) -> PathBuf {
    fs::canonicalize(path).unwrap_or_else(|_| path.to_path_buf())
}

fn display_path(root: &Path, path: &Path) -> String {
    path.strip_prefix(root)
        .unwrap_or(path)
        .display()
        .to_string()
}

fn sort_diagnostics(diagnostics: &mut [Diagnostic]) {
    diagnostics.sort_by(|left, right| {
        left.path
            .cmp(&right.path)
            .then(left.line.cmp(&right.line))
            .then(left.message.cmp(&right.message))
    });
}
