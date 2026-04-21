/**
@module SPECIAL.TESTS.SUPPORT.CLI.SPECS
Spec-oriented fixture writers and config helpers for CLI integration tests.
*/
// @fileimplements SPECIAL.TESTS.SUPPORT.CLI.SPECS
use std::fs;
use std::path::{Path, PathBuf};

pub fn write_current_and_planned_fixture(root: &Path) {
    fs::write(root.join("special.toml"), "version = \"1\"\nroot = \".\"\n")
        .expect("special.toml should be written");
    fs::write(
        root.join("specs.rs"),
        include_str!("../../fixtures/cli/live_and_planned/specs.txt"),
    )
    .expect("spec fixture should be written");
    fs::write(
        root.join("checks.rs"),
        include_str!("../../fixtures/cli/live_and_planned/checks.txt"),
    )
    .expect("verify fixture should be written");
}

pub fn write_planned_release_fixture(root: &Path) {
    fs::write(root.join("special.toml"), "version = \"1\"\nroot = \".\"\n")
        .expect("special.toml should be written");
    fs::write(
        root.join("specs.rs"),
        include_str!("../../fixtures/cli/planned_release/specs.txt"),
    )
    .expect("planned release fixture should be written");
}

pub fn write_deprecated_release_fixture(root: &Path) {
    fs::write(root.join("special.toml"), "version = \"1\"\nroot = \".\"\n")
        .expect("special.toml should be written");
    fs::write(
        root.join("specs.rs"),
        include_str!("../../fixtures/cli/deprecated_release/specs.txt"),
    )
    .expect("deprecated release fixture should be written");
}

pub fn write_file_verify_fixture(root: &Path) {
    fs::write(root.join("special.toml"), "version = \"1\"\nroot = \".\"\n")
        .expect("special.toml should be written");
    fs::write(
        root.join("specs.rs"),
        "/**\n@spec DEMO\nDemo root claim.\n*/\n",
    )
    .expect("file verify spec fixture should be written");
    fs::write(
        root.join("checks.rs"),
        "// @fileverifies DEMO\nfn verifies_demo_root() {}\n",
    )
    .expect("file verify fixture should be written");
}

pub fn write_file_attest_fixture(root: &Path) {
    fs::write(root.join("special.toml"), "version = \"1\"\nroot = \".\"\n")
        .expect("special.toml should be written");
    fs::write(
        root.join("review.md"),
        "### @spec DEMO\nDemo root claim.\n\n### @fileattests DEMO\nartifact: docs/review.md\nowner: qa\nlast_reviewed: 2026-04-16\n\n# Review Notes\n\nThe file-level review body stays attached to the whole markdown artifact.\n",
    )
    .expect("file attest fixture should be written");
}

pub fn write_lint_error_fixture(root: &Path) {
    fs::write(
        root.join("specs.rs"),
        include_str!("../../fixtures/cli/lint_error/specs.txt"),
    )
    .expect("spec fixture should be written");
    fs::write(
        root.join("checks.rs"),
        include_str!("../../fixtures/cli/lint_error/checks.txt"),
    )
    .expect("verify fixture should be written");
}

pub fn write_orphan_verify_fixture(root: &Path) {
    fs::write(
        root.join("specs.rs"),
        include_str!("../../fixtures/cli/orphan_verify/specs.txt"),
    )
    .expect("orphan verify spec fixture should be written");
    fs::write(
        root.join("checks.rs"),
        include_str!("../../fixtures/cli/orphan_verify/checks.txt"),
    )
    .expect("orphan verify fixture should be written");
}

pub fn write_special_toml_root_fixture(root: &Path) {
    let configured_root = root.join("workspace");
    fs::create_dir_all(&configured_root).expect("configured root should be created");
    fs::write(
        root.join("special.toml"),
        "version = \"1\"\nroot = \"workspace\"\n",
    )
    .expect("special.toml should be written");

    fs::write(
        configured_root.join("specs.rs"),
        "/**\n@spec DEMO\nConfig-root spec.\n*/\n",
    )
    .expect("config-root spec fixture should be written");
    fs::write(
        configured_root.join("checks.rs"),
        "// @verifies DEMO\nfn verifies_demo() {}\n",
    )
    .expect("config-root verify fixture should be written");

    fs::write(
        root.join("outside.rs"),
        "/**\n@spec OUTSIDE\nThis spec should stay outside the configured root.\n*/\n",
    )
    .expect("outside spec fixture should be written");
}

pub fn write_special_toml_dot_root_fixture(root: &Path) -> PathBuf {
    let nested = root.join("nested/deeper");
    fs::create_dir_all(&nested).expect("nested dir should be created");
    fs::write(root.join("special.toml"), "version = \"1\"\nroot = \".\"\n")
        .expect("special.toml should be written");

    fs::write(
        root.join("specs.rs"),
        "/**\n@spec DEMO\nConfig-root spec.\n*/\n",
    )
    .expect("config-root spec fixture should be written");
    fs::write(
        root.join("checks.rs"),
        "// @verifies DEMO\nfn verifies_demo() {}\n",
    )
    .expect("config-root verify fixture should be written");

    nested
}

pub fn write_missing_version_fixture(root: &Path) {
    fs::write(root.join("special.toml"), "root = \".\"\n").expect("special.toml should be written");
    fs::write(
        root.join("specs.rs"),
        include_str!("../../fixtures/cli/missing_version/specs.txt"),
    )
    .expect("missing version fixture should be written");
}

pub fn write_non_adjacent_planned_v1_fixture(root: &Path) {
    fs::write(root.join("special.toml"), "version = \"1\"\nroot = \".\"\n")
        .expect("special.toml should be written");
    fs::write(
        root.join("specs.rs"),
        include_str!("../../fixtures/cli/non_adjacent_planned_v1/specs.txt"),
    )
    .expect("planned scope fixture should be written");
}

pub fn write_unverified_current_fixture(root: &Path) {
    fs::write(
        root.join("specs.rs"),
        include_str!("../../fixtures/cli/unsupported_live/specs.txt"),
    )
    .expect("unsupported-current spec fixture should be written");
    fs::write(
        root.join("checks.rs"),
        include_str!("../../fixtures/cli/unsupported_live/checks.txt"),
    )
    .expect("unsupported-live verify fixture should be written");
}

pub fn write_supported_fixture_without_config(root: &Path) {
    fs::write(
        root.join("specs.rs"),
        "/**\n@spec DEMO\nSupported demo spec.\n*/\n",
    )
    .expect("supported spec fixture should be written");
    fs::write(
        root.join("checks.rs"),
        "// @verifies DEMO\nfn verifies_demo() {}\n",
    )
    .expect("supported verify fixture should be written");
}
