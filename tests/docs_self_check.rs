/**
@module SPECIAL.TESTS.DOCS_SELF_CHECK
Self-hosting docs-as-code checks for Special's own generated documentation structure.
*/
// @fileimplements SPECIAL.TESTS.DOCS_SELF_CHECK
#[path = "support/cli.rs"]
mod support;

use std::path::PathBuf;

use serde_json::Value;

use support::run_special;

fn repo_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
}

fn run_repo_json(args: &[&str]) -> Value {
    let output = run_special(&repo_root(), args);
    assert!(
        output.status.success(),
        "command should succeed: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    serde_json::from_slice(&output.stdout).expect("stdout should be valid json")
}

fn number_at<'a>(json: &'a Value, path: &[&str]) -> u64 {
    path.iter()
        .fold(json, |value, key| &value[*key])
        .as_u64()
        .unwrap_or_else(|| panic!("json number should exist at {}", path.join(".")))
}

fn string_at<'a>(json: &'a Value, path: &[&str]) -> &'a str {
    path.iter()
        .fold(json, |value, key| &value[*key])
        .as_str()
        .unwrap_or_else(|| panic!("json string should exist at {}", path.join(".")))
}

fn array_at<'a>(json: &'a Value, path: &[&str]) -> &'a Vec<Value> {
    path.iter()
        .fold(json, |value, key| &value[*key])
        .as_array()
        .unwrap_or_else(|| panic!("json array should exist at {}", path.join(".")))
}

fn entry_with_value<'a>(entries: &'a [Value], key: &str, value: &str) -> &'a Value {
    entries
        .iter()
        .find(|entry| entry[key].as_str() == Some(value))
        .unwrap_or_else(|| panic!("entry should exist for {key}={value}"))
}

fn docs_target_kind<'a>(metrics: &'a Value, kind: &str) -> &'a Value {
    entry_with_value(
        array_at(metrics, &["metrics", "target_kinds"]),
        "kind",
        kind,
    )
}

fn docs_coverage_kind<'a>(metrics: &'a Value, kind: &str) -> &'a Value {
    entry_with_value(
        array_at(metrics, &["metrics", "coverage", "target_kinds"]),
        "kind",
        kind,
    )
}

fn grouped_count(entries: &[Value], value: &str) -> u64 {
    entry_with_value(entries, "value", value)["count"]
        .as_u64()
        .unwrap_or_else(|| panic!("grouped count should exist for {value}"))
}

#[test]
fn docs_self_check_metrics_keep_generated_docs_connected_and_traceable() {
    let json = run_repo_json(&["docs", "--metrics", "--json", "--target", "docs/src"]);

    let total_references = number_at(&json, &["metrics", "total_references"]);
    assert!(total_references >= 280);
    assert_eq!(
        number_at(&json, &["metrics", "link_references"]),
        total_references
    );
    assert_eq!(
        number_at(&json, &["metrics", "documents_line_references"]),
        0
    );
    assert_eq!(
        number_at(&json, &["metrics", "file_documents_line_references"]),
        0
    );

    let generated_pages = number_at(&json, &["metrics", "generated_pages"]);
    assert!(generated_pages >= 22);
    assert!(number_at(&json, &["metrics", "local_doc_links"]) >= 25);
    assert_eq!(number_at(&json, &["metrics", "broken_local_doc_links"]), 0);
    assert_eq!(number_at(&json, &["metrics", "orphan_pages"]), 0);
    assert_eq!(
        number_at(&json, &["metrics", "reachable_pages_from_entrypoints"]),
        generated_pages
    );
    assert!(number_at(&json, &["metrics", "entrypoint_pages"]) >= 1);

    let spec_references = docs_target_kind(&json, "spec");
    assert!(spec_references["references"].as_u64().unwrap_or_default() >= 280);
    assert!(spec_references["generated"].as_u64().unwrap_or_default() >= 200);
    let pattern_references = docs_target_kind(&json, "pattern");
    assert!(pattern_references["generated"].as_u64().unwrap_or_default() >= 4);

    let spec_coverage = docs_coverage_kind(&json, "spec");
    assert!(spec_coverage["documented"].as_u64().unwrap_or_default() >= 200);

    let targets_with_issues = array_at(&json, &["metrics", "target_audit"])
        .iter()
        .filter(|target| {
            target["issues"]
                .as_array()
                .is_some_and(|issues| !issues.is_empty())
        })
        .count();
    assert_eq!(targets_with_issues, 0);
}

