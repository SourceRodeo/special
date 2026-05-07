use std::fs;
use std::path::{Path, PathBuf};

use anyhow::Result;
use tree_sitter::{Node, Parser};

use crate::model::{ArchitectureLongExactProseAssertion, ArchitectureRepoSignalsSummary};
use crate::source_paths::looks_like_test_path;

const MAX_EXACT_PROSE_WORDS: usize = 10;

#[derive(Debug, Clone, Copy)]
enum TestLanguage {
    Rust,
    TypeScript,
    Tsx,
    Go,
    Python,
}

impl TestLanguage {
    fn for_path(path: &Path) -> Option<Self> {
        match path.extension().and_then(|ext| ext.to_str()) {
            Some("rs") => Some(Self::Rust),
            Some("ts") => Some(Self::TypeScript),
            Some("tsx") => Some(Self::Tsx),
            Some("go") => Some(Self::Go),
            Some("py") => Some(Self::Python),
            _ => None,
        }
    }

    fn name(self) -> &'static str {
        match self {
            Self::Rust => "rust",
            Self::TypeScript => "typescript",
            Self::Tsx => "tsx",
            Self::Go => "go",
            Self::Python => "python",
        }
    }

    fn set_parser_language(self, parser: &mut Parser) -> Result<(), tree_sitter::LanguageError> {
        match self {
            Self::Rust => parser.set_language(&tree_sitter_rust::LANGUAGE.into()),
            Self::TypeScript => {
                parser.set_language(&tree_sitter_typescript::LANGUAGE_TYPESCRIPT.into())
            }
            Self::Tsx => parser.set_language(&tree_sitter_typescript::LANGUAGE_TSX.into()),
            Self::Go => parser.set_language(&tree_sitter_go::LANGUAGE.into()),
            Self::Python => parser.set_language(&tree_sitter_python::LANGUAGE.into()),
        }
    }
}

pub(super) fn apply_long_exact_prose_assertion_summary(
    root: &Path,
    files: &[PathBuf],
    summary: &mut ArchitectureRepoSignalsSummary,
) -> Result<()> {
    let mut findings = Vec::new();
    for path in files {
        let Some(language) = TestLanguage::for_path(path) else {
            continue;
        };
        if !is_test_surface(path) {
            continue;
        }
        let source = fs::read_to_string(path)?;
        findings.extend(find_long_exact_prose_assertions(
            root, path, language, &source,
        ));
    }
    findings.sort_by(|left, right| {
        left.path
            .cmp(&right.path)
            .then_with(|| left.line.cmp(&right.line))
            .then_with(|| left.callee.cmp(&right.callee))
    });
    summary.long_exact_prose_assertions = findings.len();
    summary.long_exact_prose_assertion_details = findings;
    Ok(())
}

fn is_test_surface(path: &Path) -> bool {
    if looks_like_test_path(path) {
        return true;
    }
    let text = path.to_string_lossy();
    text.contains("test_fixtures") || text.contains("/fixtures/")
}

fn find_long_exact_prose_assertions(
    root: &Path,
    path: &Path,
    language: TestLanguage,
    source: &str,
) -> Vec<ArchitectureLongExactProseAssertion> {
    let mut parser = Parser::new();
    if language.set_parser_language(&mut parser).is_err() {
        return Vec::new();
    }
    let Some(tree) = parser.parse(source, None) else {
        return Vec::new();
    };
    if tree.root_node().has_error() {
        return Vec::new();
    }
    let mut findings = Vec::new();
    collect_findings(
        root,
        path,
        language,
        tree.root_node(),
        source,
        &mut findings,
    );
    findings
}

fn collect_findings(
    root: &Path,
    path: &Path,
    language: TestLanguage,
    node: Node<'_>,
    source: &str,
    findings: &mut Vec<ArchitectureLongExactProseAssertion>,
) {
    match language {
        TestLanguage::Rust => collect_rust_node(root, path, node, source, findings),
        TestLanguage::Python if node.kind() == "assert_statement" => {
            collect_literal_descendants(root, path, language, node, source, "assert", findings);
        }
        _ if node.kind() == "call_expression" => {
            collect_exact_call_literals(root, path, language, node, source, findings);
        }
        _ => {}
    }

    let mut cursor = node.walk();
    for child in node.named_children(&mut cursor) {
        collect_findings(root, path, language, child, source, findings);
    }
}

