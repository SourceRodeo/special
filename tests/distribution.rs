/**
@group SPECIAL.DISTRIBUTION.CRATES_IO
special crates.io package identity.

@spec SPECIAL.DISTRIBUTION.CRATES_IO.PACKAGE_NAME
special publishes the package as `special-cli`.

@spec SPECIAL.DISTRIBUTION.CRATES_IO.BINARY_NAME
special installs the `special` binary from the `special-cli` package.

@spec SPECIAL.DISTRIBUTION.CRATES_IO.RUST_VERSION
special's Cargo package declares the minimum supported Rust version required by its Rust edition and source syntax.

@group SPECIAL.DISTRIBUTION.GITHUB_RELEASES
special GitHub release distribution.

@group SPECIAL.DISTRIBUTION.SOURCE_DEPENDENCIES
special source dependencies used by GitHub release builds.

@group SPECIAL.DISTRIBUTION.BUILD_SCRIPT
special Cargo build-script distribution behavior.

@group SPECIAL.DISTRIBUTION.HOMEBREW
special Homebrew distribution.

@group SPECIAL.DISTRIBUTION.CODEX_PLUGIN
special Codex plugin distribution.

@group SPECIAL.DISTRIBUTION.SKILL_DOCS
special generated skill documentation distribution.

@spec SPECIAL.DISTRIBUTION.GITHUB_RELEASES.REPOSITORY_URL
special release automation declares the `https://github.com/sourcerodeo/special` repository URL.

@spec SPECIAL.DISTRIBUTION.SOURCE_DEPENDENCIES.PARSER_CRATE
special consumes `parse-source-annotations` from the `SourceRodeo/crates` Git monorepo package instead of requiring crates.io publication or a local sibling checkout.

@spec SPECIAL.DISTRIBUTION.SOURCE_DEPENDENCIES.NO_LOCAL_CHECKOUT
special release automation relies on Cargo's Git dependency resolution for the parser crate and does not recreate a local `../crates/parse-source-annotations` checkout.

@spec SPECIAL.DISTRIBUTION.GITHUB_RELEASES.WORKFLOW
special keeps a GitHub Actions release workflow in `.github/workflows/release.yml`.

@spec SPECIAL.DISTRIBUTION.GITHUB_RELEASES.LEAN_KERNEL
special GitHub release automation embeds the standalone Lean traceability kernel in host-native local artifact builds so those released `special` binaries can use the Lean kernel without requiring Lean at user runtime; cross-target artifact builds do not embed a host-built Lean executable.

@spec SPECIAL.DISTRIBUTION.BUILD_SCRIPT.LANGUAGE_PACK_REGISTRY
special's Cargo build script generates a compile-time built-in language-pack registry from top-level shipped language-pack entry files.

@spec SPECIAL.DISTRIBUTION.BUILD_SCRIPT.LANGUAGE_PACK_REGISTRY.TOP_LEVEL_ENTRIES_ONLY
special's Cargo build script treats top-level Rust source files in `src/language_packs` as built-in language-pack entries, skips only the Rust module root, and rejects top-level helper files with a clear layout error that tells maintainers to put shared helpers under a language-pack subdirectory.

@spec SPECIAL.DISTRIBUTION.GITHUB_RELEASES.PUBLISHED
special publishes GitHub Releases for versioned distribution.

@spec SPECIAL.DISTRIBUTION.GITHUB_RELEASES.ARCHIVES
special GitHub release automation publishes versioned release archives for supported target platforms.

@spec SPECIAL.DISTRIBUTION.GITHUB_RELEASES.CHECKSUMS
special GitHub release automation publishes checksums for its release archives.

@spec SPECIAL.DISTRIBUTION.HOMEBREW.FORMULA
special ships a Homebrew formula in sourcerodeo/homebrew-tap.

@spec SPECIAL.DISTRIBUTION.HOMEBREW.FORMULA.PATH
special keeps its Homebrew formula at `Formula/special.rb` in sourcerodeo/homebrew-tap.

@spec SPECIAL.DISTRIBUTION.HOMEBREW.FORMULA.PLATFORM_SELECTION
special selects its platform-specific Homebrew archive URL and checksum in explicit Homebrew platform branches.

@spec SPECIAL.DISTRIBUTION.HOMEBREW.FORMULA.TAP_METADATA_CHECK
special release validation reads the published Homebrew tap formula and verifies its version, platform archive branches, release asset digests, and checksum pairing against the GitHub release assets.

@spec SPECIAL.DISTRIBUTION.HOMEBREW.INSTALLS_SPECIAL
special installs the `special` binary from sourcerodeo/homebrew-tap.

@attests SPECIAL.DISTRIBUTION.HOMEBREW.INSTALLS_SPECIAL
artifact: `brew list --versions special` reported `special 0.9.1`; `/opt/homebrew/bin/special` resolved to `/opt/homebrew/Cellar/special/0.9.1/bin/special`; `special --version` reported `special 0.9.1`; `brew info special` loaded `sourcerodeo/tap/special` at stable `0.9.1`; and the local tap checkout was at `91df4efd7b6018e4ace1e11fac993424adea9e04`.
owner: gk
last_reviewed: 2026-05-06

@spec SPECIAL.DISTRIBUTION.CODEX_PLUGIN.SOURCE_LAYOUT
special keeps a marketplace-installable Codex plugin source tree under `codex-plugin/special/` with manifest, MCP config, and skills.

@spec SPECIAL.DISTRIBUTION.CODEX_PLUGIN.VERSION_AWARENESS
special's Codex plugin manifest version and MCP startup version argument match the package version.

@spec SPECIAL.DISTRIBUTION.CODEX_PLUGIN.SKILL_PARITY
special's Codex plugin and fallback `special skills` surface ship the same workflow skill ids.

@spec SPECIAL.DISTRIBUTION.SKILL_DOCS.GENERATED_SOURCE
special's shipped fallback and Codex plugin skill markdown files are generated from docs source files whose source carries Special documentation evidence, attach to their owning skill-level docs module, and use docs patterns for repeated internal structure.

@module SPECIAL.TESTS.DISTRIBUTION
Distribution/release asset integration tests in `tests/distribution.rs`.
*/
// @fileimplements SPECIAL.TESTS.DISTRIBUTION
use std::io::Write;
use std::path::PathBuf;
use std::process::{Command, Stdio};
use std::sync::atomic::{AtomicU64, Ordering};
use std::time::{SystemTime, UNIX_EPOCH};
use std::{fs, path::Path};

