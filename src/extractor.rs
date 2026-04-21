/**
@spec SPECIAL.PARSE.LINE_COMMENTS
special parses annotation blocks from contiguous line comments.

@spec SPECIAL.PARSE.BLOCK_COMMENTS
special parses annotation blocks from block comments.

@spec SPECIAL.PARSE.GO_LINE_COMMENTS
special parses annotation blocks from Go line comments in `.go` files.

@spec SPECIAL.PARSE.TYPESCRIPT_LINE_COMMENTS
special parses annotation blocks from TypeScript line comments in `.ts` files.

@spec SPECIAL.PARSE.TYPESCRIPT_BLOCK_COMMENTS
special parses annotation blocks from TypeScript block comments in `.ts` files.

@spec SPECIAL.PARSE.MIXED_PURPOSE_COMMENTS
special parses reserved annotations from ordinary mixed-purpose comment blocks without requiring the whole block to be special-only.

@spec SPECIAL.PARSE.LINE_START_RESERVED_TAGS
special interprets reserved annotations only when they begin the normalized comment line after comment markers and leading whitespace are stripped.

@spec SPECIAL.PARSE.FOREIGN_TAGS_NOT_ERRORS
special does not report foreign line-start `@...` and `\\...` tags as lint errors inside mixed-purpose comment blocks.

@spec SPECIAL.PARSE.SHELL_COMMENTS
special parses annotation blocks from shell-style line comments in .sh files.

@spec SPECIAL.PARSE.PYTHON_LINE_COMMENTS
special parses annotation blocks from Python `#` comments in `.py` files instead of docstring ownership.

@spec SPECIAL.PARSE.VERIFIES.ATTACHES_TO_NEXT_ITEM
special attaches a @verifies annotation block to the next supported item in comment-based languages.

@module SPECIAL.EXTRACTOR
Collects contiguous supported source comment blocks, normalizes comment syntax, and captures the next owned code item for attachment. This module does not interpret `special` tag semantics or build spec or module trees.
*/
// @fileimplements SPECIAL.EXTRACTOR
use std::fs;
use std::path::{Path, PathBuf};

use anyhow::{Context, Result};

use crate::discovery::{DiscoveryConfig, discover_annotation_files};
use crate::model::CommentBlock;

mod blocks;
mod owned_items;

pub fn collect_comment_blocks(
    root: &Path,
    ignore_patterns: &[String],
) -> Result<Vec<CommentBlock>> {
    let mut blocks = Vec::new();
    for path in discover_annotation_files(DiscoveryConfig {
        root,
        ignore_patterns,
    })?
    .source_files
    {
        let content = fs::read_to_string(&path)
            .with_context(|| format!("reading source file {}", path.display()))?;
        blocks.extend(extract_blocks_from_text(path, &content));
    }

    Ok(blocks)
}

fn extract_blocks_from_text(path: PathBuf, content: &str) -> Vec<CommentBlock> {
    blocks::extract_blocks_from_text(path, content)
}

#[cfg(test)]
mod tests;
