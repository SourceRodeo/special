/**
@module SPECIAL.TESTS.CLI_MODULES.METRICS.COUPLING
Module coupling metrics tests for `special arch --metrics`.
*/
// @fileimplements SPECIAL.TESTS.CLI_MODULES.METRICS.COUPLING
use std::fs;

use serde_json::Value;

use crate::support::{
    find_node_by_id, run_special, temp_repo_dir, write_ambiguous_coupling_module_analysis_fixture,
    write_coupling_module_analysis_fixture,
};

fn assert_module_coupling_output(stdout: &str) {
    assert!(stdout.contains("fan out: 1"));
    assert!(stdout.contains("efferent coupling: 1"));
    assert!(stdout.contains("instability: 1.00"));
    assert!(stdout.contains("fan in: 1"));
    assert!(stdout.contains("afferent coupling: 1"));
}

fn assert_module_coupling_explanations(stdout: &str) {
    assert!(stdout.contains("fan out meaning: this module reaches into other owned modules."));
    assert!(stdout.contains("fan out exact:"));
    assert!(stdout.contains("distinct outbound concrete-module dependencies"));
    assert!(stdout.contains(
        "instability exact: efferent coupling / (afferent coupling + efferent coupling)."
    ));
}

#[test]
// @verifies SPECIAL.MODULE_COMMAND.METRICS.COUPLING
fn modules_metrics_surface_module_coupling() {
    let root = temp_repo_dir("special-cli-modules-metrics-coupling");
    write_coupling_module_analysis_fixture(&root);

    let api_output = run_special(&root, &["arch", "DEMO.API", "--metrics"]);
    assert!(api_output.status.success());
    let api_stdout = String::from_utf8(api_output.stdout).expect("stdout should be utf-8");
    assert!(api_stdout.contains("fan out: 1"));
    assert!(api_stdout.contains("efferent coupling: 1"));
    assert!(api_stdout.contains("instability: 1.00"));

    let shared_output = run_special(&root, &["arch", "DEMO.SHARED", "--metrics"]);
    assert!(shared_output.status.success());
    let shared_stdout = String::from_utf8(shared_output.stdout).expect("stdout should be utf-8");
    assert!(shared_stdout.contains("fan in: 1"));
    assert!(shared_stdout.contains("afferent coupling: 1"));
    assert!(shared_stdout.contains("instability: 0.00"));

    fs::remove_dir_all(&root).expect("temp repo should be cleaned up");
}

#[test]
// @verifies SPECIAL.MODULE_COMMAND.METRICS.COUPLING.EXPLANATIONS
fn modules_metrics_surface_module_coupling_explanations() {
    let root = temp_repo_dir("special-cli-modules-metrics-coupling-explanations");
    write_coupling_module_analysis_fixture(&root);

    let output = run_special(&root, &["arch", "--metrics", "--verbose"]);
    assert!(output.status.success());

    let stdout = String::from_utf8(output.stdout).expect("stdout should be utf-8");
    assert_module_coupling_output(&stdout);
    assert_module_coupling_explanations(&stdout);

    fs::remove_dir_all(&root).expect("temp repo should be cleaned up");
}

#[test]
// @verifies SPECIAL.MODULE_COMMAND.METRICS.RUST.COUPLING
fn modules_metrics_surface_rust_derived_module_coupling() {
    let root = temp_repo_dir("special-cli-modules-metrics-rust-coupling");
    write_coupling_module_analysis_fixture(&root);

    let api_output = run_special(&root, &["arch", "DEMO.API", "--metrics"]);
    assert!(api_output.status.success());
    let api_stdout = String::from_utf8(api_output.stdout).expect("stdout should be utf-8");
    assert!(api_stdout.contains("fan out: 1"));
    assert!(api_stdout.contains("efferent coupling: 1"));
    assert!(api_stdout.contains("instability: 1.00"));

    let shared_output = run_special(&root, &["arch", "DEMO.SHARED", "--metrics"]);
    assert!(shared_output.status.success());
    let shared_stdout = String::from_utf8(shared_output.stdout).expect("stdout should be utf-8");
    assert!(shared_stdout.contains("fan in: 1"));
    assert!(shared_stdout.contains("afferent coupling: 1"));
    assert!(shared_stdout.contains("instability: 0.00"));

    fs::remove_dir_all(&root).expect("temp repo should be cleaned up");
}

