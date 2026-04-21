/**
@module SPECIAL.TESTS.CLI_MODULES.METRICS.ITEM_SIGNALS
Item-signal and unreached-code metric tests for `special arch --metrics`.
*/
// @fileimplements SPECIAL.TESTS.CLI_MODULES.METRICS.ITEM_SIGNALS
use std::fs;

use serde_json::Value;

use crate::support::{
    find_node_by_id, run_special, temp_repo_dir,
    write_item_scoped_item_signals_module_analysis_fixture,
    write_item_signals_module_analysis_fixture, write_unreached_code_module_analysis_fixture,
};

fn assert_item_signals_output(stdout: &str) {
    assert!(stdout.contains("item signals analyzed: 6"));
    assert!(stdout.contains("connected item: core_helper"));
    assert!(stdout.contains("outbound-heavy item: outbound_heavy"));
    assert!(stdout.contains("isolated item: isolated_external"));
    assert!(stdout.contains("highest complexity item: complex_hotspot"));
    assert!(stdout.contains("parameter-heavy item: outbound_heavy"));
    assert!(stdout.contains("stringly boundary item: outbound_heavy"));
    assert!(stdout.contains("panic-heavy item: outbound_heavy"));
}

#[test]
// @verifies SPECIAL.MODULE_COMMAND.METRICS.UNREACHED_CODE
fn modules_metrics_surface_unreached_code_within_owned_implementation() {
    let root = temp_repo_dir("special-cli-modules-unreached-code");
    write_unreached_code_module_analysis_fixture(&root);

    let output = run_special(&root, &["arch", "--metrics", "--verbose"]);
    assert!(output.status.success());

    let stdout = String::from_utf8(output.stdout).expect("stdout should be utf-8");
    assert!(stdout.contains("unreached items: 2"));
    assert!(stdout.contains("unreached items meaning:"));
    assert!(stdout.contains("unreached item meaning:"));
    assert!(stdout.contains("unreached item exact:"));
    assert!(stdout.contains("unreached item: unreached_cluster_entry"));
    assert!(stdout.contains("unreached item: unreached_cluster_leaf"));

    fs::remove_dir_all(&root).expect("temp repo should be cleaned up");
}

#[test]
// @verifies SPECIAL.MODULE_COMMAND.METRICS.JSON.UNREACHED_CODE
fn modules_metrics_json_includes_structured_unreached_code() {
    let root = temp_repo_dir("special-cli-modules-unreached-code-json");
    write_unreached_code_module_analysis_fixture(&root);

    let output = run_special(&root, &["arch", "--metrics", "--json", "--verbose"]);
    assert!(output.status.success());

    let json: Value =
        serde_json::from_slice(&output.stdout).expect("json output should be valid json");

    let demo = json["nodes"]
        .as_array()
        .and_then(|nodes| nodes.iter().find_map(|node| find_node_by_id(node, "DEMO")))
        .expect("demo module should be present");
    assert_eq!(
        demo["analysis"]["item_signals"]["unreached_items"]
            .as_array()
            .expect("unreached items should be an array")
            .len(),
        2
    );
    assert!(
        demo["analysis"]["item_signals"]["unreached_items"]
            .as_array()
            .expect("unreached items should be an array")
            .iter()
            .any(|item| item["name"].as_str() == Some("unreached_cluster_entry"))
    );
    assert!(
        demo["analysis"]["item_signals"]["unreached_items"]
            .as_array()
            .expect("unreached items should be an array")
            .iter()
            .any(|item| item["name"].as_str() == Some("unreached_cluster_leaf"))
    );

    fs::remove_dir_all(&root).expect("temp repo should be cleaned up");
}

#[test]
// @verifies SPECIAL.MODULE_COMMAND.METRICS.ITEM_SIGNALS.EXPLANATIONS
fn modules_metrics_explains_item_signal_evidence_in_text_and_html() {
    let root = temp_repo_dir("special-cli-modules-metrics-item-signal-explanations");
    write_item_signals_module_analysis_fixture(&root);

    let text_output = run_special(&root, &["arch", "--metrics", "--verbose"]);
    assert!(text_output.status.success());
    let text = String::from_utf8(text_output.stdout).expect("stdout should be utf-8");
    assert!(text.contains("connected item meaning:"));
    assert!(text.contains("highest complexity item exact:"));
    assert!(text.contains("stringly boundary item meaning:"));

    let html_output = run_special(&root, &["arch", "--metrics", "--html", "--verbose"]);
    assert!(html_output.status.success());
    let html = String::from_utf8(html_output.stdout).expect("stdout should be utf-8");
    assert!(html.contains("connected item meaning:"));
    assert!(html.contains("highest complexity item exact:"));
    assert!(html.contains("stringly boundary item meaning:"));

    fs::remove_dir_all(&root).expect("temp repo should be cleaned up");
}

