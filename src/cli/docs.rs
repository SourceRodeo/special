/**
@module SPECIAL.CLI.DOCS
Documentation command behavior for validating docs relationships and materializing public docs output.
*/
// @fileimplements SPECIAL.CLI.DOCS
use std::path::{Path, PathBuf};
use std::process::ExitCode;

use anyhow::{Result, bail};
use clap::Args;

use super::common::{report_cache_stats, resolve_cli_paths};
use super::status::{CommandStatus, StatusStep};
use crate::cache::{reset_cache_stats, with_cache_status_notifier};
use crate::config::{DocsOutputConfig, resolve_project_root};
use crate::docs::{build_docs_document, materialize_path, render_docs_text};
use crate::model::LintReport;
use crate::render::render_lint_text;

#[derive(Debug, Args)]
pub(super) struct DocsArgs {
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
        help = "Materialize docs to an output path; omit PATH to use configured docs outputs"
    )]
    output: Option<Option<PathBuf>>,
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
                let report = materialize_path(
                    &resolution.root,
                    &resolution.ignore_patterns,
                    resolution.version,
                    input,
                    output,
                )?;
                Ok((report, None))
            }
            ([], Some(None), None) => {
                let report = materialize_configured_outputs(
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

fn materialize_configured_outputs(
    root: &Path,
    ignore_patterns: &[String],
    version: crate::config::SpecialVersion,
    outputs: &[DocsOutputConfig],
) -> Result<LintReport> {
    if outputs.is_empty() {
        bail!("special docs --output requires at least one [[docs.outputs]] entry in special.toml");
    }

    let mut diagnostics = Vec::new();
    for output in outputs {
        let report = materialize_path(
            root,
            ignore_patterns,
            version,
            &configured_docs_path(root, &output.source),
            &configured_docs_path(root, &output.output),
        )?;
        let has_errors = report.has_errors();
        diagnostics.extend(report.diagnostics);
        if has_errors {
            break;
        }
    }
    Ok(LintReport { diagnostics })
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
