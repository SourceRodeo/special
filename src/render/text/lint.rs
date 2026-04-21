/**
@module SPECIAL.RENDER.TEXT.LINT
Renders lint diagnostics into human-readable text output.
*/
// @fileimplements SPECIAL.RENDER.TEXT.LINT
use askama::Template;

use crate::model::{DiagnosticSeverity, LintReport};
use crate::render::templates::render_template;

#[derive(Template)]
#[template(path = "render/lint.txt", escape = "none")]
struct LintTextTemplate<'a> {
    report: &'a LintReport,
}

impl LintTextTemplate<'_> {
    fn severity_label(&self, severity: &DiagnosticSeverity) -> &'static str {
        match severity {
            DiagnosticSeverity::Warning => "warning",
            DiagnosticSeverity::Error => "error",
        }
    }
}

pub(in crate::render) fn render_lint_text(report: &LintReport) -> String {
    if report.diagnostics.is_empty() {
        return "Lint clean.".to_string();
    }

    render_template(&LintTextTemplate { report })
        .trim_end()
        .to_string()
}
