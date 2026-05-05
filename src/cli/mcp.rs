/**
@module SPECIAL.CLI.MCP
MCP server boundary for agent-facing access to Special's existing command projections.

@spec SPECIAL.MCP_COMMAND
special mcp runs a stdio MCP server that speaks newline-delimited JSON-RPC and keeps stdout reserved for protocol messages.

@spec SPECIAL.MCP_COMMAND.TOOLS
special mcp exposes controlled tools for root discovery plus existing Special inspection and validation surfaces.

@spec SPECIAL.MCP_COMMAND.DOCS_OUTPUT
special mcp exposes docs output as a bounded write tool that preserves the same target/output safety policy as the CLI.

@spec SPECIAL.MCP_COMMAND.PLUGIN_VERSION_NOTICE
when the Codex plugin starts special mcp with an expected Special binary version that differs from the installed binary, initialize returns a nonfatal instruction telling the agent how to update Special.
*/
// @fileimplements SPECIAL.CLI.MCP
use std::io::{self, BufRead, Write};
use std::path::{Path, PathBuf};
use std::process::ExitCode;

use anyhow::{Result, anyhow, bail};
use clap::Args;
use serde::Serialize;
use serde_json::{Value, json};

use super::common::resolve_cli_paths;
use super::spec::{add_config_lint_warnings, normalize_report};
use crate::config::{RootResolution, RootSource, SpecialVersion, resolve_project_root};
use crate::docs::{build_docs_document, render_docs_text, write_docs_path, write_docs_paths};
use crate::index::{build_lint_report, build_spec_document};
use crate::model::{
    DeclaredStateFilter, LintReport, ModuleAnalysisOptions, ModuleFilter, PatternFilter, SpecFilter,
};
use crate::modules::{
    RepoDocumentOptions, build_module_document, build_module_lint_report, build_repo_document,
};
use crate::overview::build_overview_document;
use crate::patterns::build_pattern_document;
use crate::render::{
    render_lint_text, render_module_html, render_module_json, render_module_text,
    render_overview_json, render_overview_text, render_pattern_json, render_pattern_text,
    render_repo_html, render_repo_json, render_repo_text, render_spec_html, render_spec_json,
    render_spec_text,
};

const PROTOCOL_VERSION: &str = "2025-06-18";

#[derive(Debug, Args)]
pub(super) struct McpArgs {
    #[arg(long = "special-version", hide = true)]
    pub(super) plugin_special_version: Option<String>,
}

struct McpStartup<'a> {
    plugin_special_version: Option<&'a str>,
}

pub(super) fn execute_mcp(args: McpArgs, current_dir: &Path) -> Result<ExitCode> {
    let stdin = io::stdin();
    let stdout = io::stdout();
    run_stdio_server(
        current_dir,
        stdin.lock(),
        stdout.lock(),
        &McpStartup {
            plugin_special_version: args.plugin_special_version.as_deref(),
        },
    )?;
    Ok(ExitCode::SUCCESS)
}

fn run_stdio_server<R, W>(
    current_dir: &Path,
    reader: R,
    mut writer: W,
    startup: &McpStartup<'_>,
) -> Result<()>
where
    R: BufRead,
    W: Write,
{
    for line in reader.lines() {
        let line = line?;
        let trimmed = line.trim();
        if trimmed.is_empty() {
            continue;
        }
        let response = match serde_json::from_str::<Value>(trimmed) {
            Ok(message) => handle_message(current_dir, message, startup),
            Err(error) => Some(error_response(
                Value::Null,
                -32700,
                &format!("parse error: {error}"),
            )),
        };
        if let Some(response) = response {
            serde_json::to_writer(&mut writer, &response)?;
            writer.write_all(b"\n")?;
            writer.flush()?;
        }
    }
    Ok(())
}

