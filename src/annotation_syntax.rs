/**
@module SPECIAL.ANNOTATION_SYNTAX
Shared recognition rules for reserved `special` annotations and foreign tag boundaries inside ordinary comments. This module does not extract comments or build spec or module trees.
*/
// @fileimplements SPECIAL.ANNOTATION_SYNTAX
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum ReservedSpecialAnnotation {
    Spec,
    Group,
    Planned,
    Deprecated,
    Verifies,
    FileVerifies,
    Attests,
    FileAttests,
    Documents,
    FileDocuments,
    Module,
    Area,
    Implements,
    FileImplements,
    Pattern,
    Strictness,
    Applies,
    FileApplies,
}

impl ReservedSpecialAnnotation {
    fn keyword(self) -> &'static str {
        match self {
            Self::Spec => "@spec",
            Self::Group => "@group",
            Self::Planned => "@planned",
            Self::Deprecated => "@deprecated",
            Self::Verifies => "@verifies",
            Self::FileVerifies => "@fileverifies",
            Self::Attests => "@attests",
            Self::FileAttests => "@fileattests",
            Self::Documents => "@documents",
            Self::FileDocuments => "@filedocuments",
            Self::Module => "@module",
            Self::Area => "@area",
            Self::Implements => "@implements",
            Self::FileImplements => "@fileimplements",
            Self::Pattern => "@pattern",
            Self::Strictness => "@strictness",
            Self::Applies => "@applies",
            Self::FileApplies => "@fileapplies",
        }
    }
}

const RESERVED_SPECIAL_ANNOTATIONS: &[ReservedSpecialAnnotation] = &[
    ReservedSpecialAnnotation::Spec,
    ReservedSpecialAnnotation::Group,
    ReservedSpecialAnnotation::Planned,
    ReservedSpecialAnnotation::Deprecated,
    ReservedSpecialAnnotation::Verifies,
    ReservedSpecialAnnotation::FileVerifies,
    ReservedSpecialAnnotation::Attests,
    ReservedSpecialAnnotation::FileAttests,
    ReservedSpecialAnnotation::Documents,
    ReservedSpecialAnnotation::FileDocuments,
    ReservedSpecialAnnotation::Module,
    ReservedSpecialAnnotation::Area,
    ReservedSpecialAnnotation::Implements,
    ReservedSpecialAnnotation::FileImplements,
    ReservedSpecialAnnotation::Pattern,
    ReservedSpecialAnnotation::Strictness,
    ReservedSpecialAnnotation::Applies,
    ReservedSpecialAnnotation::FileApplies,
];

pub(crate) fn reserved_special_annotation(text: &str) -> Option<ReservedSpecialAnnotation> {
    let trimmed = text.trim();
    RESERVED_SPECIAL_ANNOTATIONS
        .iter()
        .copied()
        .find(|annotation| reserved_special_annotation_rest(trimmed, *annotation).is_some())
}

pub(crate) fn reserved_special_annotation_rest(
    text: &str,
    annotation: ReservedSpecialAnnotation,
) -> Option<&str> {
    parse_source_annotations::directive_rest(text, annotation.keyword())
}

pub(crate) fn is_reserved_special_annotation(text: &str) -> bool {
    reserved_special_annotation(text).is_some()
}

pub(crate) fn is_foreign_tag_boundary(text: &str) -> bool {
    let trimmed = text.trim();
    if reserved_special_annotation(trimmed).is_some() {
        return false;
    }
    starts_with_tag_like_boundary(trimmed)
}

pub(crate) fn is_any_tag_boundary(text: &str) -> bool {
    is_reserved_special_annotation(text) || is_foreign_tag_boundary(text)
}

pub(crate) fn normalize_markdown_annotation_line(line: &str) -> Option<&str> {
    if special_annotation_starts_inside_code_span(line) {
        return None;
    }
    parse_source_annotations::markdown_annotation_line(line).map(|line| line.text)
}

pub(crate) fn normalize_markdown_declaration_line(line: &str) -> Option<&str> {
    if special_annotation_starts_inside_code_span(line) {
        return None;
    }
    parse_source_annotations::markdown_declaration_line(line)
}

fn starts_with_tag_like_boundary(text: &str) -> bool {
    parse_source_annotations::starts_with_tag_like_boundary(text)
}

