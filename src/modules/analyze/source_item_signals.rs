/**
@module SPECIAL.MODULES.ANALYZE.SOURCE_ITEM_SIGNALS
Builds generic per-item connectivity, unreached, complexity, and craftsmanship summaries from normalized source-item graphs so lightweight language providers can share one item-signal implementation.
*/
// @fileimplements SPECIAL.MODULES.ANALYZE.SOURCE_ITEM_SIGNALS
use std::collections::{BTreeMap, BTreeSet, VecDeque};

use crate::model::{ModuleItemKind, ModuleItemSignal, ModuleItemSignalsSummary};
use crate::syntax::{SourceItem, SourceItemKind};

#[derive(Debug, Clone, Copy, Default)]
pub(crate) struct SourceItemMetrics {
    pub(crate) parameter_count: usize,
    pub(crate) bool_parameter_count: usize,
    pub(crate) raw_string_parameter_count: usize,
    pub(crate) cyclomatic: usize,
    pub(crate) cognitive: usize,
    pub(crate) panic_site_count: usize,
}

pub(crate) fn summarize_source_item_signals_with_metrics(
    items: &[SourceItem],
    metrics_by_stable_id: &BTreeMap<String, SourceItemMetrics>,
) -> ModuleItemSignalsSummary {
    let mut records = items
        .iter()
        .map(|item| ItemSignalRecord::from_source_item(item, metrics_by_stable_id))
        .collect::<Vec<_>>();
    let local_names = records
        .iter()
        .map(|item| item.name.clone())
        .collect::<Vec<_>>();
    for item in &mut records {
        item.observe_edges(&local_names);
    }

    let mut inbound_counts: BTreeMap<String, usize> = BTreeMap::new();
    for item in &records {
        for callee in &item.internal_callees {
            *inbound_counts.entry(callee.clone()).or_default() += 1;
        }
    }
    for item in &mut records {
        item.inbound_internal_refs = inbound_counts.get(&item.name).copied().unwrap_or(0);
    }

    let mut connected_items = records
        .iter()
        .filter(|item| item.internal_refs + item.inbound_internal_refs > 0)
        .cloned()
        .collect::<Vec<_>>();
    connected_items.sort_by(|left, right| {
        (right.internal_refs + right.inbound_internal_refs)
            .cmp(&(left.internal_refs + left.inbound_internal_refs))
            .then_with(|| right.internal_refs.cmp(&left.internal_refs))
            .then_with(|| left.name.cmp(&right.name))
    });

    let mut outbound_heavy_items = records
        .iter()
        .filter(|item| item.external_refs > item.internal_refs)
        .cloned()
        .collect::<Vec<_>>();
    outbound_heavy_items.sort_by(|left, right| {
        (right.external_refs as isize - right.internal_refs as isize)
            .cmp(&(left.external_refs as isize - left.internal_refs as isize))
            .then_with(|| right.external_refs.cmp(&left.external_refs))
            .then_with(|| left.name.cmp(&right.name))
    });

    let mut isolated_items = records
        .iter()
        .filter(|item| {
            item.internal_refs == 0 && item.inbound_internal_refs == 0 && item.external_refs > 0
        })
        .cloned()
        .collect::<Vec<_>>();
    isolated_items.sort_by(|left, right| {
        right
            .external_refs
            .cmp(&left.external_refs)
            .then_with(|| left.name.cmp(&right.name))
    });

    let reachable_names = reachable_from_roots(&records);
    let mut unreached_items = records
        .iter()
        .filter(|item| {
            !item.root_visible
                && !item.is_test
                && !reachable_names.iter().any(|name| name == &item.name)
        })
        .cloned()
        .collect::<Vec<_>>();
    unreached_items.sort_by(|left, right| {
        left.name
            .cmp(&right.name)
            .then_with(|| left.kind.cmp(&right.kind))
    });

    let mut highest_complexity_items = records.to_vec();
    highest_complexity_items.sort_by(|left, right| {
        right
            .cognitive
            .cmp(&left.cognitive)
            .then_with(|| right.cyclomatic.cmp(&left.cyclomatic))
            .then_with(|| left.name.cmp(&right.name))
    });
    highest_complexity_items.retain(|item| item.cognitive > 0 || item.cyclomatic > 1);

    let mut parameter_heavy_items = records
        .iter()
        .filter(|item| item.parameter_count > 1)
        .cloned()
        .collect::<Vec<_>>();
    parameter_heavy_items.sort_by(|left, right| {
        right
            .parameter_count
            .cmp(&left.parameter_count)
            .then_with(|| {
                right
                    .raw_string_parameter_count
                    .cmp(&left.raw_string_parameter_count)
            })
            .then_with(|| left.name.cmp(&right.name))
    });

    let mut stringly_boundary_items = records
        .iter()
        .filter(|item| item.public && item.raw_string_parameter_count > 0)
        .cloned()
        .collect::<Vec<_>>();
    stringly_boundary_items.sort_by(|left, right| {
        right
            .raw_string_parameter_count
            .cmp(&left.raw_string_parameter_count)
            .then_with(|| right.parameter_count.cmp(&left.parameter_count))
            .then_with(|| left.name.cmp(&right.name))
    });

    let mut panic_heavy_items = records
        .iter()
        .filter(|item| item.panic_site_count > 0)
        .cloned()
        .collect::<Vec<_>>();
    panic_heavy_items.sort_by(|left, right| {
        right
            .panic_site_count
            .cmp(&left.panic_site_count)
            .then_with(|| right.cognitive.cmp(&left.cognitive))
            .then_with(|| left.name.cmp(&right.name))
    });

    ModuleItemSignalsSummary {
        analyzed_items: records.len(),
        unreached_item_count: unreached_items.len(),
        connected_items: connected_items
            .into_iter()
            .map(ItemSignalRecord::into_summary)
            .collect(),
        outbound_heavy_items: outbound_heavy_items
            .into_iter()
            .map(ItemSignalRecord::into_summary)
            .collect(),
        isolated_items: isolated_items
            .into_iter()
            .map(ItemSignalRecord::into_summary)
            .collect(),
        unreached_items: unreached_items
            .into_iter()
            .map(ItemSignalRecord::into_summary)
            .collect(),
        highest_complexity_items: highest_complexity_items
            .into_iter()
            .map(ItemSignalRecord::into_summary)
            .collect(),
        parameter_heavy_items: parameter_heavy_items
            .into_iter()
            .map(ItemSignalRecord::into_summary)
            .collect(),
        stringly_boundary_items: stringly_boundary_items
            .into_iter()
            .map(ItemSignalRecord::into_summary)
            .collect(),
        panic_heavy_items: panic_heavy_items
            .into_iter()
            .map(ItemSignalRecord::into_summary)
            .collect(),
    }
}