fn handle_message(current_dir: &Path, message: Value, startup: &McpStartup<'_>) -> Option<Value> {
    let id = message.get("id").cloned();
    let method = message.get("method").and_then(Value::as_str);

    let id = id?;
    let Some(method) = method else {
        return Some(error_response(id, -32600, "request is missing method"));
    };

    match method {
        "initialize" => Some(success_response(id, initialize_result(startup))),
        "ping" => Some(success_response(id, json!({}))),
        "tools/list" => Some(success_response(id, json!({ "tools": tool_definitions() }))),
        "tools/call" => Some(success_response(
            id,
            handle_tool_call(current_dir, message.get("params")).unwrap_or_else(tool_error_result),
        )),
        _ => Some(error_response(id, -32601, "method not found")),
    }
}

fn initialize_result(startup: &McpStartup<'_>) -> Value {
    json!({
        "protocolVersion": PROTOCOL_VERSION,
        "capabilities": {
            "tools": {}
        },
        "serverInfo": {
            "name": "special",
            "version": env!("CARGO_PKG_VERSION")
        },
        "instructions": startup_instructions(startup)
    })
}

fn startup_instructions(startup: &McpStartup<'_>) -> String {
    let Some(plugin_version) = startup.plugin_special_version else {
        return String::new();
    };
    if plugin_version == env!("CARGO_PKG_VERSION") {
        return String::new();
    }
    format!(
        "The Special Codex plugin was built for special {plugin_version}, but the installed special binary is {}. Continue working if possible. For Homebrew installs, run `brew upgrade special`; for GitHub Release installs, download the matching release from https://github.com/SourceRodeo/special/releases.",
        env!("CARGO_PKG_VERSION")
    )
}

fn handle_tool_call(current_dir: &Path, params: Option<&Value>) -> Result<Value> {
    let params = params.ok_or_else(|| anyhow!("tools/call requires params"))?;
    let name = params
        .get("name")
        .and_then(Value::as_str)
        .ok_or_else(|| anyhow!("tools/call requires string field `name`"))?;
    let arguments = params.get("arguments").unwrap_or(&Value::Null);
    let result = match name {
        "special_status" => status_tool(current_dir),
        "special_overview" => overview_tool(current_dir, arguments),
        "special_specs" => specs_tool(current_dir, arguments),
        "special_arch" => arch_tool(current_dir, arguments),
        "special_patterns" => patterns_tool(current_dir, arguments),
        "special_docs" => docs_tool(current_dir, arguments),
        "special_docs_output" => docs_output_tool(current_dir, arguments),
        "special_lint" => lint_tool(current_dir),
        "special_health" => health_tool(current_dir, arguments),
        _ => bail!("unknown Special MCP tool `{name}`"),
    }?;
    Ok(tool_text_result(result.text, result.is_error))
}

fn tool_definitions() -> Vec<Value> {
    vec![
        tool(
            "special_status",
            "Resolve the active Special project root and configuration source.",
            object_schema(vec![]),
        ),
        tool(
            "special_overview",
            "Render the compact Special repo overview.",
            output_schema(),
        ),
        tool(
            "special_specs",
            "Render Special spec claims, with optional id scope and filters.",
            object_schema(vec![
                string_property("id", "Optional spec or group id to scope the view."),
                bool_property("current", "Show only current specs."),
                bool_property("planned", "Show only planned specs."),
                bool_property(
                    "unverified",
                    "Show current specs with no verifies or attests.",
                ),
                bool_property("metrics", "Include spec metrics."),
                bool_property("verbose", "Show attached evidence detail."),
                output_format_property(),
            ]),
        ),
        tool(
            "special_arch",
            "Render Special architecture declarations, ownership, and optional metrics.",
            object_schema(vec![
                string_property("id", "Optional module or area id to scope the view."),
                bool_property("current", "Show only current architecture nodes."),
                bool_property("planned", "Show only planned architecture nodes."),
                bool_property(
                    "unimplemented",
                    "Show current modules with no direct implementations.",
                ),
                bool_property("metrics", "Include architecture metrics."),
                bool_property("verbose", "Show implementation detail."),
                output_format_property(),
            ]),
        ),
        tool(
            "special_patterns",
            "Render adopted patterns and optional pattern metrics.",
            object_schema(vec![
                string_property("id", "Optional pattern id to scope the view."),
                bool_property("metrics", "Include pattern metrics."),
                string_array_property("target", "Optional file or subtree metric target paths."),
                string_array_property("within", "Optional file or subtree comparison paths."),
                string_property("symbol", "Optional source item name for metric scoping."),
                bool_property(
                    "verbose",
                    "Show pattern definitions and application bodies.",
                ),
                text_or_json_property(),
            ]),
        ),
        tool(
            "special_docs",
            "Validate documentation links and render the docs relationship dump.",
            object_schema(vec![string_array_property(
                "target",
                "Optional docs file or subtree paths to validate.",
            )]),
        ),
        tool(
            "special_docs_output",
            "Write configured docs outputs, or one explicit target/output pair, using CLI safety checks.",
            object_schema(vec![
                string_property(
                    "target",
                    "Docs source file or directory for explicit output writing.",
                ),
                string_property(
                    "output",
                    "Output file or directory for explicit output writing.",
                ),
            ]),
        ),
        tool(
            "special_lint",
            "Run Special structural lint across specs, architecture, patterns, and docs.",
            object_schema(vec![]),
        ),
        tool(
            "special_health",
            "Render code-health and traceability signals.",
            object_schema(vec![
                string_array_property(
                    "target",
                    "Optional file or subtree paths for the health view.",
                ),
                string_array_property(
                    "within",
                    "Optional file or subtree paths for the analysis corpus.",
                ),
                string_property(
                    "symbol",
                    "Optional source item name; requires exactly one target.",
                ),
                bool_property("metrics", "Include deeper health metrics."),
                bool_property("verbose", "Show item-level detail."),
                output_format_property(),
            ]),
        ),
    ]
}