fn special_annotation_starts_inside_code_span(line: &str) -> bool {
    let Some(at_index) = line.find('@') else {
        return false;
    };
    let mut in_code_span = false;
    let mut escaped = false;

    for (index, ch) in line.char_indices() {
        if index >= at_index {
            break;
        }
        if escaped {
            escaped = false;
            continue;
        }
        if ch == '\\' {
            escaped = true;
            continue;
        }
        if ch == '`' {
            in_code_span = !in_code_span;
        }
    }

    in_code_span
}

#[cfg(test)]
mod tests {
    use super::{
        ReservedSpecialAnnotation, is_any_tag_boundary, is_foreign_tag_boundary,
        is_reserved_special_annotation, normalize_markdown_annotation_line,
        normalize_markdown_declaration_line, reserved_special_annotation,
        reserved_special_annotation_rest,
    };

    #[test]
    fn recognizes_reserved_special_annotations() {
        assert!(is_reserved_special_annotation("@spec EXPORT.CSV"));
        assert!(is_reserved_special_annotation("@planned 0.4.0"));
        assert!(is_reserved_special_annotation("@deprecated 0.6.0"));
        assert!(is_reserved_special_annotation("@module SPECIAL.RENDER"));
        assert!(is_reserved_special_annotation("@implements SPECIAL.RENDER"));
        assert!(is_reserved_special_annotation(
            "@fileimplements SPECIAL.RENDER"
        ));
        assert!(is_reserved_special_annotation(
            "@pattern APP.READ_EDIT_PANEL"
        ));
        assert!(is_reserved_special_annotation("@strictness high"));
        assert!(is_reserved_special_annotation(
            "@applies APP.READ_EDIT_PANEL"
        ));
        assert!(is_reserved_special_annotation("@fileverifies EXPORT.CSV"));
        assert!(is_reserved_special_annotation("@fileattests EXPORT.CSV"));
        assert!(is_reserved_special_annotation("@documents spec EXPORT.CSV"));
        assert!(is_reserved_special_annotation(
            "@filedocuments pattern CACHE.FILL"
        ));
        assert!(!is_reserved_special_annotation("@param file output path"));
        assert!(!is_reserved_special_annotation("\\param file output path"));
    }

    #[test]
    fn recognizes_foreign_tag_boundaries() {
        assert!(is_foreign_tag_boundary("@param file output path"));
        assert!(is_foreign_tag_boundary("\\param file output path"));
        assert!(!is_foreign_tag_boundary("Human prose with @spec inline."));
        assert!(!is_foreign_tag_boundary("{@link Exporter}"));
    }

    #[test]
    fn recognizes_any_boundary_kind() {
        assert!(is_any_tag_boundary("@spec EXPORT.CSV"));
        assert!(is_any_tag_boundary("@returns CSV output"));
        assert!(is_any_tag_boundary("\\param file output path"));
        assert!(!is_any_tag_boundary("CSV exports include a header row."));
    }

    #[test]
    fn extracts_reserved_annotation_rest_using_whitespace_boundaries() {
        assert_eq!(
            reserved_special_annotation_rest("@spec EXPORT.CSV", ReservedSpecialAnnotation::Spec),
            Some("EXPORT.CSV")
        );
        assert_eq!(
            reserved_special_annotation_rest("@spec", ReservedSpecialAnnotation::Spec),
            Some("")
        );
        assert_eq!(
            reserved_special_annotation("@spec\tEXPORT.CSV"),
            Some(ReservedSpecialAnnotation::Spec)
        );
        assert_eq!(reserved_special_annotation("@specly EXPORT.CSV"), None);
    }

    #[test]
    fn markdown_declaration_lines_exclude_lists_blockquotes_and_code_spans() {
        assert_eq!(
            normalize_markdown_declaration_line("@spec EXPORT.CSV"),
            Some("@spec EXPORT.CSV")
        );
        assert_eq!(
            normalize_markdown_declaration_line("### @spec EXPORT.CSV"),
            Some("@spec EXPORT.CSV")
        );
        assert_eq!(
            normalize_markdown_declaration_line("### `@spec EXPORT.CSV`"),
            None
        );
        assert_eq!(
            normalize_markdown_annotation_line("`@spec EXPORT.CSV`"),
            None
        );
        assert_eq!(
            normalize_markdown_declaration_line("- @spec EXPORT.CSV"),
            None
        );
        assert_eq!(
            normalize_markdown_declaration_line("> @spec EXPORT.CSV"),
            None
        );
    }
}
