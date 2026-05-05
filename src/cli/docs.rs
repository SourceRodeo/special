/**
@module SPECIAL.CLI.DOCS
Documentation command behavior for validating docs relationships and writing public docs output.
*/
// @fileimplements SPECIAL.CLI.DOCS
use std::path::{Path, PathBuf};
use std::process::ExitCode;

use anyhow::{Result, bail};
use clap::{Args, Subcommand};

use super::common::{report_cache_stats, resolve_cli_paths};
use super::status::{CommandStatus, StatusStep};
use crate::cache::{reset_cache_stats, with_cache_status_notifier};
use crate::config::{DocsOutputConfig, resolve_project_root};
use crate::docs::{build_docs_document, render_docs_text, write_docs_path, write_docs_paths};
use crate::model::LintReport;
use crate::render::render_lint_text;

#[derive(Debug, Args)]
pub(super) struct DocsArgs {
    #[command(subcommand)]
    command: Option<DocsCommand>,

    #[arg(value_name = "PATH", hide = true)]
    positional_paths: Vec<PathBuf>,

    #[arg(
        long = "target",
        value_name = "PATH",
        help = "Limit the current docs view to one file or subtree"
    )]
    targets: Vec<PathBuf>,

    #[arg(
        long = "output",
        value_name = "PATH",
        num_args = 0..=1,
        help = "Write docs to an output path; omit PATH to use configured docs outputs"
    )]
    output: Option<Option<PathBuf>>,
}

#[derive(Debug, Subcommand)]
enum DocsCommand {
    #[command(about = "Write configured docs outputs or one explicit docs output")]
    Build(DocsBuildArgs),
}

#[derive(Debug, Args)]
struct DocsBuildArgs {
    #[arg(value_name = "SOURCE")]
    source: Option<PathBuf>,

    #[arg(value_name = "OUTPUT", requires = "source")]
    positional_output: Option<PathBuf>,

    #[arg(
        long = "target",
        value_name = "PATH",
        help = "Input file or subtree to write from"
    )]
    targets: Vec<PathBuf>,

    #[arg(
        long = "output",
        value_name = "PATH",
        help = "Output file or directory"
    )]
    output: Option<PathBuf>,
}

const DOCS_PLAN: &[StatusStep] = &[
    StatusStep::new("resolving project root", 1),
    StatusStep::new("validating documentation links", 8),
    StatusStep::new("rendering output", 1),
];

// @applies COMMAND.PROJECTION_PIPELINE
pub(super) fn execute_docs(args: DocsArgs, current_dir: &Path) -> Result<ExitCode> {
    let status = CommandStatus::with_plan("special docs", DOCS_PLAN);
    reset_cache_stats();
    status.phase("resolving project root");
    if let Some(command) = args.command {
        if !args.positional_paths.is_empty() || !args.targets.is_empty() || args.output.is_some() {
            bail!("special docs build cannot be combined with parent docs options");
        }
        return match command {
            DocsCommand::Build(build_args) => execute_docs_build(build_args, current_dir, status),
        };
    }
    if !args.positional_paths.is_empty() {
        bail!(
            "docs path scopes must use --target PATH; try `special docs --target {}`",
            args.positional_paths[0].display()
        );
    }
    let output_requested = args.output.is_some();
    if args.targets.len() > 1 && output_requested {
        bail!("--output requires exactly one --target path");
    }
    let resolution = resolve_project_root(current_dir)?;
    if let Some(warning) = resolution.warning() {
        eprintln!("{warning}");
    }

    status.phase("validating documentation links");
    let target_paths = resolve_cli_paths(current_dir, &args.targets);
    let output_path = args
        .output
        .as_ref()
        .and_then(|output| output.as_ref())
        .map(|output| resolve_cli_path(current_dir, output));
    let (report, rendered_document) = with_cache_status_notifier(status.notifier(), || {
        match (
            target_paths.as_slice(),
            args.output.as_ref(),
            output_path.as_deref(),
        ) {
            (targets, None, None) => {
                let (document, report) = build_docs_document(
                    &resolution.root,
                    &resolution.ignore_patterns,
                    resolution.version,
                    targets,
                )?;
                let rendered =
                    (!report.has_errors()).then(|| render_docs_text(&resolution.root, &document));
                Ok((report, rendered))
            }
            ([input], Some(Some(_)), Some(output)) => {
                let report = write_docs_path(
                    &resolution.root,
                    &resolution.ignore_patterns,
                    resolution.version,
                    input,
                    output,
                )?;
                Ok((report, None))
            }
            ([], Some(None), None) => {
                let report = render_configured_outputs(
                    &resolution.root,
                    &resolution.ignore_patterns,
                    resolution.version,
                    &resolution.docs_outputs,
                )?;
                Ok((report, None))
            }
            ([], Some(Some(_)), Some(_)) => {
                bail!("special docs --output PATH requires --target PATH")
            }
            ([_], Some(None), None) => bail!("special docs --target PATH --output requires PATH"),
            ([_, _, ..], Some(_), _) => bail!("--output requires exactly one --target path"),
            (_, Some(Some(_)), None) => unreachable!("explicit output path should resolve"),
            (_, _, Some(_)) => unreachable!("output path exists only when output is requested"),
        }
    })?;
    report_cache_stats(&status);

    status.phase("rendering output");
    println!(
        "{}",
        render_docs_report(&report, rendered_document.as_deref())
    );
    status.finish();

    Ok(if report.has_errors() {
        ExitCode::from(1)
    } else {
        ExitCode::SUCCESS
    })
}

