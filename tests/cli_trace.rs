/**
@module SPECIAL.TESTS.CLI_TRACE
`special trace` integration tests for deterministic relationship packet output.
*/
// @fileimplements SPECIAL.TESTS.CLI_TRACE
#[path = "support/cli.rs"]
mod support;

use std::fs;

use serde_json::Value;

use support::{run_special, temp_repo_dir};

#[test]
// @verifies SPECIAL.TRACE_COMMAND.SPECS
fn trace_specs_json_includes_verifier_evidence_body() {
    let root = temp_repo_dir("special-cli-trace-specs");
    write_trace_fixture(&root);

    let output = run_special(&root, &["trace", "specs", "--id", "APP.EXPORT", "--json"]);
    assert!(output.status.success());
    let json: Value = serde_json::from_slice(&output.stdout).expect("trace output should be json");

    assert_eq!(json["surface"], Value::from("specs"));
    assert_eq!(json["summary"]["packets"], Value::from(1));
    assert!(
        json["guidance"]
            .as_array()
            .expect("guidance should be present")
            .iter()
            .any(|item| item
                .as_str()
                .is_some_and(|text| text.contains("semantic alignment")))
    );
    assert_eq!(
        json["packets"][0]["target"]["id"],
        Value::from("APP.EXPORT")
    );
    assert_eq!(
        json["packets"][0]["evidence"][0]["relationship"],
        Value::from("@verifies")
    );
    assert!(
        json["packets"][0]["evidence"][0]["body"]
            .as_str()
            .expect("body should be present")
            .contains("export_rows")
    );

    fs::remove_dir_all(&root).expect("temp repo should be removed");
}

#[test]
// @verifies SPECIAL.TRACE_COMMAND.DOCS
fn trace_docs_json_includes_source_prose_and_target_evidence() {
    let root = temp_repo_dir("special-cli-trace-docs");
    write_trace_fixture(&root);

    let output = run_special(
        &root,
        &["trace", "docs", "--target", "docs/src/usage.md", "--json"],
    );
    assert!(output.status.success());
    let json: Value = serde_json::from_slice(&output.stdout).expect("trace output should be json");

    assert_eq!(json["surface"], Value::from("docs"));
    assert_eq!(json["summary"]["packets"], Value::from(1));
    assert_eq!(
        json["packets"][0]["target"]["id"],
        Value::from("APP.EXPORT")
    );
    assert!(
        json["packets"][0]["references"][0]["surrounding_prose"]
            .as_str()
            .expect("prose should be present")
            .contains("CSV export")
    );
    assert_eq!(
        json["packets"][0]["evidence"][0]["relationship"],
        Value::from("@verifies")
    );

    fs::remove_dir_all(&root).expect("temp repo should be removed");
}

#[test]
// @verifies SPECIAL.TRACE_COMMAND.DOCS
fn trace_docs_json_includes_documents_line_relationships() {
    let root = temp_repo_dir("special-cli-trace-docs-lines");
    write_trace_fixture(&root);

    let output = run_special(
        &root,
        &["trace", "docs", "--target", "docs/src/line.md", "--json"],
    );
    assert!(output.status.success());
    let json: Value = serde_json::from_slice(&output.stdout).expect("trace output should be json");

    assert_eq!(json["surface"], Value::from("docs"));
    assert_eq!(json["summary"]["packets"], Value::from(1));
    assert_eq!(
        json["packets"][0]["target"]["id"],
        Value::from("APP.EXPORT")
    );
    assert_eq!(
        json["packets"][0]["references"][0]["relationship"],
        Value::from("@documents")
    );

    fs::remove_dir_all(&root).expect("temp repo should be removed");
}

#[test]
// @verifies SPECIAL.TRACE_COMMAND.DOCS
fn trace_docs_json_includes_file_documents_relationships() {
    let root = temp_repo_dir("special-cli-trace-file-docs-lines");
    write_trace_fixture(&root);

    let output = run_special(
        &root,
        &["trace", "docs", "--target", "src/file_docs.rs", "--json"],
    );
    assert!(output.status.success());
    let json: Value = serde_json::from_slice(&output.stdout).expect("trace output should be json");

    assert_eq!(json["surface"], Value::from("docs"));
    assert_eq!(json["summary"]["packets"], Value::from(1));
    assert_eq!(
        json["packets"][0]["target"]["id"],
        Value::from("APP.EXPORT")
    );
    assert_eq!(
        json["packets"][0]["references"][0]["relationship"],
        Value::from("@filedocuments")
    );

    fs::remove_dir_all(&root).expect("temp repo should be removed");
}

