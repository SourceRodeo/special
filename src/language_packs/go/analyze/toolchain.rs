/**
@module SPECIAL.LANGUAGE_PACKS.GO.ANALYZE.TOOLCHAIN
Discovers local Go tooling and manages temporary Go analysis runtime state for the built-in Go pack analyzer.
*/
// @fileimplements SPECIAL.LANGUAGE_PACKS.GO.ANALYZE.TOOLCHAIN
use std::fs;
use std::io::ErrorKind;
use std::path::{Path, PathBuf};
use std::process::Command;
use std::sync::atomic::{AtomicU64, Ordering};

use serde::Deserialize;

pub(super) fn go_list_packages(root: &Path) -> Option<Vec<GoListPackage>> {
    let go_binary = discover_go_binary()?;
    let cache_dir = create_temp_dir("special-go-build-cache")?;
    let output = Command::new(go_binary)
        .args(["list", "-json", "./..."])
        .current_dir(root)
        .env("GOCACHE", cache_dir.path())
        .output()
        .ok()?;
    if !output.status.success() {
        return None;
    }
    let mut packages = Vec::new();
    let stream = serde_json::Deserializer::from_slice(&output.stdout).into_iter::<GoListPackage>();
    for package in stream.flatten() {
        packages.push(package);
    }
    (!packages.is_empty()).then_some(packages)
}

pub(super) fn discover_go_binary() -> Option<PathBuf> {
    let output = Command::new("mise")
        .args(["ls", "--json", "go"])
        .output()
        .ok()?;
    if !output.status.success() {
        return None;
    }
    let installs: Vec<MiseGoInstall> = serde_json::from_slice(&output.stdout).ok()?;
    let install = installs
        .into_iter()
        .filter(|install| install.installed)
        .max_by(|left, right| {
            left.active
                .cmp(&right.active)
                .then_with(|| compare_semver(&left.version, &right.version))
        })?;
    let go_binary = install.install_path.join("bin/go");
    go_binary.exists().then_some(go_binary)
}

pub(super) fn analysis_environment_fingerprint() -> String {
    let Some(go_binary) = discover_go_binary() else {
        return "go=unavailable".to_string();
    };
    let gopls = go_binary
        .parent()
        .map(|dir| dir.join("gopls"))
        .filter(|path| path.exists());
    format!(
        "go={};gopls={}",
        go_binary.display(),
        gopls
            .as_ref()
            .map(|path| path.display().to_string())
            .unwrap_or_else(|| "unavailable".to_string())
    )
}

fn compare_semver(left: &str, right: &str) -> std::cmp::Ordering {
    let parse = |value: &str| {
        value
            .split('.')
            .map(|part| part.parse::<u64>().unwrap_or(0))
            .collect::<Vec<_>>()
    };
    parse(left).cmp(&parse(right))
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
struct MiseGoInstall {
    version: String,
    install_path: PathBuf,
    installed: bool,
    #[serde(default)]
    active: bool,
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
    use super::create_temp_dir;

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
}