fn collect_rust_node(
    root: &Path,
    path: &Path,
    node: Node<'_>,
    source: &str,
    findings: &mut Vec<ArchitectureLongExactProseAssertion>,
) {
    if node.kind() == "macro_invocation" {
        collect_rust_assert_macro_literals(root, path, node, source, findings);
    }
}

fn collect_rust_assert_macro_literals(
    root: &Path,
    path: &Path,
    node: Node<'_>,
    source: &str,
    findings: &mut Vec<ArchitectureLongExactProseAssertion>,
) {
    let Some(macro_node) = node.child_by_field_name("macro") else {
        return;
    };
    let Ok(macro_name) = macro_node.utf8_text(source.as_bytes()) else {
        return;
    };
    if !matches!(macro_name, "assert" | "assert_eq" | "assert_ne") {
        return;
    }
    let Some(tokens) = node.named_child(1) else {
        return;
    };
    let Ok(token_text) = tokens.utf8_text(source.as_bytes()) else {
        return;
    };
    let args = split_top_level_args(token_text);
    if macro_name == "assert" {
        if let Some(condition) = args.first() {
            for callee in ["contains", "starts_with", "ends_with"] {
                if condition.contains(&format!("{callee}(")) {
                    collect_text_literals(
                        root,
                        path,
                        TestLanguage::Rust,
                        tokens,
                        callee,
                        condition,
                        findings,
                    );
                }
            }
        }
    } else {
        for arg in args.into_iter().take(2) {
            collect_text_literals(
                root,
                path,
                TestLanguage::Rust,
                tokens,
                macro_name,
                arg,
                findings,
            );
        }
    }
}

fn collect_exact_call_literals(
    root: &Path,
    path: &Path,
    language: TestLanguage,
    node: Node<'_>,
    source: &str,
    findings: &mut Vec<ArchitectureLongExactProseAssertion>,
) {
    let Some(function) = node
        .child_by_field_name("function")
        .or_else(|| node.child_by_field_name("callee"))
    else {
        return;
    };
    let callee_text = function
        .utf8_text(source.as_bytes())
        .unwrap_or_default()
        .to_string();
    let Some(callee) = exact_callee(&callee_text) else {
        return;
    };
    let arguments = node.child_by_field_name("arguments").unwrap_or(node);
    collect_literal_descendants(root, path, language, arguments, source, callee, findings);
}

fn collect_literal_descendants(
    root: &Path,
    path: &Path,
    language: TestLanguage,
    node: Node<'_>,
    source: &str,
    callee: &str,
    findings: &mut Vec<ArchitectureLongExactProseAssertion>,
) {
    if let Some(literal) = string_literal(node, source) {
        record_literal(root, path, language, node, callee, &literal, findings);
    }
    let mut cursor = node.walk();
    for child in node.named_children(&mut cursor) {
        collect_literal_descendants(root, path, language, child, source, callee, findings);
    }
}

fn collect_text_literals(
    root: &Path,
    path: &Path,
    language: TestLanguage,
    node: Node<'_>,
    callee: &str,
    text: &str,
    findings: &mut Vec<ArchitectureLongExactProseAssertion>,
) {
    for literal in rust_string_literals(text) {
        record_literal(root, path, language, node, callee, &literal, findings);
    }
}

fn record_literal(
    root: &Path,
    path: &Path,
    language: TestLanguage,
    node: Node<'_>,
    callee: &str,
    literal: &str,
    findings: &mut Vec<ArchitectureLongExactProseAssertion>,
) {
    let word_count = human_word_count(literal);
    if word_count <= MAX_EXACT_PROSE_WORDS {
        return;
    }
    findings.push(ArchitectureLongExactProseAssertion {
        path: path.strip_prefix(root).unwrap_or(path).to_path_buf(),
        line: node.start_position().row + 1,
        language: language.name().to_string(),
        callee: callee.to_string(),
        word_count,
        literal: literal.to_string(),
    });
}

