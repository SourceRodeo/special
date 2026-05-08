/**
@module SPECIAL.TESTS.CLI_OVERVIEW
Bare `special` root command tests in `tests/cli_overview.rs`.
*/
// @fileimplements SPECIAL.TESTS.CLI_OVERVIEW
#[path = "support/cli.rs"]
mod support;

use std::fs;

use support::{run_special, temp_repo_dir};

#[test]
// @verifies SPECIAL.HELP.ROOT_HELP
fn bare_special_prints_top_level_help() {
    let root = temp_repo_dir("special-cli-root-help");

    let output = run_special(&root, &[]);
    assert!(output.status.success());

    let stdout = String::from_utf8(output.stdout).expect("stdout should be utf-8");
    assert!(stdout.contains("Usage: special"));
    assert!(stdout.contains("Commands:"));
    assert!(stdout.contains("specs"));
    assert!(stdout.contains("health"));
    assert!(stdout.contains("Examples:"));

    fs::remove_dir_all(&root).expect("temp repo should be cleaned up");
}

#[test]
// @verifies SPECIAL.HELP.SUBCOMMAND
fn bare_special_matches_help_subcommand() {
    let root = temp_repo_dir("special-cli-root-help-matches-help");

    let root_output = run_special(&root, &[]);
    let help_output = run_special(&root, &["help"]);
    assert!(root_output.status.success());
    assert!(help_output.status.success());

    assert_eq!(root_output.stdout, help_output.stdout);

    fs::remove_dir_all(&root).expect("temp repo should be cleaned up");
}

#[test]
fn bare_special_json_requires_a_command() {
    let root = temp_repo_dir("special-cli-root-json");

    let output = run_special(&root, &["--json"]);
    assert!(!output.status.success());

    let stderr = String::from_utf8(output.stderr).expect("stderr should be utf-8");
    assert!(stderr.contains("top-level --json has no root document"));
    assert!(stderr.contains("special --help"));

    fs::remove_dir_all(&root).expect("temp repo should be cleaned up");
}

#[test]
fn subcommand_help_explains_parallel_metrics_and_verbose_flags() {
    let root = temp_repo_dir("special-cli-help-parallel-flags");

    let specs_help = run_special(&root, &["specs", "--help"]);
    assert!(specs_help.status.success());
    let specs_stdout = String::from_utf8(specs_help.stdout).expect("stdout should be utf-8");
    assert!(specs_stdout.contains("Show deeper contract analysis for the declared spec view"));
    assert!(specs_stdout.contains("Show more attached support detail within the current view"));

    let arch_help = run_special(&root, &["arch", "--help"]);
    assert!(arch_help.status.success());
    let arch_stdout = String::from_utf8(arch_help.stdout).expect("stdout should be utf-8");
    assert!(
        arch_stdout
            .contains("Show deeper implementation analysis for the current architecture view")
    );
    assert!(arch_stdout.contains("Show more implementation detail within the current view"));

    let health_help = run_special(&root, &["health", "--help"]);
    assert!(health_help.status.success());
    let health_stdout = String::from_utf8(health_help.stdout).expect("stdout should be utf-8");
    assert!(health_stdout.contains("Show grouped raw analysis queues for the current health view"));
    assert!(health_stdout.contains("Show more item-level detail within the current health view"));

    fs::remove_dir_all(&root).expect("temp repo should be cleaned up");
}
