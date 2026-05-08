/**
@module SPECIAL.TESTS.PARSER.SUPPORT
Shared parser test helpers and temporary fixtures in `src/parser/tests/support.rs`.
*/
// @fileimplements SPECIAL.TESTS.PARSER.SUPPORT
use std::fs;
use std::path::{Path, PathBuf};
use std::sync::Arc;
use std::time::{SystemTime, UNIX_EPOCH};

use crate::config::SpecialVersion;
use crate::model::{BlockLine, CommentBlock, OwnedItem, ParsedRepo, SourceLocation};

use super::super::ParseDialect;
use super::super::block::{ParseRules, parse_block};

pub(super) fn parse_with_version(block: &CommentBlock, version: SpecialVersion) -> ParsedRepo {
    let mut parsed = ParsedRepo::default();
    let dialect = match version {
        SpecialVersion::V0 => ParseDialect::CompatibilityV0,
        SpecialVersion::V1 => ParseDialect::CurrentV1,
    };
    parse_block(block, &mut parsed, ParseRules::for_dialect(dialect));
    parsed
}

pub(super) fn parse_current(block: &CommentBlock) -> ParsedRepo {
    parse_with_version(block, SpecialVersion::V1)
}

pub(super) fn source_block(lines: &[&str]) -> CommentBlock {
    block_with_path("src/example.rs", lines)
}

pub(super) fn source_block_with_owned_item(
    lines: &[&str],
    item_line: usize,
    body: &str,
) -> CommentBlock {
    block_with_owned_item("src/example.rs", lines, item_line, body)
}

pub(super) fn block_with_path(path: impl Into<PathBuf>, lines: &[&str]) -> CommentBlock {
    comment_block(path.into(), lines, None)
}

pub(super) fn block_with_source_body(
    path: impl Into<PathBuf>,
    lines: &[&str],
    source_body: &str,
) -> CommentBlock {
    let mut block = comment_block(path.into(), lines, None);
    block.source_body = Some(Arc::from(source_body));
    block
}

pub(super) fn block_with_owned_item(
    path: impl Into<PathBuf>,
    lines: &[&str],
    item_line: usize,
    body: &str,
) -> CommentBlock {
    let path = path.into();
    comment_block(
        path.clone(),
        lines,
        Some(OwnedItem {
            location: SourceLocation {
                path,
                line: item_line,
            },
            body: body.to_string(),
        }),
    )
}

fn comment_block(path: PathBuf, lines: &[&str], owned_item: Option<OwnedItem>) -> CommentBlock {
    CommentBlock {
        path,
        lines: lines
            .iter()
            .enumerate()
            .map(|(index, text)| BlockLine {
                line: index + 1,
                text: (*text).to_string(),
            })
            .collect(),
        owned_item,
        source_body: None,
    }
}

pub(super) struct TempFile {
    path: PathBuf,
}

impl TempFile {
    pub(super) fn new(prefix: &str, extension: &str, content: &str) -> Self {
        let extension = extension.trim_start_matches('.');
        let path = std::env::temp_dir().join(format!(
            "{prefix}-{}-{}.{}",
            std::process::id(),
            SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .expect("time should move forward")
                .as_nanos(),
            extension
        ));
        fs::write(&path, content).expect("fixture should be written");
        Self { path }
    }

    pub(super) fn path(&self) -> &Path {
        &self.path
    }
}

impl Drop for TempFile {
    fn drop(&mut self) {
        let _ = fs::remove_file(&self.path);
    }
}
