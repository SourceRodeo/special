/**
@module SPECIAL.TRACE
Builds deterministic relationship packets for audit workflows over specs, docs, architecture, and patterns.

@spec SPECIAL.TRACE_COMMAND
special trace emits deterministic relationship packets for explicit Special surfaces without making natural-language truth judgments.

@spec SPECIAL.TRACE_COMMAND.SPECS
special trace specs emits current spec packets with verifier and attestation evidence bodies.

@spec SPECIAL.TRACE_COMMAND.DOCS
special trace docs emits documentation relationship packets with source prose context and the linked target evidence chain.

@spec SPECIAL.TRACE_COMMAND.ARCH
special trace arch emits module and area packets with implementation and pattern-application attachments.

@spec SPECIAL.TRACE_COMMAND.PATTERNS
special trace patterns emits pattern packets with definitions, applications, and module joins.

@spec SPECIAL.TRACE_COMMAND.FILTERS
special trace supports id and path filters for focused audit packet generation.
*/
// @fileimplements SPECIAL.TRACE
use std::collections::{BTreeMap, BTreeSet};
use std::fs;
use std::path::{Path, PathBuf};

use anyhow::Result;
use serde::{Deserialize, Serialize};

use crate::cache::{load_or_parse_architecture, load_or_parse_repo};
use crate::config::SpecialVersion;
use crate::docs::{DocumentRef, DocumentRefSource, DocumentTargetKind, build_docs_document};
use crate::model::{
    ArchitectureKind, AttestRef, ImplementRef, NodeKind, ParsedArchitecture, ParsedRepo,
    PatternApplication, PatternDefinition, SourceLocation, SpecDecl, VerifyRef,
};
use crate::source_paths::matches_scope_path;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub(crate) enum TraceSurface {
    Specs,
    Docs,
    Arch,
    Patterns,
}

impl TraceSurface {
    fn as_str(self) -> &'static str {
        match self {
            Self::Specs => "specs",
            Self::Docs => "docs",
            Self::Arch => "arch",
            Self::Patterns => "patterns",
        }
    }
}

