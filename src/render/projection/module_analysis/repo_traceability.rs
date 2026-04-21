/**
@module SPECIAL.RENDER.PROJECTION.MODULE_ANALYSIS.REPO_TRACEABILITY
Projects repo-level backward-traceability summaries into shared count and detail lines for renderers.
*/
// @fileimplements SPECIAL.RENDER.PROJECTION.MODULE_ANALYSIS.REPO_TRACEABILITY
use crate::model::{ArchitectureTraceabilityItem, ArchitectureTraceabilitySummary};

use super::{
    ProjectedArchitectureTraceability, ProjectedCount, ProjectedExplanation, ProjectedMetaLine,
    count,
};

pub(in crate::render) fn project_repo_traceability_view(
    traceability: Option<&ArchitectureTraceabilitySummary>,
    unavailable_reason: Option<&str>,
) -> ProjectedArchitectureTraceability {
    let Some(traceability) = traceability else {
        return ProjectedArchitectureTraceability {
            counts: Vec::new(),
            explanations: Vec::new(),
            items: Vec::new(),
            unavailable_reason: unavailable_reason.map(ToString::to_string),
        };
    };
    let mut counts = vec![count(
        "traceability items analyzed",
        traceability.analyzed_items,
    )];
    let mut explanations = vec![ProjectedExplanation {
        label: "traceability items analyzed",
        plain: "this counts analyzable implementation items considered by backward trace in the current health view.",
        precise: "count of non-test implementation items included in repo traceability analysis for the active language pack.",
    }];
    if !traceability.current_spec_items.is_empty() {
        push_repo_traceability_count(
            &mut counts,
            &mut explanations,
            "current spec items",
            traceability.current_spec_items.len(),
            "these items have a traced path from verifying tests tied to current specs.",
            "count of analyzed implementation items reached from at least one test with item- or file-scoped support for a current spec.",
        );
    }
    if !traceability.planned_only_items.is_empty() {
        push_repo_traceability_count(
            &mut counts,
            &mut explanations,
            "planned-only items",
            traceability.planned_only_items.len(),
            "these items only trace to planned specs, not current ones.",
            "count of analyzed implementation items reached only from tests tied to planned specs and from no tests tied to current specs.",
        );
    }
    if !traceability.deprecated_only_items.is_empty() {
        push_repo_traceability_count(
            &mut counts,
            &mut explanations,
            "deprecated-only items",
            traceability.deprecated_only_items.len(),
            "these items only trace to deprecated specs, not current ones.",
            "count of analyzed implementation items reached only from tests tied to deprecated specs and from no tests tied to current or planned specs.",
        );
    }
    if !traceability.file_scoped_only_items.is_empty() {
        push_repo_traceability_count(
            &mut counts,
            &mut explanations,
            "file-scoped-only items",
            traceability.file_scoped_only_items.len(),
            "these items are justified only by file-scoped verification, not item-scoped verification.",
            "count of analyzed implementation items reached from tests with file-scoped support and with no item-scoped support attached to the item itself.",
        );
    }
    if !traceability.unverified_test_items.is_empty() {
        push_repo_traceability_count(
            &mut counts,
            &mut explanations,
            "unverified-test items",
            traceability.unverified_test_items.len(),
            "these items are touched by tests that are not tied to specs.",
            "count of analyzed implementation items reached from tests with no attached current, planned, or deprecated spec support.",
        );
    }
    if !traceability.statically_mediated_items.is_empty() {
        push_repo_traceability_count(
            &mut counts,
            &mut explanations,
            "statically mediated items",
            traceability.statically_mediated_items.len(),
            "these items are justified by a language-level static entry shape rather than a direct call chain.",
            "count of analyzed implementation items classified as traceable through mediated static edges such as trait or interface entrypoints.",
        );
    }
    if !traceability.unexplained_items.is_empty() {
        push_repo_traceability_count(
            &mut counts,
            &mut explanations,
            "unexplained items",
            traceability.unexplained_items.len(),
            "these items are still outside the currently explained spec-backed trace set.",
            "count of analyzed implementation items not classified as current, planned-only, deprecated-only, unverified-test, or statically mediated.",
        );
        for detail in unexplained_traceability_details(traceability) {
            push_repo_traceability_count(
                &mut counts,
                &mut explanations,
                detail.label,
                detail.value,
                detail.plain,
                detail.precise,
            );
        }
    }

    let mut items = Vec::new();
    push_architecture_traceability_group(
        &mut items,
        "current spec item",
        &traceability.current_spec_items,
    );
    push_architecture_traceability_group(
        &mut items,
        "planned-only item",
        &traceability.planned_only_items,
    );
    push_architecture_traceability_group(
        &mut items,
        "deprecated-only item",
        &traceability.deprecated_only_items,
    );
    push_architecture_traceability_group(
        &mut items,
        "file-scoped-only item",
        &traceability.file_scoped_only_items,
    );
    push_architecture_traceability_group(
        &mut items,
        "unverified-test item",
        &traceability.unverified_test_items,
    );
    push_architecture_traceability_group(
        &mut items,
        "statically mediated item",
        &traceability.statically_mediated_items,
    );
    push_architecture_traceability_group(
        &mut items,
        "unexplained item",
        &traceability.unexplained_items,
    );

    ProjectedArchitectureTraceability {
        counts,
        explanations,
        items,
        unavailable_reason: unavailable_reason.map(ToString::to_string),
    }
}

struct RepoTraceabilityDetail {
    label: &'static str,
    value: usize,
    plain: &'static str,
    precise: &'static str,
}

