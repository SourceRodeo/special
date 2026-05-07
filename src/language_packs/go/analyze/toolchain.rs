/**
@module SPECIAL.LANGUAGE_PACKS.GO.ANALYZE.TOOLCHAIN
Discovers local Go tooling and manages temporary Go analysis runtime state for the built-in Go pack analyzer.
*/
// @fileimplements SPECIAL.LANGUAGE_PACKS.GO.ANALYZE.TOOLCHAIN
use std::fs;
use std::io::ErrorKind;
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicU64, Ordering};

use serde::Deserialize;

use crate::config::{
    ProjectToolStatus, ProjectToolchain, probe_project_tool, standard_tool_unavailable_reason,
};

pub(super) fn go_list_packages(root: &Path) -> Option<Vec<GoListPackage>> {
    let cache_dir = create_temp_dir("special-go-build-cache")?;
    let toolchain = ProjectToolchain::discover(root).ok().flatten()?;
    run_go_list_command(root, &toolchain, cache_dir.path())
        .map(|output| {
            let mut packages = Vec::new();
            let stream =
                serde_json::Deserializer::from_slice(&output.stdout).into_iter::<GoListPackage>();
            for package in stream.flatten() {
                packages.push(package);
            }
            packages
        })
        .filter(|packages| !packages.is_empty())
}

fn run_go_list_command(
    _root: &Path,
    toolchain: &ProjectToolchain,
    cache_dir: &Path,
) -> Option<std::process::Output> {
    toolchain
        .command("go")
        .args(["list", "-json", "./..."])
        .env("GOCACHE", cache_dir)
        .output()
        .ok()
        .filter(|output| output.status.success())
}

pub(super) fn go_backward_trace_unavailable_reason(root: &Path) -> Option<String> {
    match probe_project_tool(root, "go", &["version"]).ok()? {
        ProjectToolStatus::Available => {}
        status => {
            return Some(standard_tool_unavailable_reason(
                "Go backward trace",
                "go",
                &status,
            ));
        }
    }
    match probe_project_tool(root, "gopls", &["version"]).ok()? {
        ProjectToolStatus::Available => None,
        status => Some(standard_tool_unavailable_reason(
            "Go backward trace",
            "gopls",
            &status,
        )),
    }
}

pub(super) fn analysis_environment_fingerprint(root: &Path) -> String {
    let Some(toolchain) = ProjectToolchain::discover(root).ok().flatten() else {
        return "project_toolchain=unavailable".to_string();
    };
    let go = tool_version_fingerprint(&toolchain, "go", &["version"]);
    let gopls = tool_version_fingerprint(&toolchain, "gopls", &["version"]);
    format!("project_toolchain_go={go};project_toolchain_gopls={gopls}")
}

fn tool_version_fingerprint(
    toolchain: &ProjectToolchain,
    tool: &str,
    version_args: &[&str],
) -> String {
    let available = toolchain.tool_available(tool, version_args);
    let output = toolchain.command(tool).args(version_args).output();
    let version = output
        .ok()
        .filter(|output| output.status.success())
        .map(|output| {
            let stdout = String::from_utf8_lossy(&output.stdout).trim().to_string();
            let stderr = String::from_utf8_lossy(&output.stderr).trim().to_string();
            if !stdout.is_empty() {
                stdout
            } else if !stderr.is_empty() {
                stderr
            } else {
                "available".to_string()
            }
        })
        .unwrap_or_else(|| available.to_string());
    version.replace(['\n', '\r'], " ")
}

static TEMP_DIR_COUNTER: AtomicU64 = AtomicU64::new(0);

pub(super) struct TempDirGuard {
    path: PathBuf,
}

impl TempDirGuard {
    pub(super) fn path(&self) -> &Path {
        &self.path
    }
}

impl Drop for TempDirGuard {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.path);
    }
}

pub(super) fn create_temp_dir(prefix: &str) -> Option<TempDirGuard> {
    loop {
        let unique = TEMP_DIR_COUNTER.fetch_add(1, Ordering::Relaxed);
        let path = std::env::temp_dir().join(format!("{prefix}-{}-{unique}", std::process::id()));
        match fs::create_dir(&path) {
            Ok(()) => return Some(TempDirGuard { path }),
            Err(error) if error.kind() == ErrorKind::AlreadyExists => continue,
            Err(_) => return None,
        }
    }
}

#[derive(Deserialize)]
pub(super) struct GoListPackage {
    #[serde(rename = "ImportPath")]
    pub(super) import_path: String,
    #[serde(rename = "Dir")]
    pub(super) dir: PathBuf,
    #[serde(rename = "GoFiles", default)]
    pub(super) go_files: Vec<String>,
}
#[cfg(test)]
mod tests {
    use std::fs;

    use crate::config::ProjectToolchain;

    use super::{analysis_environment_fingerprint, create_temp_dir, tool_version_fingerprint};

    #[test]
    fn create_temp_dir_uses_unique_paths_and_cleans_up_on_drop() {
        let first = create_temp_dir("special-go-temp").expect("first temp dir should exist");
        let second = create_temp_dir("special-go-temp").expect("second temp dir should exist");

        let first_path = first.path().to_path_buf();
        let second_path = second.path().to_path_buf();

        assert_ne!(first_path, second_path);
        assert!(first_path.is_dir());
        assert!(second_path.is_dir());

        drop(first);
        assert!(!first_path.exists());
        assert!(second_path.exists());

        drop(second);
        assert!(!second_path.exists());
    }

    #[test]
    fn analysis_environment_fingerprint_reports_missing_toolchain_contract() {
        let root = temp_root("special-go-toolchain-missing");

        assert_eq!(
            analysis_environment_fingerprint(&root),
            "project_toolchain=unavailable"
        );

        fs::remove_dir_all(root).expect("temp root should be removed");
    }

    #[test]
    fn tool_version_fingerprint_reports_undeclared_tools_without_shelling_real_tool() {
        let root = temp_root("special-go-toolchain-undeclared");
        fs::write(root.join(".tool-versions"), "python 3.12\n")
            .expect("tool versions should be written");
        let toolchain = ProjectToolchain::discover(&root)
            .expect("toolchain discovery should succeed")
            .expect("toolchain should exist");

        assert_eq!(
            tool_version_fingerprint(&toolchain, "go", &["version"]),
            "false"
        );

        fs::remove_dir_all(root).expect("temp root should be removed");
    }

    fn temp_root(prefix: &str) -> std::path::PathBuf {
        let unique = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .expect("system clock should be valid")
            .as_nanos();
        let path = std::env::temp_dir().join(format!("{prefix}-{}-{unique}", std::process::id()));
        fs::create_dir_all(&path).expect("temp root should exist");
        path
    }
}
