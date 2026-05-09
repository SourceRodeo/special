/**
@module SPECIAL.TESTS.CLI_DOCS
CLI integration tests for documentation relationship validation and generated docs output.
*/
// @fileimplements SPECIAL.TESTS.CLI_DOCS
#[path = "support/cli.rs"]
mod support;

use std::fs;

use serde_json::Value;

use support::{run_special, temp_repo_dir};

#[test]
// @verifies SPECIAL.DOCS_COMMAND.OUTPUT
fn docs_output_rewrites_document_links_and_removes_document_lines() {
    let root = temp_repo_dir("special-cli-docs-output");
    write_docs_fixture(&root);
    fs::create_dir_all(root.join("docs/src/nested")).expect("nested docs dir should be created");
    fs::write(
        root.join("docs/src/README.md"),
        concat!(
            "# Guide\n\n",
            "@documents spec EXPORT.CSV.HEADERS\n",
            "[CSV exports include headers](documents://spec/EXPORT.CSV.HEADERS).\n",
            "[Parser ownership](documents://module/APP.PARSER) is documented.\n",
            "[Cache fill](documents://pattern/CACHE.SINGLE_FLIGHT_FILL) is intentional.\n",
        ),
    )
    .expect("docs source should be written");
    fs::write(root.join("docs/src/nested/asset.txt"), "plain\n")
        .expect("plain asset should be written");

    let output = run_special(&root, &["docs", "build", "docs/src", "docs/dist"]);

    assert!(
        output.status.success(),
        "docs output should succeed: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    let rendered =
        fs::read_to_string(root.join("docs/dist/README.md")).expect("rendered docs should exist");
    assert!(rendered.contains("CSV exports include headers."));
    assert!(rendered.contains("Parser ownership is documented."));
    assert!(rendered.contains("Cache fill is intentional."));
    assert!(!rendered.contains("documents://"));
    assert!(!rendered.contains("@documents"));
    assert_eq!(
        fs::read_to_string(root.join("docs/dist/nested/asset.txt"))
            .expect("plain asset should be copied"),
        "plain\n"
    );
}

#[test]
// @verifies SPECIAL.DOCS_COMMAND.OUTPUT.DIRECTORY
fn docs_output_mirrors_directory_tree() {
    let root = temp_repo_dir("special-cli-docs-output-directory");
    write_docs_fixture(&root);
    fs::create_dir_all(root.join("docs/src/nested")).expect("nested docs dir should be created");
    fs::write(
        root.join("docs/src/README.md"),
        "[CSV exports include headers](documents://spec/EXPORT.CSV.HEADERS).\n",
    )
    .expect("docs source should be written");
    fs::write(root.join("docs/src/nested/asset.txt"), "plain\n")
        .expect("plain asset should be written");

    let output = run_special(&root, &["docs", "build", "docs/src", "docs/dist"]);

    assert!(
        output.status.success(),
        "docs directory output should succeed: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    assert!(root.join("docs/dist/README.md").exists());
    assert_eq!(
        fs::read_to_string(root.join("docs/dist/nested/asset.txt"))
            .expect("plain asset should be copied"),
        "plain\n"
    );
}

#[test]
// @verifies SPECIAL.DOCS_COMMAND.OUTPUT.AUTHORING_LINES
fn docs_output_removes_markdown_architecture_authoring_lines() {
    let root = temp_repo_dir("special-cli-docs-output-authoring-lines");
    write_docs_fixture(&root);
    fs::write(
        root.join("source.md"),
        concat!(
            "# Guide\n\n",
            "@fileimplements APP.PARSER\n",
            "@fileapplies CACHE.SINGLE_FLIGHT_FILL\n",
            "@implements APP.PARSER\n",
            "@applies CACHE.SINGLE_FLIGHT_FILL\n",
            "[CSV](documents://spec/EXPORT.CSV.HEADERS).\n\n",
            "`@implements APP.EXAMPLE`\n\n",
            "```markdown\n",
            "@applies APP.EXAMPLE\n",
            "```\n",
        ),
    )
    .expect("docs source should be written");

    let output = run_special(&root, &["docs", "build", "source.md", "public.md"]);

    assert!(
        output.status.success(),
        "docs output should succeed: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    let rendered = fs::read_to_string(root.join("public.md")).expect("output should be written");
    assert!(!rendered.contains("@fileimplements APP.PARSER"));
    assert!(!rendered.contains("@fileapplies CACHE.SINGLE_FLIGHT_FILL"));
    assert!(!rendered.contains("@implements APP.PARSER"));
    assert!(!rendered.contains("@applies CACHE.SINGLE_FLIGHT_FILL"));
    assert!(rendered.contains("CSV."));
    assert!(rendered.contains("`@implements APP.EXAMPLE`"));
    assert!(rendered.contains("@applies APP.EXAMPLE"));
}

#[test]
// @verifies SPECIAL.DOCS.LINKS.OUTPUT
fn docs_output_rewrites_document_links_to_plain_text() {
    let root = temp_repo_dir("special-cli-docs-link-output");
    write_docs_fixture(&root);
    fs::write(
        root.join("source.md"),
        "[CSV exports include headers](documents://spec/EXPORT.CSV.HEADERS).\n",
    )
    .expect("source docs markdown should be written");

    let output = run_special(&root, &["docs", "build", "source.md", "public.md"]);

    assert!(
        output.status.success(),
        "docs output should succeed: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    let rendered = fs::read_to_string(root.join("public.md")).expect("output should be written");
    assert_eq!(rendered, "CSV exports include headers.\n");
}

#[test]
// @verifies SPECIAL.DOCS.DOCUMENTS_LINES
fn docs_lint_rejects_stacked_document_relationship_lines() {
    let root = temp_repo_dir("special-cli-docs-stacked-document-lines");
    write_docs_fixture(&root);
    fs::write(
        root.join("docs.md"),
        concat!(
            "@documents spec EXPORT.CSV.HEADERS\n",
            "@filedocuments module APP.PARSER\n",
        ),
    )
    .expect("docs markdown should be written");

    let output = run_special(&root, &["lint"]);

    assert!(!output.status.success());
    let stdout = String::from_utf8(output.stdout).expect("stdout should be utf-8");
    assert!(stdout.contains("documentation relationship lines may not be stacked"));
}

#[test]
// @verifies SPECIAL.DOCS.LINKS.POLYMORPHIC
fn docs_links_accept_polymorphic_targets() {
    let root = temp_repo_dir("special-cli-docs-polymorphic-links");
    write_docs_fixture(&root);
    fs::write(
        root.join("docs.md"),
        concat!(
            "[Export group](documents://group/EXPORT).\n",
            "[CSV spec](documents://spec/EXPORT.CSV.HEADERS).\n",
            "[App area](documents://area/APP).\n",
            "[Parser module](documents://module/APP.PARSER).\n",
            "[Cache pattern](documents://pattern/CACHE.SINGLE_FLIGHT_FILL).\n",
        ),
    )
    .expect("docs markdown should be written");

    let output = run_special(&root, &["docs"]);

    assert!(
        output.status.success(),
        "docs validation should succeed: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    let stdout = String::from_utf8(output.stdout).expect("stdout should be utf-8");
    assert!(stdout.contains("group EXPORT"));
    assert!(stdout.contains("spec EXPORT.CSV.HEADERS"));
    assert!(stdout.contains("area APP"));
    assert!(stdout.contains("module APP.PARSER"));
    assert!(stdout.contains("pattern CACHE.SINGLE_FLIGHT_FILL"));
}

#[test]
// @verifies SPECIAL.CONFIG.SPECIAL_TOML.DOCS_PATHS
fn special_toml_accepts_docs_output_mappings() {
    let root = temp_repo_dir("special-cli-docs-config-docs-paths");
    write_docs_fixture(&root);
    fs::write(
        root.join("special.toml"),
        "version = \"1\"\nroot = \".\"\n\n[[docs.outputs]]\nsource = \"docs/src\"\noutput = \"docs/dist\"\n",
    )
    .expect("special.toml should be written");
    fs::create_dir_all(root.join("docs/src")).expect("docs source dir should be created");
    fs::write(
        root.join("docs/src/README.md"),
        "[CSV exports include headers](documents://spec/EXPORT.CSV.HEADERS).\n",
    )
    .expect("docs source should be written");

    let output = run_special(&root, &["docs", "build"]);

    assert!(
        output.status.success(),
        "configured docs output should succeed: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    assert!(root.join("docs/dist/README.md").exists());
}

#[test]
// @verifies SPECIAL.DOCS_COMMAND.OUTPUT.CONFIG
fn docs_output_uses_configured_source_and_output_paths() {
    let root = temp_repo_dir("special-cli-docs-output-config");
    write_docs_fixture(&root);
    fs::write(
        root.join("special.toml"),
        "version = \"1\"\nroot = \".\"\n\n[[docs.outputs]]\nsource = \"docs/src\"\noutput = \"docs/dist\"\n\n[[docs.outputs]]\nsource = \"docs/src/README.md\"\noutput = \"README.md\"\n",
    )
    .expect("special.toml should be written");
    fs::create_dir_all(root.join("docs/src/nested")).expect("nested docs dir should be created");
    fs::write(
        root.join("docs/src/README.md"),
        "[CSV exports include headers](documents://spec/EXPORT.CSV.HEADERS).\n",
    )
    .expect("docs source should be written");
    fs::write(root.join("docs/src/nested/asset.txt"), "plain\n")
        .expect("plain asset should be written");

    let output = run_special(&root, &["docs", "build"]);

    assert!(
        output.status.success(),
        "configured docs output should succeed: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    let rendered =
        fs::read_to_string(root.join("docs/dist/README.md")).expect("rendered docs should exist");
    assert!(rendered.contains("CSV exports include headers."));
    let readme = fs::read_to_string(root.join("README.md")).expect("root README should exist");
    assert!(readme.contains("CSV exports include headers."));
    assert_eq!(
        fs::read_to_string(root.join("docs/dist/nested/asset.txt"))
            .expect("plain asset should preserve its relative tree path"),
        "plain\n"
    );
}

#[test]
// @verifies SPECIAL.DOCS_COMMAND.METRICS.RELATIONSHIPS
fn docs_metrics_reports_documentation_relationship_inventory() {
    let root = temp_repo_dir("special-cli-docs-metrics-coverage");
    write_docs_metrics_fixture(&root);

    let output = run_special(&root, &["docs", "--metrics"]);

    assert!(
        output.status.success(),
        "docs metrics should succeed: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    let stdout = String::from_utf8(output.stdout).expect("stdout should be utf-8");
    assert!(stdout.contains("special docs metrics"));
    assert!(stdout.contains("relationship inventory"));
    assert!(stdout.contains("specs: 2 reference(s)"));
    assert!(stdout.contains("2 referenced target(s)"));
    assert!(stdout.contains("1 internal-only"));
    assert!(stdout.contains("modules: 1 reference(s)"));
    assert!(stdout.contains("patterns: 1 reference(s)"));
    assert!(stdout.contains("groups: 0 reference(s)"));
    assert!(!stdout.contains("undocumented groups"));

    let verbose = run_special(&root, &["docs", "--metrics", "--verbose"]);
    assert!(verbose.status.success());
    let verbose_stdout = String::from_utf8(verbose.stdout).expect("stdout should be utf-8");
    assert!(verbose_stdout.contains("sources: 1 link"));
    assert!(verbose_stdout.contains("1 @documents"));
}

#[test]
// @verifies SPECIAL.DOCS_COMMAND.METRICS.COVERAGE
fn docs_metrics_text_surfaces_public_target_coverage() {
    let root = temp_repo_dir("special-cli-docs-target-coverage");
    write_docs_metrics_fixture(&root);

    let output = run_special(&root, &["docs", "--metrics"]);
    assert!(output.status.success());

    let stdout = String::from_utf8(output.stdout).expect("stdout should be utf-8");
    assert!(stdout.contains("public target coverage"));
    assert!(stdout.contains("undocumented current specs: 1"));
    assert!(stdout.contains("internal-only documented targets: 1"));

    fs::remove_dir_all(&root).expect("temp repo should be cleaned up");
}

#[test]
// @verifies SPECIAL.DOCS_COMMAND.METRICS.COVERAGE
fn docs_metrics_json_includes_public_target_coverage() {
    let root = temp_repo_dir("special-cli-docs-target-coverage-json");
    write_docs_metrics_fixture(&root);

    let output = run_special(&root, &["docs", "--metrics", "--json"]);
    assert!(output.status.success());

    let json: Value = serde_json::from_slice(&output.stdout).expect("stdout should be json");
    assert_eq!(
        json["metrics"]["coverage"]["undocumented_current_specs"],
        Value::from(1)
    );
    assert_eq!(
        json["metrics"]["coverage"]["undocumented_current_spec_ids"],
        Value::Array(vec![Value::from("EXPORT.INTERNAL")])
    );
    assert_eq!(
        json["metrics"]["coverage"]["internal_only_documented_targets"],
        Value::from(1)
    );

    fs::remove_dir_all(&root).expect("temp repo should be cleaned up");
}

#[test]
// @verifies SPECIAL.DOCS_COMMAND.METRICS.COVERAGE.DOCS_SOURCE_DECLARATIONS
fn docs_coverage_excludes_docs_source_architecture_targets() {
    let root = temp_repo_dir("special-cli-docs-source-target-coverage");
    write_docs_source_target_coverage_fixture(&root);

    let output = run_special(&root, &["docs", "--metrics", "--json"]);
    assert!(output.status.success());

    let json: Value = serde_json::from_slice(&output.stdout).expect("stdout should be json");
    assert_eq!(
        json["metrics"]["coverage"]["undocumented_module_ids"],
        Value::Array(vec![Value::from("APP.PARSER")])
    );
    assert_eq!(
        json["metrics"]["coverage"]["undocumented_pattern_ids"],
        Value::Array(vec![Value::from("CACHE.SINGLE_FLIGHT_FILL")])
    );

    fs::remove_dir_all(&root).expect("temp repo should be cleaned up");
}

#[test]
// @verifies SPECIAL.DOCS_COMMAND.METRICS
fn docs_metrics_json_includes_coverage_and_omits_target_audit() {
    let root = temp_repo_dir("special-cli-docs-target-audit");
    write_docs_metrics_fixture(&root);

    let output = run_special(&root, &["docs", "--metrics", "--json"]);

    assert!(
        output.status.success(),
        "docs metrics json should succeed: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    let json: Value = serde_json::from_slice(&output.stdout).expect("stdout should be json");
    assert!(json["metrics"]["coverage"].is_object());
    assert!(json["metrics"]["target_audit"].is_null());
}

#[test]
// @verifies SPECIAL.DOCS_COMMAND.METRICS.INTERCONNECTIVITY
fn docs_metrics_reports_generated_docs_graph() {
    let root = temp_repo_dir("special-cli-docs-metrics-graph");
    write_docs_metrics_fixture(&root);

    let output = run_special(&root, &["docs", "--metrics"]);

    assert!(
        output.status.success(),
        "docs metrics should succeed: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    let stdout = String::from_utf8(output.stdout).expect("stdout should be utf-8");
    assert!(stdout.contains("generated pages: 2"));
    assert!(stdout.contains("local doc links: 1"));
    assert!(stdout.contains("broken local doc links: 0"));
    assert!(stdout.contains("orphan pages: 0"));
    assert!(stdout.contains("reachable from entrypoints: 2/2 page(s), 1 entrypoint(s)"));
}

#[test]
// @verifies SPECIAL.DOCS_COMMAND.METRICS
fn docs_metrics_json_exposes_structured_counts() {
    let root = temp_repo_dir("special-cli-docs-metrics-json");
    write_docs_metrics_fixture(&root);

    let output = run_special(&root, &["docs", "--metrics", "--json"]);

    assert!(
        output.status.success(),
        "docs metrics json should succeed: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    let json: Value = serde_json::from_slice(&output.stdout).expect("stdout should be json");
    assert_eq!(json["metrics"]["generated_pages"], 2);
    assert_eq!(json["metrics"]["reachable_pages_from_entrypoints"], 2);
    let specs = json["metrics"]["target_kinds"]
        .as_array()
        .expect("target kinds should be an array")
        .iter()
        .find(|kind| kind["kind"] == "spec")
        .expect("spec metrics should exist");
    assert_eq!(specs["references"], 2);
    assert_eq!(specs["documented_targets"], 2);
    assert_eq!(specs["generated"], 1);
    assert_eq!(specs["internal_only"], 1);
    assert!(json["metrics"]["coverage"].is_object());
    assert!(json["metrics"]["target_audit"].is_null());
}

#[test]
// @verifies SPECIAL.CONFIG.SPECIAL_TOML.DOCS_ENTRYPOINTS
fn docs_metrics_uses_configured_entrypoints_for_reachability() {
    let root = temp_repo_dir("special-cli-docs-metrics-entrypoints");
    write_docs_metrics_fixture(&root);

    let output = run_special(&root, &["docs", "--metrics", "--json"]);

    assert!(
        output.status.success(),
        "docs metrics json should succeed: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    let json: Value = serde_json::from_slice(&output.stdout).expect("stdout should be json");
    assert_eq!(json["metrics"]["entrypoint_pages"], 1);
    assert_eq!(json["metrics"]["reachable_pages_from_entrypoints"], 2);
}

#[test]
// @verifies SPECIAL.DOCS_COMMAND.OUTPUT.SAFETY
fn docs_configured_outputs_reject_duplicate_output_paths_before_writing() {
    let root = temp_repo_dir("special-cli-docs-duplicate-config-output");
    write_docs_fixture(&root);
    fs::write(
        root.join("special.toml"),
        "version = \"1\"\nroot = \".\"\n\n[[docs.outputs]]\nsource = \"docs/a.md\"\noutput = \"README.md\"\n\n[[docs.outputs]]\nsource = \"docs/b.md\"\noutput = \"README.md\"\n",
    )
    .expect("special.toml should be written");
    fs::create_dir_all(root.join("docs")).expect("docs dir should be created");
    fs::write(
        root.join("docs/a.md"),
        "First [CSV](documents://spec/EXPORT.CSV.HEADERS).\n",
    )
    .expect("first docs source should be written");
    fs::write(
        root.join("docs/b.md"),
        "Second [CSV](documents://spec/EXPORT.CSV.HEADERS).\n",
    )
    .expect("second docs source should be written");

    let output = run_special(&root, &["docs", "build"]);

    assert!(!output.status.success());
    assert!(
        !root.join("README.md").exists(),
        "configured output writing should not partially write duplicate output plans"
    );
    let stderr = String::from_utf8(output.stderr).expect("stderr should be utf-8");
    assert!(stderr.contains("maps multiple inputs"));
}

#[test]
// @verifies SPECIAL.DOCS_COMMAND.OUTPUT.SAFETY
fn docs_directory_output_rejects_expanded_files_inside_input_tree() {
    let root = temp_repo_dir("special-cli-docs-output-inside-input");
    write_docs_fixture(&root);
    fs::create_dir_all(root.join("docs/src/src")).expect("nested source dir should be created");
    fs::write(root.join("docs/src/src/guide.md"), "plain\n")
        .expect("nested docs source should be written");

    let output = run_special(&root, &["docs", "build", "docs/src", "docs"]);

    assert!(!output.status.success());
    assert!(
        !root.join("docs/src/guide.md").exists(),
        "docs output should not write generated content back into the source tree"
    );
    let stderr = String::from_utf8(output.stderr).expect("stderr should be utf-8");
    assert!(stderr.contains("docs output file must not be inside the input directory"));
}

#[test]
// @verifies SPECIAL.DOCS_COMMAND
fn docs_validate_prints_relationship_dump() {
    let root = temp_repo_dir("special-cli-docs-dump");
    write_docs_fixture(&root);
    fs::write(
        root.join("docs.md"),
        "[CSV exports include headers](documents://spec/EXPORT.CSV.HEADERS).\n",
    )
    .expect("docs markdown should be written");

    let output = run_special(&root, &["docs"]);

    assert!(
        output.status.success(),
        "docs validation should succeed: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    let stdout = String::from_utf8(output.stdout).expect("stdout should be utf-8");
    assert!(stdout.contains("Docs"));
    assert!(stdout.contains("spec EXPORT.CSV.HEADERS"));
    assert!(stdout.contains("docs.md:1 link: CSV exports include headers"));
}

#[test]
// @verifies SPECIAL.DOCS_COMMAND
fn docs_validate_ignores_inline_code_examples() {
    let root = temp_repo_dir("special-cli-docs-inline-code");
    write_docs_fixture(&root);
    fs::write(
        root.join("docs.md"),
        concat!(
            "`@documents spec EXPORT.MISSING`\n",
            "`[Missing](documents://spec/EXPORT.MISSING)`\n",
        ),
    )
    .expect("docs markdown should be written");
    fs::write(
        root.join("examples.rs"),
        concat!(
            "/**\n",
            "```text\n",
            "@documents spec EXPORT.MISSING\n",
            "```\n",
            "*/\n",
            "fn example() {}\n",
        ),
    )
    .expect("source example should be written");

    let output = run_special(&root, &["docs"]);

    assert!(
        output.status.success(),
        "inline code examples should not become docs evidence: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    let stdout = String::from_utf8(output.stdout).expect("stdout should be utf-8");
    assert!(!stdout.contains("EXPORT.MISSING"));
}

#[test]
// @verifies SPECIAL.DOCS_COMMAND.TARGET
fn docs_target_scopes_validation_without_writing_files() {
    let root = temp_repo_dir("special-cli-docs-target");
    write_docs_fixture(&root);
    fs::write(
        root.join("good.md"),
        "[CSV exports include headers](documents://spec/EXPORT.CSV.HEADERS).\n",
    )
    .expect("good docs markdown should be written");
    fs::write(
        root.join("bad.md"),
        "[Missing behavior](documents://spec/EXPORT.MISSING).\n",
    )
    .expect("bad docs markdown should be written");

    let output = run_special(&root, &["docs", "--target", "good.md"]);

    assert!(
        output.status.success(),
        "scoped docs validation should ignore off-target docs errors: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    assert!(
        !root.join("docs/dist").exists(),
        "targeted validation without docs build should not write rendered files"
    );
    let stdout = String::from_utf8(output.stdout).expect("stdout should be utf-8");
    assert!(stdout.contains("good.md:1 link: CSV exports include headers"));
    assert!(!stdout.contains("EXPORT.MISSING"));
}

#[test]
// @verifies SPECIAL.DOCS_COMMAND.OUTPUT
fn docs_help_names_output_as_path() {
    let root = temp_repo_dir("special-cli-docs-help-output-path");
    write_docs_fixture(&root);

    let output = run_special(&root, &["docs", "build", "--help"]);

    assert!(output.status.success());
    let stdout = String::from_utf8(output.stdout).expect("stdout should be utf-8");
    assert!(stdout.contains("--output"));
    assert!(stdout.contains("PATH"));
    assert!(!stdout.contains("--output <OUTPUT>"));
    assert!(stdout.contains("SOURCE"));
}

#[test]
// @verifies SPECIAL.DOCS_COMMAND
fn docs_validate_reports_unknown_targets() {
    let root = temp_repo_dir("special-cli-docs-unknown");
    write_docs_fixture(&root);
    fs::write(
        root.join("docs.md"),
        "[Missing behavior](documents://spec/EXPORT.MISSING).\n",
    )
    .expect("docs markdown should be written");

    let output = run_special(&root, &["docs"]);

    assert!(!output.status.success());
    let stdout = String::from_utf8(output.stdout).expect("stdout should be utf-8");
    assert!(stdout.contains("unknown documentation target spec `EXPORT.MISSING`"));
}

#[test]
// @verifies SPECIAL.DOCS_COMMAND.PATH_SCOPE_SYNTAX
fn docs_rejects_hidden_positional_path_scope() {
    let root = temp_repo_dir("special-cli-docs-positional");
    write_docs_fixture(&root);
    fs::write(
        root.join("docs.md"),
        "[CSV exports include headers](documents://spec/EXPORT.CSV.HEADERS).\n",
    )
    .expect("docs markdown should be written");

    let output = run_special(&root, &["docs", "docs.md"]);

    assert!(!output.status.success());
    let stderr = String::from_utf8(output.stderr).expect("stderr should be utf-8");
    assert!(stderr.contains("docs path scopes must use --target PATH"));
}

#[test]
// @verifies SPECIAL.DOCS.LINKS
fn docs_output_reports_malformed_document_links() {
    let root = temp_repo_dir("special-cli-docs-malformed");
    write_docs_fixture(&root);
    fs::write(root.join("docs.md"), "[Broken](documents://spec).\n")
        .expect("docs markdown should be written");

    let output = run_special(&root, &["docs", "build", "docs.md", "public.md"]);

    assert!(!output.status.success());
    assert!(
        !root.join("public.md").exists(),
        "output writing should not write output when docs links are malformed"
    );
    let stdout = String::from_utf8(output.stdout).expect("stdout should be utf-8");
    assert!(stdout.contains("malformed documents URI `documents://spec`"));
}

#[test]
// @verifies SPECIAL.DOCS.LINKS
fn docs_output_rejects_reserved_special_links() {
    let root = temp_repo_dir("special-cli-docs-reserved-special-uri");
    write_docs_fixture(&root);
    fs::write(
        root.join("docs.md"),
        "[CSV exports include headers](special://spec/EXPORT.CSV.HEADERS).\n",
    )
    .expect("docs markdown should be written");

    let output = run_special(&root, &["docs", "build", "docs.md", "public.md"]);

    assert!(!output.status.success());
    assert!(
        !root.join("public.md").exists(),
        "output writing should not write output when docs links use the reserved URI scheme"
    );
    let stdout = String::from_utf8(output.stdout).expect("stdout should be utf-8");
    assert!(stdout.contains("`special://` is reserved"));
    assert!(stdout.contains("documents://kind/ID"));
}

#[test]
// @verifies SPECIAL.DOCS_COMMAND.OUTPUT.SAFETY
fn docs_output_refuses_to_overwrite_docs_evidence() {
    let root = temp_repo_dir("special-cli-docs-overwrite-evidence");
    write_docs_fixture(&root);
    fs::write(
        root.join("source.md"),
        "[CSV exports include headers](documents://spec/EXPORT.CSV.HEADERS).\n",
    )
    .expect("source docs markdown should be written");
    fs::write(
        root.join("public.md"),
        "[Authored evidence](documents://spec/EXPORT.CSV.HEADERS).\n",
    )
    .expect("existing evidence-bearing output should be written");

    let output = run_special(&root, &["docs", "build", "source.md", "public.md"]);

    assert!(!output.status.success());
    let stderr = String::from_utf8(output.stderr).expect("stderr should be utf-8");
    assert!(stderr.contains("refusing to overwrite docs evidence"));
}

#[cfg(unix)]
#[test]
// @verifies SPECIAL.DOCS_COMMAND.OUTPUT.SAFETY
fn docs_output_refuses_to_overwrite_symlinked_docs_evidence() {
    let root = temp_repo_dir("special-cli-docs-overwrite-symlinked-evidence");
    write_docs_fixture(&root);
    fs::write(
        root.join("source.md"),
        "[CSV exports include headers](documents://spec/EXPORT.CSV.HEADERS).\n",
    )
    .expect("source docs markdown should be written");
    fs::write(
        root.join("canonical.md"),
        "[Authored evidence](documents://spec/EXPORT.CSV.HEADERS).\n",
    )
    .expect("existing evidence-bearing target should be written");
    std::os::unix::fs::symlink(root.join("canonical.md"), root.join("public.md"))
        .expect("output symlink should be created");

    let output = run_special(&root, &["docs", "build", "source.md", "public.md"]);

    assert!(!output.status.success());
    let stderr = String::from_utf8(output.stderr).expect("stderr should be utf-8");
    assert!(stderr.contains("refusing to overwrite docs evidence"));
}

#[test]
// @verifies SPECIAL.DOCS_COMMAND.OUTPUT.SAFETY
fn docs_output_refuses_to_overwrite_markdown_authoring_lines() {
    let root = temp_repo_dir("special-cli-docs-overwrite-authoring-lines");
    write_docs_fixture(&root);
    fs::write(
        root.join("source.md"),
        "[CSV exports include headers](documents://spec/EXPORT.CSV.HEADERS).\n",
    )
    .expect("source docs markdown should be written");
    fs::write(root.join("public.md"), "@implements APP.PARSER\n")
        .expect("existing authoring output should be written");

    let output = run_special(&root, &["docs", "build", "source.md", "public.md"]);

    assert!(!output.status.success());
    let stderr = String::from_utf8(output.stderr).expect("stderr should be utf-8");
    assert!(stderr.contains("refusing to overwrite docs evidence"));
}

#[test]
// @verifies SPECIAL.DOCS_COMMAND.OUTPUT.SAFETY
fn docs_output_allows_overwriting_fenced_docs_examples() {
    let root = temp_repo_dir("special-cli-docs-overwrite-fenced-example");
    write_docs_fixture(&root);
    fs::write(
        root.join("source.md"),
        "[CSV exports include headers](documents://spec/EXPORT.CSV.HEADERS).\n",
    )
    .expect("source docs markdown should be written");
    fs::write(
        root.join("public.md"),
        "```markdown\n@filedocuments spec APP.CONFIG\n[Config](documents://spec/APP.CONFIG)\n```\n\n`@documents` lines and `documents://spec/APP.CONFIG` links are examples here.\n",
    )
    .expect("existing docs example should be written");

    let output = run_special(&root, &["docs", "build", "source.md", "public.md"]);

    assert!(
        output.status.success(),
        "fenced examples should not block overwrite: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    let rendered = fs::read_to_string(root.join("public.md")).expect("output should be written");
    assert_eq!(rendered, "CSV exports include headers.\n");
}

fn write_docs_fixture(root: &std::path::Path) {
    fs::write(root.join("special.toml"), "version = \"1\"\nroot = \".\"\n")
        .expect("special.toml should be written");
    fs::write(
        root.join("specs.md"),
        "### @group EXPORT\nExports.\n\n### @spec EXPORT.CSV.HEADERS\nCSV headers.\n",
    )
    .expect("specs should be written");
    fs::write(
        root.join("architecture.md"),
        "### @area APP\nApplication.\n\n### @module APP.PARSER\nParser.\n",
    )
    .expect("architecture should be written");
    fs::write(
        root.join("patterns.md"),
        "### @pattern CACHE.SINGLE_FLIGHT_FILL\nSingle-flight cache fill.\n",
    )
    .expect("patterns should be written");
    fs::write(
        root.join("src.rs"),
        "// @fileimplements APP.PARSER\n// @documents spec EXPORT.CSV.HEADERS\nfn parse() {}\n",
    )
    .expect("source should be written");
}

fn write_docs_metrics_fixture(root: &std::path::Path) {
    fs::write(
        root.join("special.toml"),
        concat!(
            "version = \"1\"\n",
            "root = \".\"\n",
            "[docs]\n",
            "entrypoints = [\"docs/README.md\"]\n",
            "\n",
            "[[docs.outputs]]\n",
            "source = \"docs/src\"\n",
            "output = \"docs\"\n",
        ),
    )
    .expect("special.toml should be written");
    fs::write(
        root.join("specs.md"),
        concat!(
            "### @group EXPORT\n",
            "Exports.\n\n",
            "### @spec EXPORT.CSV.HEADERS\n",
            "CSV headers.\n\n",
            "### @spec EXPORT.INTERNAL\n",
            "Internal export.\n",
        ),
    )
    .expect("specs should be written");
    fs::write(
        root.join("architecture.md"),
        concat!(
            "### @area APP\n",
            "App.\n\n",
            "### @module APP.PARSER\n",
            "Parser.\n",
        ),
    )
    .expect("architecture should be written");
    fs::write(
        root.join("patterns.md"),
        "### @pattern CACHE.SINGLE_FLIGHT_FILL\nCache fill.\n",
    )
    .expect("patterns should be written");
    fs::write(
        root.join("src.rs"),
        concat!(
            "// @fileimplements APP.PARSER\n",
            "// @documents spec EXPORT.INTERNAL\n",
            "fn parse() {}\n",
        ),
    )
    .expect("source should be written");
    fs::create_dir_all(root.join("docs/src")).expect("docs source dir should be created");
    fs::write(
        root.join("docs/src/README.md"),
        concat!(
            "[Guide](guide.md)\n",
            "[CSV](documents://spec/EXPORT.CSV.HEADERS)\n",
        ),
    )
    .expect("docs index should be written");
    fs::write(
        root.join("docs/src/guide.md"),
        concat!(
            "[Parser](documents://module/APP.PARSER)\n",
            "[Cache](documents://pattern/CACHE.SINGLE_FLIGHT_FILL)\n",
        ),
    )
    .expect("docs guide should be written");
}

fn write_docs_source_target_coverage_fixture(root: &std::path::Path) {
    fs::write(
        root.join("special.toml"),
        concat!(
            "version = \"1\"\n",
            "root = \".\"\n",
            "[docs]\n",
            "entrypoints = [\"README.md\"]\n",
            "\n",
            "[[docs.outputs]]\n",
            "source = \"docs/src/README.md\"\n",
            "output = \"README.md\"\n",
        ),
    )
    .expect("special.toml should be written");
    fs::write(
        root.join("specs.md"),
        concat!(
            "### @group EXPORT\n",
            "Exports.\n\n",
            "### @spec EXPORT.CSV.HEADERS\n",
            "CSV headers.\n\n",
            "### @spec EXPORT.INTERNAL\n",
            "Internal export.\n",
        ),
    )
    .expect("specs should be written");
    fs::write(
        root.join("architecture.md"),
        "### @area APP\nApp.\n\n### @module APP.PARSER\nParser.\n",
    )
    .expect("architecture should be written");
    fs::write(
        root.join("src.rs"),
        "// @fileimplements APP.PARSER\npub fn parse() {}\n",
    )
    .expect("source should be written");
    fs::write(
        root.join("patterns.md"),
        "### @pattern CACHE.SINGLE_FLIGHT_FILL\nCache fill.\n",
    )
    .expect("patterns should be written");
    fs::write(
        root.join("docs-architecture.md"),
        concat!(
            "### @area DOCS\n",
            "Docs architecture.\n\n",
            "### @module DOCS.README\n",
            "README docs section.\n\n",
            "### @pattern DOCS.TRACEABLE_EXAMPLE\n",
            "Traceable docs example.\n",
        ),
    )
    .expect("docs architecture should be written");
    fs::create_dir_all(root.join("docs/src")).expect("docs source dir should be created");
    fs::write(
        root.join("docs/src/README.md"),
        concat!(
            "@implements DOCS.README\n",
            "@applies DOCS.TRACEABLE_EXAMPLE\n",
            "## README\n",
            "[CSV](documents://spec/EXPORT.CSV.HEADERS)\n",
        ),
    )
    .expect("docs source should be written");
}
