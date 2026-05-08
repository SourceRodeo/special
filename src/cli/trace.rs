/**
@module SPECIAL.CLI.TRACE
Command-line boundary for deterministic Special relationship trace packets.
*/
// @fileimplements SPECIAL.CLI.TRACE
// @fileverifies SPECIAL.TRACE_COMMAND
use std::fs;
use std::path::{Path, PathBuf};
use std::process::ExitCode;

use anyhow::{Result, bail};
use clap::{Args, Subcommand};

use super::common::{report_cache_stats, resolve_cli_path, resolve_cli_paths};
use super::status::{CommandStatus, StatusStep};
use crate::config::resolve_project_root;
use crate::trace::{
    TraceOptions, TraceSurface, build_trace_document, render_trace_json, render_trace_text,
};

#[derive(Debug, Args)]
pub(super) struct TraceArgs {
    #[command(subcommand)]
    command: TraceCommand,
}

#[derive(Debug, Subcommand)]
enum TraceCommand {
    #[command(about = "Build spec-to-evidence trace packets")]
    Specs(TraceSurfaceArgs),
    #[command(about = "Build docs-to-target trace packets")]
    Docs(TraceSurfaceArgs),
    #[command(about = "Build architecture ownership trace packets")]
    Arch(TraceSurfaceArgs),
    #[command(about = "Build pattern definition and application trace packets")]
    Patterns(TraceSurfaceArgs),
}

#[derive(Debug, Args)]
struct TraceSurfaceArgs {
    #[arg(value_name = "PATH", hide = true)]
    positional_scope: Option<PathBuf>,
    #[arg(
        long = "id",
        value_name = "ID",
        help = "Limit trace packets to one id or id subtree"
    )]
    id: Option<String>,
    #[arg(
        long = "target",
        value_name = "PATH",
        help = "Limit trace packets to a source file or subtree"
    )]
    targets: Vec<PathBuf>,
    #[arg(long = "json", help = "Render trace packets as JSON")]
    json: bool,
    #[arg(
        long = "output",
        value_name = "PATH",
        help = "Write trace packets to a file instead of stdout"
    )]
    output: Option<PathBuf>,
}

const TRACE_PLAN: &[StatusStep] = &[
    StatusStep::new("resolving project root", 1),
    StatusStep::new("building trace packets", 8),
    StatusStep::new("rendering output", 1),
];

pub(super) fn execute_trace(args: TraceArgs, current_dir: &Path) -> Result<ExitCode> {
    let status = CommandStatus::with_plan("special trace", TRACE_PLAN);
    status.phase("resolving project root");
    let resolution = resolve_project_root(current_dir)?;
    if let Some(warning) = resolution.warning() {
        status.note(&warning);
    }

    let (surface, surface_args) = match args.command {
        TraceCommand::Specs(args) => (TraceSurface::Specs, args),
        TraceCommand::Docs(args) => (TraceSurface::Docs, args),
        TraceCommand::Arch(args) => (TraceSurface::Arch, args),
        TraceCommand::Patterns(args) => (TraceSurface::Patterns, args),
    };
    if let Some(path) = &surface_args.positional_scope {
        bail!(
            "trace path scopes must use --target PATH; try `special trace {} --target {}`",
            surface_name(surface),
            path.display()
        );
    }

    status.phase("building trace packets");
    let options = TraceOptions {
        id: surface_args.id.clone(),
        target_paths: resolve_cli_paths(current_dir, &surface_args.targets),
    };
    let document = build_trace_document(
        &resolution.root,
        &resolution.ignore_patterns,
        resolution.version,
        surface,
        options,
    )?;
    report_cache_stats(&status);

    status.phase("rendering output");
    let rendered = if surface_args.json {
        render_trace_json(&document)?
    } else {
        render_trace_text(&document)
    };
    if let Some(output) = &surface_args.output {
        let output = resolve_cli_path(current_dir, output);
        if let Some(parent) = output.parent() {
            fs::create_dir_all(parent)?;
        }
        fs::write(&output, rendered)?;
    } else {
        print!("{rendered}");
    }
    status.finish();
    Ok(ExitCode::SUCCESS)
}

fn surface_name(surface: TraceSurface) -> &'static str {
    match surface {
        TraceSurface::Specs => "specs",
        TraceSurface::Docs => "docs",
        TraceSurface::Arch => "arch",
        TraceSurface::Patterns => "patterns",
    }
}