fn push_repo_traceability_count(
    counts: &mut Vec<ProjectedCount>,
    explanations: &mut Vec<ProjectedExplanation>,
    label: &'static str,
    value: usize,
    plain: &'static str,
    precise: &'static str,
) {
    counts.push(count(label, value));
    explanations.push(ProjectedExplanation {
        label,
        plain,
        precise,
    });
}

fn unexplained_traceability_details(
    traceability: &ArchitectureTraceabilitySummary,
) -> [RepoTraceabilityDetail; 9] {
    [
        RepoTraceabilityDetail {
            label: "unexplained review-surface items",
            value: traceability.unexplained_review_surface_items(),
            plain: "these unexplained items are the main review pile: public API or root-visible entrypoints that behave like product surface.",
            precise: "count of unexplained implementation items marked public or root-visible by the active language pack, including process entrypoints such as `main`.",
        },
        RepoTraceabilityDetail {
            label: "unexplained public items",
            value: traceability.unexplained_public_items(),
            plain: "these unexplained items are public entrypoints or exported API surface.",
            precise: "count of unexplained implementation items marked public by the active language pack.",
        },
        RepoTraceabilityDetail {
            label: "unexplained internal items",
            value: traceability.unexplained_internal_items(),
            plain: "these unexplained items are internal implementation, not public API.",
            precise: "count of unexplained implementation items not marked public by the active language pack.",
        },
        RepoTraceabilityDetail {
            label: "unexplained test-file items",
            value: traceability.unexplained_test_file_items(),
            plain: "these unexplained items sit in files recognized as test files.",
            precise: "count of unexplained implementation items whose source path is under a tests directory or in a file named tests.",
        },
        RepoTraceabilityDetail {
            label: "unexplained module-owned items",
            value: traceability.unexplained_module_owned_items(),
            plain: "these unexplained items still belong to at least one declared module.",
            precise: "count of unexplained implementation items with one or more declared owning module ids.",
        },
        RepoTraceabilityDetail {
            label: "unexplained module-backed items",
            value: traceability.unexplained_module_backed_items(),
            plain: "these unexplained items sit in modules that already have current-spec-traced code somewhere else.",
            precise: "count of unexplained implementation items whose declared owning module ids include at least one module with current-spec-backed traced implementation.",
        },
        RepoTraceabilityDetail {
            label: "unexplained module-connected items",
            value: traceability.unexplained_module_connected_items(),
            plain: "these unexplained items also connect inside those modules to code that is already current-spec-traced.",
            precise: "count of unexplained implementation items in current-spec-backed modules that share a same-module call or reference component with current-spec-traced implementation.",
        },
        RepoTraceabilityDetail {
            label: "unexplained module-isolated items",
            value: traceability.unexplained_module_isolated_items(),
            plain: "these unexplained items are in current-spec-backed modules but still sit outside the connected traced cluster in those modules.",
            precise: "count of unexplained implementation items in current-spec-backed modules that do not share a same-module call or reference component with current-spec-traced implementation.",
        },
        RepoTraceabilityDetail {
            label: "unexplained unowned items",
            value: traceability.unexplained_unowned_items(),
            plain: "these unexplained items are outside all declared modules.",
            precise: "count of unexplained implementation items with no declared owning module ids.",
        },
    ]
}

fn push_architecture_traceability_group(
    meta_lines: &mut Vec<ProjectedMetaLine>,
    label: &'static str,
    items: &[ArchitectureTraceabilityItem],
) {
    meta_lines.extend(items.iter().map(|item| ProjectedMetaLine {
        label,
        value: architecture_traceability_value(item),
    }));
}

fn architecture_traceability_value(item: &ArchitectureTraceabilityItem) -> String {
    let mut suffix = Vec::new();
    suffix.push(if item.review_surface {
        if item.public {
            "review surface; public".to_string()
        } else {
            "review surface; root-visible entrypoint".to_string()
        }
    } else {
        "internal".to_string()
    });
    if item.module_backed_by_current_specs {
        suffix.push("module-backed".to_string());
        suffix.push(if item.module_connected_to_current_specs {
            "connected inside module".to_string()
        } else {
            "isolated inside module".to_string()
        });
    }
    if item.test_file {
        suffix.push("test file".to_string());
    }
    if item.module_ids.is_empty() {
        suffix.push("unowned".to_string());
    } else {
        suffix.push(format!("modules {}", item.module_ids.join(", ")));
    }
    if !item.current_specs.is_empty() {
        suffix.push(format!("current specs {}", item.current_specs.join(", ")));
    }
    if !item.planned_specs.is_empty() {
        suffix.push(format!("planned specs {}", item.planned_specs.join(", ")));
    }
    if !item.deprecated_specs.is_empty() {
        suffix.push(format!(
            "deprecated specs {}",
            item.deprecated_specs.join(", ")
        ));
    }
    if !item.verifying_tests.is_empty() {
        suffix.push(format!(
            "verifying tests {}",
            item.verifying_tests.join(", ")
        ));
    }
    if !item.unverified_tests.is_empty() {
        suffix.push(format!(
            "unverified tests {}",
            item.unverified_tests.join(", ")
        ));
    }
    if let Some(reason) = &item.mediated_reason {
        suffix.push(format!("mediated reason {reason}"));
    }
    let base = format!("{}:{}", item.path.display(), item.name);
    if suffix.is_empty() {
        base
    } else {
        format!("{base} [{}]", suffix.join("; "))
    }
}
