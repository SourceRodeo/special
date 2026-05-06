/**
@module SPECIAL.TESTS.CLI_MODULES.PARSE
Architecture declaration, attachment, and lint behavior tests for `special arch`.
*/
// @fileimplements SPECIAL.TESTS.CLI_MODULES.PARSE
use std::fs;

use serde_json::Value;

use crate::support::{
    find_node_by_id, rendered_spec_node_ids, run_special, temp_repo_dir,
    write_area_implements_fixture, write_area_modules_fixture,
    write_duplicate_file_scoped_implements_fixture, write_duplicate_item_scoped_implements_fixture,
    write_implements_with_trailing_content_fixture, write_missing_intermediate_modules_fixture,
    write_mixed_purpose_source_local_module_fixture, write_modules_fixture,
    write_planned_area_fixture, write_planned_area_invalid_suffix_fixture,
    write_source_local_modules_fixture, write_unimplemented_module_fixture,
    write_unknown_implements_fixture,
};

fn expect_single_implementation<'a>(node: &'a Value, message: &str) -> &'a Value {
    let implementations = node["implements"]
        .as_array()
        .expect("implements should be an array");
    assert_eq!(implementations.len(), 1, "{message}");
    &implementations[0]
}

#[test]
// @verifies SPECIAL.MODULE_PARSE.MARKDOWN_DECLARATIONS
fn modules_read_markdown_architecture_declarations_when_present() {
    let root = temp_repo_dir("special-cli-modules-architecture-doc");
    write_area_modules_fixture(&root);

    let output = run_special(&root, &["arch", "--json"]);
    assert!(output.status.success());

    let json: Value =
        serde_json::from_slice(&output.stdout).expect("json output should be valid json");
    let area = json["nodes"]
        .as_array()
        .and_then(|nodes| nodes.iter().find_map(|node| find_node_by_id(node, "DEMO")))
        .expect("DEMO area should be present");
    assert_eq!(area["id"], Value::String("DEMO".to_string()));

    fs::remove_dir_all(&root).expect("temp repo should be cleaned up");
}

#[test]
// @verifies SPECIAL.MODULE_PARSE.MARKDOWN_DECLARATIONS
fn modules_read_bare_markdown_architecture_declaration_lines() {
    let root = temp_repo_dir("special-cli-modules-bare-architecture-lines");
    fs::write(
        root.join("architecture.md"),
        "@area DEMO\nDemo area.\n\n@module DEMO.API @planned\nAPI module.\n",
    )
    .expect("bare markdown architecture declarations should be written");

    let output = run_special(&root, &["arch", "--json"]);
    assert!(output.status.success());

    let json: Value =
        serde_json::from_slice(&output.stdout).expect("json output should be valid json");
    let module = json["nodes"]
        .as_array()
        .and_then(|nodes| {
            nodes
                .iter()
                .find_map(|node| find_node_by_id(node, "DEMO.API"))
        })
        .expect("DEMO.API module should be present");
    assert_eq!(module["text"], Value::String("API module.".to_string()));

    fs::remove_dir_all(&root).expect("temp repo should be cleaned up");
}

#[test]
// @verifies SPECIAL.MODULE_PARSE.MODULE_DECLARATIONS
fn modules_parse_source_local_module_declarations() {
    let root = temp_repo_dir("special-cli-modules-source-local-decls");
    write_source_local_modules_fixture(&root);

    let output = run_special(&root, &["arch", "--json"]);
    assert!(output.status.success());

    let json: Value =
        serde_json::from_slice(&output.stdout).expect("json output should be valid json");
    let demo = json["nodes"]
        .as_array()
        .and_then(|nodes| nodes.iter().find_map(|node| find_node_by_id(node, "DEMO")))
        .expect("DEMO module should be present");
    let local = json["nodes"]
        .as_array()
        .and_then(|nodes| {
            nodes
                .iter()
                .find_map(|node| find_node_by_id(node, "DEMO.LOCAL"))
        })
        .expect("DEMO.LOCAL module should be present");
    assert_eq!(demo["kind"], Value::String("module".to_string()));
    assert_eq!(local["kind"], Value::String("module".to_string()));

    fs::remove_dir_all(&root).expect("temp repo should be cleaned up");
}

