/**
@module SPECIAL.CLI.DIFF
Command boundary for explicit relationship fingerprint review.
*/
// @fileimplements SPECIAL.CLI.DIFF
use std::path::{Path, PathBuf};
use std::process::Command;
use std::process::ExitCode;

use anyhow::{Context, Result, bail};
use clap::Args;

use super::common::{report_cache_stats, resolve_cli_paths};
use super::status::{CommandStatus, StatusStep};
use crate::cache::{reset_cache_stats, with_cache_status_notifier};
use crate::config::{RootSource, VcsKind, resolve_project_root};
use crate::diff::{
    RelationshipDiffOptions, build_relationship_diff_document, render_relationship_diff_json,
    render_relationship_diff_text,
};

#[derive(Debug, Args)]
pub(super) struct DiffArgs {
    #[arg(value_name = "PATH", hide = true)]
    positional_paths: Vec<PathBuf>,

    #[arg(
        long = "target",
        value_name = "PATH",
        help = "Limit explicit relationships to one file or subtree"
    )]
    targets: Vec<PathBuf>,

    #[arg(
        long = "id",
        value_name = "ID",
        help = "Limit explicit relationships to one target id or id subtree"
    )]
    id: Option<String>,

    #[arg(
        long = "symbol",
        value_name = "SYMBOL",
        help = "Limit explicit relationships whose source body or target id mentions a symbol"
    )]
    symbol: Option<String>,

    #[arg(long = "json", help = "Render the view as JSON")]
    json: bool,

    #[arg(short = 'v', long = "verbose", help = "Show current endpoint content")]
    verbose: bool,

    #[arg(
        short = 'm',
        long = "metrics",
        help = "Show affected relationship counts by kind and path"
    )]
    metrics: bool,
}

const DIFF_PLAN: &[StatusStep] = &[
    StatusStep::new("resolving project root", 1),
    StatusStep::new("fingerprinting explicit relationships", 8),
    StatusStep::new("rendering output", 1),
];

// @applies COMMAND.PROJECTION_PIPELINE
pub(super) fn execute_diff(args: DiffArgs, current_dir: &Path) -> Result<ExitCode> {
    let status = CommandStatus::with_plan("special diff", DIFF_PLAN);
    reset_cache_stats();
    status.phase("resolving project root");
    if !args.positional_paths.is_empty() {
        bail!(
            "diff path scopes must use --target PATH; try `special diff --target {}`",
            args.positional_paths[0].display()
        );
    }
    let resolution = resolve_project_root(current_dir)?;
    if let Some(warning) = resolution.warning() {
        eprintln!("{warning}");
    }
    let full_relationship_view = resolution.vcs.is_none() || resolution.vcs == Some(VcsKind::None);
    if resolution.vcs.is_none() && resolution.source == RootSource::SpecialToml {
        status
            .note("no `vcs` declared in special.toml; showing the full explicit relationship view");
    } else if resolution.vcs == Some(VcsKind::None) {
        status.note("`vcs = \"none\"`; showing the full explicit relationship view");
    }

    status.phase("fingerprinting explicit relationships");
    let changed_paths = match (full_relationship_view, resolution.vcs) {
        (true, _) => Vec::new(),
        (false, Some(vcs)) => changed_paths_for_vcs(vcs, &resolution.root)?,
        (false, None) => unreachable!("full view handles missing vcs"),
    };
    let document = with_cache_status_notifier(status.notifier(), || {
        build_relationship_diff_document(
            &resolution.root,
            &resolution.ignore_patterns,
            resolution.version,
            RelationshipDiffOptions {
                target_paths: resolve_cli_paths(current_dir, &args.targets),
                changed_paths,
                full_view: full_relationship_view,
                id: args.id.clone(),
                symbol: args.symbol.clone(),
                include_content: args.verbose,
            },
        )
    })?;
    report_cache_stats(&status);

    status.phase("rendering output");
    let rendered = if args.json {
        render_relationship_diff_json(&document)?
    } else {
        render_relationship_diff_text(&document, args.verbose, args.metrics)
    };
    println!("{rendered}");
    status.finish();

    Ok(ExitCode::SUCCESS)
}

fn changed_paths_for_vcs(vcs: VcsKind, root: &Path) -> Result<Vec<PathBuf>> {
    match vcs {
        VcsKind::Jj => changed_paths_from_jj(root),
        VcsKind::Git => changed_paths_from_git(root),
        VcsKind::None => Ok(Vec::new()),
    }
}

fn changed_paths_from_jj(root: &Path) -> Result<Vec<PathBuf>> {
    let output = Command::new("jj")
        .arg("diff")
        .arg("--summary")
        .current_dir(root)
        .output()
        .context("failed to run `jj diff --summary`")?;
    if !output.status.success() {
        bail!(
            "`jj diff --summary` failed: {}",
            String::from_utf8_lossy(&output.stderr).trim()
        );
    }
    Ok(String::from_utf8_lossy(&output.stdout)
        .lines()
        .filter_map(parse_jj_summary_path)
        .map(|path| root.join(path))
        .collect())
}

fn parse_jj_summary_path(line: &str) -> Option<&str> {
    let trimmed = line.trim();
    if trimmed.len() < 2 {
        return None;
    }
    let path = trimmed.get(2..)?.trim();
    (!path.is_empty()).then_some(path)
}

fn changed_paths_from_git(root: &Path) -> Result<Vec<PathBuf>> {
    let output = Command::new("git")
        .arg("status")
        .arg("--porcelain=v1")
        .arg("-uall")
        .current_dir(root)
        .output()
        .context("failed to run `git status --porcelain=v1 -uall`")?;
    if !output.status.success() {
        bail!(
            "`git status --porcelain=v1 -uall` failed: {}",
            String::from_utf8_lossy(&output.stderr).trim()
        );
    }
    Ok(String::from_utf8_lossy(&output.stdout)
        .lines()
        .filter_map(parse_git_status_path)
        .map(|path| root.join(path))
        .collect())
}

fn parse_git_status_path(line: &str) -> Option<&str> {
    if line.len() < 4 {
        return None;
    }
    let path = line.get(3..)?.trim();
    let path = path
        .rsplit_once(" -> ")
        .map(|(_, path)| path)
        .unwrap_or(path);
    (!path.is_empty()).then_some(path.trim_matches('"'))
}