fn execute_docs_build(
    args: DocsBuildArgs,
    current_dir: &Path,
    status: CommandStatus,
) -> Result<ExitCode> {
    if args.source.is_some() && (!args.targets.is_empty() || args.output.is_some()) {
        bail!("special docs build accepts either positional SOURCE OUTPUT or --target/--output");
    }
    let resolution = resolve_project_root(current_dir)?;
    if let Some(warning) = resolution.warning() {
        eprintln!("{warning}");
    }

    status.phase("validating documentation links");
    let target_paths = resolve_cli_paths(current_dir, &args.targets);
    let report = with_cache_status_notifier(status.notifier(), || {
        match (
            args.source.as_ref(),
            args.positional_output.as_ref(),
            target_paths.as_slice(),
            args.output.as_ref(),
        ) {
            (None, None, [], None) => render_configured_outputs(
                &resolution.root,
                &resolution.ignore_patterns,
                resolution.version,
                &resolution.docs_outputs,
            ),
            (Some(input), Some(output), [], None) => write_docs_path(
                &resolution.root,
                &resolution.ignore_patterns,
                resolution.version,
                &resolve_cli_path(current_dir, input),
                &resolve_cli_path(current_dir, output),
            ),
            (None, None, [input], Some(output)) => write_docs_path(
                &resolution.root,
                &resolution.ignore_patterns,
                resolution.version,
                input,
                &resolve_cli_path(current_dir, output),
            ),
            (Some(_), None, [], None) => {
                bail!("special docs build requires SOURCE and OUTPUT paths")
            }
            (None, None, [], Some(_)) => {
                bail!("special docs build --output PATH requires --target PATH")
            }
            (None, None, [_], None) => {
                bail!("special docs build --target PATH requires --output PATH")
            }
            (None, None, [_, _, ..], _) => {
                bail!("special docs build accepts exactly one --target path")
            }
            _ => bail!(
                "special docs build accepts either positional SOURCE OUTPUT or --target/--output"
            ),
        }
    })?;
    report_cache_stats(&status);

    status.phase("rendering output");
    println!("{}", render_docs_report(&report, None));
    status.finish();

    Ok(if report.has_errors() {
        ExitCode::from(1)
    } else {
        ExitCode::SUCCESS
    })
}

fn render_configured_outputs(
    root: &Path,
    ignore_patterns: &[String],
    version: crate::config::SpecialVersion,
    outputs: &[DocsOutputConfig],
) -> Result<LintReport> {
    if outputs.is_empty() {
        bail!("special docs build requires at least one [[docs.outputs]] entry in special.toml");
    }

    let mappings = outputs
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

fn render_docs_report(report: &LintReport, rendered_document: Option<&str>) -> String {
    if report.has_errors() {
        render_lint_text(report)
    } else if let Some(rendered_document) = rendered_document {
        rendered_document.to_string()
    } else if report.diagnostics.is_empty() {
        "Docs clean.".to_string()
    } else {
        render_lint_text(report)
    }
}
