/**
@module SPECIAL.TESTS.CLI_MODULES.METRICS.QUALITY
Quality metric and explanation tests for `special arch --metrics`.
*/
// @fileimplements SPECIAL.TESTS.CLI_MODULES.METRICS.QUALITY
use std::fs;

use serde_json::Value;

use crate::support::{
    find_node_by_id, run_special, temp_repo_dir, write_item_signals_module_analysis_fixture,
    write_quality_module_analysis_fixture,
};

fn assert_quality_output(stdout: &str) {
    assert!(stdout.contains("quality public functions: 1"));
    assert!(stdout.contains("quality parameters: 3"));
    assert!(stdout.contains("quality bool params: 1"));
    assert!(stdout.contains("quality raw string params: 2"));
    assert!(stdout.contains("quality panic sites: 1"));
}

#[test]
// @verifies SPECIAL.MODULE_COMMAND.METRICS.QUALITY
fn modules_metrics_surface_quality_evidence() {
    let root = temp_repo_dir("special-cli-modules-metrics-quality");
    write_quality_module_analysis_fixture(&root);

    let output = run_special(&root, &["arch", "DEMO", "--metrics"]);
    assert!(output.status.success());

    let stdout = String::from_utf8(output.stdout).expect("stdout should be utf-8");
    assert_quality_output(&stdout);

    fs::remove_dir_all(&root).expect("temp repo should be cleaned up");
}

#[test]
// @verifies SPECIAL.MODULE_COMMAND.METRICS.RUST.QUALITY
fn modules_metrics_surface_rust_quality_evidence() {
    let root = temp_repo_dir("special-cli-modules-metrics-rust-quality");
    write_quality_module_analysis_fixture(&root);

    let output = run_special(&root, &["arch", "DEMO", "--metrics"]);
    assert!(output.status.success());

    let stdout = String::from_utf8(output.stdout).expect("stdout should be utf-8");
    assert_quality_output(&stdout);

    fs::remove_dir_all(&root).expect("temp repo should be cleaned up");
}

#[test]
// @verifies SPECIAL.MODULE_COMMAND.METRICS.JSON.QUALITY
fn modules_metrics_json_includes_structured_quality_summary() {
    let root = temp_repo_dir("special-cli-modules-metrics-quality-json");
    write_quality_module_analysis_fixture(&root);

    let output = run_special(&root, &["arch", "DEMO", "--metrics", "--json"]);
    assert!(output.status.success());

    let json: Value =
        serde_json::from_slice(&output.stdout).expect("json output should be valid json");
    let demo = json["nodes"]
        .as_array()
        .and_then(|nodes| nodes.iter().find_map(|node| find_node_by_id(node, "DEMO")))
        .expect("demo module should be present");
    assert_eq!(
        demo["analysis"]["quality"]["public_function_count"],
        Value::from(1)
    );
    assert_eq!(
        demo["analysis"]["quality"]["parameter_count"],
        Value::from(3)
    );
    assert_eq!(
        demo["analysis"]["quality"]["bool_parameter_count"],
        Value::from(1)
    );
    assert_eq!(
        demo["analysis"]["quality"]["raw_string_parameter_count"],
        Value::from(2)
    );
    assert_eq!(
        demo["analysis"]["quality"]["panic_site_count"],
        Value::from(1)
    );

    fs::remove_dir_all(&root).expect("temp repo should be cleaned up");
}

#[test]
// @verifies SPECIAL.MODULE_COMMAND.METRICS.JSON.RUST.QUALITY
fn modules_metrics_json_includes_structured_rust_quality_evidence() {
    let root = temp_repo_dir("special-cli-modules-metrics-rust-quality-json");
    write_quality_module_analysis_fixture(&root);

    let output = run_special(&root, &["arch", "DEMO", "--metrics", "--json"]);
    assert!(output.status.success());

    let json: Value =
        serde_json::from_slice(&output.stdout).expect("json output should be valid json");
    let demo = json["nodes"]
        .as_array()
        .and_then(|nodes| nodes.iter().find_map(|node| find_node_by_id(node, "DEMO")))
        .expect("demo module should be present");
    assert_eq!(
        demo["analysis"]["quality"]["public_function_count"],
        Value::from(1)
    );
    assert_eq!(
        demo["analysis"]["quality"]["parameter_count"],
        Value::from(3)
    );
    assert_eq!(
        demo["analysis"]["quality"]["bool_parameter_count"],
        Value::from(1)
    );
    assert_eq!(
        demo["analysis"]["quality"]["raw_string_parameter_count"],
        Value::from(2)
    );
    assert_eq!(
        demo["analysis"]["quality"]["panic_site_count"],
        Value::from(1)
    );

    fs::remove_dir_all(&root).expect("temp repo should be cleaned up");
}

#[test]
fn modules_metrics_keep_explanations_out_of_default_metrics_view() {
    let root = temp_repo_dir("special-cli-modules-metrics-default-summary");
    write_item_signals_module_analysis_fixture(&root);

    let text_output = run_special(&root, &["arch", "--metrics"]);
    assert!(text_output.status.success());
    let text = String::from_utf8(text_output.stdout).expect("stdout should be utf-8");
    assert!(!text.contains("cyclomatic total meaning:"));
    assert!(!text.contains("quality parameters meaning:"));
    assert!(!text.contains("connected item meaning:"));

    fs::remove_dir_all(&root).expect("temp repo should be cleaned up");
}

#[test]
// @verifies SPECIAL.MODULE_COMMAND.METRICS.QUALITY.EXPLANATIONS
fn modules_metrics_explains_quality_evidence_in_text_and_html() {
    let root = temp_repo_dir("special-cli-modules-metrics-quality-explanations");
    write_item_signals_module_analysis_fixture(&root);

    let text_output = run_special(&root, &["arch", "--metrics", "--verbose"]);
    assert!(text_output.status.success());
    let text = String::from_utf8(text_output.stdout).expect("stdout should be utf-8");
    assert!(text.contains("quality parameters meaning:"));
    assert!(text.contains("quality raw string params exact:"));
    assert!(text.contains("quality panic sites meaning:"));

    let html_output = run_special(&root, &["arch", "--metrics", "--html", "--verbose"]);
    assert!(html_output.status.success());
    let html = String::from_utf8(html_output.stdout).expect("stdout should be utf-8");
    assert!(html.contains("quality parameters meaning:"));
    assert!(html.contains("quality raw string params exact:"));
    assert!(html.contains("quality panic sites meaning:"));

    fs::remove_dir_all(&root).expect("temp repo should be cleaned up");
}
