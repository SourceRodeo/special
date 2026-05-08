/**
@module SPECIAL.TESTS.CLI_DIFF
`special diff` integration tests for VCS-scoped explicit relationship review.
*/
// @fileimplements SPECIAL.TESTS.CLI_DIFF
#[path = "support/cli.rs"]
mod support;

use std::fs;
use std::process::Command;

use serde_json::Value;

use support::{run_special, temp_repo_dir};

#[test]
// @verifies SPECIAL.DIFF_COMMAND.NO_VCS
fn diff_without_vcs_config_shows_full_relationship_view() {
    let root = temp_repo_dir("special-cli-diff-no-vcs");
    write_diff_fixture(&root, None);

    let output = run_special(&root, &["diff", "--json"]);
    assert!(output.status.success());

    let stderr = String::from_utf8(output.stderr).expect("stderr should be utf-8");
    assert!(stderr.contains("no `vcs` declared"));
    let json: Value =
        serde_json::from_slice(&output.stdout).expect("json output should be valid json");
    assert_eq!(json["summary"]["changed_paths"], Value::from(0));
    assert_eq!(json["summary"]["affected_relationships"], Value::from(3));
    assert_eq!(
        json["relationships"]
            .as_array()
            .expect("relationships")
            .len(),
        3
    );

    fs::remove_dir_all(&root).expect("temp repo should be cleaned up");
}

#[test]
// @verifies SPECIAL.DIFF_COMMAND.NO_VCS
fn diff_with_vcs_none_shows_full_relationship_view() {
    let root = temp_repo_dir("special-cli-diff-vcs-none");
    write_diff_fixture(&root, Some("none"));

    let output = run_special(&root, &["diff", "--json"]);
    assert!(output.status.success());

    let stderr = String::from_utf8(output.stderr).expect("stderr should be utf-8");
    assert!(stderr.contains("vcs = \"none\""));
    let json: Value =
        serde_json::from_slice(&output.stdout).expect("json output should be valid json");
    assert_eq!(json["summary"]["changed_paths"], Value::from(0));
    assert_eq!(json["summary"]["affected_relationships"], Value::from(3));

    fs::remove_dir_all(&root).expect("temp repo should be cleaned up");
}

#[test]
// @verifies SPECIAL.DIFF_COMMAND
fn diff_uses_declared_git_changed_paths_for_relationship_review() {
    let root = temp_repo_dir("special-cli-diff-git");
    write_diff_fixture(&root, Some("git"));
    run_git(&root, &["init"]);
    run_git(&root, &["add", "."]);
    run_git(&root, &["commit", "-m", "initial"]);
    fs::write(
        root.join("src/feature.rs"),
        "/// @module APP.FEATURE\n/// Feature module.\n// @fileimplements APP.FEATURE\npub fn feature() -> bool { false }\n",
    )
    .expect("changed feature source should be written");

    let output = run_special(
        &root,
        &[
            "diff",
            "--target",
            "src/feature.rs",
            "--metrics",
            "--verbose",
        ],
    );
    assert!(output.status.success());

    let stdout = String::from_utf8(output.stdout).expect("stdout should be utf-8");
    assert!(stdout.contains("changed paths: 1"));
    assert!(stdout.contains("affected relationships: 1"));
    assert!(stdout.contains("@implements module APP.FEATURE"));
    assert!(stdout.contains("relationship kinds:"));
    assert!(stdout.contains("target kinds:"));
    assert!(stdout.contains("source src/feature.rs"));
    assert!(stdout.contains("target src/feature.rs"));
    assert!(!stdout.contains("APP.FEATURE.WORKS"));

    fs::remove_dir_all(&root).expect("temp repo should be cleaned up");
}

#[test]
// @verifies SPECIAL.CLI.EXPLICIT_PATH_SCOPE
fn diff_rejects_positional_path_scope() {
    let root = temp_repo_dir("special-cli-diff-no-positional-scope");
    write_diff_fixture(&root, Some("none"));

    let output = run_special(&root, &["diff", "src/feature.rs"]);
    assert!(!output.status.success());

    let stderr = String::from_utf8(output.stderr).expect("stderr should be utf-8");
    assert!(stderr.contains("diff path scopes must use --target PATH"));
    assert!(stderr.contains("special diff --target src/feature.rs"));

    fs::remove_dir_all(&root).expect("temp repo should be cleaned up");
}

#[test]
// @verifies SPECIAL.DIFF_COMMAND.METRICS
fn diff_metrics_reports_relationship_breakdowns() {
    let root = temp_repo_dir("special-cli-diff-metrics");
    write_diff_fixture(&root, Some("none"));

    let output = run_special(&root, &["diff", "--metrics"]);
    assert!(output.status.success());

    let stdout = String::from_utf8(output.stdout).expect("stdout should be utf-8");
    assert!(stdout.contains("relationship kinds:"));
    assert!(stdout.contains("@implements: 1"));
    assert!(stdout.contains("@verifies: 1"));
    assert!(stdout.contains("documents://: 1"));
    assert!(stdout.contains("target kinds:"));
    assert!(stdout.contains("top source paths:"));

    fs::remove_dir_all(&root).expect("temp repo should be cleaned up");
}

#[test]
// @verifies SPECIAL.DIFF_COMMAND.VERBOSE
fn diff_verbose_reports_current_endpoint_content() {
    let root = temp_repo_dir("special-cli-diff-verbose");
    write_diff_fixture(&root, Some("none"));

    let output = run_special(&root, &["diff", "--verbose"]);
    assert!(output.status.success());

    let stdout = String::from_utf8(output.stdout).expect("stdout should be utf-8");
    assert!(stdout.contains("source src/feature.rs"));
    assert!(stdout.contains("target src/feature.rs"));
    assert!(stdout.contains("Feature module."));

    fs::remove_dir_all(&root).expect("temp repo should be cleaned up");
}

fn write_diff_fixture(root: &std::path::Path, vcs: Option<&str>) {
    fs::create_dir_all(root.join("src")).expect("src dir should be created");
    let mut config = "version = \"1\"\nroot = \".\"\n".to_string();
    if let Some(vcs) = vcs {
        config.push_str(&format!("vcs = \"{vcs}\"\n"));
    }
    fs::write(root.join("special.toml"), config).expect("special.toml should be written");
    fs::write(
        root.join("src/feature.rs"),
        "/// @module APP.FEATURE\n/// Feature module.\n// @fileimplements APP.FEATURE\npub fn feature() -> bool { true }\n",
    )
    .expect("feature source should be written");
    fs::write(
        root.join("tests.rs"),
        "/// @spec APP.FEATURE.WORKS\n/// Feature returns true.\n/// @verifies APP.FEATURE.WORKS\nfn verifies_feature() { assert!(true); }\n",
    )
    .expect("test source should be written");
    fs::write(
        root.join("docs.md"),
        "# Docs\n\nThe [feature](documents://module/APP.FEATURE) exists.\n",
    )
    .expect("docs source should be written");
}

fn run_git(root: &std::path::Path, args: &[&str]) {
    let output = Command::new("git")
        .args(args)
        .current_dir(root)
        .env("GIT_AUTHOR_NAME", "Special Test")
        .env("GIT_AUTHOR_EMAIL", "special@example.com")
        .env("GIT_COMMITTER_NAME", "Special Test")
        .env("GIT_COMMITTER_EMAIL", "special@example.com")
        .output()
        .expect("git command should run");
    assert!(
        output.status.success(),
        "git {:?} failed: {}",
        args,
        String::from_utf8_lossy(&output.stderr)
    );
}
