/**
@module SPECIAL.MODULES.ANALYZE.COUPLING
Aggregates language-provider dependency evidence into shared module-to-module coupling summaries without inventing architecture verdicts.
*/
// @fileimplements SPECIAL.MODULES.ANALYZE.COUPLING
use std::collections::{BTreeMap, BTreeSet};
use std::path::PathBuf;

use crate::model::{
    ArchitectureKind, ModuleAnalysisSummary, ModuleCouplingSummary, ParsedArchitecture,
};

#[derive(Debug, Clone)]
enum ModuleTargetResolution {
    Unique(String),
    Ambiguous,
}

#[derive(Debug, Clone, Default)]
pub(crate) struct ModuleCouplingInput {
    pub internal_files: BTreeSet<PathBuf>,
    pub external_targets: BTreeSet<String>,
}

pub(crate) fn apply_module_coupling(
    parsed: &ParsedArchitecture,
    inputs: &BTreeMap<String, ModuleCouplingInput>,
    modules: &mut BTreeMap<String, ModuleAnalysisSummary>,
) {
    let module_index = index_file_modules(parsed);
    let concrete_modules: BTreeSet<String> = parsed
        .modules
        .iter()
        .filter(|module| module.kind() == ArchitectureKind::Module)
        .map(|module| module.id.clone())
        .collect();
    let mut outbound: BTreeMap<String, BTreeSet<String>> = BTreeMap::new();
    let mut inbound: BTreeMap<String, BTreeSet<String>> = BTreeMap::new();
    let mut ambiguous_counts: BTreeMap<String, usize> = BTreeMap::new();
    let mut unresolved_counts: BTreeMap<String, usize> = BTreeMap::new();

    for (module_id, input) in inputs {
        for target_file in &input.internal_files {
            match module_index.get(target_file) {
                Some(ModuleTargetResolution::Unique(target_module)) => {
                    if target_module == module_id {
                        continue;
                    }
                    outbound
                        .entry(module_id.clone())
                        .or_default()
                        .insert(target_module.clone());
                    inbound
                        .entry(target_module.clone())
                        .or_default()
                        .insert(module_id.clone());
                }
                Some(ModuleTargetResolution::Ambiguous) => {
                    *ambiguous_counts.entry(module_id.clone()).or_default() += 1;
                }
                None => {
                    *unresolved_counts.entry(module_id.clone()).or_default() += 1;
                }
            }
        }
    }

    for module_id in concrete_modules {
        let fan_out = outbound.get(&module_id).map_or(0, BTreeSet::len);
        let fan_in = inbound.get(&module_id).map_or(0, BTreeSet::len);
        let afferent_coupling = fan_in;
        let efferent_coupling = fan_out;
        let instability = if afferent_coupling + efferent_coupling == 0 {
            0.0
        } else {
            efferent_coupling as f64 / (afferent_coupling + efferent_coupling) as f64
        };
        let external_target_count = inputs
            .get(&module_id)
            .map_or(0, |input| input.external_targets.len());
        let ambiguous_internal_target_count =
            ambiguous_counts.get(&module_id).copied().unwrap_or(0);
        let unresolved_internal_target_count =
            unresolved_counts.get(&module_id).copied().unwrap_or(0);

        if let Some(analysis) = modules.get_mut(&module_id) {
            analysis.coupling = Some(ModuleCouplingSummary {
                fan_in,
                fan_out,
                afferent_coupling,
                efferent_coupling,
                instability,
                external_target_count,
                ambiguous_internal_target_count,
                unresolved_internal_target_count,
            });
        }
    }
}

fn index_file_modules(parsed: &ParsedArchitecture) -> BTreeMap<PathBuf, ModuleTargetResolution> {
    let concrete_modules: BTreeSet<String> = parsed
        .modules
        .iter()
        .filter(|module| module.kind() == ArchitectureKind::Module)
        .map(|module| module.id.clone())
        .collect();
    let mut raw: BTreeMap<PathBuf, BTreeSet<String>> = BTreeMap::new();

    for implementation in &parsed.implements {
        if !concrete_modules.contains(&implementation.module_id) {
            continue;
        }
        raw.entry(implementation.location.path.clone())
            .or_default()
            .insert(implementation.module_id.clone());
    }

    raw.into_iter()
        .filter_map(|(path, module_ids)| {
            if module_ids.is_empty() {
                return None;
            }
            Some((
                path,
                if module_ids.len() == 1 {
                    ModuleTargetResolution::Unique(
                        module_ids
                            .into_iter()
                            .next()
                            .expect("single module id should be present"),
                    )
                } else {
                    ModuleTargetResolution::Ambiguous
                },
            ))
        })
        .collect()
}