#[derive(Clone)]
struct ItemSignalRecord {
    name: String,
    kind: ModuleItemKind,
    public: bool,
    root_visible: bool,
    is_test: bool,
    parameter_count: usize,
    bool_parameter_count: usize,
    raw_string_parameter_count: usize,
    cyclomatic: usize,
    cognitive: usize,
    panic_site_count: usize,
    internal_refs: usize,
    inbound_internal_refs: usize,
    external_refs: usize,
    internal_callees: Vec<String>,
    observed_call_names: Vec<String>,
}

impl ItemSignalRecord {
    fn from_source_item(
        item: &SourceItem,
        metrics_by_stable_id: &BTreeMap<String, SourceItemMetrics>,
    ) -> Self {
        let metrics = metrics_by_stable_id
            .get(&item.stable_id)
            .copied()
            .unwrap_or_default();
        Self {
            name: item.name.clone(),
            kind: match item.kind {
                SourceItemKind::Function => ModuleItemKind::Function,
                SourceItemKind::Method => ModuleItemKind::Method,
            },
            public: item.public,
            root_visible: item.root_visible || is_process_entrypoint(item),
            is_test: item.is_test,
            parameter_count: metrics.parameter_count,
            bool_parameter_count: metrics.bool_parameter_count,
            raw_string_parameter_count: metrics.raw_string_parameter_count,
            cyclomatic: metrics.cyclomatic,
            cognitive: metrics.cognitive,
            panic_site_count: metrics.panic_site_count,
            internal_refs: 0,
            inbound_internal_refs: 0,
            external_refs: 0,
            internal_callees: Vec::new(),
            observed_call_names: item.calls.iter().map(|call| call.name.clone()).collect(),
        }
    }

    fn observe_edges(&mut self, local_names: &[String]) {
        for call_name in &self.observed_call_names {
            if local_names.iter().any(|name| name == call_name) {
                self.internal_refs += 1;
                self.internal_callees.push(call_name.clone());
            } else {
                self.external_refs += 1;
            }
        }
    }

    fn into_summary(self) -> ModuleItemSignal {
        ModuleItemSignal {
            name: self.name,
            kind: self.kind,
            public: self.public,
            parameter_count: self.parameter_count,
            bool_parameter_count: self.bool_parameter_count,
            raw_string_parameter_count: self.raw_string_parameter_count,
            internal_refs: self.internal_refs,
            inbound_internal_refs: self.inbound_internal_refs,
            external_refs: self.external_refs,
            cyclomatic: self.cyclomatic,
            cognitive: self.cognitive,
            panic_site_count: self.panic_site_count,
        }
    }
}

fn reachable_from_roots(items: &[ItemSignalRecord]) -> Vec<String> {
    let adjacency = items
        .iter()
        .map(|item| (item.name.clone(), item.internal_callees.clone()))
        .collect::<BTreeMap<_, _>>();
    let mut queue = items
        .iter()
        .filter(|item| item.root_visible || item.is_test)
        .map(|item| item.name.clone())
        .collect::<VecDeque<_>>();
    let mut seen = BTreeSet::new();

    while let Some(name) = queue.pop_front() {
        if !seen.insert(name.clone()) {
            continue;
        }
        if let Some(callees) = adjacency.get(&name) {
            queue.extend(callees.iter().cloned());
        }
    }

    seen.into_iter().collect()
}

fn is_process_entrypoint(item: &SourceItem) -> bool {
    item.kind == SourceItemKind::Function && item.name == "main"
}
