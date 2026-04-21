/**
@module SPECIAL.TESTS.CLI_MODULES.METRICS.COMPLEXITY
Complexity metric and explanation tests for `special arch --metrics`.
*/
// @fileimplements SPECIAL.TESTS.CLI_MODULES.METRICS.COMPLEXITY
use std::fs;

use serde_json::Value;

use crate::support::{
    find_node_by_id, run_special, temp_repo_dir,
    write_cognitive_complexity_module_analysis_fixture, write_complexity_module_analysis_fixture,
    write_item_signals_module_analysis_fixture,
};

#[test]
// @verifies SPECIAL.MODULE_COMMAND.METRICS.RUST.COMPLEXITY
fn modules_metrics_surface_rust_complexity_summary() {
    let root = temp_repo_dir("special-cli-modules-metrics-complexity");
    write_complexity_module_analysis_fixture(&root);

    let output = run_special(&root, &["arch", "DEMO", "--metrics"]);
    assert!(output.status.success());

    let stdout = String::from_utf8(output.stdout).expect("stdout should be utf-8");
    assert!(stdout.contains("complexity functions: 2"));
    assert!(stdout.contains("cyclomatic total: 8"));
    assert!(stdout.contains("cyclomatic max: 7"));

    fs::remove_dir_all(&root).expect("temp repo should be cleaned up");
}

#[test]
// @verifies SPECIAL.MODULE_COMMAND.METRICS.JSON.RUST.COMPLEXITY
fn modules_metrics_json_includes_structured_complexity_summary() {
    let root = temp_repo_dir("special-cli-modules-metrics-complexity-json");
    write_complexity_module_analysis_fixture(&root);

    let output = run_special(&root, &["arch", "--metrics", "--json", "--verbose"]);
    assert!(output.status.success());

    let json: Value =
        serde_json::from_slice(&output.stdout).expect("json output should be valid json");
    let demo = json["nodes"]
        .as_array()
        .and_then(|nodes| nodes.iter().find_map(|node| find_node_by_id(node, "DEMO")))
        .expect("demo module should be present");
    assert_eq!(
        demo["analysis"]["complexity"]["function_count"],
        Value::from(2)
    );
    assert_eq!(
        demo["analysis"]["complexity"]["total_cyclomatic"],
        Value::from(8)
    );
    assert_eq!(
        demo["analysis"]["complexity"]["max_cyclomatic"],
        Value::from(7)
    );

    fs::remove_dir_all(&root).expect("temp repo should be cleaned up");
}

#[test]
// @verifies SPECIAL.MODULE_COMMAND.METRICS.RUST.COMPLEXITY.COGNITIVE
fn modules_metrics_surface_rust_cognitive_complexity_summary() {
    let root = temp_repo_dir("special-cli-modules-metrics-cognitive");
    write_cognitive_complexity_module_analysis_fixture(&root);

    let output = run_special(&root, &["arch", "DEMO", "--metrics"]);
    assert!(output.status.success());

    let stdout = String::from_utf8(output.stdout).expect("stdout should be utf-8");
    assert!(stdout.contains("cognitive total: 3"));
    assert!(stdout.contains("cognitive max: 3"));

    fs::remove_dir_all(&root).expect("temp repo should be cleaned up");
}

#[test]
// @verifies SPECIAL.MODULE_COMMAND.METRICS.JSON.RUST.COMPLEXITY.COGNITIVE
fn modules_metrics_json_includes_structured_cognitive_complexity_summary() {
    let root = temp_repo_dir("special-cli-modules-metrics-cognitive-json");
    write_cognitive_complexity_module_analysis_fixture(&root);

    let output = run_special(&root, &["arch", "DEMO", "--metrics", "--json"]);
    assert!(output.status.success());

    let json: Value =
        serde_json::from_slice(&output.stdout).expect("json output should be valid json");
    let demo = json["nodes"]
        .as_array()
        .and_then(|nodes| nodes.iter().find_map(|node| find_node_by_id(node, "DEMO")))
        .expect("demo module should be present");
    assert_eq!(
        demo["analysis"]["complexity"]["total_cognitive"],
        Value::from(3)
    );
    assert_eq!(
        demo["analysis"]["complexity"]["max_cognitive"],
        Value::from(3)
    );

    fs::remove_dir_all(&root).expect("temp repo should be cleaned up");
}

#[test]
// @verifies SPECIAL.MODULE_COMMAND.METRICS.COMPLEXITY.EXPLANATIONS
fn modules_metrics_explains_complexity_evidence_in_text_and_html() {
    let root = temp_repo_dir("special-cli-modules-metrics-complexity-explanations");
    write_item_signals_module_analysis_fixture(&root);

    let text_output = run_special(&root, &["arch", "--metrics", "--verbose"]);
    assert!(text_output.status.success());
    let text = String::from_utf8(text_output.stdout).expect("stdout should be utf-8");
    assert!(text.contains("cyclomatic total meaning:"));
    assert!(text.contains("cyclomatic total exact:"));
    assert!(text.contains("cognitive max meaning:"));

    let html_output = run_special(&root, &["arch", "--metrics", "--html", "--verbose"]);
    assert!(html_output.status.success());
    let html = String::from_utf8(html_output.stdout).expect("stdout should be utf-8");
    assert!(html.contains("cyclomatic total meaning:"));
    assert!(html.contains("cyclomatic total exact:"));
    assert!(html.contains("cognitive max meaning:"));

    fs::remove_dir_all(&root).expect("temp repo should be cleaned up");
}