#[test]
// @verifies SPECIAL.MODULE_COMMAND.METRICS.RUST.ITEM_SIGNALS
fn modules_metrics_surface_rust_item_signals() {
    let root = temp_repo_dir("special-cli-modules-metrics-rust-item-signals");
    write_item_signals_module_analysis_fixture(&root);

    let output = run_special(&root, &["arch", "--metrics", "--verbose"]);
    assert!(output.status.success());

    let stdout = String::from_utf8(output.stdout).expect("stdout should be utf-8");
    assert_item_signals_output(&stdout);

    fs::remove_dir_all(&root).expect("temp repo should be cleaned up");
}

#[test]
// @verifies SPECIAL.MODULE_COMMAND.METRICS.RUST.ITEM_SIGNALS.COMPLEXITY
fn modules_metrics_surface_rust_item_complexity_drilldown() {
    let root = temp_repo_dir("special-cli-modules-metrics-rust-item-complexity");
    write_item_signals_module_analysis_fixture(&root);

    let output = run_special(&root, &["arch", "--metrics", "--verbose"]);
    assert!(output.status.success());

    let stdout = String::from_utf8(output.stdout).expect("stdout should be utf-8");
    assert!(stdout.contains("highest complexity item: complex_hotspot"));

    fs::remove_dir_all(&root).expect("temp repo should be cleaned up");
}

#[test]
// @verifies SPECIAL.MODULE_COMMAND.METRICS.RUST.ITEM_SIGNALS.QUALITY
fn modules_metrics_surface_rust_item_quality_drilldown() {
    let root = temp_repo_dir("special-cli-modules-metrics-rust-item-quality");
    write_item_signals_module_analysis_fixture(&root);

    let output = run_special(&root, &["arch", "--metrics", "--verbose"]);
    assert!(output.status.success());

    let stdout = String::from_utf8(output.stdout).expect("stdout should be utf-8");
    assert!(stdout.contains("parameter-heavy item: outbound_heavy"));
    assert!(stdout.contains("stringly boundary item: outbound_heavy"));
    assert!(stdout.contains("panic-heavy item: outbound_heavy"));

    fs::remove_dir_all(&root).expect("temp repo should be cleaned up");
}

#[test]
// @verifies SPECIAL.MODULE_COMMAND.METRICS.RUST.ITEM_SIGNALS
fn modules_metrics_surface_rust_item_signals_for_item_scoped_implements() {
    let root = temp_repo_dir("special-cli-modules-metrics-rust-item-signals-item-scoped");
    write_item_scoped_item_signals_module_analysis_fixture(&root);

    let output = run_special(&root, &["arch", "--metrics", "--verbose"]);
    assert!(output.status.success());

    let stdout = String::from_utf8(output.stdout).expect("stdout should be utf-8");
    assert!(stdout.contains("item signals analyzed: 3"));
    assert!(stdout.contains("connected item: connected"));
    assert!(stdout.contains("isolated item: isolated_external"));

    fs::remove_dir_all(&root).expect("temp repo should be cleaned up");
}

#[test]
// @verifies SPECIAL.MODULE_COMMAND.METRICS.JSON.RUST.ITEM_SIGNALS
fn modules_metrics_json_includes_structured_rust_item_signals() {
    let root = temp_repo_dir("special-cli-modules-metrics-rust-item-signals-json");
    write_item_signals_module_analysis_fixture(&root);

    let output = run_special(&root, &["arch", "--metrics", "--json", "--verbose"]);
    assert!(output.status.success());

    let json: Value =
        serde_json::from_slice(&output.stdout).expect("json output should be valid json");
    let demo = json["nodes"]
        .as_array()
        .and_then(|nodes| nodes.iter().find_map(|node| find_node_by_id(node, "DEMO")))
        .expect("demo module should be present");
    assert_eq!(
        demo["analysis"]["item_signals"]["analyzed_items"],
        Value::from(6)
    );
    assert_eq!(
        demo["analysis"]["item_signals"]["isolated_items"][0]["name"],
        Value::from("isolated_external")
    );
    assert_eq!(
        demo["analysis"]["item_signals"]["outbound_heavy_items"][0]["name"],
        Value::from("outbound_heavy")
    );
    assert_eq!(
        demo["analysis"]["item_signals"]["connected_items"][0]["name"],
        Value::from("helper_leaf")
    );

    fs::remove_dir_all(&root).expect("temp repo should be cleaned up");
}

