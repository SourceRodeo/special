/**
@module SPECIAL.MODULES.ANALYZE.MODULE_SUMMARY
Owns module-facing analysis aggregation over provider results and concrete ownership attachments.
*/
// @fileimplements SPECIAL.MODULES.ANALYZE.MODULE_SUMMARY
use std::collections::BTreeMap;
use std::path::{Path, PathBuf};

use anyhow::Result;

use crate::model::{ModuleAnalysisOptions, ModuleAnalysisSummary, ParsedArchitecture};

use super::{FileOwnership, coupling, registry};

#[path = "module_summary/coverage.rs"]
mod coverage;
#[path = "module_summary/per_module.rs"]
mod per_module;
#[path = "module_summary/provider.rs"]
mod provider;

pub(super) fn build_module_analysis(
    root: &Path,
    parsed: &ParsedArchitecture,
    file_ownership: &BTreeMap<PathBuf, FileOwnership<'_>>,
    repo_contexts: &registry::RepoAnalysisContexts,
    options: ModuleAnalysisOptions,
) -> Result<BTreeMap<String, ModuleAnalysisSummary>> {
    let mut modules = BTreeMap::new();
    let mut coupling_inputs: BTreeMap<String, coupling::ModuleCouplingInput> = BTreeMap::new();

    for module in &parsed.modules {
        if let Some((module_id, analysis, coupling_input)) = per_module::build_module_summary(
            root,
            module,
            parsed,
            file_ownership,
            repo_contexts,
            options,
        )? {
            if let Some(coupling_input) = coupling_input {
                coupling_inputs.insert(module_id.clone(), coupling_input);
            }
            modules.insert(module_id, analysis);
        }
    }

    if options.metrics {
        coupling::apply_module_coupling(parsed, &coupling_inputs, &mut modules);
    }

    Ok(modules)
}