use serde_json::{Value, json};

static HOMEBREW_VERIFIER_COUNTER: AtomicU64 = AtomicU64::new(0);

fn repo_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
}

fn read_repo_file(path: impl AsRef<Path>) -> String {
    fs::read_to_string(repo_root().join(path)).expect("repo file should be readable")
}

fn cargo_metadata() -> Value {
    let output = Command::new("mise")
        .args([
            "exec",
            "--",
            "cargo",
            "metadata",
            "--no-deps",
            "--format-version",
            "1",
        ])
        .current_dir(repo_root())
        .output()
        .expect("cargo metadata should run");

    assert!(
        output.status.success(),
        "stdout:\n{}\n\nstderr:\n{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );

    serde_json::from_slice(&output.stdout).expect("cargo metadata output should be valid json")
}

fn release_assets() -> Value {
    serde_json::from_str(include_str!("../scripts/release-assets.json"))
        .expect("release assets json should be valid")
}

fn current_package_version() -> String {
    package_metadata()["version"]
        .as_str()
        .expect("package version should be a string")
        .to_string()
}

fn plugin_manifest() -> Value {
    serde_json::from_str(&read_repo_file(
        "codex-plugin/special/.codex-plugin/plugin.json",
    ))
    .expect("plugin manifest should be valid json")
}

fn plugin_mcp_config() -> Value {
    serde_json::from_str(&read_repo_file("codex-plugin/special/.mcp.json"))
        .expect("plugin MCP config should be valid json")
}

fn package_metadata() -> Value {
    cargo_metadata()["packages"]
        .as_array()
        .expect("packages should be an array")
        .iter()
        .find(|package| package["name"].as_str() == Some("special-cli"))
        .cloned()
        .expect("cargo metadata should include the special-cli package")
}

fn base64_encode(input: &str) -> String {
    let mut child = Command::new("mise")
        .args([
            "exec",
            "--",
            "python3",
            "-c",
            "import base64,sys; print(base64.b64encode(sys.stdin.buffer.read()).decode())",
        ])
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .spawn()
        .expect("python3 should run");
    child
        .stdin
        .as_mut()
        .expect("stdin should be piped")
        .write_all(input.as_bytes())
        .expect("input should be written");
    let output = child
        .wait_with_output()
        .expect("python output should be captured");
    assert!(
        output.status.success(),
        "python3 base64 helper should succeed: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    String::from_utf8(output.stdout)
        .expect("base64 output should be utf-8")
        .trim()
        .to_string()
}

fn homebrew_selector_arms() -> Vec<(&'static str, &'static str, &'static str)> {
    vec![
        ("special-cli-aarch64-apple-darwin.tar.xz", "macos", "arm"),
        ("special-cli-x86_64-apple-darwin.tar.xz", "macos", "intel"),
        (
            "special-cli-aarch64-unknown-linux-gnu.tar.xz",
            "linux",
            "arm",
        ),
        (
            "special-cli-x86_64-unknown-linux-gnu.tar.xz",
            "linux",
            "intel",
        ),
    ]
}

fn expected_release_sha(index: usize) -> String {
    format!("{:064x}", index + 1)
}

fn expected_release_sha_for_archive(name: &str) -> String {
    release_assets()["homebrew_formula_archives"]
        .as_array()
        .expect("homebrew formula archives should be an array")
        .iter()
        .enumerate()
        .find_map(|(index, value)| {
            (value.as_str() == Some(name)).then(|| expected_release_sha(index))
        })
        .unwrap_or_else(|| panic!("archive {name} should exist in release assets"))
}

fn valid_release_assets_json(mut mutate: impl FnMut(&mut Vec<Value>)) -> String {
    let release_assets = release_assets();
    let required = release_assets["homebrew_formula_archives"]
        .as_array()
        .expect("homebrew formula archives should be an array");
    let mut assets = required
        .iter()
        .enumerate()
        .map(|(index, name)| {
            let name = name.as_str().expect("archive name should be a string");
            json!({
                "name": name,
                "digest": format!("sha256:{}", expected_release_sha(index)),
            })
        })
        .collect::<Vec<_>>();
    mutate(&mut assets);
    json!({ "assets": assets }).to_string()
}

fn valid_formula_for_release(version: &str, sha_override: Option<(&str, String)>) -> String {
    let mut sha_macos_arm = String::new();
    let mut sha_macos_intel = String::new();
    let mut sha_linux_arm = String::new();
    let mut sha_linux_intel = String::new();

    for (archive, os_name, arch) in homebrew_selector_arms() {
        let sha = if let Some((target_archive, override_sha)) = &sha_override {
            if *target_archive == archive {
                override_sha.clone()
            } else {
                expected_release_sha_for_archive(archive)
            }
        } else {
            expected_release_sha_for_archive(archive)
        };
        match (os_name, arch) {
            ("macos", "arm") => sha_macos_arm = sha,
            ("macos", "intel") => sha_macos_intel = sha,
            ("linux", "arm") => sha_linux_arm = sha,
            ("linux", "intel") => sha_linux_intel = sha,
            _ => unreachable!("unexpected selector arm"),
        }
    }

    format!(
        r#"class Special < Formula
  version "{version}"
  if OS.mac? && Hardware::CPU.arm?
    url "https://github.com/sourcerodeo/special/releases/download/v{version}/special-cli-aarch64-apple-darwin.tar.xz"
    sha256 "{sha_macos_arm}"
  elsif OS.mac?
    url "https://github.com/sourcerodeo/special/releases/download/v{version}/special-cli-x86_64-apple-darwin.tar.xz"
    sha256 "{sha_macos_intel}"
  elsif OS.linux? && Hardware::CPU.arm?
    url "https://github.com/sourcerodeo/special/releases/download/v{version}/special-cli-aarch64-unknown-linux-gnu.tar.xz"
    sha256 "{sha_linux_arm}"
  elsif OS.linux?
    url "https://github.com/sourcerodeo/special/releases/download/v{version}/special-cli-x86_64-unknown-linux-gnu.tar.xz"
    sha256 "{sha_linux_intel}"
  end

  def install
    bin.install "special"
  end
end
"#
    )
}

fn run_homebrew_formula_verifier(release_json: &str, formula: &str) -> std::process::Output {
    let counter = HOMEBREW_VERIFIER_COUNTER.fetch_add(1, Ordering::Relaxed);
    let nanos = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("clock should be after unix epoch")
        .as_nanos();
    let temp = std::env::temp_dir().join(format!(
        "special-homebrew-verifier-{}-{nanos}-{counter}",
        std::process::id()
    ));
    let bin_dir = temp.join("bin");
    fs::create_dir_all(&bin_dir).expect("temp bin dir should be created");

    let gh_script = format!(
        r#"#!/usr/bin/env bash
set -euo pipefail
if [[ "${{1:-}}" == "release" && "${{2:-}}" == "view" ]]; then
cat <<'JSON'
{release_json}
JSON
elif [[ "${{1:-}}" == "api" ]]; then
cat <<'B64'
{formula_b64}
B64
else
echo "unexpected gh invocation: $*" >&2
exit 1
fi
"#,
        release_json = release_json,
        formula_b64 = base64_encode(formula),
    );
    let gh_path = bin_dir.join("gh");
    fs::write(&gh_path, gh_script).expect("fake gh script should be written");
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        let mut permissions = fs::metadata(&gh_path)
            .expect("fake gh metadata should exist")
            .permissions();
        permissions.set_mode(0o755);
        fs::set_permissions(&gh_path, permissions).expect("fake gh should be executable");
    }

    let path = std::env::var("PATH").expect("PATH should be set");
    let output = Command::new("bash")
        .arg("scripts/verify-homebrew-formula.sh")
        .current_dir(repo_root())
        .env("PATH", format!("{}:{path}", bin_dir.display()))
        .output()
        .expect("homebrew verifier should run");
    fs::remove_dir_all(&temp).expect("homebrew verifier temp dir should be removed");
    output
}

fn dist_command() -> Command {
    let mut command = Command::new("mise");
    command.args(["exec", "--", "dist"]);
    command.current_dir(repo_root());
    command
}

fn dist_manifest() -> Value {
    let package = package_metadata();
    let version = package["version"]
        .as_str()
        .expect("package version should be a string");

    let output = dist_command()
        .args([
            "manifest",
            "--artifacts=all",
            "--output-format=json",
            "--no-local-paths",
            "--tag",
            &format!("v{version}"),
            "--allow-dirty",
        ])
        .output()
        .expect("dist manifest should run");

    assert!(
        output.status.success(),
        "stdout:\n{}\n\nstderr:\n{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );

    serde_json::from_slice(&output.stdout).expect("dist manifest output should be valid json")
}

#[test]
// @verifies SPECIAL.DISTRIBUTION.CODEX_PLUGIN.SOURCE_LAYOUT
fn codex_plugin_source_has_marketplace_installable_layout() {
    let manifest = plugin_manifest();
    let mcp = plugin_mcp_config();

    assert_eq!(manifest["name"], "special");
    assert_eq!(manifest["skills"], "./skills/");
    assert_eq!(manifest["mcpServers"], "./.mcp.json");
    assert!(
        repo_root()
            .join("codex-plugin/special/skills/special-workflow/SKILL.md")
            .exists()
    );
    assert!(
        repo_root()
            .join("codex-plugin/special/skills/install-or-update-special/SKILL.md")
            .exists()
    );
    assert!(
        repo_root()
            .join("codex-plugin/special/skills/setup-special-project/SKILL.md")
            .exists()
    );
    assert_eq!(mcp["mcpServers"]["special"]["command"], "special");
    assert_eq!(mcp["mcpServers"]["special"]["args"][0], "mcp");
}

#[test]
// @verifies SPECIAL.DISTRIBUTION.CODEX_PLUGIN.VERSION_AWARENESS
fn codex_plugin_versions_match_package_version() {
    let version = current_package_version();
    let manifest = plugin_manifest();
    let mcp = plugin_mcp_config();
    let args = mcp["mcpServers"]["special"]["args"]
        .as_array()
        .expect("MCP args should be an array");

    assert_eq!(manifest["version"], version);
    assert!(
        args.iter()
            .any(|arg| arg.as_str() == Some(&format!("--special-version={version}")))
    );
}

fn markdown_files_under(root: &Path) -> Vec<PathBuf> {
    fn visit(path: &Path, files: &mut Vec<PathBuf>) {
        for entry in fs::read_dir(path).expect("directory should be readable") {
            let entry = entry.expect("directory entry should be readable");
            let path = entry.path();
            if path.is_dir() {
                visit(&path, files);
            } else if path.extension().and_then(|value| value.to_str()) == Some("md") {
                files.push(path);
            }
        }
    }

    let mut files = Vec::new();
    visit(root, &mut files);
    files.sort();
    files
}

fn skill_ids_under(root: &Path) -> Vec<String> {
    let mut ids = fs::read_dir(root)
        .unwrap_or_else(|error| panic!("{}: {error}", root.display()))
        .map(|entry| {
            entry
                .unwrap_or_else(|error| panic!("{}: {error}", root.display()))
                .path()
        })
        .filter(|path| path.is_dir())
        .filter(|path| path.join("SKILL.md").is_file())
        .map(|path| {
            path.file_name()
                .and_then(|name| name.to_str())
                .expect("skill directory should be utf-8")
                .to_string()
        })
        .collect::<Vec<_>>();
    ids.sort();
    ids
}

fn skill_module_id(prefix: &str, relative: &Path) -> String {
    let skill_name = relative
        .components()
        .next()
        .and_then(|component| component.as_os_str().to_str())
        .expect("skill source path should start with a skill directory");
    let normalized = skill_name.replace('-', "_").to_ascii_uppercase();
    format!("SPECIAL.DOCUMENTATION.SKILLS.{prefix}.{normalized}")
}

#[test]
// @verifies SPECIAL.DISTRIBUTION.CODEX_PLUGIN.SKILL_PARITY
fn codex_plugin_skills_do_not_lag_fallback_workflows() {
    let fallback_source = skill_ids_under(&repo_root().join("docs/src/skills/templates/skills"));
    let plugin_source = skill_ids_under(&repo_root().join("docs/src/skills/codex-plugin/skills"));
    let fallback_shipped = skill_ids_under(&repo_root().join("templates/skills"));
    let plugin_shipped = skill_ids_under(&repo_root().join("codex-plugin/special/skills"));

    assert_eq!(
        fallback_source, plugin_source,
        "plugin skill source should expose the same workflow ids as fallback source"
    );
    assert_eq!(
        fallback_shipped, fallback_source,
        "fallback shipped skills should match fallback source"
    );
    assert_eq!(
        plugin_shipped, plugin_source,
        "plugin shipped skills should match plugin source"
    );
}

#[test]
// @verifies SPECIAL.DISTRIBUTION.SKILL_DOCS.GENERATED_SOURCE
fn shipped_skill_docs_are_generated_from_annotated_sources() {
    let roots = [
        (
            "FALLBACK",
            repo_root().join("docs/src/skills/templates/skills"),
            repo_root().join("templates/skills"),
        ),
        (
            "PLUGIN",
            repo_root().join("docs/src/skills/codex-plugin/skills"),
            repo_root().join("codex-plugin/special/skills"),
        ),
    ];

    for (module_prefix, source_root, shipped_root) in roots {
        for shipped_path in markdown_files_under(&shipped_root) {
            let relative = shipped_path
                .strip_prefix(&shipped_root)
                .expect("shipped skill path should be under shipped root");
            let source_path = source_root.join(relative);
            let source = fs::read_to_string(&source_path)
                .unwrap_or_else(|error| panic!("{}: {error}", source_path.display()));
            let shipped = fs::read_to_string(&shipped_path)
                .unwrap_or_else(|error| panic!("{}: {error}", shipped_path.display()));

            assert!(
                source.contains("documents://")
                    || source.lines().any(|line| line.starts_with("@documents "))
                    || source
                        .lines()
                        .any(|line| line.starts_with("@filedocuments ")),
                "{} should carry docs evidence in source",
                source_path.display()
            );
            let expected_module = skill_module_id(module_prefix, relative);
            let implemented_modules = source
                .lines()
                .filter_map(|line| line.strip_prefix("@implements "))
                .filter(|id| id.starts_with("SPECIAL.DOCUMENTATION.SKILLS."))
                .collect::<Vec<_>>();
            assert!(
                implemented_modules
                    .iter()
                    .any(|module| *module == expected_module),
                "{} should attach to its owning skill docs module {expected_module}",
                source_path.display()
            );
            assert!(
                implemented_modules
                    .iter()
                    .all(|module| *module == expected_module),
                "{} should not model repeated skill body sections as docs modules: {:?}",
                source_path.display(),
                implemented_modules
            );
            assert!(
                source
                    .lines()
                    .any(|line| line.starts_with("@applies DOCS.SKILL_")),
                "{} should apply a generated skill docs pattern",
                source_path.display()
            );
            if relative.file_name().and_then(|name| name.to_str()) == Some("SKILL.md") {
                assert!(
                    source.contains("documents://"),
                    "{} should link concrete skill claims to Special docs targets",
                    source_path.display()
                );
            }
            assert!(
                !shipped.lines().any(|line| {
                    line.contains("(documents://") || line.starts_with("@filedocuments ")
                }),
                "{} should be scrubbed generated output",
                shipped_path.display()
            );
        }
    }
}

#[test]
// @verifies SPECIAL.DISTRIBUTION.CRATES_IO.PACKAGE_NAME
fn crates_io_package_name_is_special_cli() {
    let package = package_metadata();
    let package_name = package["name"]
        .as_str()
        .expect("package name should be a string");

    assert_eq!(package_name, "special-cli");
}

#[test]
// @verifies SPECIAL.DISTRIBUTION.CRATES_IO.BINARY_NAME
fn cargo_package_installs_special_binary() {
    let package = package_metadata();
    let targets = package["targets"]
        .as_array()
        .expect("targets should be an array");

    let has_special_bin = targets.iter().any(|target| {
        target["name"].as_str() == Some("special")
            && target["kind"]
                .as_array()
                .map(|kinds| kinds.iter().any(|kind| kind.as_str() == Some("bin")))
                .unwrap_or(false)
    });

    assert!(
        has_special_bin,
        "cargo metadata should expose a `special` binary target"
    );
}

#[test]
// @verifies SPECIAL.DISTRIBUTION.CRATES_IO.RUST_VERSION
fn cargo_package_declares_minimum_rust_version() {
    let package = package_metadata();
    let rust_version = package["rust_version"]
        .as_str()
        .expect("package metadata should include rust_version");

    assert_eq!(rust_version, "1.85");
}

#[test]
// @verifies SPECIAL.DISTRIBUTION.GITHUB_RELEASES.REPOSITORY_URL
fn github_release_repository_url_is_declared() {
    let package = package_metadata();
    let repository = package["repository"]
        .as_str()
        .expect("repository should be a string");

    assert_eq!(repository, "https://github.com/sourcerodeo/special");
}

#[test]
// @verifies SPECIAL.DISTRIBUTION.SOURCE_DEPENDENCIES.PARSER_CRATE
fn shared_parser_crate_uses_github_monorepo_dependency() {
    let package = package_metadata();
    let dependencies = package["dependencies"]
        .as_array()
        .expect("dependencies should be an array");
    let dependency = dependencies
        .iter()
        .find(|dependency| dependency["name"].as_str() == Some("parse-source-annotations"))
        .expect("special should depend on parse-source-annotations");
    assert_eq!(
        dependency["source"],
        "git+https://github.com/SourceRodeo/crates"
    );

    let manifest = read_repo_file("Cargo.toml");
    assert!(manifest.contains("parse-source-annotations = {"));
    assert!(manifest.contains("version = \"0.1.0\""));
    assert!(manifest.contains("git = \"https://github.com/SourceRodeo/crates\""));
    assert!(manifest.contains("package = \"parse-source-annotations\""));
    assert!(!manifest.contains("../crates/parse-source-annotations"));
}

#[test]
// @verifies SPECIAL.DISTRIBUTION.SOURCE_DEPENDENCIES.PARSER_CRATE
fn parser_crate_git_monorepo_contract_is_documented() {
    let docs = read_repo_file("docs/contributor/release.md");
    assert!(docs.contains("SourceRodeo/crates"));
    assert!(docs.contains("parse-source-annotations"));
    assert!(docs.contains("Cargo"));
    assert!(docs.contains("Git dependency"));
    assert!(docs.contains("release builds"));
    assert!(!docs.contains("../crates/parse-source-annotations"));
}

#[test]
// @verifies SPECIAL.DISTRIBUTION.SOURCE_DEPENDENCIES.NO_LOCAL_CHECKOUT
fn release_workflow_does_not_clone_parser_source_dependency() {
    let workflow = read_repo_file(".github/workflows/release.yml");

    assert!(!workflow.contains("Checkout parser source dependency"));
    assert!(!workflow.contains("SOURCE_DEPENDENCIES_TOKEN"));
    assert!(!workflow.contains("../crates/parse-source-annotations"));
    assert!(!workflow.contains("sourcerodeo/parse-source-annotations"));
}

#[test]
// @verifies SPECIAL.DISTRIBUTION.GITHUB_RELEASES.WORKFLOW
fn github_release_workflow_is_committed_and_in_sync() {
    assert!(
        repo_root().join(".github/workflows/release.yml").is_file(),
        "release workflow should be committed"
    );
    let workflow = read_repo_file(".github/workflows/release.yml");
    assert!(workflow.contains("tags:\n      - '**[0-9]+.[0-9]+.[0-9]+*'"));
    assert!(!workflow.contains("Validate release tag shape"));
    assert!(workflow.contains("rm -f artifacts/*-dist-manifest.json"));

    let package = package_metadata();
    let version = package["version"]
        .as_str()
        .expect("package version should be a string");

    let output = dist_command()
        .args([
            "host",
            "--steps=create",
            "--tag",
            &format!("v{version}"),
            "--output-format=json",
        ])
        .output()
        .expect("dist host --steps=create should run");

    assert!(
        output.status.success(),
        "stdout:\n{}\n\nstderr:\n{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
}

#[test]
// @verifies SPECIAL.DISTRIBUTION.BUILD_SCRIPT.LANGUAGE_PACK_REGISTRY
fn build_script_generates_the_builtin_language_pack_registry() {
    let build_script = read_repo_file("build.rs");

    assert!(
        build_script.contains("src/language_packs"),
        "build script should discover shipped language pack sources"
    );
    assert!(
        build_script.contains("language_pack_registry.rs"),
        "build script should emit the generated language pack registry"
    );
    assert!(
        build_script.contains("REGISTERED_LANGUAGE_PACKS"),
        "generated registry should expose the built-in language pack descriptor list"
    );
}

#[test]
// @verifies SPECIAL.DISTRIBUTION.BUILD_SCRIPT.LANGUAGE_PACK_REGISTRY.TOP_LEVEL_ENTRIES_ONLY
fn build_script_rejects_top_level_language_pack_helpers() {
    let build_script = read_repo_file("build.rs");

    assert!(
        build_script.contains("assert_language_pack_entry_shape"),
        "registry generation should validate top-level language-pack entry shape"
    );
    assert!(
        build_script.contains("move shared helpers under a language-pack subdirectory"),
        "registry generation should explain where shared helpers belong"
    );
    assert!(
        build_script.contains("stem == \"mod\""),
        "registry generation should skip only the Rust module root"
    );
}

fn built_in_language_pack_ids() -> [&'static str; 4] {
    ["go", "python", "rust", "typescript"]
}

fn read_language_pack_source(id: &str) -> String {
    read_repo_file(format!("src/language_packs/{id}.rs"))
}

#[test]
// @verifies SPECIAL.LANGUAGE_PACKS.ADMISSION.REGISTRATION
fn built_in_language_pack_admission_uses_descriptor_registration() {
    let build_script = read_repo_file("build.rs");
    let syntax_registry = read_repo_file("src/syntax/registry.rs");
    let syntax_core = read_repo_file("src/syntax.rs");

    for id in built_in_language_pack_ids() {
        let source = read_language_pack_source(id);
        assert!(source.contains("DESCRIPTOR"));
        assert!(source.contains("LanguagePackDescriptor"));
        assert!(source.contains("parse_source_graph"));
    }
    assert!(build_script.contains("language_pack_registry.rs"));
    assert!(build_script.contains("assert_language_pack_entry_shape"));
    assert!(syntax_registry.contains("language_packs::descriptors()"));
    assert!(syntax_core.contains("registry::parse_source_graph_at_path"));
}

#[test]
// @verifies SPECIAL.LANGUAGE_PACKS.ADMISSION.PARSER_SURFACE
fn built_in_language_pack_admission_has_parser_surface_tests() {
    let syntax_core = read_repo_file("src/syntax.rs");

    for id in built_in_language_pack_ids() {
        let provider = read_repo_file(format!("src/syntax/{id}.rs"));
        assert!(provider.contains("impl SyntaxProvider"));
        assert!(provider.contains("ParsedSourceGraph"));
        assert!(provider.contains("SourceItem"));
        assert!(provider.contains("call"));
        assert!(provider.contains("provider_facade"));
    }
    for spec in [
        "SPECIAL.SYNTAX.PROVIDERS.GO_ITEMS_AND_CALLS",
        "SPECIAL.SYNTAX.PROVIDERS.PYTHON_ITEMS_AND_CALLS",
        "SPECIAL.SYNTAX.PROVIDERS.RUST_ITEMS_AND_CALLS",
        "SPECIAL.SYNTAX.PROVIDERS.TYPESCRIPT_ITEMS_AND_CALLS",
    ] {
        assert!(syntax_core.contains(spec));
    }
}

#[test]
// @verifies SPECIAL.LANGUAGE_PACKS.ADMISSION.TRACEABILITY
fn built_in_language_pack_admission_has_traceability_parity_fixtures() {
    let scoped_tests = read_repo_file("tests/scoped_health_proof_boundary.rs");
    let repo_tests = read_repo_file("tests/cli_repo.rs");

    for token in [
        "write_go_traceability_fixture",
        "write_python_traceability_fixture",
        "write_traceability_imported_call_fixture",
        "write_typescript_traceability_fixture",
        "matches_full_then_filtered",
        "does_not_build_language_pack_fact_blobs",
    ] {
        assert!(scoped_tests.contains(token));
    }
    for token in [
        "repo_surfaces_go_traceability",
        "repo_surfaces_python_traceability",
        "repo_surfaces_traceability",
        "repo_surfaces_typescript_traceability",
    ] {
        assert!(repo_tests.contains(token));
    }
}

#[test]
// @verifies SPECIAL.LANGUAGE_PACKS.ADMISSION.DEGRADATION
fn built_in_language_pack_admission_declares_tooling_or_parser_boundary() {
    for id in ["go", "rust", "typescript"] {
        let source = read_language_pack_source(id);
        assert!(source.contains("project_tooling: Some(&PROJECT_TOOLING)"));
        assert!(source.contains("ProjectToolRequirement"));
        assert!(source.contains("traceability_unavailable_reason"));
    }

    let python = read_language_pack_source("python");
    assert!(python.contains("project_tooling: None"));
    assert!(python.contains("parser-backed"));
    assert!(python.contains("traceability_unavailable_reason"));
}

#[test]
// @verifies SPECIAL.DISTRIBUTION.GITHUB_RELEASES.LEAN_KERNEL
fn github_release_workflow_embeds_lean_traceability_kernel_for_host_native_builds() {
    let workflow = read_repo_file(".github/workflows/release.yml");
    let dist_config = read_repo_file("dist-workspace.toml");
    let build_script = read_repo_file("build.rs");
    let verifier = read_repo_file("scripts/verify-lean-traceability-kernel.py");
    assert!(
        workflow.contains("uses: jdx/mise-action@v2"),
        "release builds should install the project tool manager before building the Lean kernel"
    );
    assert!(
        workflow.contains("SPECIAL_BUILD_LEAN_KERNEL: \"1\""),
        "release builds should request the standalone Lean traceability kernel"
    );
    assert!(
        dist_config.contains("allow-dirty = [\"ci\"]"),
        "cargo-dist should allow the intentional release workflow customization"
    );
    assert!(
        build_script.contains("lean_kernel_target_matches_host")
            && build_script.contains("skipping embedded Lean traceability kernel for cross target"),
        "cross builds should not embed a Lean executable built for the host architecture"
    );
    assert!(
        !build_script.contains("will use the Rust reference kernel"),
        "cross builds should not claim a production fallback to the debug/test Rust reference kernel"
    );
    assert!(
        verifier.contains("\"-Krelease\", \"build\", \"special_traceability_kernel\"")
            && verifier.contains("SPECIAL_TRACEABILITY_KERNEL_EXE")
            && verifier.contains("SPECIAL_REQUIRE_LEAN_KERNEL_TESTS"),
        "Lean verification should exercise the release-built kernel through required Rust-vs-Lean equivalence tests"
    );
}

#[test]
// @verifies SPECIAL.DISTRIBUTION.HOMEBREW.FORMULA.PATH
fn homebrew_formula_uses_the_standard_formula_path() {
    let updater = read_repo_file("scripts/update-homebrew-formula.py");
    assert!(
        updater.contains("FORMULA_PATH = \"Formula/special.rb\""),
        "Homebrew updater should target the standard Formula path"
    );

    let verifier = read_repo_file("scripts/verify-homebrew-formula.sh");
    assert!(
        verifier.contains("contents/Formula/special.rb"),
        "Homebrew verification should read the standard Formula path"
    );
}

#[test]
// @verifies SPECIAL.DISTRIBUTION.HOMEBREW.FORMULA.PLATFORM_SELECTION
fn homebrew_formula_uses_standard_platform_selection_helpers() {
    let updater = read_repo_file("scripts/update-homebrew-formula.py");

    assert!(
        updater.contains("if OS.mac? && Hardware::CPU.arm?"),
        "Homebrew updater should branch on mac arm"
    );
    assert!(
        updater.contains("elsif OS.linux? && Hardware::CPU.arm?"),
        "Homebrew updater should branch on linux arm"
    );
    assert!(
        updater.contains("sha256 \"{archive_sha("),
        "Homebrew updater should keep checksum selection inside platform branches"
    );
    assert!(updater.contains("github.com/sourcerodeo/special/releases"));
    assert!(updater.contains("special-cli-aarch64-apple-darwin.tar.xz"));

    let version = current_package_version();
    let release_json = valid_release_assets_json(|_| {});
    let formula = valid_formula_for_release(&version, None);
    let success = run_homebrew_formula_verifier(&release_json, &formula);
    assert!(
        success.status.success(),
        "stdout:\n{}\n\nstderr:\n{}",
        String::from_utf8_lossy(&success.stdout),
        String::from_utf8_lossy(&success.stderr)
    );

    let missing_url = run_homebrew_formula_verifier(
        &release_json,
        &formula.replace(
            &format!(
                "url \"https://github.com/sourcerodeo/special/releases/download/v{version}/special-cli-aarch64-apple-darwin.tar.xz\""
            ),
            "",
        ),
    );
    assert!(!missing_url.status.success());
    assert!(
        String::from_utf8_lossy(&missing_url.stderr).contains("formula is missing archive branch"),
        "stderr:\n{}",
        String::from_utf8_lossy(&missing_url.stderr)
    );

    let missing_selector = run_homebrew_formula_verifier(
        &release_json,
        &formula.replace(
            "special-cli-aarch64-apple-darwin.tar.xz",
            "wrong-archive.tar.xz",
        ),
    );
    assert!(!missing_selector.status.success());
    assert!(
        String::from_utf8_lossy(&missing_selector.stderr)
            .contains("formula is missing archive branch"),
        "stderr:\n{}",
        String::from_utf8_lossy(&missing_selector.stderr)
    );
}

#[test]
// @verifies SPECIAL.DISTRIBUTION.HOMEBREW.FORMULA.TAP_METADATA_CHECK
fn homebrew_formula_verifier_requires_release_asset_digests() {
    let version = current_package_version();
    let release_json = valid_release_assets_json(|assets| {
        assets[0]
            .as_object_mut()
            .expect("asset should be an object")
            .remove("digest");
    });
    let formula = valid_formula_for_release(&version, None);
    let output = run_homebrew_formula_verifier(&release_json, &formula);

    assert!(!output.status.success());
    assert!(
        String::from_utf8_lossy(&output.stderr).contains("release asset is missing digest"),
        "stderr:\n{}",
        String::from_utf8_lossy(&output.stderr)
    );
}

#[test]
// @verifies SPECIAL.DISTRIBUTION.HOMEBREW.FORMULA.TAP_METADATA_CHECK
fn homebrew_formula_verifier_checks_selector_checksum_pairing() {
    let version = current_package_version();
    let release_json = valid_release_assets_json(|_| {});
    let formula = valid_formula_for_release(
        &version,
        Some((
            "special-cli-x86_64-unknown-linux-gnu.tar.xz",
            "f".repeat(64),
        )),
    );
    let output = run_homebrew_formula_verifier(&release_json, &formula);

    assert!(!output.status.success());
    assert!(
        String::from_utf8_lossy(&output.stderr).contains("formula is missing archive branch"),
        "stderr:\n{}",
        String::from_utf8_lossy(&output.stderr)
    );
}

#[test]
// @verifies SPECIAL.DISTRIBUTION.GITHUB_RELEASES.ARCHIVES
fn github_release_plan_contains_versioned_archives() {
    let manifest = dist_manifest();
    let artifacts = manifest["artifacts"]
        .as_object()
        .expect("artifacts should be an object");

    let release_archives: Vec<_> = artifacts
        .values()
        .filter(|artifact| artifact["kind"].as_str() == Some("executable-zip"))
        .collect();

    let mut archive_names: Vec<_> = release_archives
        .iter()
        .map(|archive| {
            archive["name"]
                .as_str()
                .expect("archive name should be a string")
        })
        .collect();
    archive_names.sort_unstable();

    let release_assets = release_assets();
    let mut expected_archive_names: Vec<_> = release_assets["archives"]
        .as_array()
        .expect("archives should be an array")
        .iter()
        .map(|archive| archive.as_str().expect("archive should be a string"))
        .collect();
    expected_archive_names.sort_unstable();

    assert_eq!(archive_names, expected_archive_names);

    for archive in release_archives {
        let name = archive["name"]
            .as_str()
            .expect("archive name should be a string");
        let assets = archive["assets"]
            .as_array()
            .expect("archive assets should be an array");

        assert!(
            name.starts_with("special-cli-"),
            "archive name should be versioned under the package identity: {name}"
        );
        assert!(
            assets
                .iter()
                .any(|asset| asset["kind"].as_str() == Some("executable")),
            "archive should include the special executable: {name}"
        );
    }
}

#[test]
// @verifies SPECIAL.DISTRIBUTION.GITHUB_RELEASES.CHECKSUMS
fn github_release_plan_contains_checksums_for_archives() {
    let manifest = dist_manifest();
    let artifacts = manifest["artifacts"]
        .as_object()
        .expect("artifacts should be an object");
    let release_assets = release_assets();

    assert!(
        artifacts
            .get("sha256.sum")
            .and_then(|artifact| artifact["kind"].as_str())
            == Some("unified-checksum"),
        "dist manifest should define a unified checksum artifact"
    );

    for artifact in artifacts.values() {
        if artifact["kind"].as_str() == Some("executable-zip") {
            let checksum_name = artifact["checksum"]
                .as_str()
                .expect("archive should declare a checksum artifact");

            assert!(
                artifacts
                    .get(checksum_name)
                    .and_then(|checksum| checksum["kind"].as_str())
                    == Some("checksum"),
                "archive checksum artifact should exist for {checksum_name}"
            );
        }
    }

    for archive_name in release_assets["archives"]
        .as_array()
        .expect("archives should be an array")
        .iter()
        .map(|archive| archive.as_str().expect("archive should be a string"))
    {
        let checksum_name = format!("{archive_name}.sha256");
        assert!(
            artifacts
                .get(&checksum_name)
                .and_then(|artifact| artifact["kind"].as_str())
                == Some("checksum"),
            "dist manifest should define checksum artifact {checksum_name}"
        );
    }
}
