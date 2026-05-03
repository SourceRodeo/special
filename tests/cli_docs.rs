/**
@module SPECIAL.TESTS.CLI_DOCS
CLI integration tests for documentation relationship validation and materialized public docs output.
*/
// @fileimplements SPECIAL.TESTS.CLI_DOCS
#[path = "support/cli.rs"]
mod support;

use std::fs;

use support::{run_special, temp_repo_dir};

#[test]
// @verifies SPECIAL.DOCS_COMMAND.MATERIALIZE
fn docs_materialize_rewrites_special_links_and_removes_document_lines() {
    let root = temp_repo_dir("special-cli-docs-materialize");
    write_docs_fixture(&root);
    fs::create_dir_all(root.join("docs/src/nested")).expect("nested docs dir should be created");
    fs::write(
        root.join("docs/src/README.md"),
        concat!(
            "# Guide\n\n",
            "@documents spec EXPORT.CSV.HEADERS\n",
            "@filedocuments module APP.PARSER\n",
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
        "docs materialize should succeed: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    let rendered =
        fs::read_to_string(root.join("docs/dist/README.md")).expect("rendered docs should exist");
    assert!(rendered.contains("CSV exports include headers."));
    assert!(rendered.contains("Parser ownership is documented."));
    assert!(rendered.contains("Cache fill is intentional."));
    assert!(!rendered.contains("special://"));
    assert!(!rendered.contains("@documents"));
    assert!(!rendered.contains("@filedocuments"));
    assert_eq!(
        fs::read_to_string(root.join("docs/dist/nested/asset.txt"))
            .expect("plain asset should be copied"),
        "plain\n"
    );
}

#[test]
// @verifies SPECIAL.DOCS_COMMAND.MATERIALIZE.CONFIG
fn docs_materialize_uses_configured_source_and_output_paths() {
    let root = temp_repo_dir("special-cli-docs-materialize-config");
    write_docs_fixture(&root);
    fs::write(
        root.join("special.toml"),
        "version = \"1\"\nroot = \".\"\n\n[docs]\nsource = \"docs/src\"\noutput = \"docs/dist\"\n",
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
        "configured docs materialization should succeed: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    let rendered =
        fs::read_to_string(root.join("docs/dist/README.md")).expect("rendered docs should exist");
    assert!(rendered.contains("CSV exports include headers."));
    assert_eq!(
        fs::read_to_string(root.join("docs/dist/nested/asset.txt"))
            .expect("plain asset should preserve its relative tree path"),
        "plain\n"
    );
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
        "targeted validation without --output should not write materialized files"
    );
    let stdout = String::from_utf8(output.stdout).expect("stdout should be utf-8");
    assert!(stdout.contains("good.md:1 link: CSV exports include headers"));
    assert!(!stdout.contains("EXPORT.MISSING"));
}

#[test]
// @verifies SPECIAL.DOCS_COMMAND.MATERIALIZE
fn docs_help_names_output_as_path() {
    let root = temp_repo_dir("special-cli-docs-help-output-path");
    write_docs_fixture(&root);

    let output = run_special(&root, &["docs", "--help"]);

    assert!(output.status.success());
    let stdout = String::from_utf8(output.stdout).expect("stdout should be utf-8");
    assert!(stdout.contains("--output"));
    assert!(stdout.contains("PATH"));
    assert!(!stdout.contains("--output <OUTPUT>"));
    assert!(!stdout.contains("--materialize"));
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
fn docs_materialize_reports_malformed_special_links() {
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
        "materialization should not write output when docs links are malformed"
    );
    let stdout = String::from_utf8(output.stdout).expect("stdout should be utf-8");
    assert!(stdout.contains("malformed Special docs URI `special://spec`"));
}

#[test]
// @verifies SPECIAL.DOCS_COMMAND.MATERIALIZE.SAFETY
fn docs_materialize_refuses_to_overwrite_docs_evidence() {
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
