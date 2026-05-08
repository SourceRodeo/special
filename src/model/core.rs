/**
@module SPECIAL.MODEL.CORE
Shared core domain primitives for spec, architecture, and report modeling.
*/
// @fileimplements SPECIAL.MODEL.CORE
use std::fmt;
use std::path::PathBuf;
use std::sync::Arc;

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, PartialOrd, Ord)]
#[serde(rename_all = "snake_case")]
pub enum DiagnosticSeverity {
    Error,
    Warning,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum NodeKind {
    Spec,
    Group,
}

impl NodeKind {
    pub(crate) fn as_annotation(self) -> &'static str {
        match self {
            Self::Spec => "@spec",
            Self::Group => "@group",
        }
    }
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum ArchitectureKind {
    Module,
    Area,
}

impl ArchitectureKind {
    pub fn as_annotation(self) -> &'static str {
        match self {
            Self::Module => "@module",
            Self::Area => "@area",
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct SourceLocation {
    pub path: PathBuf,
    pub line: usize,
}

#[derive(Debug, Clone)]
pub struct OwnedItem {
    pub location: SourceLocation,
    pub body: String,
}

#[derive(Debug, Clone)]
pub struct CommentBlock {
    pub path: PathBuf,
    pub lines: Vec<BlockLine>,
    pub owned_item: Option<OwnedItem>,
    pub source_body: Option<Arc<str>>,
}

#[derive(Debug, Clone)]
pub struct BlockLine {
    pub line: usize,
    pub text: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PlannedRelease(String);

impl PlannedRelease {
    pub fn new(value: impl Into<String>) -> Result<Self, ModelInvariantError> {
        let value = value.into();
        let trimmed = value.trim();
        if trimmed.is_empty() {
            return Err(ModelInvariantError::empty_planned_release());
        }
        Ok(Self(trimmed.to_string()))
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DeprecatedRelease(String);

impl DeprecatedRelease {
    pub fn new(value: impl Into<String>) -> Result<Self, ModelInvariantError> {
        let value = value.into();
        let trimmed = value.trim();
        if trimmed.is_empty() {
            return Err(ModelInvariantError::empty_deprecated_release());
        }
        Ok(Self(trimmed.to_string()))
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ModelInvariantError {
    message: String,
}

impl ModelInvariantError {
    pub(crate) fn planned_group(kind: NodeKind) -> Self {
        Self {
            message: format!("`{}` nodes may not be planned", kind.as_annotation()),
        }
    }

    pub(crate) fn empty_planned_release() -> Self {
        Self {
            message: "planned release metadata must not be empty".to_string(),
        }
    }

    pub(crate) fn deprecated_group(kind: NodeKind) -> Self {
        Self {
            message: format!("`{}` nodes may not be deprecated", kind.as_annotation()),
        }
    }

    pub(crate) fn empty_deprecated_release() -> Self {
        Self {
            message: "deprecated release metadata must not be empty".to_string(),
        }
    }

    pub(crate) fn conflicting_spec_lifecycle() -> Self {
        Self {
            message: "@spec may not be both planned and deprecated".to_string(),
        }
    }

    pub(crate) fn deprecated_release_without_deprecated() -> Self {
        Self {
            message: "@deprecated release metadata requires @deprecated".to_string(),
        }
    }

    pub(crate) fn planned_area() -> Self {
        Self {
            message: "`@area` nodes may not be planned".to_string(),
        }
    }
}

impl fmt::Display for ModelInvariantError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.message)
    }
}

impl std::error::Error for ModelInvariantError {}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum PlanState {
    Current,
    Planned { release: Option<PlannedRelease> },
}

impl PlanState {
    pub fn current() -> Self {
        Self::Current
    }

    pub fn planned(release: Option<PlannedRelease>) -> Self {
        Self::Planned { release }
    }

    pub fn is_planned(&self) -> bool {
        matches!(self, Self::Planned { .. })
    }

    pub fn release(&self) -> Option<&str> {
        match self {
            Self::Current => None,
            Self::Planned { release } => release.as_ref().map(PlannedRelease::as_str),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DeclaredStateFilter {
    All,
    Current,
    Planned,
}
