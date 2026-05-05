/**
@module SPECIAL.TESTS.CLI_DOCS
CLI integration tests for documentation relationship validation and public docs output.
*/
// @fileimplements SPECIAL.TESTS.CLI_DOCS
#[path = "support/cli.rs"]
mod support;

use std::fs;

use support::{run_special, temp_repo_dir};

#[test]
// @verifies SPECIAL.DOCS_COMMAND.OUTPUT.DIRECTORY
fn docs_output_rewrites_special_links_and_removes_document_lines() {
    let root = temp_repo_dir("special-cli-docs-output");
    write_docs_fixture(&root);
    fs::create_dir_all(root.join("docs/src/nested")).expect("nested docs dir should be created");
    fs::write(
        root.join("docs/src/README.md"),
        concat!(
            "# Guide\n\n",
            "@documents spec EXPORT.CSV.HEADERS\n",
            "[CSV exports include headers](special://spec/EXPORT.CSV.HEADERS).\n",
            "[Parser ownership](special://module/APP.PARSER) is documented.\n",
            "[Cache fill](special://pattern/CACHE.SINGLE_FLIGHT_FILL) is intentional.\n",
        ),
    )
    .expect("docs source should be written");
    fs::write(root.join("docs/src/nested/asset.txt"), "plain\n")
        .expect("plain asset should be written");

    let output = run_special(
        &root,
        &["docs", "--target", "docs/src", "--output", "docs/dist"],
    );

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
    assert!(!rendered.contains("special://"));
    assert!(!rendered.contains("@documents"));
    assert_eq!(
        fs::read_to_string(root.join("docs/dist/nested/asset.txt"))
            .expect("plain asset should be copied"),
        "plain\n"
    );
}

#[test]
// @verifies SPECIAL.DOCS.LINKS.OUTPUT
fn docs_output_rewrites_special_links_to_plain_text() {
    let root = temp_repo_dir("special-cli-docs-link-output");
    write_docs_fixture(&root);
    fs::write(
        root.join("source.md"),
        "[CSV exports include headers](special://spec/EXPORT.CSV.HEADERS).\n",
    )
    .expect("source docs markdown should be written");

    let output = run_special(
        &root,
        &["docs", "--target", "source.md", "--output", "public.md"],
    );

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
            "[Export group](special://group/EXPORT).\n",
            "[CSV spec](special://spec/EXPORT.CSV.HEADERS).\n",
            "[App area](special://area/APP).\n",
            "[Parser module](special://module/APP.PARSER).\n",
            "[Cache pattern](special://pattern/CACHE.SINGLE_FLIGHT_FILL).\n",
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
        "[CSV exports include headers](special://spec/EXPORT.CSV.HEADERS).\n",
    )
    .expect("docs source should be written");

    let output = run_special(&root, &["docs", "--output"]);

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
        "[CSV exports include headers](special://spec/EXPORT.CSV.HEADERS).\n",
    )
    .expect("docs source should be written");
    fs::write(root.join("docs/src/nested/asset.txt"), "plain\n")
        .expect("plain asset should be written");

    let output = run_special(&root, &["docs", "--output"]);

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
        "First [CSV](special://spec/EXPORT.CSV.HEADERS).\n",
    )
    .expect("first docs source should be written");
    fs::write(
        root.join("docs/b.md"),
        "Second [CSV](special://spec/EXPORT.CSV.HEADERS).\n",
    )
    .expect("second docs source should be written");

    let output = run_special(&root, &["docs", "--output"]);

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

    let output = run_special(&root, &["docs", "--target", "docs/src", "--output", "docs"]);

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
        "[CSV exports include headers](special://spec/EXPORT.CSV.HEADERS).\n",
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
            "`[Missing](special://spec/EXPORT.MISSING)`\n",
        ),
    )
    .expect("docs markdown should be written");

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
        "[CSV exports include headers](special://spec/EXPORT.CSV.HEADERS).\n",
    )
    .expect("good docs markdown should be written");
    fs::write(
        root.join("bad.md"),
        "[Missing behavior](special://spec/EXPORT.MISSING).\n",
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
        "targeted validation without --output should not write rendered files"
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

    let output = run_special(&root, &["docs", "--help"]);

    assert!(output.status.success());
    let stdout = String::from_utf8(output.stdout).expect("stdout should be utf-8");
    assert!(stdout.contains("--output"));
    assert!(stdout.contains("PATH"));
    assert!(!stdout.contains("--output <OUTPUT>"));
    assert!(!stdout.contains("--render"));
}