#[test]
// @verifies SPECIAL.MODULE_COMMAND.METRICS.JSON.RUST.ITEM_SIGNALS.COMPLEXITY
fn modules_metrics_json_includes_structured_rust_item_complexity_drilldown() {
    let root = temp_repo_dir("special-cli-modules-metrics-rust-item-complexity-json");
    write_item_signals_module_analysis_fixture(&root);

    let output = run_special(&root, &["arch", "--metrics", "--json", "--verbose"]);
    assert!(output.status.success());

    let json: Value =
        serde_json::from_slice(&output.stdout).expect("json output should be valid json");
    let demo = json["nodes"]
        .as_array()
        .and_then(|nodes| nodes.iter().find_map(|node| find_node_by_id(node, "DEMO")))
        .expect("demo module should be present");
    assert_eq!(
        demo["analysis"]["item_signals"]["highest_complexity_items"][0]["name"],
        Value::from("complex_hotspot")
    );
    assert_eq!(
        demo["analysis"]["item_signals"]["highest_complexity_items"][0]["cognitive"],
        demo["analysis"]["complexity"]["max_cognitive"]
    );

    fs::remove_dir_all(&root).expect("temp repo should be cleaned up");
}

#[test]
// @verifies SPECIAL.MODULE_COMMAND.METRICS.JSON.RUST.ITEM_SIGNALS.QUALITY
fn modules_metrics_json_includes_structured_rust_item_quality_drilldown() {
    let root = temp_repo_dir("special-cli-modules-metrics-rust-item-quality-json");
    write_item_signals_module_analysis_fixture(&root);

    let output = run_special(&root, &["arch", "--metrics", "--json", "--verbose"]);
    assert!(output.status.success());

    let json: Value =
        serde_json::from_slice(&output.stdout).expect("json output should be valid json");
    let demo = json["nodes"]
        .as_array()
        .and_then(|nodes| nodes.iter().find_map(|node| find_node_by_id(node, "DEMO")))
        .expect("demo module should be present");
    assert_eq!(
        demo["analysis"]["item_signals"]["parameter_heavy_items"][0]["name"],
        Value::from("outbound_heavy")
    );
    assert_eq!(
        demo["analysis"]["item_signals"]["parameter_heavy_items"][0]["parameter_count"],
        Value::from(3)
    );
    assert_eq!(
        demo["analysis"]["item_signals"]["stringly_boundary_items"][0]["raw_string_parameter_count"],
        Value::from(2)
    );
    assert_eq!(
        demo["analysis"]["item_signals"]["panic_heavy_items"][0]["panic_site_count"],
        Value::from(1)
    );

    fs::remove_dir_all(&root).expect("temp repo should be cleaned up");
}

#[test]
// @verifies SPECIAL.MODULE_COMMAND.METRICS.JSON.RUST.ITEM_SIGNALS
fn modules_metrics_json_includes_structured_rust_item_signals_for_item_scoped_implements() {
    let root = temp_repo_dir("special-cli-modules-metrics-rust-item-signals-item-scoped-json");
    write_item_scoped_item_signals_module_analysis_fixture(&root);

    let output = run_special(&root, &["arch", "--metrics", "--json", "--verbose"]);
    assert!(output.status.success());

    let json: Value =
        serde_json::from_slice(&output.stdout).expect("json output should be valid json");
    let demo = json["nodes"]
        .as_array()
        .and_then(|nodes| nodes.iter().find_map(|node| find_node_by_id(node, "DEMO")))
        .expect("demo module should be present");
    assert_eq!(
        demo["analysis"]["item_signals"]["analyzed_items"],
        Value::from(3)
    );
    assert_eq!(
        demo["analysis"]["item_signals"]["connected_items"][0]["name"],
        Value::from("connected")
    );
    assert_eq!(
        demo["analysis"]["item_signals"]["isolated_items"][0]["name"],
        Value::from("isolated_external")
    );

    fs::remove_dir_all(&root).expect("temp repo should be cleaned up");
}