fn exact_callee(callee: &str) -> Option<&'static str> {
    for candidate in [
        "contains",
        "starts_with",
        "ends_with",
        "includes",
        "startsWith",
        "endsWith",
        "toContain",
        "toEqual",
        "toStrictEqual",
        "toBe",
        "Equal",
        "NotEqual",
        "Contains",
        "HasPrefix",
        "HasSuffix",
        "assertEqual",
        "assertIn",
        "assertTrue",
    ] {
        if callee == candidate || callee.ends_with(&format!(".{candidate}")) {
            return Some(candidate);
        }
    }
    None
}

fn string_literal(node: Node<'_>, source: &str) -> Option<String> {
    let text = node.utf8_text(source.as_bytes()).ok()?;
    match node.kind() {
        "string_literal" | "interpreted_string_literal" | "string" => {
            strip_quoted(text).map(ToString::to_string)
        }
        "raw_string_literal" | "template_string" => strip_rawish(text).map(ToString::to_string),
        _ => None,
    }
}

fn strip_quoted(text: &str) -> Option<&str> {
    text.strip_prefix(['"', '\''])
        .and_then(|inner| inner.strip_suffix(['"', '\'']))
}

fn strip_rawish(text: &str) -> Option<&str> {
    if text.starts_with('`') {
        return text
            .strip_prefix('`')
            .and_then(|inner| inner.strip_suffix('`'));
    }
    let start = text.find('"')? + 1;
    let end = text[start..].rfind('"')? + start;
    Some(&text[start..end])
}

fn human_word_count(text: &str) -> usize {
    text.split(|ch: char| !ch.is_alphabetic())
        .filter(|word| word.len() > 1)
        .count()
}

fn split_top_level_args(text: &str) -> Vec<&str> {
    let inner = text
        .strip_prefix(['(', '[', '{'])
        .and_then(|stripped| stripped.strip_suffix([')', ']', '}']))
        .unwrap_or(text);
    let mut args = Vec::new();
    let mut start = 0;
    let mut depth = 0_i32;
    let mut in_string = false;
    let mut escaped = false;
    for (index, ch) in inner.char_indices() {
        if in_string {
            if escaped {
                escaped = false;
                continue;
            }
            if ch == '\\' {
                escaped = true;
                continue;
            }
            if ch == '"' {
                in_string = false;
            }
            continue;
        }
        match ch {
            '"' => in_string = true,
            '(' | '[' | '{' => depth += 1,
            ')' | ']' | '}' => depth -= 1,
            ',' if depth == 0 => {
                args.push(inner[start..index].trim());
                start = index + ch.len_utf8();
            }
            _ => {}
        }
    }
    args.push(inner[start..].trim());
    args
}

fn rust_string_literals(text: &str) -> Vec<String> {
    let mut literals = Vec::new();
    let bytes = text.as_bytes();
    let mut index = 0;
    while index < bytes.len() {
        if bytes[index] == b'"' {
            if let Some((literal, next)) = quoted_literal(text, index) {
                literals.push(literal);
                index = next;
                continue;
            }
        }
        if bytes[index] == b'r'
            && let Some((literal, next)) = raw_rust_literal(text, index)
        {
            literals.push(literal);
            index = next;
            continue;
        }
        index += 1;
    }
    literals
}

fn quoted_literal(text: &str, start: usize) -> Option<(String, usize)> {
    let mut escaped = false;
    for (offset, ch) in text[start + 1..].char_indices() {
        let index = start + 1 + offset;
        if escaped {
            escaped = false;
            continue;
        }
        if ch == '\\' {
            escaped = true;
            continue;
        }
        if ch == '"' {
            return Some((text[start + 1..index].to_string(), index + 1));
        }
    }
    None
}

fn raw_rust_literal(text: &str, start: usize) -> Option<(String, usize)> {
    let rest = &text[start..];
    if !rest.starts_with("r\"") && !rest.starts_with("r#") {
        return None;
    }
    let quote = rest.find('"')?;
    let hashes = &rest[1..quote];
    let body_start = start + quote + 1;
    let terminator = format!("\"{hashes}");
    let body_end = text[body_start..].find(&terminator)? + body_start;
    Some((
        text[body_start..body_end].to_string(),
        body_end + terminator.len(),
    ))
}
