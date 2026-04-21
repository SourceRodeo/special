/**
@module SPECIAL.TESTS.SUPPORT.CLI.RENDER
Output-parsing helpers for CLI integration tests.
*/
// @fileimplements SPECIAL.TESTS.SUPPORT.CLI.RENDER
use std::fs;
use std::path::Path;

use serde_json::Value;

pub fn top_level_help_command_names(output: &str) -> Vec<String> {
    top_level_help_commands(output)
        .into_iter()
        .map(|(name, _)| name)
        .collect()
}

pub fn top_level_help_commands(output: &str) -> Vec<(String, String)> {
    section_items(output, "Commands:")
        .into_iter()
        .filter_map(|line| {
            let trimmed = line.trim_start();
            let mut parts = trimmed.split_whitespace();
            let name = parts.next()?;
            let summary = trimmed[name.len()..].trim_start();
            Some((name.to_string(), summary.to_string()))
        })
        .collect()
}

pub fn top_level_help_command_summaries(output: &str) -> Vec<String> {
    top_level_help_commands(output)
        .into_iter()
        .map(|(_, summary)| summary)
        .collect()
}

pub fn rendered_spec_node_lines(output: &str) -> Vec<String> {
    let mut lines = Vec::new();

    for line in output.lines() {
        let trimmed = line.trim_start();
        let id = trimmed.split_whitespace().next().unwrap_or_default();
        if !id.is_empty()
            && id.chars().all(|ch| {
                ch.is_ascii_uppercase() || ch.is_ascii_digit() || matches!(ch, '.' | '_' | '-')
            })
        {
            lines.push(trimmed.to_string());
        }
    }

    lines
}

pub fn rendered_spec_node_ids(output: &str) -> Vec<String> {
    rendered_spec_node_lines(output)
        .into_iter()
        .filter_map(|line| line.split_whitespace().next().map(|id| id.to_string()))
        .collect()
}

pub fn rendered_spec_node_line(output: &str, id: &str) -> Option<String> {
    rendered_spec_node_lines(output)
        .into_iter()
        .find(|line| line.split_whitespace().next() == Some(id))
}

pub fn html_node_has_badge(output: &str, id: &str, badge_class: &str, badge_text: &str) -> bool {
    let needle = format!("<span class=\"node-id\">{id}</span>");
    let Some(start) = output.find(&needle) else {
        return false;
    };
    let after_id = &output[start + needle.len()..];
    let Some(header_end) = after_id.find("</div>") else {
        return false;
    };
    after_id[..header_end].contains(&format!(
        "<span class=\"badge {badge_class}\">{badge_text}</span>"
    ))
}

pub fn installed_skill_ids(skills_root: &Path) -> Vec<String> {
    let mut ids = fs::read_dir(skills_root)
        .expect("skills directory should be readable")
        .map(|entry| entry.expect("skill entry should be readable").path())
        .filter(|path| path.is_dir())
        .map(|path| {
            path.file_name()
                .and_then(|name| name.to_str())
                .expect("skill directory name should be utf-8")
                .to_string()
        })
        .collect::<Vec<_>>();
    ids.sort();
    ids
}

pub fn listed_skill_ids(output: &str) -> Vec<String> {
    let mut ids = Vec::new();
    let mut in_skill_section = false;

    for line in output.lines() {
        if line == "Available skill ids:" {
            in_skill_section = true;
            continue;
        }
        if in_skill_section && line.trim().is_empty() {
            break;
        }
        if in_skill_section {
            let id = line.split_whitespace().next().unwrap_or_default();
            if !id.is_empty() {
                ids.push(id.to_string());
            }
        }
    }

    ids.sort();
    ids
}

pub fn find_node_by_id<'a>(node: &'a Value, id: &str) -> Option<&'a Value> {
    if node["id"].as_str() == Some(id) {
        return Some(node);
    }
    node["children"]
        .as_array()
        .into_iter()
        .flatten()
        .find_map(|child| find_node_by_id(child, id))
}

fn section_items(output: &str, section_heading: &str) -> Vec<String> {
    let mut items = Vec::new();
    let mut in_section = false;

    for line in output.lines() {
        if line.trim() == section_heading {
            in_section = true;
            continue;
        }
        if in_section {
            if line.trim().is_empty() {
                break;
            }
            if line.starts_with("  ") && !line.starts_with("    ") {
                items.push(line.trim_start().to_string());
            }
        }
    }

    items
}
