/**
@module SPECIAL.CLI
Thin command-line boundary for the Rust application.

@group SPECIAL.HELP
special help surface and top-level command descriptions.

@group SPECIAL.VERSION
special top-level version output surface.

@spec SPECIAL.HELP.TOP_LEVEL_COMMANDS
special `--help` lists the top-level commands with purpose-oriented summaries.

@spec SPECIAL.HELP.SUBCOMMAND
special `help` prints the same top-level help surface as `special --help`.

@spec SPECIAL.HELP.SPECS_COMMAND_PLURAL_PRIMARY
special help text presents the spec command as `special specs`.

@spec SPECIAL.HELP.ARCH_COMMAND_PRIMARY
special help text presents the architecture command as `special arch`.

@spec SPECIAL.HELP.HEALTH_COMMAND
special help text presents the code-health command as `special health`.

@spec SPECIAL.HELP.ROOT_OVERVIEW
special help text explains that bare `special` prints a compact health overview.

@spec SPECIAL.HELP.SKILLS_COMMAND_SHAPES
special help text explains the `skills`, `skills SKILL_ID`, and `skills install [SKILL_ID]` command shapes.

@spec SPECIAL.HELP.TASK_ORIENTED_EXAMPLES
special help text groups examples by user task instead of listing every command shape flatly.

@spec SPECIAL.VERSION.FLAGS
special `-v` and `--version` print the current CLI version and exit successfully.

@spec SPECIAL.DIFF_COMMAND
special diff uses the VCS backend declared in special.toml to report explicit Special relationships whose source or target endpoint intersects the current changed path set.

@spec SPECIAL.DIFF_COMMAND.NO_VCS
special diff gracefully degrades to a full explicit relationship view when special.toml omits `vcs` or declares `vcs = "none"`.

@spec SPECIAL.DIFF_COMMAND.METRICS
special diff --metrics reports affected relationship counts by relationship kind, target kind, and source path.

@spec SPECIAL.DIFF_COMMAND.VERBOSE
special diff --verbose includes current endpoint content for relationship review.
*/
// @fileimplements SPECIAL.CLI
use std::env;
use std::ffi::OsStr;
use std::process::ExitCode;

use anyhow::Result;
use clap::{CommandFactory, Parser, Subcommand};

mod common;
mod diff;
mod docs;
mod init;
mod mcp;
mod modules;
mod overview;
mod patterns;
mod repo;
mod skills;
mod spec;
mod status;

use self::diff::{DiffArgs, execute_diff};
use self::docs::{DocsArgs, execute_docs};
use self::init::execute_init;
use self::mcp::{McpArgs, execute_mcp};
use self::modules::{ModulesArgs, execute_modules};
use self::overview::{OverviewArgs, execute_overview};
use self::patterns::{PatternsArgs, execute_patterns};
use self::repo::{HealthArgs, execute_health};
use self::skills::{SkillsArgs, execute_skills};
use self::spec::{SpecArgs, execute_lint, execute_spec};

#[derive(Debug, Parser)]
#[command(
    name = "special",
    bin_name = "special",
    about = "Connect repo claims, proof, ownership, patterns, docs, and health signals. Run with no subcommand for a compact health overview.",
    after_help = "Examples:\n  Start a fresh project:\n    special init\n    special specs APP.EXPORT --verbose\n    special arch APP.EXPORT --verbose\n    special docs build\n    special lint\n\n  Understand an existing project:\n    special init\n    special health --metrics\n    special patterns --metrics\n    special health --target src/export.ts --symbol exportCsv\n\n  Work one surface:\n    special specs --unverified\n    special arch --unimplemented\n    special patterns APP.ROW_NORMALIZER --verbose\n    special docs --metrics\n\n  Use with agents and skills:\n    special mcp\n    special skills\n    special skills install define-product-specs",
    args_conflicts_with_subcommands = true,
    disable_help_subcommand = true
)]
struct Cli {
    #[arg(long = "json")]
    json: bool,

    #[command(subcommand)]
    command: Option<Command>,
}

