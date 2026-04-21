/**
@module SPECIAL.TESTS.EXTRACTOR.COMMENTS
Extractor comment-style tests in `src/extractor/tests/comments.rs`.
*/
// @fileimplements SPECIAL.TESTS.EXTRACTOR.COMMENTS
use super::support::extract;

#[test]
// @verifies SPECIAL.PARSE.LINE_COMMENTS
fn extracts_line_comment_blocks() {
    let blocks = extract(
        "src/example.rs",
        "// @spec AUTH\n// Auth works.\nfn main() {}\n",
    );

    assert_eq!(blocks.len(), 1);
    assert_eq!(blocks[0].lines[0].text, "@spec AUTH");
    assert_eq!(
        blocks[0]
            .owned_item
            .as_ref()
            .expect("owned item should be present")
            .body,
        "fn main() {}"
    );
}

#[test]
// @verifies SPECIAL.PARSE.GO_LINE_COMMENTS
fn extracts_go_line_comment_blocks() {
    let blocks = extract(
        "src/example.go",
        "// @spec AUTH.LOGIN\n// Auth works.\nfunc main() {}\n",
    );

    assert_eq!(blocks.len(), 1);
    assert_eq!(blocks[0].lines[0].text, "@spec AUTH.LOGIN");
    assert_eq!(
        blocks[0]
            .owned_item
            .as_ref()
            .expect("owned item should be present")
            .body,
        "func main() {}"
    );
}

#[test]
// @verifies SPECIAL.PARSE.TYPESCRIPT_LINE_COMMENTS
fn extracts_typescript_line_comment_blocks() {
    let blocks = extract(
        "src/example.ts",
        "// @spec AUTH.LOGIN\n// Auth works.\nexport const ok = true;\n",
    );

    assert_eq!(blocks.len(), 1);
    assert_eq!(blocks[0].lines[0].text, "@spec AUTH.LOGIN");
    assert_eq!(
        blocks[0]
            .owned_item
            .as_ref()
            .expect("owned item should be present")
            .body,
        "export const ok = true;"
    );
}

#[test]
// @verifies SPECIAL.PARSE.BLOCK_COMMENTS
fn extracts_generic_block_comment_blocks() {
    let blocks = extract(
        "src/example.rs",
        "/**\n * @spec AUTH.LOGIN\n * Auth works.\n */\nfn main() {}\n",
    );

    assert_eq!(blocks.len(), 1);
    assert_eq!(blocks[0].lines[1].text, "@spec AUTH.LOGIN");
    assert_eq!(
        blocks[0]
            .owned_item
            .as_ref()
            .expect("owned item should be present")
            .body,
        "fn main() {}"
    );
}

#[test]
// @verifies SPECIAL.PARSE.TYPESCRIPT_BLOCK_COMMENTS
fn extracts_block_comment_blocks() {
    let blocks = extract(
        "src/example.ts",
        "/**\n * @verifies AUTH.LOGIN\n */\nexport {};\n",
    );

    assert_eq!(blocks.len(), 1);
    assert_eq!(blocks[0].lines[1].text, "@verifies AUTH.LOGIN");
    assert_eq!(
        blocks[0]
            .owned_item
            .as_ref()
            .expect("owned item should be present")
            .body,
        "export {};"
    );
}

#[test]
// @verifies SPECIAL.PARSE.SHELL_COMMENTS
fn extracts_shell_comment_blocks() {
    let blocks = extract(
        "scripts/verify.sh",
        "#!/usr/bin/env bash\n# @fileverifies SPECIAL.QUALITY.RUST.CLIPPY.SPEC_OWNED\nset -euo pipefail\n\nexec mise exec -- cargo clippy --all-targets --all-features -- -D warnings\n",
    );

    assert_eq!(blocks.len(), 1);
    assert_eq!(
        blocks[0].lines[1].text,
        "@fileverifies SPECIAL.QUALITY.RUST.CLIPPY.SPEC_OWNED"
    );
    assert_eq!(
        blocks[0]
            .owned_item
            .as_ref()
            .expect("owned item should be present")
            .body,
        "set -euo pipefail\n\nexec mise exec -- cargo clippy --all-targets --all-features -- -D warnings"
    );
}

#[test]
// @verifies SPECIAL.PARSE.PYTHON_LINE_COMMENTS
fn extracts_python_line_comment_blocks() {
    let blocks = extract(
        "src/example.py",
        "# @verifies AUTH.LOGIN\n\ndef test_auth_login():\n    assert True\n",
    );

    assert_eq!(blocks.len(), 1);
    assert_eq!(blocks[0].lines[0].text, "@verifies AUTH.LOGIN");
    assert_eq!(
        blocks[0]
            .owned_item
            .as_ref()
            .expect("owned item should be present")
            .body,
        "def test_auth_login():\n    assert True"
    );
}
