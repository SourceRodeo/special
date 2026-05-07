/**
@module SPECIAL.LANGUAGE_PACKS.RUST.ANALYZE.DEPENDENCIES
Extracts Rust-specific `use`-path dependency evidence from owned Rust implementation without inferring architecture verdicts.
*/
// @fileimplements SPECIAL.LANGUAGE_PACKS.RUST.ANALYZE.DEPENDENCIES
use std::collections::{BTreeMap, BTreeSet};
use std::path::{Path, PathBuf};

use syn::{Item, ItemUse};

use crate::model::ModuleDependencySummary;
use crate::modules::analyze::ModuleCouplingInput;
use crate::modules::analyze::build_dependency_summary;
use super::use_tree::flatten_use_tree;

#[derive(Debug, Default)]
pub(super) struct RustDependencySummary {
    targets: BTreeMap<String, usize>,
    internal_files: BTreeSet<PathBuf>,
    external_targets: BTreeSet<String>,
}

impl RustDependencySummary {
    pub(super) fn observe(&mut self, root: &Path, source_path: &Path, text: &str) {
        if let Ok(file) = syn::parse_file(text) {
            self.observe_items(root, source_path, &file.items);
            return;
        }

        if let Ok(item) = syn::parse_str::<Item>(text) {
            self.observe_item(root, source_path, &item);
        }
    }

    pub(super) fn summary(&self) -> ModuleDependencySummary {
        build_dependency_summary(&self.targets)
    }

    pub(super) fn coupling_input(&self) -> ModuleCouplingInput {
        ModuleCouplingInput {
            internal_files: self.internal_files.clone(),
            external_targets: self.external_targets.clone(),
        }
    }

    fn observe_items(&mut self, root: &Path, source_path: &Path, items: &[Item]) {
        for item in items {
            self.observe_item(root, source_path, item);
        }
    }

    fn observe_item(&mut self, root: &Path, source_path: &Path, item: &Item) {
        match item {
            Item::Use(item_use) => self.observe_use(root, source_path, item_use),
            Item::Mod(item_mod) => {
                if let Some((_, nested)) = &item_mod.content {
                    self.observe_items(root, source_path, nested);
                }
            }
            _ => {}
        }
    }

    fn observe_use(&mut self, root: &Path, source_path: &Path, node: &ItemUse) {
        for path in flatten_use_tree(&node.tree) {
            *self.targets.entry(path.clone()).or_default() += 1;
            if let Some(file) = resolve_internal_file(root, source_path, &path) {
                self.internal_files.insert(file);
            } else if !is_internal_target(&path) {
                self.external_targets.insert(path);
            }
        }
    }
}

fn is_internal_target(path: &str) -> bool {
    path.starts_with("crate::") || path.starts_with("self::") || path.starts_with("super::")
}

fn resolve_internal_file(root: &Path, source_path: &Path, path: &str) -> Option<PathBuf> {
    let mut segments = path.split("::");
    let anchor = segments.next()?;
    let remainder: Vec<&str> = segments.collect();
    if remainder.is_empty() {
        return None;
    }

    let source_dir = root
        .join(source_path)
        .parent()
        .map(Path::to_path_buf)
        .unwrap_or_else(|| root.to_path_buf());
    let module_dir = match anchor {
        "crate" => root.to_path_buf(),
        "self" => source_dir,
        "super" => {
            let mut dir = source_dir
                .parent()
                .unwrap_or_else(|| Path::new(""))
                .to_path_buf();
            let mut remaining = remainder.as_slice();
            while matches!(remaining.first(), Some(segment) if *segment == "super") {
                dir = dir.parent().unwrap_or_else(|| Path::new("")).to_path_buf();
                remaining = &remaining[1..];
            }
            return resolve_internal_segments(root, dir, remaining);
        }
        _ => return None,
    };

    resolve_internal_segments(root, module_dir, &remainder)
}

fn resolve_internal_segments(
    _root: &Path,
    module_dir: PathBuf,
    remainder: &[&str],
) -> Option<PathBuf> {
    for prefix_len in (1..=remainder.len()).rev() {
        let candidate = &remainder[..prefix_len];
        let file_candidate = module_dir.join(candidate.join("/")).with_extension("rs");
        if file_candidate.exists() {
            return Some(file_candidate);
        }

        let mod_candidate = module_dir.join(candidate.join("/")).join("mod.rs");
        if mod_candidate.exists() {
            return Some(mod_candidate);
        }
    }
    None
}

#[cfg(test)]
mod tests {
    use std::fs;

    use super::*;

    #[test]
    fn provider_dependency_summary_resolves_internal_rust_modules() {
        let root = temp_root("special-rust-dependencies");
        fs::create_dir_all(root.join("nested")).expect("nested dir should exist");
        fs::write(root.join("api.rs"), "").expect("api module should exist");
        fs::write(root.join("nested").join("mod.rs"), "").expect("nested module should exist");
        fs::write(root.join("nested").join("sibling.rs"), "")
            .expect("sibling module should exist");

        assert!(is_internal_target("crate::api::Item"));
        assert!(!is_internal_target("serde::Serialize"));
        assert_eq!(
            resolve_internal_file(&root, Path::new("lib.rs"), "crate::api::Item"),
            Some(root.join("api.rs"))
        );
        assert_eq!(
            resolve_internal_file(&root, Path::new("lib.rs"), "self::nested::Thing"),
            Some(root.join("nested").join("mod.rs"))
        );
        assert_eq!(
            resolve_internal_file(&root, Path::new("nested/current.rs"), "super::api::Item"),
            Some(root.join("api.rs"))
        );
        assert_eq!(
            resolve_internal_segments(&root, root.join("nested"), &["sibling", "Item"]),
            Some(root.join("nested").join("sibling.rs"))
        );

        fs::remove_dir_all(root).expect("temp root should be removed");
    }

    fn temp_root(prefix: &str) -> PathBuf {
        let unique = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .expect("system clock should be valid")
            .as_nanos();
        let path = std::env::temp_dir().join(format!("{prefix}-{}-{unique}", std::process::id()));
        fs::create_dir_all(&path).expect("temp root should exist");
        path
    }
}
