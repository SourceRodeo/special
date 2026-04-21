/**
@module SPECIAL.TESTS.INDEX.SUPPORT
Shared index test helpers and temporary repository fixtures in `src/index/tests/support.rs`.
*/
// @fileimplements SPECIAL.TESTS.INDEX.SUPPORT
use std::fs;
use std::path::PathBuf;
use std::time::{SystemTime, UNIX_EPOCH};

use crate::config::SpecialVersion;
use crate::model::{
    AttestRef, AttestScope, DeclaredStateFilter, NodeKind, ParsedRepo, PlanState, SourceLocation,
    SpecDecl, SpecDocument, SpecFilter, VerifyRef,
};

use super::super::{build_spec_document, lint_from_parsed, materialize_spec};

pub(super) fn spec_decl(
    id: &str,
    kind: NodeKind,
    text: &str,
    planned: bool,
    line: usize,
) -> SpecDecl {
    let plan = if planned {
        PlanState::planned(None)
    } else {
        PlanState::current()
    };
    SpecDecl::new(
        id.to_string(),
        kind,
        text.to_string(),
        plan,
        false,
        None,
        SourceLocation {
            path: "src/lib.rs".into(),
            line,
        },
    )
    .expect("test helper should construct valid spec decls")
}

pub(super) fn group_decl(id: &str, text: &str, path: &str, line: usize) -> SpecDecl {
    SpecDecl::new(
        id.to_string(),
        NodeKind::Group,
        text.to_string(),
        PlanState::current(),
        false,
        None,
        SourceLocation {
            path: path.into(),
            line,
        },
    )
    .expect("test should construct valid group decl")
}

pub(super) fn verify_ref(spec_id: &str, path: &str, line: usize, body: &str) -> VerifyRef {
    VerifyRef {
        spec_id: spec_id.to_string(),
        location: SourceLocation {
            path: path.into(),
            line,
        },
        body_location: None,
        body: Some(body.to_string()),
    }
}

pub(super) fn block_attest_ref(spec_id: &str, path: &str, line: usize, body: &str) -> AttestRef {
    AttestRef {
        spec_id: spec_id.to_string(),
        artifact: "docs/report.pdf".to_string(),
        owner: "security".to_string(),
        last_reviewed: "2026-04-12".to_string(),
        review_interval_days: None,
        scope: AttestScope::Block,
        location: SourceLocation {
            path: path.into(),
            line,
        },
        body: Some(body.to_string()),
    }
}

pub(super) fn file_attest_ref(spec_id: &str, path: &str, line: usize, body: &str) -> AttestRef {
    AttestRef {
        spec_id: spec_id.to_string(),
        artifact: "docs/review.md".to_string(),
        owner: "security".to_string(),
        last_reviewed: "2026-04-12".to_string(),
        review_interval_days: None,
        scope: AttestScope::File,
        location: SourceLocation {
            path: path.into(),
            line,
        },
        body: Some(body.to_string()),
    }
}

pub(super) fn parsed_repo(
    specs: Vec<SpecDecl>,
    verifies: Vec<VerifyRef>,
    attests: Vec<AttestRef>,
) -> ParsedRepo {
    ParsedRepo {
        specs,
        verifies,
        attests,
        diagnostics: Vec::new(),
    }
}

pub(super) fn materialize_current(parsed: &ParsedRepo) -> SpecDocument {
    materialize_spec(
        parsed,
        SpecFilter {
            state: DeclaredStateFilter::Current,
            unverified_only: false,
            scope: None,
        },
        false,
        None,
    )
}

pub(super) fn materialize_all(parsed: &ParsedRepo) -> SpecDocument {
    materialize_spec(
        parsed,
        SpecFilter {
            state: DeclaredStateFilter::All,
            unverified_only: false,
            scope: None,
        },
        false,
        None,
    )
}

pub(super) fn materialize_unverified_current(parsed: &ParsedRepo) -> SpecDocument {
    materialize_spec(
        parsed,
        SpecFilter {
            state: DeclaredStateFilter::Current,
            unverified_only: true,
            scope: None,
        },
        false,
        None,
    )
}

pub(super) fn lint(parsed: &ParsedRepo) -> crate::model::LintReport {
    lint_from_parsed(parsed)
}

pub(super) struct TempRepo {
    root: PathBuf,
}

impl TempRepo {
    pub(super) fn new(prefix: &str) -> Self {
        let unique = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("time should move forward")
            .as_nanos();
        let root = std::env::temp_dir().join(format!("{prefix}-{unique}"));
        fs::create_dir_all(&root).expect("temp repo dir should be created");
        Self { root }
    }

    pub(super) fn write(&self, relative: &str, content: &str) {
        let path = self.root.join(relative);
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent).expect("fixture parent dir should be created");
        }
        fs::write(path, content).expect("fixture should be written");
    }

    pub(super) fn build_current_document(&self) -> (SpecDocument, crate::model::LintReport) {
        build_spec_document(
            &self.root,
            &[],
            SpecialVersion::V1,
            SpecFilter {
                state: DeclaredStateFilter::Current,
                unverified_only: false,
                scope: None,
            },
            false,
        )
        .expect("document build should succeed")
    }
}

impl Drop for TempRepo {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.root);
    }
}