#[test]
fn docs_self_check_arch_metrics_cover_docs_source_modules() {
    let json = run_repo_json(&["arch", "SPECIAL.DOCUMENTATION", "--metrics", "--json"]);

    let metrics = &json["metrics"];
    let total_modules = metrics["total_modules"].as_u64().unwrap_or_default();
    assert!(total_modules >= 62);
    assert_eq!(metrics["unimplemented_modules"].as_u64(), Some(0));
    assert_eq!(metrics["file_scoped_implements"].as_u64(), Some(0));
    assert_eq!(
        metrics["item_scoped_implements"].as_u64(),
        Some(total_modules)
    );
    assert!(metrics["owned_lines"].as_u64().unwrap_or_default() >= 1_700);

    let modules_by_area = array_at(metrics, &["modules_by_area"]);
    assert!(grouped_count(modules_by_area, "SPECIAL.DOCUMENTATION.PUBLIC") >= 54);
    assert!(grouped_count(modules_by_area, "SPECIAL.DOCUMENTATION.CONTRIBUTOR") >= 8);
}

#[test]
fn docs_self_check_pattern_metrics_cover_docs_patterns() {
    let expected = [
        ("DOCS.SURFACE_GUIDE_PAGE", 5),
        ("DOCS.CONTRIBUTOR_RUNBOOK_PAGE", 8),
        ("DOCS.REFERENCE_CATALOG_PAGE", 3),
        ("DOCS.TRACEABLE_DOCS_EXAMPLE", 3),
    ];

    for (pattern_id, minimum_applications) in expected {
        let json = run_repo_json(&["patterns", pattern_id, "--metrics", "--json"]);
        assert!(number_at(&json, &["metrics", "total_patterns"]) >= 20);
        assert!(number_at(&json, &["metrics", "total_applications"]) >= 140);

        let pattern = array_at(&json, &["patterns"])
            .first()
            .expect("scoped pattern should be present");
        assert_eq!(pattern["id"].as_str(), Some(pattern_id));
        assert_eq!(string_at(pattern, &["metrics", "strictness"]), "low");
        assert!(number_at(pattern, &["metrics", "scored_applications"]) >= minimum_applications);
        assert_eq!(
            string_at(pattern, &["metrics", "benchmark_estimate"]),
            "tighter_than_expected"
        );
    }
}

#[test]
fn docs_self_check_health_metrics_keep_docs_scope_clean() {
    let json = run_repo_json(&[
        "health",
        "--metrics",
        "--json",
        "--target",
        "docs/src",
        "--within",
        "docs/src",
    ]);

    assert_eq!(
        number_at(&json, &["metrics", "global", "raw_investigation_queues"]),
        0
    );
    assert_eq!(
        number_at(
            &json,
            &["metrics", "architecture", "source_outside_architecture"]
        ),
        0
    );
    assert_eq!(
        number_at(&json, &["metrics", "patterns", "duplicate_source_shapes"]),
        0
    );
    assert_eq!(
        number_at(&json, &["metrics", "patterns", "possible_pattern_clusters"]),
        0
    );
    assert_eq!(
        number_at(
            &json,
            &["metrics", "patterns", "possible_missing_applications"]
        ),
        0
    );
    assert_eq!(
        number_at(&json, &["metrics", "docs", "long_prose_outside_docs"]),
        0
    );
    assert_eq!(
        number_at(&json, &["metrics", "tests", "exact_long_prose_assertions"]),
        0
    );
}