#[test]
// @verifies SPECIAL.MODULE_PARSE.FOREIGN_TAG_BOUNDARIES
fn modules_stop_text_before_foreign_comment_tags() {
    let root = temp_repo_dir("special-cli-modules-foreign-tag-boundary");
    write_mixed_purpose_source_local_module_fixture(&root);

    let output = run_special(&root, &["arch", "--json"]);
    assert!(output.status.success());

    let json: Value =
        serde_json::from_slice(&output.stdout).expect("json output should be valid json");
    let module = json["nodes"]
        .as_array()
        .and_then(|nodes| nodes.iter().find_map(|node| find_node_by_id(node, "DEMO")))
        .expect("DEMO module should be present");
    assert_eq!(
        module["text"],
        Value::String("Renders the demo export surface.".to_string())
    );

    fs::remove_dir_all(&root).expect("temp repo should be cleaned up");
}

#[test]
// @verifies SPECIAL.MODULE_PARSE.AREA_DECLARATIONS
fn modules_parse_area_declarations_from_architecture_doc() {
    let root = temp_repo_dir("special-cli-modules-area-declarations");
    write_area_modules_fixture(&root);

    let output = run_special(&root, &["arch", "--json"]);
    assert!(output.status.success());

    let json: Value =
        serde_json::from_slice(&output.stdout).expect("json output should be valid json");
    let area = json["nodes"]
        .as_array()
        .and_then(|nodes| nodes.iter().find_map(|node| find_node_by_id(node, "DEMO")))
        .expect("DEMO area should be present");
    assert_eq!(area["kind"], Value::String("area".to_string()));

    fs::remove_dir_all(&root).expect("temp repo should be cleaned up");
}

#[test]
// @verifies SPECIAL.MODULE_PARSE.IMPLEMENTS.FILE_SCOPE
fn modules_record_top_of_file_implements_without_owned_item_body() {
    let root = temp_repo_dir("special-cli-modules-file-scope");
    write_modules_fixture(&root);

    let output = run_special(&root, &["arch", "--json", "--verbose"]);
    assert!(output.status.success());

    let json: Value =
        serde_json::from_slice(&output.stdout).expect("json output should be valid json");
    let demo = json["nodes"]
        .as_array()
        .and_then(|nodes| nodes.iter().find_map(|node| find_node_by_id(node, "DEMO")))
        .expect("DEMO module should be present");
    let implementation = expect_single_implementation(
        demo,
        "file-scoped module fixture should expose exactly one implementation",
    );
    assert!(implementation["body"].is_null());
    assert!(implementation["body_location"].is_null());

    fs::remove_dir_all(&root).expect("temp repo should be cleaned up");
}

#[test]
// @verifies SPECIAL.MODULE_PARSE.IMPLEMENTS.ITEM_SCOPE
fn modules_attach_owned_item_bodies_for_item_scoped_implements() {
    let root = temp_repo_dir("special-cli-modules-item-scope");
    write_modules_fixture(&root);

    let output = run_special(&root, &["arch", "--json", "--verbose"]);
    assert!(output.status.success());

    let json: Value =
        serde_json::from_slice(&output.stdout).expect("json output should be valid json");
    let demo_live = json["nodes"]
        .as_array()
        .and_then(|nodes| {
            nodes
                .iter()
                .find_map(|node| find_node_by_id(node, "DEMO.LIVE"))
        })
        .expect("DEMO.LIVE module should be present");
    let implementation = expect_single_implementation(
        demo_live,
        "item-scoped module fixture should expose exactly one implementation",
    );
    assert_eq!(
        implementation["body"],
        Value::String("fn implements_demo_live() {}".to_string())
    );
    assert!(implementation["body_location"].is_object());

    fs::remove_dir_all(&root).expect("temp repo should be cleaned up");
}

#[test]
// @verifies SPECIAL.MODULE_PARSE.IMPLEMENTS.MARKDOWN_SCOPE
fn modules_record_markdown_file_scoped_implements() {
    let root = temp_repo_dir("special-cli-modules-markdown-file-implements");
    fs::write(root.join("special.toml"), "version = \"1\"\nroot = \".\"\n")
        .expect("special.toml should be written");
    fs::write(
        root.join("docs.md"),
        "### `@module DOCS`\nDocs module.\n\n@fileimplements DOCS\n",
    )
    .expect("markdown fixture should be written");

    let output = run_special(&root, &["arch", "--json", "--verbose"]);
    assert!(output.status.success());

    let json: Value =
        serde_json::from_slice(&output.stdout).expect("json output should be valid json");
    let docs = json["nodes"]
        .as_array()
        .and_then(|nodes| nodes.iter().find_map(|node| find_node_by_id(node, "DOCS")))
        .expect("DOCS module should be present");
    let implementation = expect_single_implementation(
        docs,
        "markdown file-scoped implementation should be recorded",
    );
    assert!(implementation["body"].is_null());
    assert!(implementation["body_location"].is_null());

    fs::remove_dir_all(&root).expect("temp repo should be cleaned up");
}

