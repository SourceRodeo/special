/**
@module SPECIAL.LANGUAGE_PACKS.RUST.ANALYZE.USE_TREE
Shared Rust `use`-tree helpers over tree-sitter use-clause nodes for dependency and traceability analysis.
*/
// @fileimplements SPECIAL.LANGUAGE_PACKS.RUST.ANALYZE.USE_TREE
use std::collections::BTreeMap;

use tree_sitter::Node;

pub(super) fn collect_use_paths(node: Node<'_>, source: &[u8]) -> Vec<String> {
    let mut paths = Vec::new();
    collect_use_paths_with_prefix(node, source, None, &mut paths);
    paths
}

pub(super) fn collect_use_aliases(
    node: Node<'_>,
    source: &[u8],
) -> BTreeMap<String, Vec<String>> {
    let mut aliases = BTreeMap::new();
    collect_use_aliases_with_prefix(node, source, None, &mut aliases);
    aliases
}

fn collect_use_paths_with_prefix(
    node: Node<'_>,
    source: &[u8],
    prefix: Option<&str>,
    paths: &mut Vec<String>,
) {
    match node.kind() {
        "identifier" | "scoped_identifier" => {
            if let Some(path) = use_path_text(node, source, prefix) {
                paths.push(path);
            }
        }
        "use_as_clause" => {
            if let Some(path) = node
                .child_by_field_name("path")
                .and_then(|path| use_path_text(path, source, prefix))
            {
                paths.push(path);
            }
        }
        "scoped_use_list" => {
            let Some(path) = node
                .child_by_field_name("path")
                .and_then(|path| use_path_text(path, source, prefix))
            else {
                return;
            };
            if let Some(list) = node.child_by_field_name("list") {
                collect_use_paths_with_prefix(list, source, Some(&path), paths);
            }
        }
        "use_list" => {
            let mut cursor = node.walk();
            for child in node.named_children(&mut cursor) {
                collect_use_paths_with_prefix(child, source, prefix, paths);
            }
        }
        "use_wildcard" => {}
        _ => {}
    }
}

fn collect_use_aliases_with_prefix(
    node: Node<'_>,
    source: &[u8],
    prefix: Option<&str>,
    aliases: &mut BTreeMap<String, Vec<String>>,
) {
    match node.kind() {
        "identifier" | "scoped_identifier" => {
            if let Some(path) = use_path_text(node, source, prefix)
                && let Some(alias) = path.rsplit("::").next()
            {
                aliases.entry(alias.to_string()).or_default().push(path);
            }
        }
        "use_as_clause" => {
            let Some(path) = node
                .child_by_field_name("path")
                .and_then(|path| use_path_text(path, source, prefix))
            else {
                return;
            };
            let Some(alias) = node
                .child_by_field_name("alias")
                .and_then(|alias| alias.utf8_text(source).ok())
                .map(str::to_string)
            else {
                return;
            };
            aliases.entry(alias).or_default().push(path);
        }
        "scoped_use_list" => {
            let Some(path) = node
                .child_by_field_name("path")
                .and_then(|path| use_path_text(path, source, prefix))
            else {
                return;
            };
            if let Some(list) = node.child_by_field_name("list") {
                collect_use_aliases_with_prefix(list, source, Some(&path), aliases);
            }
        }
        "use_list" => {
            let mut cursor = node.walk();
            for child in node.named_children(&mut cursor) {
                collect_use_aliases_with_prefix(child, source, prefix, aliases);
            }
        }
        "use_wildcard" => {}
        _ => {}
    }
}

fn use_path_text(node: Node<'_>, source: &[u8], prefix: Option<&str>) -> Option<String> {
    let raw = node.utf8_text(source).ok()?;
    let normalized = raw
        .chars()
        .filter(|ch| !ch.is_whitespace())
        .collect::<String>();
    if normalized.is_empty() {
        return None;
    }
    Some(match prefix {
        Some(prefix) => format!("{prefix}::{normalized}"),
        None => normalized,
    })
}

#[cfg(test)]
mod tests {
    use tree_sitter::Parser;

    use super::*;

    fn with_first_use_argument(source: &str, check: impl FnOnce(Node<'_>)) {
        let mut parser = Parser::new();
        parser
            .set_language(&tree_sitter_rust::LANGUAGE.into())
            .expect("rust grammar should load");
        let tree = parser.parse(source, None).expect("source should parse");
        let root = tree.root_node();
        let mut cursor = root.walk();
        let argument = root
            .named_children(&mut cursor)
            .find(|node| node.kind() == "use_declaration")
            .and_then(|node| node.child_by_field_name("argument"))
            .expect("use declaration should have an argument");
        check(argument);
    }

    #[test]
    fn provider_use_tree_helpers_flatten_groups_and_renames() {
        let source = "use crate::{api::Client, io::Reader as R, prelude::*};";
        with_first_use_argument(source, |argument| {
            assert_eq!(
                collect_use_paths(argument, source.as_bytes()),
                vec!["crate::api::Client", "crate::io::Reader"]
            );
            assert_eq!(
                collect_use_aliases(argument, source.as_bytes()),
                BTreeMap::from([
                    ("Client".to_string(), vec!["crate::api::Client".to_string()]),
                    ("R".to_string(), vec!["crate::io::Reader".to_string()]),
                ])
            );
        });
    }
}