#[derive(Debug, Subcommand)]
enum Command {
    #[command(
        name = "specs",
        about = "Show product claims, lifecycle state, and proof attachments"
    )]
    Specs(SpecArgs),
    #[command(
        name = "arch",
        about = "Show module ownership, implementation boundaries, and gaps"
    )]
    Modules(ModulesArgs),
    #[command(
        name = "patterns",
        about = "Show adopted patterns and repeated-source candidates"
    )]
    Patterns(PatternsArgs),
    #[command(
        name = "health",
        about = "Show repo gaps across ownership, proof, docs, patterns, and traceability"
    )]
    Health(HealthArgs),
    #[command(
        name = "docs",
        about = "Check docs links, report docs metrics, or build generated docs"
    )]
    Docs(DocsArgs),
    #[command(
        name = "diff",
        about = "Fingerprint explicit relationship endpoints for review"
    )]
    Diff(DiffArgs),
    #[command(about = "Serve bounded Special tools for agents over stdio")]
    Mcp(McpArgs),
    #[command(about = "Fail on broken ids, misplaced annotations, and graph errors")]
    Lint,
    #[command(about = "Create starter special.toml repo configuration")]
    Init,
    #[command(
        about = "List, print, or install bundled agent workflow skills",
        long_about = "Use `special skills` to see available bundled skills and command shapes.\n\nCommand shapes:\n  special skills\n  special skills SKILL_ID\n  special skills install [SKILL_ID]\n  special skills install [SKILL_ID] --destination DESTINATION\n  special skills install [SKILL_ID] --destination DESTINATION --force"
    )]
    Skills(SkillsArgs),
}

pub fn run_from_env() -> ExitCode {
    let args: Vec<_> = env::args_os().collect();

    if let Some(code) = handle_top_level_shortcuts(&args) {
        return code;
    }

    let cli = match Cli::try_parse_from(&args) {
        Ok(cli) => cli,
        Err(err) => {
            let code = err.exit_code();
            let _ = err.print();
            return ExitCode::from(code.clamp(0, u8::MAX.into()) as u8);
        }
    };

    match execute(cli) {
        Ok(code) => code,
        Err(err) => {
            eprintln!("{err:#}");
            ExitCode::from(1)
        }
    }
}

fn execute(cli: Cli) -> Result<ExitCode> {
    let current_dir = env::current_dir()?;

    match cli.command {
        Some(Command::Init) => execute_init(&current_dir),
        Some(Command::Modules(args)) => execute_modules(args, &current_dir),
        Some(Command::Patterns(args)) => execute_patterns(args, &current_dir),
        Some(Command::Health(args)) => execute_health(args, &current_dir),
        Some(Command::Docs(args)) => execute_docs(args, &current_dir),
        Some(Command::Diff(args)) => execute_diff(args, &current_dir),
        Some(Command::Mcp(args)) => execute_mcp(args, &current_dir),
        Some(Command::Skills(args)) => execute_skills(args, &current_dir),
        Some(Command::Specs(args)) => execute_spec(args, &current_dir),
        Some(Command::Lint) => execute_lint(&current_dir),
        None => execute_overview(OverviewArgs { json: cli.json }, &current_dir),
    }
}

fn handle_top_level_shortcuts(args: &[std::ffi::OsString]) -> Option<ExitCode> {
    if args.len() != 2 {
        return None;
    }

    match args[1].as_os_str() {
        arg if arg == OsStr::new("help") => Some(print_top_level_help()),
        arg if arg == OsStr::new("-v") || arg == OsStr::new("--version") => {
            println!("special {}", env!("CARGO_PKG_VERSION"));
            Some(ExitCode::SUCCESS)
        }
        _ => None,
    }
}

fn print_top_level_help() -> ExitCode {
    let mut cmd = Cli::command();
    match cmd.print_help() {
        Ok(()) => {
            println!();
            ExitCode::SUCCESS
        }
        Err(err) => {
            eprintln!("{err}");
            ExitCode::from(1)
        }
    }
}