#[test]
// @verifies SPECIAL.TRACE_COMMAND.ARCH
fn trace_arch_text_includes_implementation_attachment() {
    let root = temp_repo_dir("special-cli-trace-arch");
    write_trace_fixture(&root);

    let output = run_special(&root, &["trace", "arch", "--id", "APP.EXPORT"]);
    assert!(output.status.success());
    let stdout = String::from_utf8(output.stdout).expect("stdout should be utf-8");

    assert!(stdout.contains("module APP.EXPORT"));
    assert!(stdout.contains("relationship existence"));
    assert!(stdout.contains("does not prove"));
    assert!(stdout.contains("@implements"));
    assert!(stdout.contains("export_rows"));

    fs::remove_dir_all(&root).expect("temp repo should be removed");
}

#[test]
// @verifies SPECIAL.TRACE_COMMAND.PATTERNS
fn trace_patterns_json_includes_applications_and_module_join() {
    let root = temp_repo_dir("special-cli-trace-patterns");
    write_trace_fixture(&root);

    let output = run_special(
        &root,
        &["trace", "patterns", "--id", "APP.EXPORT_PATTERN", "--json"],
    );
    assert!(output.status.success());
    let json: Value = serde_json::from_slice(&output.stdout).expect("trace output should be json");

    assert_eq!(json["surface"], Value::from("patterns"));
    assert_eq!(
        json["packets"][0]["target"]["id"],
        Value::from("APP.EXPORT_PATTERN")
    );
    assert!(
        json["packets"][0]["evidence"]
            .as_array()
            .expect("evidence should be an array")
            .iter()
            .any(|item| item["relationship"].as_str() == Some("@applies"))
    );

    fs::remove_dir_all(&root).expect("temp repo should be removed");
}

#[test]
// @verifies SPECIAL.TRACE_COMMAND.FILTERS
fn trace_rejects_positional_path_scope_and_writes_output_file() {
    let root = temp_repo_dir("special-cli-trace-output");
    write_trace_fixture(&root);

    let rejected = run_special(&root, &["trace", "docs", "docs/src/usage.md"]);
    assert!(!rejected.status.success());
    let stderr = String::from_utf8(rejected.stderr).expect("stderr should be utf-8");
    assert!(stderr.contains("trace path scopes must use --target PATH"));

    let output_path = root.join("audit/trace.json");
    let output = run_special(
        &root,
        &[
            "trace",
            "specs",
            "--id",
            "APP.EXPORT",
            "--json",
            "--output",
            "audit/trace.json",
        ],
    );
    assert!(output.status.success());
    assert!(output.stdout.is_empty());
    let json: Value = serde_json::from_str(
        &fs::read_to_string(output_path).expect("trace output should be written"),
    )
    .expect("trace file should be json");
    assert_eq!(json["summary"]["packets"], Value::from(1));

    fs::remove_dir_all(&root).expect("temp repo should be removed");
}

fn write_trace_fixture(root: &std::path::Path) {
    fs::create_dir_all(root.join("src")).expect("src dir should be created");
    fs::create_dir_all(root.join("docs/src")).expect("docs dir should be created");
    fs::write(root.join("special.toml"), "version = \"1\"\nroot = \".\"\n")
        .expect("config should be written");
    let source = [
        "/**\n",
        &annotation_line("spec", "APP.EXPORT"),
        "CSV export returns rows.\n\n",
        &annotation_line("module", "APP.EXPORT"),
        "CSV export implementation.\n\n",
        &annotation_line("pattern", "APP.EXPORT_PATTERN"),
        "Return a visible row set.\n\n",
        &annotation_line("verifies", "APP.EXPORT"),
        "*/\n",
        "fn verifies_export_rows() {\n",
        "    assert_eq!(export_rows(), vec![\"row\"]);\n",
        "}\n\n",
        "// ",
        &annotation_line("implements", "APP.EXPORT"),
        "// ",
        &annotation_line("applies", "APP.EXPORT_PATTERN"),
        "fn export_rows() -> Vec<&'static str> {\n",
        "    vec![\"row\"]\n",
        "}\n",
    ]
    .concat();
    fs::write(root.join("src/export.rs"), source).expect("source fixture should be written");
    fs::write(
        root.join("docs/src/usage.md"),
        "Use [CSV export](documents://spec/APP.EXPORT) for rows.\n",
    )
    .expect("docs fixture should be written");
    fs::write(
        root.join("docs/src/line.md"),
        [
            &annotation_line("documents", "spec APP.EXPORT"),
            "The export reference page describes the current row behavior.\n",
        ]
        .concat(),
    )
    .expect("docs line fixture should be written");
    fs::write(
        root.join("src/file_docs.rs"),
        ["// ", &annotation_line("filedocuments", "spec APP.EXPORT")].concat(),
    )
    .expect("file docs fixture should be written");
}

fn annotation_line(name: &str, id: &str) -> String {
    format!("@{name} {id}\n")
}
