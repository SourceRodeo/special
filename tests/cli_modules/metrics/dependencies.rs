/**
@module SPECIAL.TESTS.CLI_MODULES.METRICS.DEPENDENCIES
Dependency metric tests for `special arch --metrics`.
*/
// @fileimplements SPECIAL.TESTS.CLI_MODULES.METRICS.DEPENDENCIES
use std::fs;

use serde_json::Value;

use crate::support::{
    find_node_by_id, run_special, temp_repo_dir, write_dependency_module_analysis_fixture,
};

#[test]
// @verifies SPECIAL.MODULE_COMMAND.METRICS.RUST.DEPENDENCIES
fn modules_metrics_surface_rust_use_path_dependency_evidence() {
    let root = temp_repo_dir("special-cli-modules-metrics-dependencies");
    write_dependency_module_analysis_fixture(&root);

    let output = run_special(&root, &["arch", "--metrics", "--verbose"]);
    assert!(output.status.success());

    let stdout = String::from_utf8(output.stdout).expect("stdout should be utf-8");
    assert!(stdout.contains("dependency refs: 2"));
    assert!(stdout.contains("dependency targets: 2"));
    assert!(stdout.contains("dependency target: crate::shared::util::helper (1)"));
    assert!(stdout.contains("dependency target: serde_json::Value (1)"));

    fs::remove_dir_all(&root).expect("temp repo should be cleaned up");
}

#[test]
// @verifies SPECIAL.MODULE_COMMAND.METRICS.JSON.RUST.DEPENDENCIES
fn modules_metrics_json_includes_structured_dependency_targets() {
    let root = temp_repo_dir("special-cli-modules-metrics-dependencies-json");
    write_dependency_module_analysis_fixture(&root);

    let output = run_special(&root, &["arch", "--metrics", "--json", "--verbose"]);
    assert!(output.status.success());

    let json: Value =
        serde_json::from_slice(&output.stdout).expect("json output should be valid json");
    let demo = json["nodes"]
        .as_array()
        .and_then(|nodes| nodes.iter().find_map(|node| find_node_by_id(node, "DEMO")))
        .expect("demo module should be present");
    assert_eq!(
        demo["analysis"]["dependencies"]["reference_count"],
        Value::from(2)
    );
    assert_eq!(
        demo["analysis"]["dependencies"]["distinct_targets"],
        Value::from(2)
    );
    let targets = demo["analysis"]["dependencies"]["targets"]
        .as_array()
        .expect("dependency targets should be an array");
    assert!(targets.iter().any(|target| {
        target["path"] == Value::String("crate::shared::util::helper".to_string())
            && target["count"] == 1
    }));
    assert!(targets.iter().any(|target| {
        target["path"] == Value::String("serde_json::Value".to_string()) && target["count"] == 1
    }));

    fs::remove_dir_all(&root).expect("temp repo should be cleaned up");
}