#[test]
// @verifies SPECIAL.MODULE_PARSE.IMPLEMENTS.MARKDOWN_SCOPE
fn modules_attach_markdown_implements_to_heading_sections() {
    let root = temp_repo_dir("special-cli-modules-markdown-section-implements");
    fs::write(root.join("special.toml"), "version = \"1\"\nroot = \".\"\n")
        .expect("special.toml should be written");
    fs::write(
        root.join("docs.md"),
        "### `@area DOCS`\nDocs area.\n\n### `@module DOCS.QUICK`\nQuick module.\n\n@implements DOCS.QUICK\n## Quick start\nRun setup.\n\n### Details\nKeep it short.\n\n## Reference\nOptions.\n",
    )
    .expect("markdown fixture should be written");

    let output = run_special(&root, &["arch", "--json", "--verbose"]);
    assert!(output.status.success());

    let json: Value =
        serde_json::from_slice(&output.stdout).expect("json output should be valid json");
    let quick = json["nodes"]
        .as_array()
        .and_then(|nodes| {
            nodes
                .iter()
                .find_map(|node| find_node_by_id(node, "DOCS.QUICK"))
        })
        .expect("DOCS.QUICK module should be present");
    let implementation =
        expect_single_implementation(quick, "markdown section implementation should be recorded");
    let body = implementation["body"]
        .as_str()
        .expect("markdown implementation body should be text");
    assert!(body.contains("## Quick start"));
    assert!(body.contains("### Details"));
    assert!(!body.contains("## Reference"));
    assert!(implementation["body_location"].is_object());

    fs::remove_dir_all(&root).expect("temp repo should be cleaned up");
}

#[test]
// @verifies SPECIAL.MODULE_PARSE.IMPLEMENTS.MARKDOWN_SCOPE
fn modules_attach_inline_markdown_implements_to_containing_section() {
    let root = temp_repo_dir("special-cli-modules-markdown-contained-implements");
    fs::write(root.join("special.toml"), "version = \"1\"\nroot = \".\"\n")
        .expect("special.toml should be written");
    fs::write(
        root.join("docs.md"),
        "### `@area DOCS`\nDocs area.\n\n### `@module DOCS.INSTALL`\nInstall module.\n\n## Install\n@implements DOCS.INSTALL\nUse setup.\n",
    )
    .expect("markdown fixture should be written");

    let output = run_special(&root, &["arch", "--json", "--verbose"]);
    assert!(output.status.success());

    let json: Value =
        serde_json::from_slice(&output.stdout).expect("json output should be valid json");
    let install = json["nodes"]
        .as_array()
        .and_then(|nodes| {
            nodes
                .iter()
                .find_map(|node| find_node_by_id(node, "DOCS.INSTALL"))
        })
        .expect("DOCS.INSTALL module should be present");
    let implementation = expect_single_implementation(
        install,
        "contained markdown implementation should be recorded",
    );
    let body = implementation["body"]
        .as_str()
        .expect("markdown implementation body should be text");
    assert!(body.contains("## Install"));
    assert!(body.contains("Use setup."));
    assert!(!body.contains("@implements"));

    fs::remove_dir_all(&root).expect("temp repo should be cleaned up");
}

#[test]
// @verifies SPECIAL.LINT_COMMAND.UNKNOWN_IMPLEMENTS_REFS
fn lint_reports_unknown_implements_references() {
    let root = temp_repo_dir("special-cli-modules-lint-unknown");
    write_unknown_implements_fixture(&root);

    let output = run_special(&root, &["lint"]);
    assert!(!output.status.success());

    let stdout = String::from_utf8(output.stdout).expect("stdout should be utf-8");
    assert!(
        stdout.contains(
            "unknown module id `DEMO.MISSING` referenced by @implements or @fileimplements"
        )
    );

    fs::remove_dir_all(&root).expect("temp repo should be cleaned up");
}