#[test]
// @verifies SPECIAL.DOCS_COMMAND
fn docs_validate_reports_unknown_targets() {
    let root = temp_repo_dir("special-cli-docs-unknown");
    write_docs_fixture(&root);
    fs::write(
        root.join("docs.md"),
        "[Missing behavior](special://spec/EXPORT.MISSING).\n",
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
        "[CSV exports include headers](special://spec/EXPORT.CSV.HEADERS).\n",
    )
    .expect("docs markdown should be written");

    let output = run_special(&root, &["docs", "docs.md"]);

    assert!(!output.status.success());
    let stderr = String::from_utf8(output.stderr).expect("stderr should be utf-8");
    assert!(stderr.contains("docs path scopes must use --target PATH"));
}

#[test]
// @verifies SPECIAL.DOCS.LINKS
fn docs_output_reports_malformed_special_links() {
    let root = temp_repo_dir("special-cli-docs-malformed");
    write_docs_fixture(&root);
    fs::write(root.join("docs.md"), "[Broken](special://spec).\n")
        .expect("docs markdown should be written");

    let output = run_special(
        &root,
        &["docs", "--target", "docs.md", "--output", "public.md"],
    );

    assert!(!output.status.success());
    assert!(
        !root.join("public.md").exists(),
        "output writing should not write output when docs links are malformed"
    );
    let stdout = String::from_utf8(output.stdout).expect("stdout should be utf-8");
    assert!(stdout.contains("malformed Special docs URI `special://spec`"));
}

#[test]
// @verifies SPECIAL.DOCS_COMMAND.OUTPUT.SAFETY
fn docs_output_refuses_to_overwrite_docs_evidence() {
    let root = temp_repo_dir("special-cli-docs-overwrite-evidence");
    write_docs_fixture(&root);
    fs::write(
        root.join("source.md"),
        "[CSV exports include headers](special://spec/EXPORT.CSV.HEADERS).\n",
    )
    .expect("source docs markdown should be written");
    fs::write(
        root.join("public.md"),
        "[Authored evidence](special://spec/EXPORT.CSV.HEADERS).\n",
    )
    .expect("existing evidence-bearing output should be written");

    let output = run_special(
        &root,
        &["docs", "--target", "source.md", "--output", "public.md"],
    );

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
        "[CSV exports include headers](special://spec/EXPORT.CSV.HEADERS).\n",
    )
    .expect("source docs markdown should be written");
    fs::write(
        root.join("public.md"),
        "```markdown\n@filedocuments spec APP.CONFIG\n[Config](special://spec/APP.CONFIG)\n```\n\n`@documents` lines and `special://spec/APP.CONFIG` links are examples here.\n",
    )
    .expect("existing docs example should be written");

    let output = run_special(
        &root,
        &["docs", "--target", "source.md", "--output", "public.md"],
    );

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
        "### `@group EXPORT`\nExports.\n\n### `@spec EXPORT.CSV.HEADERS`\nCSV headers.\n",
    )
    .expect("specs should be written");
    fs::write(
        root.join("architecture.md"),
        "### `@area APP`\nApplication.\n\n### `@module APP.PARSER`\nParser.\n",
    )
    .expect("architecture should be written");
    fs::write(
        root.join("patterns.md"),
        "### `@pattern CACHE.SINGLE_FLIGHT_FILL`\nSingle-flight cache fill.\n",
    )
    .expect("patterns should be written");
    fs::write(
        root.join("src.rs"),
        "// @fileimplements APP.PARSER\n// @documents spec EXPORT.CSV.HEADERS\nfn parse() {}\n",
    )
    .expect("source should be written");
}