#[test]
// @verifies SPECIAL.MODULE_COMMAND.METRICS.JSON.COUPLING
fn modules_metrics_json_includes_structured_module_coupling() {
    let root = temp_repo_dir("special-cli-modules-metrics-coupling-json");
    write_coupling_module_analysis_fixture(&root);

    let api_output = run_special(&root, &["arch", "DEMO.API", "--metrics", "--json"]);
    assert!(api_output.status.success());

    let api_json: Value =
        serde_json::from_slice(&api_output.stdout).expect("json output should be valid json");
    let api = api_json["nodes"]
        .as_array()
        .and_then(|nodes| {
            nodes
                .iter()
                .find_map(|node| find_node_by_id(node, "DEMO.API"))
        })
        .expect("api module should be present");
    assert_eq!(api["analysis"]["coupling"]["fan_out"], Value::from(1));
    assert_eq!(
        api["analysis"]["coupling"]["efferent_coupling"],
        Value::from(1)
    );
    assert_eq!(api["analysis"]["coupling"]["instability"], Value::from(1.0));

    let shared_output = run_special(&root, &["arch", "DEMO.SHARED", "--metrics", "--json"]);
    assert!(shared_output.status.success());

    let shared_json: Value =
        serde_json::from_slice(&shared_output.stdout).expect("json output should be valid json");
    let shared = shared_json["nodes"]
        .as_array()
        .and_then(|nodes| {
            nodes
                .iter()
                .find_map(|node| find_node_by_id(node, "DEMO.SHARED"))
        })
        .expect("shared module should be present");
    assert_eq!(shared["analysis"]["coupling"]["fan_in"], Value::from(1));
    assert_eq!(
        shared["analysis"]["coupling"]["afferent_coupling"],
        Value::from(1)
    );
    assert_eq!(
        shared["analysis"]["coupling"]["instability"],
        Value::from(0.0)
    );

    fs::remove_dir_all(&root).expect("temp repo should be cleaned up");
}

#[test]
fn modules_metrics_distinguish_ambiguous_internal_targets_from_unresolved_ones() {
    let root = temp_repo_dir("special-cli-modules-metrics-coupling-ambiguous");
    write_ambiguous_coupling_module_analysis_fixture(&root);

    let text_output = run_special(&root, &["arch", "DEMO.API", "--metrics"]);
    assert!(text_output.status.success());
    let stdout = String::from_utf8(text_output.stdout).expect("stdout should be utf-8");
    assert!(stdout.contains("ambiguous internal dependency targets: 1"));
    assert!(stdout.contains("unresolved internal dependency targets: 0"));

    let json_output = run_special(&root, &["arch", "DEMO.API", "--metrics", "--json"]);
    assert!(json_output.status.success());
    let json: Value =
        serde_json::from_slice(&json_output.stdout).expect("json output should be valid json");
    let api = json["nodes"]
        .as_array()
        .and_then(|nodes| {
            nodes
                .iter()
                .find_map(|node| find_node_by_id(node, "DEMO.API"))
        })
        .expect("api module should be present");
    assert_eq!(
        api["analysis"]["coupling"]["ambiguous_internal_target_count"],
        Value::from(1)
    );
    assert_eq!(
        api["analysis"]["coupling"]["unresolved_internal_target_count"],
        Value::from(0)
    );

    fs::remove_dir_all(&root).expect("temp repo should be cleaned up");
}
