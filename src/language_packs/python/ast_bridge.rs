/**
@module SPECIAL.LANGUAGE_PACKS.PYTHON.AST_BRIDGE
Bridges the built-in Python pack to Python's own `ast` parser through a small helper script so Python syntax extraction can stay self-contained under the pack without embedding a separate parser in the Rust core.
*/
// @fileimplements SPECIAL.LANGUAGE_PACKS.PYTHON.AST_BRIDGE
use std::path::Path;
use std::process::{Command, Stdio};

use anyhow::{Result, anyhow};
use serde::Deserialize;

use crate::syntax::{
    CallSyntaxKind, ParsedSourceGraph, SourceCall, SourceItem, SourceItemKind, SourceInvocation,
    SourceLanguage, SourceSpan,
};

const PYTHON_AST_BRIDGE_SCRIPT: &str = include_str!("parse_source_graph.py");

pub(super) fn parse_source_graph(path: &Path, text: &str) -> Option<ParsedSourceGraph> {
    parse_source_graph_result(path, text).ok()
}

pub(super) fn parse_source_graph_result(path: &Path, text: &str) -> Result<ParsedSourceGraph> {
    let mut child = bridge_command(path)
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .map_err(|error| anyhow!("failed to launch python syntax bridge: {error}"))?;
    {
        use std::io::Write;

        child
            .stdin
            .as_mut()
            .ok_or_else(|| anyhow!("python syntax bridge stdin was unavailable"))?
            .write_all(text.as_bytes())
            .map_err(|error| anyhow!("failed to write python source to syntax bridge: {error}"))?;
    }
    let output = child
        .wait_with_output()
        .map_err(|error| anyhow!("failed to wait for python syntax bridge: {error}"))?;
    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr).trim().to_string();
        return Err(anyhow!(
            "python syntax bridge exited with status {}{}",
            output.status,
            if stderr.is_empty() {
                String::new()
            } else {
                format!(": {stderr}")
            }
        ));
    }

    let payload: PythonParsedSourceGraph = serde_json::from_slice(&output.stdout)?;
    Ok(ParsedSourceGraph {
        language: SourceLanguage::new("python"),
        items: payload.items.into_iter().map(|item| item.into_source_item(path)).collect(),
    })
}

fn bridge_command(path: &Path) -> Command {
    let mut command = Command::new("mise");
    command.args(bridge_command_args(path));
    command
}

fn bridge_command_args(path: &Path) -> Vec<String> {
    vec![
        "exec".to_string(),
        "--".to_string(),
        "python3".to_string(),
        "-c".to_string(),
        PYTHON_AST_BRIDGE_SCRIPT.to_string(),
        path.display().to_string(),
    ]
}

#[derive(Deserialize)]
struct PythonParsedSourceGraph {
    items: Vec<PythonItem>,
}

#[derive(Deserialize)]
struct PythonItem {
    name: String,
    qualified_name: String,
    module_path: Vec<String>,
    container_path: Vec<String>,
    kind: String,
    start_line: usize,
    end_line: usize,
    start_column: usize,
    end_column: usize,
    start_byte: usize,
    end_byte: usize,
    public: bool,
    root_visible: bool,
    is_test: bool,
    calls: Vec<PythonCall>,
}

impl PythonItem {
    fn into_source_item(self, path: &Path) -> SourceItem {
        let span = SourceSpan {
            start_line: self.start_line,
            end_line: self.end_line,
            start_column: self.start_column,
            end_column: self.end_column,
            start_byte: self.start_byte,
            end_byte: self.end_byte,
        };
        SourceItem {
            source_path: path.display().to_string(),
            stable_id: format!(
                "{}:{}:{}",
                path.display(),
                self.qualified_name,
                self.start_line
            ),
            name: self.name,
            qualified_name: self.qualified_name,
            module_path: self.module_path,
            container_path: self.container_path,
            shape_fingerprint: String::new(),
            shape_node_count: 0,
            kind: match self.kind.as_str() {
                "method" => SourceItemKind::Method,
                _ => SourceItemKind::Function,
            },
            span,
            public: self.public,
            root_visible: self.root_visible,
            is_test: self.is_test,
            calls: self.calls.into_iter().map(PythonCall::into_source_call).collect(),
            invocations: Vec::<SourceInvocation>::new(),
        }
    }
}

#[derive(Deserialize)]
struct PythonCall {
    name: String,
    qualifier: Option<String>,
    syntax: String,
    start_line: usize,
    end_line: usize,
    start_column: usize,
    end_column: usize,
    start_byte: usize,
    end_byte: usize,
}

impl PythonCall {
    fn into_source_call(self) -> SourceCall {
        SourceCall {
            name: self.name,
            qualifier: self.qualifier,
            syntax: match self.syntax.as_str() {
                "field" => CallSyntaxKind::Field,
                "scoped_identifier" => CallSyntaxKind::ScopedIdentifier,
                _ => CallSyntaxKind::Identifier,
            },
            span: SourceSpan {
                start_line: self.start_line,
                end_line: self.end_line,
                start_column: self.start_column,
                end_column: self.end_column,
                start_byte: self.start_byte,
                end_byte: self.end_byte,
            },
        }
    }
}

#[cfg(test)]
mod tests {
    use std::path::Path;

    use super::bridge_command_args;

    #[test]
    fn embedded_bridge_command_does_not_depend_on_manifest_relative_script_paths() {
        let args = bridge_command_args(Path::new("src/app.py"));

        assert_eq!(args[0], "exec");
        assert_eq!(args[1], "--");
        assert_eq!(args[2], "python3");
        assert_eq!(args[3], "-c");
        assert!(args[4].contains("import ast"));
        assert!(!args[4].contains("parse_source_graph.py"));
        assert!(!args[4].contains(env!("CARGO_MANIFEST_DIR")));
        assert_eq!(args[5], "src/app.py");
    }
}