#[test]
// @verifies SPECIAL.MODULE_PARSE.IMPLEMENTS.EXACT_DIRECTIVE_SHAPE
fn lint_rejects_trailing_content_after_implements_module_id() {
    let root = temp_repo_dir("special-cli-modules-lint-implements-trailing");
    write_implements_with_trailing_content_fixture(&root);

    let output = run_special(&root, &["lint"]);
    assert!(!output.status.success());

    let stdout = String::from_utf8(output.stdout).expect("stdout should be utf-8");
    assert!(stdout.contains("unexpected trailing content after @implements module id"));
    assert!(stdout.contains("unexpected trailing content after @fileimplements module id"));

    fs::remove_dir_all(&root).expect("temp repo should be cleaned up");
}

#[test]
// @verifies SPECIAL.LINT_COMMAND.INTERMEDIATE_MODULES
fn lint_reports_missing_intermediate_module_ids() {
    let root = temp_repo_dir("special-cli-modules-lint-intermediate");
    write_missing_intermediate_modules_fixture(&root);

    let output = run_special(&root, &["lint"]);
    assert!(!output.status.success());

    let stdout = String::from_utf8(output.stdout).expect("stdout should be utf-8");
    assert!(stdout.contains("missing intermediate module `DEMO`"));
    assert!(stdout.contains("missing intermediate module `DEMO.CHILD`"));

    fs::remove_dir_all(&root).expect("temp repo should be cleaned up");
}

#[test]
// @verifies SPECIAL.MODULE_PARSE.IMPLEMENTS.DUPLICATE_FILE_SCOPE
fn lint_reports_duplicate_file_scoped_implements() {
    let root = temp_repo_dir("special-cli-modules-lint-duplicate-file-scope");
    write_duplicate_file_scoped_implements_fixture(&root);

    let output = run_special(&root, &["lint"]);
    assert!(!output.status.success());

    let stdout = String::from_utf8(output.stdout).expect("stdout should be utf-8");
    assert!(stdout.contains("duplicate @fileimplements"));

    fs::remove_dir_all(&root).expect("temp repo should be cleaned up");
}

#[test]
// @verifies SPECIAL.MODULE_PARSE.IMPLEMENTS.DUPLICATE_ITEM_SCOPE
fn lint_reports_duplicate_item_scoped_implements() {
    let root = temp_repo_dir("special-cli-modules-lint-duplicate-item-scope");
    write_duplicate_item_scoped_implements_fixture(&root);

    let output = run_special(&root, &["lint"]);
    assert!(!output.status.success());

    let stdout = String::from_utf8(output.stdout).expect("stdout should be utf-8");
    assert!(stdout.contains("duplicate @implements for attached item"));

    fs::remove_dir_all(&root).expect("temp repo should be cleaned up");
}

#[test]
// @verifies SPECIAL.MODULE_COMMAND.AREA_NODES
fn modules_parse_area_nodes() {
    let root = temp_repo_dir("special-cli-modules-area-nodes");
    write_area_modules_fixture(&root);

    let text_output = run_special(&root, &["arch"]);
    assert!(text_output.status.success());
    let text_stdout = String::from_utf8(text_output.stdout).expect("stdout should be utf-8");
    let node_ids = rendered_spec_node_ids(&text_stdout);
    assert!(node_ids.contains(&"DEMO".to_string()));

    let json_output = run_special(&root, &["arch", "--json"]);
    assert!(json_output.status.success());
    let json: Value =
        serde_json::from_slice(&json_output.stdout).expect("json output should be valid json");
    let area = json["nodes"]
        .as_array()
        .and_then(|nodes| nodes.iter().find_map(|node| find_node_by_id(node, "DEMO")))
        .expect("DEMO area should be present");
    assert_eq!(area["kind"], Value::String("area".to_string()));

    fs::remove_dir_all(&root).expect("temp repo should be cleaned up");
}

