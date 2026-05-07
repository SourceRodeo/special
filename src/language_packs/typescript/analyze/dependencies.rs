/**
@module SPECIAL.LANGUAGE_PACKS.TYPESCRIPT.ANALYZE.DEPENDENCIES
Extracts TypeScript import-based dependency and coupling evidence for the built-in TypeScript pack analyzer.
*/
// @fileimplements SPECIAL.LANGUAGE_PACKS.TYPESCRIPT.ANALYZE.DEPENDENCIES
use std::collections::{BTreeMap, BTreeSet};
use std::path::{Component, Path, PathBuf};

use tree_sitter::{Node, Parser};

use crate::model::ModuleDependencySummary;
use crate::modules::analyze::{ModuleCouplingInput, build_dependency_summary};

#[derive(Default)]
pub(super) struct TypeScriptDependencySummary {
    targets: BTreeMap<String, usize>,
    internal_files: BTreeSet<PathBuf>,
    external_targets: BTreeSet<String>,
}

impl TypeScriptDependencySummary {
    pub(super) fn observe(
        &mut self,
        root: &Path,
        source_path: &Path,
        text: &str,
        tool_internal_files: &BTreeMap<PathBuf, BTreeMap<String, BTreeSet<PathBuf>>>,
    ) {
        let mut parser = Parser::new();
        if parser
            .set_language(
                &match source_path.extension().and_then(|ext| ext.to_str()) {
                    Some("tsx") => tree_sitter_typescript::LANGUAGE_TSX,
                    _ => tree_sitter_typescript::LANGUAGE_TYPESCRIPT,
                }
                .into(),
            )
            .is_err()
        {
            return;
        }
        let Some(tree) = parser.parse(text, None) else {
            return;
        };
        let mut imports = Vec::new();
        collect_import_sources(tree.root_node(), text.as_bytes(), &mut imports);
        let tool_resolved_files = tool_internal_files.get(source_path);
        for target in imports {
            *self.targets.entry(target.clone()).or_default() += 1;
            if let Some(files) = tool_resolved_files.and_then(|files| files.get(&target)) {
                self.internal_files.extend(files.iter().cloned());
            } else if let Some(file) = resolve_internal_import(root, source_path, &target) {
                self.internal_files.insert(file);
            } else if !target.starts_with('.') {
                self.external_targets.insert(target);
            }
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
}

fn collect_import_sources(node: Node<'_>, source: &[u8], imports: &mut Vec<String>) {
    if node.kind() == "import_statement"
        && let Some(import_source) = node.child_by_field_name("source")
        && let Ok(text) = import_source.utf8_text(source)
    {
        imports.push(text.trim_matches('"').trim_matches('\'').to_string());
    }

    let mut cursor = node.walk();
    for child in node.named_children(&mut cursor) {
        collect_import_sources(child, source, imports);
    }
}

fn resolve_internal_import(root: &Path, source_path: &Path, target: &str) -> Option<PathBuf> {
    if !target.starts_with('.') {
        return None;
    }

    let source_dir = normalize_path(
        &root
            .join(source_path)
            .parent()
            .map(Path::to_path_buf)
            .unwrap_or_else(|| root.to_path_buf()),
    );
    let candidate_base = source_dir.join(target);
    let candidates = [
        candidate_base.with_extension("ts"),
        candidate_base.with_extension("tsx"),
        candidate_base.join("index.ts"),
        candidate_base.join("index.tsx"),
    ];

    candidates
        .into_iter()
        .map(|candidate| normalize_path(&candidate))
        .find(|candidate| candidate.exists())
}

fn normalize_path(path: &Path) -> PathBuf {
    if let Ok(canonical) = path.canonicalize() {
        return canonical;
    }
    let mut normalized = PathBuf::new();
    for component in path.components() {
        match component {
            Component::CurDir => {}
            Component::ParentDir => {
                normalized.pop();
            }
            _ => normalized.push(component.as_os_str()),
        }
    }
    normalized
}

#[cfg(test)]
mod tests {
    use std::collections::BTreeMap;
    use std::fs;
    use std::path::Path;

    use super::TypeScriptDependencySummary;

    #[test]
    fn provider_dependency_summary_tracks_external_and_relative_imports() {
        let root = temp_root("special-ts-dependencies");
        let src = root.join("src");
        fs::create_dir_all(&src).expect("source dir should exist");
        fs::write(src.join("local.ts"), "export const value = 1;\n")
            .expect("local source should be written");

        let mut summary = TypeScriptDependencySummary::default();
        summary.observe(
            &root,
            Path::new("src/app.ts"),
            "import { value } from './local';\nimport * as path from 'node:path';\n",
            &BTreeMap::new(),
        );

        let dependencies = summary.summary();
        assert_eq!(dependencies.reference_count, 2);
        assert_eq!(summary.coupling_input().internal_files.len(), 1);
        assert_eq!(summary.coupling_input().external_targets.len(), 1);

        fs::remove_dir_all(root).expect("temp root should be removed");
    }

    fn temp_root(prefix: &str) -> std::path::PathBuf {
        let unique = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .expect("system clock should be after unix epoch")
            .as_nanos();
        let path = std::env::temp_dir().join(format!("{prefix}-{}-{unique}", std::process::id()));
        fs::create_dir_all(&path).expect("temp root should exist");
        path
    }
}
