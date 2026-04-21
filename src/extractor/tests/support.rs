/**
@module SPECIAL.TESTS.EXTRACTOR.SUPPORT
Shared extractor test helpers in `src/extractor/tests/support.rs`.
*/
// @fileimplements SPECIAL.TESTS.EXTRACTOR.SUPPORT
use std::path::PathBuf;

use crate::model::CommentBlock;

use super::super::extract_blocks_from_text;

pub(super) fn extract(path: &str, content: &str) -> Vec<CommentBlock> {
    extract_blocks_from_text(PathBuf::from(path), content)
}