#[test]
// @verifies SPECIAL.MODULE_COMMAND.KIND_LABELS
fn modules_label_area_nodes_by_kind_in_output() {
    let root = temp_repo_dir("special-cli-modules-kind-labels");
    write_area_modules_fixture(&root);

    let text_output = run_special(&root, &["arch"]);
    assert!(text_output.status.success());
    let text_stdout = String::from_utf8(text_output.stdout).expect("stdout should be utf-8");
    assert!(text_stdout.contains("DEMO [area]"));

    let json_output = run_special(&root, &["arch", "--json"]);
    assert!(json_output.status.success());
    let json: Value =
        serde_json::from_slice(&json_output.stdout).expect("json output should be valid json");
    let area = json["nodes"]
        .as_array()
        .and_then(|nodes| nodes.iter().find_map(|node| find_node_by_id(node, "DEMO")))
        .expect("DEMO area should be present");
    assert_eq!(area["kind"], Value::String("area".to_string()));

    let html_output = run_special(&root, &["arch", "--html"]);
    assert!(html_output.status.success());
    let html_stdout = String::from_utf8(html_output.stdout).expect("stdout should be utf-8");
    assert!(html_stdout.contains(">area</span>"));

    fs::remove_dir_all(&root).expect("temp repo should be cleaned up");
}

#[test]
// @verifies SPECIAL.MODULE_PARSE.IMPLEMENTS.MODULE_ONLY
fn lint_rejects_implements_on_area_ids() {
    let root = temp_repo_dir("special-cli-modules-area-implements");
    write_area_implements_fixture(&root);

    let output = run_special(&root, &["lint"]);
    assert!(!output.status.success());

    let stdout = String::from_utf8(output.stdout).expect("stdout should be utf-8");
    assert!(stdout.contains("@implements and @fileimplements may only reference @module ids"));

    fs::remove_dir_all(&root).expect("temp repo should be cleaned up");
}

#[test]
// @verifies SPECIAL.MODULE_PARSE.PLANNED.MODULE_ONLY
fn lint_rejects_planned_areas() {
    let root = temp_repo_dir("special-cli-modules-planned-area");
    write_planned_area_fixture(&root);

    let output = run_special(&root, &["lint"]);
    assert!(!output.status.success());

    let stdout = String::from_utf8(output.stdout).expect("stdout should be utf-8");
    assert!(stdout.contains("@planned may only apply to @module, not @area"));

    fs::remove_dir_all(&root).expect("temp repo should be cleaned up");
}

#[test]
// @verifies SPECIAL.MODULE_PARSE.PLANNED.EXACT_STANDALONE_MARKER
fn lint_rejects_standalone_planned_suffixes_in_module_declarations() {
    let root = temp_repo_dir("special-cli-modules-planned-invalid-suffix");
    write_planned_area_invalid_suffix_fixture(&root);

    let output = run_special(&root, &["lint"]);
    assert!(!output.status.success());

    let stdout = String::from_utf8(output.stdout).expect("stdout should be utf-8");
    assert!(stdout.contains("use an exact standalone `@planned` marker with no trailing suffix"));
    assert!(!stdout.contains("planned areas"));

    fs::remove_dir_all(&root).expect("temp repo should be cleaned up");
}

#[test]
// @verifies SPECIAL.MODULE_PARSE.CURRENT_MODULES_REQUIRE_IMPLEMENTATION
fn lint_rejects_current_modules_without_direct_implements() {
    let root = temp_repo_dir("special-cli-modules-unimplemented");
    write_unimplemented_module_fixture(&root);

    let output = run_special(&root, &["lint"]);
    assert!(!output.status.success());

    let stdout = String::from_utf8(output.stdout).expect("stdout should be utf-8");
    assert!(stdout.contains("current module `DEMO` has no ownership"));
    assert!(stdout.contains("mark the module @planned"));

    fs::remove_dir_all(&root).expect("temp repo should be cleaned up");
}

#[test]
// @verifies SPECIAL.MODULE_PARSE.AREAS_ARE_STRUCTURAL_ONLY
fn modules_unimplemented_filter_does_not_treat_areas_as_missing_implementation() {
    let root = temp_repo_dir("special-cli-modules-areas-structural-only");
    write_area_modules_fixture(&root);

    let output = run_special(&root, &["arch", "--unimplemented"]);
    assert!(output.status.success());

    let stdout = String::from_utf8(output.stdout).expect("stdout should be utf-8");
    let node_ids = rendered_spec_node_ids(&stdout);
    assert!(!node_ids.contains(&"DEMO".to_string()));

    fs::remove_dir_all(&root).expect("temp repo should be cleaned up");
}
