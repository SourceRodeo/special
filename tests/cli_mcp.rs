/**
@module SPECIAL.TESTS.CLI_MCP
CLI integration tests for the Special MCP stdio server.
*/
// @fileimplements SPECIAL.TESTS.CLI_MCP
#[path = "support/cli.rs"]
mod support;

use std::fs;
use std::path::Path;

use serde_json::{Value, json};
use support::{run_special_with_input, temp_repo_dir};

#[test]
// @verifies SPECIAL.MCP_COMMAND
fn mcp_initializes_and_lists_special_tools_as_jsonrpc_lines() {
    let root = temp_repo_dir("special-cli-mcp-list-tools");
    write_mcp_fixture(&root);

    let output = run_special_with_input(
        &root,
        &["mcp"],
        concat!(
            "{\"jsonrpc\":\"2.0\",\"id\":1,\"method\":\"initialize\",\"params\":{}}\n",
            "{\"jsonrpc\":\"2.0\",\"method\":\"notifications/initialized\"}\n",
            "{\"jsonrpc\":\"2.0\",\"id\":2,\"method\":\"tools/list\",\"params\":{}}\n",
        ),
    );

    assert!(
        output.status.success(),
        "mcp server should exit cleanly: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    let responses = jsonrpc_responses(output.stdout);
    assert_eq!(responses.len(), 2);
    assert_eq!(responses[0]["result"]["serverInfo"]["name"], "special");
    let tools = responses[1]["result"]["tools"]
        .as_array()
        .expect("tools should be an array");
    let tool_names = tools
        .iter()
        .map(|tool| tool["name"].as_str().expect("tool should have name"))
        .collect::<Vec<_>>();
    assert!(tool_names.contains(&"special_status"));
    assert!(tool_names.contains(&"special_specs"));
    assert!(tool_names.contains(&"special_docs_materialize"));
}

#[test]
// @verifies SPECIAL.MCP_COMMAND.TOOLS
fn mcp_specs_tool_returns_special_projection_content() {
    let root = temp_repo_dir("special-cli-mcp-specs-tool");
    write_mcp_fixture(&root);

    let output = run_special_with_input(
        &root,
        &["mcp"],
        &format!(
            "{}\n",
            json!({
                "jsonrpc": "2.0",
                "id": 1,
                "method": "tools/call",
                "params": {
                    "name": "special_specs",
                    "arguments": {
                        "id": "EXPORT.CSV.HEADERS",
                        "verbose": true
                    }
                }
            })
        ),
    );

    assert!(output.status.success());
    let responses = jsonrpc_responses(output.stdout);
    let text = tool_text(&responses[0]);
    assert!(text.contains("EXPORT.CSV.HEADERS"));
    assert_eq!(responses[0]["result"]["isError"], false);
}

#[test]
// @verifies SPECIAL.MCP_COMMAND.DOCS_MATERIALIZE
fn mcp_docs_materialize_tool_scrubs_docs_output() {
    let root = temp_repo_dir("special-cli-mcp-docs-materialize");
    write_mcp_fixture(&root);
    fs::create_dir_all(root.join("docs/src")).expect("docs source dir should be created");
    fs::write(
        root.join("docs/src/README.md"),
        "[CSV exports include headers](special://spec/EXPORT.CSV.HEADERS).\n",
    )
    .expect("docs source should be written");

    let output = run_special_with_input(
        &root,
        &["mcp"],
        &format!(
            "{}\n",
            json!({
                "jsonrpc": "2.0",
                "id": 1,
                "method": "tools/call",
                "params": {
                    "name": "special_docs_materialize",
                    "arguments": {
                        "target": "docs/src",
                        "output": "docs/dist"
                    }
                }
            })
        ),
    );

    assert!(
        output.status.success(),
        "mcp docs materialization should succeed: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    let responses = jsonrpc_responses(output.stdout);
    assert_eq!(responses[0]["result"]["isError"], false);
    let rendered =
        fs::read_to_string(root.join("docs/dist/README.md")).expect("rendered docs should exist");
    assert!(rendered.contains("CSV exports include headers."));
    assert!(!rendered.contains("special://"));
}

fn write_mcp_fixture(root: &Path) {
    fs::write(
        root.join("special.toml"),
        "version = \"1\"\nroot = \".\"\nignore = [\"docs/dist\"]\n",
    )
    .expect("special.toml should be written");
    fs::create_dir_all(root.join("specs")).expect("specs dir should be created");
    fs::write(
        root.join("specs/root.md"),
        concat!(
            "@group EXPORT\n",
            "Export behavior.\n\n",
            "@group EXPORT.CSV\n",
            "CSV export behavior.\n\n",
            "@spec EXPORT.CSV.HEADERS\n",
            "CSV exports include headers.\n",
        ),
    )
    .expect("spec fixture should be written");
}

fn jsonrpc_responses(stdout: Vec<u8>) -> Vec<Value> {
    String::from_utf8(stdout)
        .expect("stdout should be utf-8")
        .lines()
        .map(|line| serde_json::from_str(line).expect("stdout line should be json"))
        .collect()
}

fn tool_text(response: &Value) -> &str {
    response["result"]["content"][0]["text"]
        .as_str()
        .expect("tool response should include text")
}
