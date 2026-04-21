/**
@module SPECIAL.MODULES.ANALYZE.OWNERSHIP
Owns file-level implementation joins and shared owned-text access for analysis and language packs.
*/
// @fileimplements SPECIAL.MODULES.ANALYZE.OWNERSHIP
use std::collections::BTreeMap;
use std::fs;
use std::path::{Path, PathBuf};

use anyhow::{Context, Result};

use crate::model::{ImplementRef, ModuleMetricsSummary, ParsedArchitecture};

#[derive(Debug, Default)]
pub(crate) struct FileOwnership<'a> {
    pub(crate) file_scoped: Vec<&'a ImplementRef>,
    pub(crate) item_scoped: Vec<&'a ImplementRef>,
}

pub(super) fn index_file_ownership<'a>(
    parsed: &'a ParsedArchitecture,
) -> BTreeMap<PathBuf, FileOwnership<'a>> {
    let mut files: BTreeMap<PathBuf, FileOwnership<'a>> = BTreeMap::new();
    for implementation in &parsed.implements {
        let entry = files
            .entry(implementation.location.path.clone())
            .or_default();
        if implementation.body_location.is_some() {
            entry.item_scoped.push(implementation);
        } else {
            entry.file_scoped.push(implementation);
        }
    }
    files
}

pub(crate) fn display_path(root: &Path, path: &Path) -> PathBuf {
    path.strip_prefix(root)
        .map(Path::to_path_buf)
        .unwrap_or_else(|_| path.to_path_buf())
}

pub(super) fn build_module_metrics(
    root: &Path,
    implementations: &[&ImplementRef],
    file_ownership: &BTreeMap<PathBuf, FileOwnership<'_>>,
) -> Result<ModuleMetricsSummary> {
    let mut summary = ModuleMetricsSummary::default();
    visit_owned_texts(root, implementations, file_ownership, |_path, text| {
        summary.owned_lines += line_count(text);
        Ok(())
    })?;
    Ok(summary)
}

pub(crate) fn visit_owned_texts<F>(
    root: &Path,
    implementations: &[&ImplementRef],
    file_ownership: &BTreeMap<PathBuf, FileOwnership<'_>>,
    mut visit: F,
) -> Result<()>
where
    F: FnMut(&Path, &str) -> Result<()>,
{
    for implementation in implementations {
        let path = &implementation.location.path;
        match &implementation.body {
            Some(body) => visit(path, body)?,
            None => {
                let Some(ownership) = file_ownership.get(path) else {
                    continue;
                };
                if !ownership.item_scoped.is_empty() {
                    continue;
                }
                let content = read_owned_file_text(root, path)?;
                visit(path, &content)?;
            }
        }
    }
    Ok(())
}

pub(crate) fn read_owned_file_text(root: &Path, path: &Path) -> Result<String> {
    let full_path = root.join(path);
    fs::read_to_string(&full_path)
        .or_else(|_| fs::read_to_string(path))
        .with_context(|| {
            format!(
                "failed to read owned file {}",
                display_path(root, path).display()
            )
        })
}

fn line_count(text: &str) -> usize {
    text.lines().count().max(1)
}