#[derive(Debug, Clone, Default)]
pub(crate) struct TraceOptions {
    pub id: Option<String>,
    pub target_paths: Vec<PathBuf>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub(crate) struct TraceDocument {
    pub surface: TraceSurface,
    pub summary: TraceSummary,
    pub packets: Vec<TracePacket>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub(crate) struct TraceSummary {
    pub packets: usize,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub id_filter: Option<String>,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub target_paths: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub(crate) struct TracePacket {
    pub target: TraceTarget,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub references: Vec<TraceReference>,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub evidence: Vec<TraceEvidence>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub(crate) struct TraceTarget {
    pub kind: String,
    pub id: String,
    pub text: String,
    pub location: TraceLocation,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub lifecycle: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub(crate) struct TraceReference {
    pub source: String,
    pub line: usize,
    pub relationship: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub text: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub surrounding_prose: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub(crate) struct TraceEvidence {
    pub relationship: String,
    pub location: TraceLocation,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub body_location: Option<TraceLocation>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub body: Option<String>,
    #[serde(skip_serializing_if = "BTreeMap::is_empty")]
    pub fields: BTreeMap<String, String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub(crate) struct TraceLocation {
    pub path: String,
    pub line: usize,
}

pub(crate) fn build_trace_document(
    root: &Path,
    ignore_patterns: &[String],
    version: SpecialVersion,
    surface: TraceSurface,
    options: TraceOptions,
) -> Result<TraceDocument> {
    let parsed_repo = load_or_parse_repo(root, ignore_patterns, version)?;
    let parsed_architecture = load_or_parse_architecture(root, ignore_patterns)?;
    let packets = match surface {
        TraceSurface::Specs => spec_packets(root, &parsed_repo, &options),
        TraceSurface::Docs => docs_packets(
            root,
            ignore_patterns,
            version,
            &parsed_repo,
            &parsed_architecture,
            &options,
        )?,
        TraceSurface::Arch => arch_packets(root, &parsed_architecture, &options),
        TraceSurface::Patterns => pattern_packets(root, &parsed_architecture, &options),
    };
    let summary = TraceSummary {
        packets: packets.len(),
        id_filter: options.id,
        target_paths: options
            .target_paths
            .iter()
            .map(|path| display_path(root, path))
            .collect(),
    };
    Ok(TraceDocument {
        surface,
        summary,
        packets,
    })
}

pub(crate) fn render_trace_json(document: &TraceDocument) -> Result<String> {
    Ok(serde_json::to_string_pretty(document)? + "\n")
}

pub(crate) fn render_trace_text(document: &TraceDocument) -> String {
    let mut output = String::new();
    output.push_str(&format!("special trace {}\n", document.surface.as_str()));
    output.push_str(&format!("packets: {}\n", document.summary.packets));
    if let Some(id) = &document.summary.id_filter {
        output.push_str(&format!("id filter: {id}\n"));
    }
    if !document.summary.target_paths.is_empty() {
        output.push_str("target paths:\n");
        for path in &document.summary.target_paths {
            output.push_str(&format!("  {path}\n"));
        }
    }
    output.push('\n');

    for packet in &document.packets {
        output.push_str(&format!(
            "{} {} @ {}:{}\n",
            packet.target.kind,
            packet.target.id,
            packet.target.location.path,
            packet.target.location.line
        ));
        if let Some(lifecycle) = &packet.target.lifecycle {
            output.push_str(&format!("  lifecycle: {lifecycle}\n"));
        }
        if !packet.target.text.is_empty() {
            output.push_str(&format!("  text: {}\n", packet.target.text));
        }
        if !packet.references.is_empty() {
            output.push_str("  references:\n");
            for reference in &packet.references {
                output.push_str(&format!(
                    "    {} @ {}:{}\n",
                    reference.relationship, reference.source, reference.line
                ));
                if let Some(text) = &reference.text {
                    output.push_str(&format!("      text: {text}\n"));
                }
                if let Some(prose) = &reference.surrounding_prose {
                    output.push_str("      surrounding prose:\n");
                    for line in prose.lines() {
                        output.push_str(&format!("        {line}\n"));
                    }
                }
            }
        }
        if !packet.evidence.is_empty() {
            output.push_str("  evidence:\n");
            for evidence in &packet.evidence {
                output.push_str(&format!(
                    "    {} @ {}:{}\n",
                    evidence.relationship, evidence.location.path, evidence.location.line
                ));
                if let Some(body_location) = &evidence.body_location {
                    output.push_str(&format!(
                        "      body @ {}:{}\n",
                        body_location.path, body_location.line
                    ));
                }
                for (key, value) in &evidence.fields {
                    output.push_str(&format!("      {key}: {value}\n"));
                }
                if let Some(body) = &evidence.body {
                    output.push_str("      body:\n");
                    for line in body.lines() {
                        output.push_str(&format!("        {line}\n"));
                    }
                }
            }
        }
        output.push('\n');
    }
    output
}

fn spec_packets(root: &Path, parsed_repo: &ParsedRepo, options: &TraceOptions) -> Vec<TracePacket> {
    parsed_repo
        .specs
        .iter()
        .filter(|decl| decl.kind() == NodeKind::Spec && !decl.is_planned() && !decl.is_deprecated())
        .filter(|decl| id_matches(&decl.id, options.id.as_deref()))
        .filter(|decl| {
            trace_location_matches(&decl.location, &options.target_paths)
                || parsed_repo
                    .verifies
                    .iter()
                    .filter(|verify| verify.spec_id == decl.id)
                    .any(|verify| trace_location_matches(&verify.location, &options.target_paths))
                || parsed_repo
                    .attests
                    .iter()
                    .filter(|attest| attest.spec_id == decl.id)
                    .any(|attest| trace_location_matches(&attest.location, &options.target_paths))
        })
        .map(|decl| TracePacket {
            target: spec_target(root, decl),
            references: Vec::new(),
            evidence: spec_evidence(root, parsed_repo, &decl.id),
        })
        .collect()
}

fn docs_packets(
    root: &Path,
    ignore_patterns: &[String],
    version: SpecialVersion,
    parsed_repo: &ParsedRepo,
    parsed_architecture: &ParsedArchitecture,
    options: &TraceOptions,
) -> Result<Vec<TracePacket>> {
    let (document, _) = build_docs_document(root, ignore_patterns, version, &options.target_paths)?;
    let index = TargetIndex::new(root, parsed_repo, parsed_architecture);
    let mut packets = Vec::new();
    for reference in document.references {
        if !id_matches(&reference.target_id, options.id.as_deref()) {
            continue;
        }
        if !trace_location_matches(&reference.location, &options.target_paths) {
            continue;
        }
        let target = index
            .target(reference.target_kind, &reference.target_id)
            .unwrap_or_else(|| missing_target(root, reference.target_kind, &reference.target_id));
        let evidence = index.evidence(root, reference.target_kind, &reference.target_id);
        packets.push(TracePacket {
            target,
            references: vec![docs_reference(root, &reference)],
            evidence,
        });
    }
    Ok(packets)
}

fn arch_packets(
    root: &Path,
    parsed_architecture: &ParsedArchitecture,
    options: &TraceOptions,
) -> Vec<TracePacket> {
    parsed_architecture
        .modules
        .iter()
        .filter(|decl| id_matches(&decl.id, options.id.as_deref()))
        .filter(|decl| {
            trace_location_matches(&decl.location, &options.target_paths)
                || parsed_architecture
                    .implements
                    .iter()
                    .filter(|implementation| implementation.module_id == decl.id)
                    .any(|implementation| {
                        trace_location_matches(&implementation.location, &options.target_paths)
                    })
        })
        .map(|decl| TracePacket {
            target: module_target(root, decl),
            references: Vec::new(),
            evidence: architecture_evidence(root, parsed_architecture, &decl.id),
        })
        .collect()
}

fn pattern_packets(
    root: &Path,
    parsed_architecture: &ParsedArchitecture,
    options: &TraceOptions,
) -> Vec<TracePacket> {
    let modules_by_pattern = modules_by_pattern(parsed_architecture);
    parsed_architecture
        .patterns
        .iter()
        .filter(|definition| id_matches(&definition.pattern_id, options.id.as_deref()))
        .filter(|definition| {
            trace_location_matches(&definition.location, &options.target_paths)
                || parsed_architecture
                    .pattern_applications
                    .iter()
                    .filter(|application| application.pattern_id == definition.pattern_id)
                    .any(|application| {
                        trace_location_matches(&application.location, &options.target_paths)
                    })
        })
        .map(|definition| TracePacket {
            target: pattern_target(root, definition),
            references: Vec::new(),
            evidence: pattern_evidence(
                root,
                parsed_architecture,
                &modules_by_pattern,
                &definition.pattern_id,
            ),
        })
        .collect()
}

struct TargetIndex<'a> {
    root: &'a Path,
    specs: BTreeMap<&'a str, &'a SpecDecl>,
    modules: BTreeMap<&'a str, &'a crate::model::ModuleDecl>,
    patterns: BTreeMap<&'a str, &'a PatternDefinition>,
    parsed_repo: &'a ParsedRepo,
    parsed_architecture: &'a ParsedArchitecture,
}

impl<'a> TargetIndex<'a> {
    fn new(
        root: &'a Path,
        parsed_repo: &'a ParsedRepo,
        parsed_architecture: &'a ParsedArchitecture,
    ) -> Self {
        Self {
            root,
            specs: parsed_repo
                .specs
                .iter()
                .map(|decl| (decl.id.as_str(), decl))
                .collect(),
            modules: parsed_architecture
                .modules
                .iter()
                .map(|decl| (decl.id.as_str(), decl))
                .collect(),
            patterns: parsed_architecture
                .patterns
                .iter()
                .map(|definition| (definition.pattern_id.as_str(), definition))
                .collect(),
            parsed_repo,
            parsed_architecture,
        }
    }

    fn target(&self, kind: DocumentTargetKind, id: &str) -> Option<TraceTarget> {
        match kind {
            DocumentTargetKind::Spec | DocumentTargetKind::Group => {
                self.specs.get(id).map(|decl| spec_target(self.root, decl))
            }
            DocumentTargetKind::Module | DocumentTargetKind::Area => self
                .modules
                .get(id)
                .map(|decl| module_target(self.root, decl)),
            DocumentTargetKind::Pattern => self
                .patterns
                .get(id)
                .map(|definition| pattern_target(self.root, definition)),
        }
    }

    fn evidence(&self, root: &Path, kind: DocumentTargetKind, id: &str) -> Vec<TraceEvidence> {
        match kind {
            DocumentTargetKind::Spec | DocumentTargetKind::Group => {
                spec_evidence(root, self.parsed_repo, id)
            }
            DocumentTargetKind::Module | DocumentTargetKind::Area => {
                architecture_evidence(root, self.parsed_architecture, id)
            }
            DocumentTargetKind::Pattern => pattern_evidence(
                root,
                self.parsed_architecture,
                &modules_by_pattern(self.parsed_architecture),
                id,
            ),
        }
    }
}

fn spec_target(root: &Path, decl: &SpecDecl) -> TraceTarget {
    TraceTarget {
        kind: match decl.kind() {
            NodeKind::Spec => "spec".to_string(),
            NodeKind::Group => "group".to_string(),
        },
        id: decl.id.clone(),
        text: decl.text.clone(),
        location: trace_location(root, &decl.location),
        lifecycle: spec_lifecycle(decl),
    }
}

fn module_target(root: &Path, decl: &crate::model::ModuleDecl) -> TraceTarget {
    TraceTarget {
        kind: match decl.kind() {
            ArchitectureKind::Module => "module".to_string(),
            ArchitectureKind::Area => "area".to_string(),
        },
        id: decl.id.clone(),
        text: decl.text.clone(),
        location: trace_location(root, &decl.location),
        lifecycle: decl.is_planned().then(|| "planned".to_string()),
    }
}

fn pattern_target(root: &Path, definition: &PatternDefinition) -> TraceTarget {
    TraceTarget {
        kind: "pattern".to_string(),
        id: definition.pattern_id.clone(),
        text: definition.text.clone(),
        location: trace_location(root, &definition.location),
        lifecycle: Some(format!("strictness: {}", definition.strictness.as_str())),
    }
}

fn missing_target(root: &Path, kind: DocumentTargetKind, id: &str) -> TraceTarget {
    TraceTarget {
        kind: document_target_kind_label(kind).to_string(),
        id: id.to_string(),
        text: "missing target".to_string(),
        location: TraceLocation {
            path: display_path(root, root),
            line: 0,
        },
        lifecycle: None,
    }
}

fn spec_lifecycle(decl: &SpecDecl) -> Option<String> {
    if let Some(release) = decl.planned_release() {
        return Some(format!("planned {release}"));
    }
    if decl.is_planned() {
        return Some("planned".to_string());
    }
    if let Some(release) = decl.deprecated_release() {
        return Some(format!("deprecated {release}"));
    }
    if decl.is_deprecated() {
        return Some("deprecated".to_string());
    }
    None
}

fn spec_evidence(root: &Path, parsed_repo: &ParsedRepo, id: &str) -> Vec<TraceEvidence> {
    parsed_repo
        .verifies
        .iter()
        .filter(|verify| verify.spec_id == id)
        .map(|verify| verify_evidence(root, verify))
        .chain(
            parsed_repo
                .attests
                .iter()
                .filter(|attest| attest.spec_id == id)
                .map(|attest| attest_evidence(root, attest)),
        )
        .collect()
}

fn architecture_evidence(
    root: &Path,
    parsed_architecture: &ParsedArchitecture,
    id: &str,
) -> Vec<TraceEvidence> {
    parsed_architecture
        .implements
        .iter()
        .filter(|implementation| implementation.module_id == id)
        .map(|implementation| implementation_evidence(root, implementation))
        .chain(
            parsed_architecture
                .pattern_applications
                .iter()
                .filter_map(|application| {
                    let module_id =
                        owning_module_for_application(parsed_architecture, application)?;
                    (module_id == id).then(|| pattern_application_evidence(root, application, None))
                }),
        )
        .collect()
}

fn pattern_evidence(
    root: &Path,
    parsed_architecture: &ParsedArchitecture,
    modules_by_pattern: &BTreeMap<String, BTreeSet<String>>,
    id: &str,
) -> Vec<TraceEvidence> {
    let mut evidence = parsed_architecture
        .pattern_applications
        .iter()
        .filter(|application| application.pattern_id == id)
        .map(|application| {
            pattern_application_evidence(
                root,
                application,
                owning_module_for_application(parsed_architecture, application),
            )
        })
        .collect::<Vec<_>>();
    if let Some(modules) = modules_by_pattern.get(id) {
        for module_id in modules {
            let mut fields = BTreeMap::new();
            fields.insert("module_id".to_string(), module_id.clone());
            evidence.push(TraceEvidence {
                relationship: "module_join".to_string(),
                location: TraceLocation {
                    path: String::new(),
                    line: 0,
                },
                body_location: None,
                body: None,
                fields,
            });
        }
    }
    evidence
}

fn verify_evidence(root: &Path, verify: &VerifyRef) -> TraceEvidence {
    TraceEvidence {
        relationship: "@verifies".to_string(),
        location: trace_location(root, &verify.location),
        body_location: verify
            .body_location
            .as_ref()
            .map(|location| trace_location(root, location)),
        body: verify.body.clone(),
        fields: BTreeMap::new(),
    }
}

fn attest_evidence(root: &Path, attest: &AttestRef) -> TraceEvidence {
    let mut fields = BTreeMap::new();
    fields.insert("artifact".to_string(), attest.artifact.clone());
    fields.insert("owner".to_string(), attest.owner.clone());
    fields.insert("last_reviewed".to_string(), attest.last_reviewed.clone());
    if let Some(days) = attest.review_interval_days {
        fields.insert("review_interval_days".to_string(), days.to_string());
    }
    fields.insert(
        "scope".to_string(),
        format!("{:?}", attest.scope).to_lowercase(),
    );
    TraceEvidence {
        relationship: attest.scope.as_annotation().to_string(),
        location: trace_location(root, &attest.location),
        body_location: None,
        body: attest.body.clone(),
        fields,
    }
}

fn implementation_evidence(root: &Path, implementation: &ImplementRef) -> TraceEvidence {
    TraceEvidence {
        relationship: "@implements".to_string(),
        location: trace_location(root, &implementation.location),
        body_location: implementation
            .body_location
            .as_ref()
            .map(|location| trace_location(root, location)),
        body: implementation.body.clone(),
        fields: BTreeMap::new(),
    }
}

fn pattern_application_evidence(
    root: &Path,
    application: &PatternApplication,
    module_id: Option<String>,
) -> TraceEvidence {
    let mut fields = BTreeMap::new();
    if let Some(module_id) = module_id {
        fields.insert("module_id".to_string(), module_id);
    }
    TraceEvidence {
        relationship: "@applies".to_string(),
        location: trace_location(root, &application.location),
        body_location: application
            .body_location
            .as_ref()
            .map(|location| trace_location(root, location)),
        body: application.body.clone(),
        fields,
    }
}

fn docs_reference(root: &Path, reference: &DocumentRef) -> TraceReference {
    TraceReference {
        source: display_path(root, &reference.location.path),
        line: reference.location.line,
        relationship: document_ref_source_label(reference.source).to_string(),
        text: reference.text.clone(),
        surrounding_prose: prose_context(&reference.location.path, reference.location.line),
    }
}

fn prose_context(path: &Path, line: usize) -> Option<String> {
    let text = fs::read_to_string(path).ok()?;
    let lines = text.lines().collect::<Vec<_>>();
    if line == 0 || line > lines.len() {
        return None;
    }
    let index = line - 1;
    let mut start = index;
    while start > 0 && !lines[start - 1].trim().is_empty() {
        start -= 1;
    }
    let mut end = index;
    while end + 1 < lines.len() && !lines[end + 1].trim().is_empty() {
        end += 1;
    }
    let mut context = (start..=end)
        .map(|line_index| format!("{}: {}", line_index + 1, lines[line_index]))
        .collect::<Vec<_>>();
    if context.len() > 8 {
        let low = index.saturating_sub(3);
        let high = (index + 3).min(lines.len() - 1);
        context = (low..=high)
            .map(|line_index| format!("{}: {}", line_index + 1, lines[line_index]))
            .collect();
    }
    Some(context.join("\n"))
}

fn modules_by_pattern(
    parsed_architecture: &ParsedArchitecture,
) -> BTreeMap<String, BTreeSet<String>> {
    let mut modules = BTreeMap::<String, BTreeSet<String>>::new();
    for application in &parsed_architecture.pattern_applications {
        if let Some(module_id) = owning_module_for_application(parsed_architecture, application) {
            modules
                .entry(application.pattern_id.clone())
                .or_default()
                .insert(module_id);
        }
    }
    modules
}

fn owning_module_for_application(
    parsed_architecture: &ParsedArchitecture,
    application: &PatternApplication,
) -> Option<String> {
    parsed_architecture
        .implements
        .iter()
        .filter(|implementation| implementation.location.path == application.location.path)
        .filter(|implementation| implementation.location.line <= application.location.line)
        .max_by_key(|implementation| implementation.location.line)
        .map(|implementation| implementation.module_id.clone())
}

fn trace_location(root: &Path, location: &SourceLocation) -> TraceLocation {
    TraceLocation {
        path: display_path(root, &location.path),
        line: location.line,
    }
}

fn trace_location_matches(location: &SourceLocation, scope_paths: &[PathBuf]) -> bool {
    scope_paths.is_empty() || matches_scope_path(&location.path, scope_paths)
}

fn id_matches(id: &str, scope: Option<&str>) -> bool {
    let Some(scope) = scope else {
        return true;
    };
    id == scope
        || id
            .strip_prefix(scope)
            .is_some_and(|suffix| suffix.starts_with('.'))
}

fn display_path(root: &Path, path: &Path) -> String {
    path.strip_prefix(root)
        .unwrap_or(path)
        .to_string_lossy()
        .to_string()
}

fn document_target_kind_label(kind: DocumentTargetKind) -> &'static str {
    match kind {
        DocumentTargetKind::Spec => "spec",
        DocumentTargetKind::Group => "group",
        DocumentTargetKind::Module => "module",
        DocumentTargetKind::Area => "area",
        DocumentTargetKind::Pattern => "pattern",
    }
}

fn document_ref_source_label(source: DocumentRefSource) -> &'static str {
    match source {
        DocumentRefSource::Link => "documents:// link",
        DocumentRefSource::DocumentsLine => "@documents",
        DocumentRefSource::FileDocumentsLine => "@filedocuments",
    }
}