fn tool(name: &str, description: &str, input_schema: Value) -> Value {
    json!({
        "name": name,
        "description": description,
        "inputSchema": input_schema
    })
}

fn object_schema(properties: Vec<(&'static str, Value)>) -> Value {
    let mut map = serde_json::Map::new();
    for (name, schema) in properties {
        map.insert(name.to_string(), schema);
    }
    json!({
        "type": "object",
        "properties": map,
        "additionalProperties": false
    })
}

fn bool_property(description: &'static str, detail: &'static str) -> (&'static str, Value) {
    (
        description,
        json!({ "type": "boolean", "description": detail }),
    )
}

fn string_property(name: &'static str, description: &'static str) -> (&'static str, Value) {
    (
        name,
        json!({ "type": "string", "description": description }),
    )
}

fn string_array_property(name: &'static str, description: &'static str) -> (&'static str, Value) {
    (
        name,
        json!({
            "type": "array",
            "items": { "type": "string" },
            "description": description
        }),
    )
}

fn output_format_property() -> (&'static str, Value) {
    (
        "format",
        json!({
            "type": "string",
            "enum": ["text", "json", "html"],
            "description": "Output format. Defaults to text."
        }),
    )
}

fn text_or_json_property() -> (&'static str, Value) {
    (
        "format",
        json!({
            "type": "string",
            "enum": ["text", "json"],
            "description": "Output format. Defaults to text."
        }),
    )
}

fn output_schema() -> Value {
    object_schema(vec![text_or_json_property()])
}

#[derive(Debug)]
struct ToolOutput {
    text: String,
    is_error: bool,
}

fn status_tool(current_dir: &Path) -> Result<ToolOutput> {
    let resolution = resolve_project_root(current_dir)?;
    let status = StatusOutput::from_resolution(&resolution);
    Ok(ToolOutput {
        text: serde_json::to_string_pretty(&status)?,
        is_error: false,
    })
}

#[derive(Serialize)]
struct StatusOutput {
    root: String,
    source: &'static str,
    version: String,
    version_explicit: bool,
    config_path: Option<String>,
    warning: Option<String>,
}

impl StatusOutput {
    fn from_resolution(resolution: &RootResolution) -> Self {
        Self {
            root: resolution.root.display().to_string(),
            source: match resolution.source {
                RootSource::SpecialToml => "special.toml",
                RootSource::Vcs => "vcs",
                RootSource::CurrentDir => "current-dir",
            },
            version: resolution.version.as_str().to_string(),
            version_explicit: resolution.version_explicit,
            config_path: resolution
                .config_path
                .as_ref()
                .map(|path| path.display().to_string()),
            warning: resolution.warning(),
        }
    }
}

fn overview_tool(current_dir: &Path, arguments: &Value) -> Result<ToolOutput> {
    let resolution = resolve_project_root(current_dir)?;
    let document = build_overview_document(
        &resolution.root,
        &resolution.ignore_patterns,
        resolution.version,
    )?;
    let rendered = if format(arguments)? == OutputFormat::Json {
        render_overview_json(&document)?
    } else {
        render_overview_text(&document)
    };
    Ok(ToolOutput {
        text: rendered,
        is_error: document.lint.errors > 0,
    })
}

fn specs_tool(current_dir: &Path, arguments: &Value) -> Result<ToolOutput> {
    let resolution = resolve_project_root(current_dir)?;
    let state = declared_state_filter(arguments, "spec")?;
    let document_filter = SpecFilter {
        state,
        unverified_only: bool_arg(arguments, "unverified")?,
        scope: optional_string_arg(arguments, "id")?,
    };
    let (document, mut lint) = build_spec_document(
        &resolution.root,
        &resolution.ignore_patterns,
        resolution.version,
        document_filter,
        bool_arg(arguments, "metrics")?,
    )?;
    add_config_lint_warnings(&mut lint, &resolution);
    let rendered = match format(arguments)? {
        OutputFormat::Text => render_spec_text(&document, bool_arg(arguments, "verbose")?),
        OutputFormat::Json => render_spec_json(&document, bool_arg(arguments, "verbose")?)?,
        OutputFormat::Html => render_spec_html(&document, bool_arg(arguments, "verbose")?),
    };
    Ok(render_with_lint(rendered, &lint))
}

fn arch_tool(current_dir: &Path, arguments: &Value) -> Result<ToolOutput> {
    let resolution = resolve_project_root(current_dir)?;
    let state = declared_state_filter(arguments, "arch")?;
    let (document, lint) = build_module_document(
        &resolution.root,
        &resolution.ignore_patterns,
        resolution.version,
        ModuleFilter {
            state,
            unimplemented_only: bool_arg(arguments, "unimplemented")?,
            scope: optional_string_arg(arguments, "id")?,
        },
        ModuleAnalysisOptions {
            coverage: bool_arg(arguments, "metrics")?,
            metrics: bool_arg(arguments, "metrics")?,
            traceability: false,
        },
    )?;
    let rendered = match format(arguments)? {
        OutputFormat::Text => render_module_text(&document, bool_arg(arguments, "verbose")?),
        OutputFormat::Json => render_module_json(&document, bool_arg(arguments, "verbose")?)?,
        OutputFormat::Html => render_module_html(&document, bool_arg(arguments, "verbose")?),
    };
    Ok(render_with_lint(rendered, &lint))
}

fn patterns_tool(current_dir: &Path, arguments: &Value) -> Result<ToolOutput> {
    let resolution = resolve_project_root(current_dir)?;
    let (document, lint) = build_pattern_document(
        &resolution.root,
        &resolution.ignore_patterns,
        PatternFilter {
            scope: optional_string_arg(arguments, "id")?,
            metrics: bool_arg(arguments, "metrics")?,
            target_paths: resolve_cli_paths(current_dir, &path_args(arguments, "target")?),
            comparison_paths: resolve_cli_paths(current_dir, &path_args(arguments, "within")?),
            symbol: optional_string_arg(arguments, "symbol")?,
        },
        resolution.pattern_benchmarks,
    )?;
    let rendered = match format(arguments)? {
        OutputFormat::Text => render_pattern_text(&document, bool_arg(arguments, "verbose")?),
        OutputFormat::Json => render_pattern_json(&document, bool_arg(arguments, "verbose")?)?,
        OutputFormat::Html => bail!("special_patterns supports only text or json format"),
    };
    Ok(render_with_lint(rendered, &lint))
}

fn docs_tool(current_dir: &Path, arguments: &Value) -> Result<ToolOutput> {
    let resolution = resolve_project_root(current_dir)?;
    let target_paths = resolve_cli_paths(current_dir, &path_args(arguments, "target")?);
    let (document, report) = build_docs_document(
        &resolution.root,
        &resolution.ignore_patterns,
        resolution.version,
        &target_paths,
    )?;
    let rendered = if report.has_errors() {
        render_lint_text(&report)
    } else {
        render_docs_text(&resolution.root, &document)
    };
    Ok(ToolOutput {
        text: rendered,
        is_error: report.has_errors(),
    })
}

fn docs_output_tool(current_dir: &Path, arguments: &Value) -> Result<ToolOutput> {
    let resolution = resolve_project_root(current_dir)?;
    let target = optional_path_arg(arguments, "target")?;
    let output = optional_path_arg(arguments, "output")?;
    let report = match (target, output) {
        (None, None) => render_configured_outputs(
            &resolution.root,
            &resolution.ignore_patterns,
            resolution.version,
            &resolution,
        )?,
        (Some(target), Some(output)) => write_docs_path(
            &resolution.root,
            &resolution.ignore_patterns,
            resolution.version,
            &resolve_cli_path(current_dir, &target),
            &resolve_cli_path(current_dir, &output),
        )?,
        (Some(_), None) => bail!("special_docs_output target requires output"),
        (None, Some(_)) => bail!("special_docs_output output requires target"),
    };
    let rendered = if report.has_errors() {
        render_lint_text(&report)
    } else if report.diagnostics.is_empty() {
        "Docs clean.".to_string()
    } else {
        render_lint_text(&report)
    };
    Ok(ToolOutput {
        text: rendered,
        is_error: report.has_errors(),
    })
}

fn render_configured_outputs(
    root: &Path,
    ignore_patterns: &[String],
    version: SpecialVersion,
    resolution: &RootResolution,
) -> Result<LintReport> {
    if resolution.docs_outputs.is_empty() {
        bail!("special docs build requires at least one [[docs.outputs]] entry in special.toml");
    }
    let mappings = resolution
        .docs_outputs
        .iter()
        .map(|output| {
            (
                configured_docs_path(root, &output.source),
                configured_docs_path(root, &output.output),
            )
        })
        .collect::<Vec<_>>();
    write_docs_paths(root, ignore_patterns, version, &mappings)
}

fn lint_tool(current_dir: &Path) -> Result<ToolOutput> {
    let resolution = resolve_project_root(current_dir)?;
    let mut report = build_lint_report(
        &resolution.root,
        &resolution.ignore_patterns,
        resolution.version,
    )?;
    report.diagnostics.extend(
        build_module_lint_report(&resolution.root, &resolution.ignore_patterns)?.diagnostics,
    );
    report.diagnostics.extend(
        crate::docs::build_docs_lint_report(
            &resolution.root,
            &resolution.ignore_patterns,
            resolution.version,
        )?
        .diagnostics,
    );
    add_config_lint_warnings(&mut report, &resolution);
    normalize_report(&mut report);
    Ok(ToolOutput {
        text: render_lint_text(&report),
        is_error: report.has_errors(),
    })
}

fn health_tool(current_dir: &Path, arguments: &Value) -> Result<ToolOutput> {
    let resolution = resolve_project_root(current_dir)?;
    let target_paths = resolve_cli_paths(current_dir, &path_args(arguments, "target")?);
    let within_paths = resolve_cli_paths(current_dir, &path_args(arguments, "within")?);
    if optional_string_arg(arguments, "symbol")?.is_some() && target_paths.len() != 1 {
        bail!("symbol requires exactly one target path");
    }
    let (document, lint) = build_repo_document(
        &resolution.root,
        &resolution.ignore_patterns,
        resolution.version,
        RepoDocumentOptions {
            metrics: bool_arg(arguments, "metrics")?,
            health_ignore_unexplained_patterns: &resolution.health_ignore_unexplained_patterns,
            target_scope_paths: (!target_paths.is_empty()).then_some(target_paths.as_slice()),
            within_scope_paths: (!within_paths.is_empty()).then_some(within_paths.as_slice()),
            symbol: optional_string_arg(arguments, "symbol")?.as_deref(),
        },
    )?;
    let rendered = match format(arguments)? {
        OutputFormat::Text => render_repo_text(&document, bool_arg(arguments, "verbose")?),
        OutputFormat::Json => render_repo_json(&document, bool_arg(arguments, "verbose")?)?,
        OutputFormat::Html => render_repo_html(&document, bool_arg(arguments, "verbose")?),
    };
    Ok(render_with_lint(rendered, &lint))
}

fn render_with_lint(rendered: String, lint: &LintReport) -> ToolOutput {
    if lint.diagnostics.is_empty() {
        ToolOutput {
            text: rendered,
            is_error: false,
        }
    } else {
        ToolOutput {
            text: format!("{}\n\n{}", render_lint_text(lint), rendered),
            is_error: lint.has_errors(),
        }
    }
}

fn declared_state_filter(arguments: &Value, tool: &str) -> Result<DeclaredStateFilter> {
    let current = bool_arg(arguments, "current")?;
    let planned = bool_arg(arguments, "planned")?;
    if current && planned {
        bail!("special_{tool} current and planned filters conflict");
    }
    if planned {
        Ok(DeclaredStateFilter::Planned)
    } else if current || bool_arg(arguments, "unverified")? || bool_arg(arguments, "unimplemented")?
    {
        Ok(DeclaredStateFilter::Current)
    } else {
        Ok(DeclaredStateFilter::All)
    }
}

#[derive(Clone, Copy, PartialEq, Eq)]
enum OutputFormat {
    Text,
    Json,
    Html,
}

fn format(arguments: &Value) -> Result<OutputFormat> {
    match optional_string_arg(arguments, "format")?.as_deref() {
        None | Some("text") => Ok(OutputFormat::Text),
        Some("json") => Ok(OutputFormat::Json),
        Some("html") => Ok(OutputFormat::Html),
        Some(format) => bail!("unsupported format `{format}`"),
    }
}

fn bool_arg(arguments: &Value, name: &str) -> Result<bool> {
    match arguments.get(name) {
        None => Ok(false),
        Some(Value::Bool(value)) => Ok(*value),
        Some(_) => bail!("argument `{name}` must be boolean"),
    }
}

fn optional_string_arg(arguments: &Value, name: &str) -> Result<Option<String>> {
    match arguments.get(name) {
        None | Some(Value::Null) => Ok(None),
        Some(Value::String(value)) => Ok(Some(value.clone())),
        Some(_) => bail!("argument `{name}` must be string"),
    }
}

fn optional_path_arg(arguments: &Value, name: &str) -> Result<Option<PathBuf>> {
    Ok(optional_string_arg(arguments, name)?.map(PathBuf::from))
}

fn path_args(arguments: &Value, name: &str) -> Result<Vec<PathBuf>> {
    match arguments.get(name) {
        None | Some(Value::Null) => Ok(Vec::new()),
        Some(Value::String(value)) => Ok(vec![PathBuf::from(value)]),
        Some(Value::Array(values)) => values
            .iter()
            .map(|value| {
                value
                    .as_str()
                    .map(PathBuf::from)
                    .ok_or_else(|| anyhow!("argument `{name}` must contain only strings"))
            })
            .collect(),
        Some(_) => bail!("argument `{name}` must be string or array of strings"),
    }
}

fn configured_docs_path(root: &Path, path: &Path) -> PathBuf {
    if path.is_absolute() {
        path.to_path_buf()
    } else {
        root.join(path)
    }
}

fn resolve_cli_path(current_dir: &Path, path: &Path) -> PathBuf {
    if path.is_absolute() {
        path.to_path_buf()
    } else {
        current_dir.join(path)
    }
}

fn tool_text_result(text: String, is_error: bool) -> Value {
    json!({
        "content": [
            {
                "type": "text",
                "text": text
            }
        ],
        "isError": is_error
    })
}

fn tool_error_result(error: anyhow::Error) -> Value {
    tool_text_result(format!("{error:#}"), true)
}

fn success_response(id: Value, result: Value) -> Value {
    json!({
        "jsonrpc": "2.0",
        "id": id,
        "result": result
    })
}

fn error_response(id: Value, code: i64, message: &str) -> Value {
    json!({
        "jsonrpc": "2.0",
        "id": id,
        "error": {
            "code": code,
            "message": message
        }
    })
}
