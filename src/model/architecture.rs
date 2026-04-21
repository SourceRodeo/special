/**
@module SPECIAL.MODEL.ARCHITECTURE
Architecture declaration, attachment, and rendered module-tree domain types.
*/
// @fileimplements SPECIAL.MODEL.ARCHITECTURE
use serde::ser::SerializeStruct;
use serde::{Deserialize, Serialize, Serializer};

use super::{
    ArchitectureKind, DeclaredStateFilter, ModelInvariantError, ModuleAnalysisSummary, PlanState,
    SourceLocation,
};

#[derive(Debug, Default, Clone)]
pub struct ParsedArchitecture {
    pub modules: Vec<ModuleDecl>,
    pub implements: Vec<ImplementRef>,
    pub diagnostics: Vec<super::Diagnostic>,
}

#[derive(Debug, Clone)]
pub struct ModuleDecl {
    pub id: String,
    kind: ArchitectureKind,
    pub text: String,
    plan: PlanState,
    pub location: SourceLocation,
}

impl ModuleDecl {
    pub fn new(
        id: String,
        kind: ArchitectureKind,
        text: String,
        plan: PlanState,
        location: SourceLocation,
    ) -> Result<Self, ModelInvariantError> {
        ensure_valid_architecture_plan(kind, &plan)?;
        Ok(Self {
            id,
            kind,
            text,
            plan,
            location,
        })
    }

    pub fn is_planned(&self) -> bool {
        self.plan.is_planned()
    }

    pub fn kind(&self) -> ArchitectureKind {
        self.kind
    }

    pub fn plan(&self) -> &PlanState {
        &self.plan
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ImplementRef {
    pub module_id: String,
    pub location: SourceLocation,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub body_location: Option<SourceLocation>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub body: Option<String>,
}

#[derive(Debug, Clone)]
pub struct ModuleNode {
    pub id: String,
    kind: ArchitectureKind,
    pub text: String,
    plan: PlanState,
    pub location: SourceLocation,
    pub implements: Vec<ImplementRef>,
    pub analysis: Option<ModuleAnalysisSummary>,
    pub children: Vec<ModuleNode>,
}

impl ModuleNode {
    pub fn new(
        decl: ModuleDecl,
        implements: Vec<ImplementRef>,
        analysis: Option<ModuleAnalysisSummary>,
        children: Vec<ModuleNode>,
    ) -> Self {
        Self {
            id: decl.id,
            kind: decl.kind,
            text: decl.text,
            plan: decl.plan,
            location: decl.location,
            implements,
            analysis,
            children,
        }
    }

    pub(crate) fn is_planned(&self) -> bool {
        self.plan.is_planned()
    }

    pub(crate) fn kind(&self) -> ArchitectureKind {
        self.kind
    }

    pub(crate) fn planned_release(&self) -> Option<&str> {
        self.plan.release()
    }

    pub(crate) fn is_unimplemented(&self) -> bool {
        self.kind == ArchitectureKind::Module && !self.is_planned() && self.implements.is_empty()
    }
}

impl Serialize for ModuleNode {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let mut state = serializer.serialize_struct("ModuleNode", 9)?;
        state.serialize_field("id", &self.id)?;
        state.serialize_field("kind", &self.kind)?;
        state.serialize_field("text", &self.text)?;
        state.serialize_field("planned", &self.is_planned())?;
        if let Some(planned_release) = self.planned_release() {
            state.serialize_field("planned_release", planned_release)?;
        }
        state.serialize_field("location", &self.location)?;
        state.serialize_field("implements", &self.implements)?;
        if let Some(analysis) = &self.analysis {
            state.serialize_field("analysis", analysis)?;
        }
        state.serialize_field("children", &self.children)?;
        state.end()
    }
}

#[derive(Debug, Clone)]
pub struct ModuleFilter {
    pub state: DeclaredStateFilter,
    pub unimplemented_only: bool,
    pub scope: Option<String>,
}

fn ensure_valid_architecture_plan(
    kind: ArchitectureKind,
    plan: &PlanState,
) -> Result<(), ModelInvariantError> {
    if kind == ArchitectureKind::Area && plan.is_planned() {
        return Err(ModelInvariantError::planned_area());
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::{ArchitectureKind, ModuleDecl, PlanState, SourceLocation};

    #[test]
    fn rejects_planned_areas_at_construction_time() {
        let error = ModuleDecl::new(
            "SPECIAL.AREA".to_string(),
            ArchitectureKind::Area,
            "Structural area.".to_string(),
            PlanState::planned(None),
            SourceLocation {
                path: "ARCHITECTURE.md".into(),
                line: 1,
            },
        )
        .expect_err("areas should not accept planned state");

        assert_eq!(error.to_string(), "`@area` nodes may not be planned");
    }
}
