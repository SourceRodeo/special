#!/usr/bin/env python3
# @module SPECIAL.RELEASE_REVIEW
# Local-only code-quality review entrypoint that orchestrates release review and surfaces release warnings for tagging. Context planning, payload validation, Codex invocation policy, and response merging live in dedicated helper modules.
# @fileimplements SPECIAL.RELEASE_REVIEW
# @fileverifies SPECIAL.QUALITY.RUST.RELEASE_REVIEW.SPEC_OWNED

from __future__ import annotations

import sys

sys.dont_write_bytecode = True

import argparse
import concurrent.futures
import datetime as dt
import json
import os
from pathlib import Path

SCRIPT_DIR = Path(__file__).resolve().parent
if str(SCRIPT_DIR) not in sys.path:
    sys.path.insert(0, str(SCRIPT_DIR))

from release_review_contract import validate_review_payload, validate_review_preview
from release_review_invoke import (
    DEFAULT_MODEL,
    FAST_MODEL,
    MOCK_ALLOW_ENV,
    SMART_MODEL,
    CodexInvocationError,
    codex_invocation_config,
    invoke_codex,
)
from release_review_merge import merge_pass_responses
from release_review_pipeline import (
    MAX_CONCURRENT_REVIEW_CHUNKS,
    build_file_contexts,
    build_pass_chunks,
    build_review_passes,
    changed_files_from_diff,
    command_exists,
    discover_latest_semver_tag,
    diff_text_for_paths,
    extract_context_ranges,
    full_scan_files,
    parse_changed_line_ranges,
)
from release_tooling import package_version


SCHEMA_PATH = Path(__file__).with_name("rust-release-review.schema.json")
EXPENSIVE_REVIEW_NOTE = (
    "EXPENSIVE REVIEW PRESERVATION: output is written to disk as chunks complete. "
    "Do not rerun to recover terminal output; inspect the JSON file instead. "
    "When launched from Codex, run this script with sandbox escalation because nested codex exec "
    "sessions must access Codex state outside the project sandbox."
)


def repo_root() -> Path:
    return Path(__file__).resolve().parent.parent


def parse_args() -> argparse.Namespace:
    parser = argparse.ArgumentParser(
        description=(
            "Run a warn-only Codex release review for Rust code-quality issues outside clippy's scope."
        )
    )
    parser.add_argument(
        "--base",
        help="Explicit base tag or revision to diff against. Defaults to the latest semver tag.",
    )
    parser.add_argument(
        "--full",
        action="store_true",
        help="Review the full Rust-relevant code surface instead of only the release diff.",
    )
    parser.add_argument("--head", help=argparse.SUPPRESS)
    model_group = parser.add_mutually_exclusive_group()
    model_group.add_argument(
        "--fast",
        action="store_true",
        help=f"Use the faster {FAST_MODEL} review model.",
    )
    model_group.add_argument(
        "--smart",
        action="store_true",
        help=f"Use the stronger {SMART_MODEL} review model.",
    )
    parser.add_argument(
        "--dry-run",
        action="store_true",
        help="Print the assembled review payload as JSON instead of invoking Codex.",
    )
    parser.add_argument(
        "--output",
        help=(
            "Write merged review JSON to this path. Full scans and smart-model reviews "
            "default to _project/release/reviews when this is omitted."
        ),
    )
    parser.add_argument("--allow-mock", action="store_true", help=argparse.SUPPRESS)
    return parser.parse_args()


def running_in_ci() -> bool:
    return any(
        os.environ.get(name, "").strip()
        for name in ("CI", "GITHUB_ACTIONS", "BUILDKITE", "JENKINS_URL")
    )


def selected_model(args: argparse.Namespace) -> tuple[str, str]:
    if args.fast:
        return ("fast", FAST_MODEL)
    if args.smart:
        return ("smart", SMART_MODEL)
    return ("default", DEFAULT_MODEL)


def status(message: str) -> None:
    print(f"[rust-release-review] {message}", file=sys.stderr, flush=True)


def is_expensive_review(args: argparse.Namespace, review_mode: str, model: str) -> bool:
    return args.full or review_mode == "smart" or model == SMART_MODEL


def default_output_path(root: Path, review_mode: str, full_scan: bool) -> Path:
    timestamp = dt.datetime.now(dt.UTC).strftime("%Y%m%dT%H%M%SZ")
    scope = "full" if full_scan else "diff"
    return (
        root
        / "_project"
        / "release"
        / "reviews"
        / f"rust-release-review-{scope}-{review_mode}-{timestamp}.json"
    )


def resolve_output_path(
    args: argparse.Namespace, root: Path, review_mode: str, model: str
) -> Path | None:
    if args.dry_run:
        return None
    if args.output:
        return Path(args.output)
    if is_expensive_review(args, review_mode, model):
        return default_output_path(root, review_mode, args.full)
    return None


def flush_status(output_path: Path | None) -> str:
    return "partial output flushed" if output_path else "no output file configured"


def has_jj_root(root: Path) -> bool:
    return root.joinpath(".jj").exists() and command_exists("jj")


def has_git_root(root: Path) -> bool:
    return root.joinpath(".git").exists() and command_exists("git")


def load_version(root: Path) -> str:
    return package_version(root)


def validate_response_shape(response: dict) -> dict:
    return validate_review_payload(response, subject="review response")


def write_merged_review(
    output_path: Path | None,
    base: str | None,
    full_scan: bool,
    responses: list[tuple[str, int, dict]],
    runner_warnings: list[str],
    completed_chunks: int,
    total_chunks: int,
    complete: bool,
) -> dict:
    payload = merge_pass_responses(base, full_scan, responses, runner_warnings)
    output_payload = {
        **payload,
        "complete": complete,
        "completed_chunks": completed_chunks,
        "total_chunks": total_chunks,
        "runner_warnings": runner_warnings,
    }
    if output_path:
        output_path.parent.mkdir(parents=True, exist_ok=True)
        output_path.write_text(json.dumps(output_payload, indent=2) + "\n", encoding="utf-8")
    return output_payload


def main() -> int:
    args = parse_args()
    root = repo_root()

    if not args.allow_mock:
        forbidden_mock_envs = [
            name
            for name in (
                MOCK_ALLOW_ENV,
                "SPECIAL_RUST_RELEASE_REVIEW_MOCK_OUTPUT",
                "SPECIAL_RUST_RELEASE_REVIEW_MOCK_EXIT_CODE",
            )
            if os.environ.get(name) is not None
        ]
        if forbidden_mock_envs:
            raise SystemExit(
                "release review mock controls are test-only; unset "
                + ", ".join(sorted(forbidden_mock_envs))
            )

    if has_jj_root(root):
        backend = "jj"
    elif has_git_root(root):
        backend = "git"
    else:
        raise SystemExit("repository root must contain .jj or .git")

    review_mode, model = selected_model(args)
    output_path = resolve_output_path(args, root, review_mode, model)
    if not args.dry_run:
        if output_path:
            status(EXPENSIVE_REVIEW_NOTE)
            status(f"model: {model} ({review_mode}); output: {output_path}")
        else:
            status(
                "stdout-only diff review. Use --output for durable capture before running any expensive review."
            )
    version = load_version(root)
    head = args.head or ("@" if backend == "jj" else "HEAD")
    base = None if args.full else (args.base or discover_latest_semver_tag(root, backend, head))
    review_files = (
        full_scan_files(root, backend)
        if args.full
        else changed_files_from_diff(root, backend, base, head)
    )
    review_passes = build_review_passes(review_files)

    changed_line_ranges: dict[str, list[tuple[int, int]]] = {}
    if not args.full and review_files and base is not None:
        changed_line_ranges = parse_changed_line_ranges(
            diff_text_for_paths(root, backend, base, head, review_files)
        )

    chunk_records: list[dict[str, object]] = []
    runner_warnings: list[str] = []

    for review_pass in review_passes:
        pass_files = list(review_pass["files"])
        file_contexts = build_file_contexts(root, pass_files, changed_line_ranges, args.full)
        chunks, pass_runner_warnings = build_pass_chunks(
            root,
            version,
            backend,
            base,
            head,
            args.full,
            review_pass,
            file_contexts,
        )
        runner_warnings.extend(pass_runner_warnings)
        chunk_records.extend(chunks)

    if args.dry_run:
        preview = validate_review_preview(
            {
                "model": model,
                "review_mode": review_mode,
                "codex_invocation": codex_invocation_config(model),
                "schema_path": str(SCHEMA_PATH),
                "backend": backend,
                "baseline": base,
                "head": head,
                "full_scan": args.full,
                "changed_files": review_files,
                "runner_warnings": runner_warnings,
                "review_passes": [
                    {
                        "name": review_pass["name"],
                        "focus": review_pass["focus"],
                        "files": review_pass["files"],
                        "chunks": [
                            {
                                "chunk_index": chunk["chunk_index"],
                                "chunk_count": chunk["chunk_count"],
                                "files": chunk["files"],
                                "estimated_chars": chunk["estimated_chars"],
                                "file_contexts": chunk["file_contexts"],
                                "prompt": chunk["prompt"],
                            }
                            for chunk in chunk_records
                            if chunk["name"] == review_pass["name"]
                        ],
                    }
                    for review_pass in review_passes
                ],
            },
            subject="review dry-run preview",
        )
        print(json.dumps(preview, indent=2))
        return 0

    if running_in_ci():
        raise SystemExit(
            "rust release review is local-only and must not invoke Codex from CI; "
            "use --dry-run for wrapper verification"
        )

    if not chunk_records:
        payload = write_merged_review(output_path, base, args.full, [], runner_warnings, 0, 0, True)
        for warning in runner_warnings:
            print(warning, file=sys.stderr)
        if output_path:
            print(payload["summary"])
            print(f"Wrote review JSON to {output_path}")
        else:
            print(json.dumps(merge_pass_responses(base, args.full, [], runner_warnings), indent=2))
        return 1 if runner_warnings else 0

    responses: list[tuple[str, int, dict]] = []
    max_workers = min(len(chunk_records), MAX_CONCURRENT_REVIEW_CHUNKS)
    completed_chunks = 0
    total_chunks = len(chunk_records)

    status(
        f"planned {total_chunks} review chunk(s) across {len(review_files)} file(s); max workers: {max_workers or 1}"
    )
    write_merged_review(
        output_path,
        base,
        args.full,
        responses,
        runner_warnings,
        completed_chunks,
        total_chunks,
        False,
    )

    with concurrent.futures.ThreadPoolExecutor(max_workers=max_workers or 1) as executor:
        future_to_chunk = {
            executor.submit(
                invoke_codex,
                root,
                str(chunk["prompt"]),
                model,
                review_mode,
                SCHEMA_PATH,
                validate_response_shape,
            ): chunk
            for chunk in chunk_records
        }
        for future in concurrent.futures.as_completed(future_to_chunk):
            chunk = future_to_chunk[future]
            try:
                response = future.result()
            except CodexInvocationError as error:
                runner_warnings.append(
                    f"review chunk {chunk['name']}#{chunk['chunk_index']} failed: {error}"
                )
                completed_chunks += 1
                status(
                    f"failed {chunk['name']}#{chunk['chunk_index']}/{chunk['chunk_count']} "
                    f"({completed_chunks}/{total_chunks} chunk(s) completed); {flush_status(output_path)}"
                )
                write_merged_review(
                    output_path,
                    base,
                    args.full,
                    responses,
                    runner_warnings,
                    completed_chunks,
                    total_chunks,
                    False,
                )
                continue
            except Exception as error:
                runner_warnings.append(
                    f"review chunk {chunk['name']}#{chunk['chunk_index']} crashed: {error}"
                )
                completed_chunks += 1
                status(
                    f"failed {chunk['name']}#{chunk['chunk_index']}/{chunk['chunk_count']} "
                    f"({completed_chunks}/{total_chunks} chunk(s) completed); {flush_status(output_path)}"
                )
                write_merged_review(
                    output_path,
                    base,
                    args.full,
                    responses,
                    runner_warnings,
                    completed_chunks,
                    total_chunks,
                    False,
                )
                continue
            responses.append((str(chunk["name"]), int(chunk["chunk_index"]), response))
            completed_chunks += 1
            warning_count = len(response.get("warnings", []))
            status(
                f"completed {chunk['name']}#{chunk['chunk_index']}/{chunk['chunk_count']} "
                f"with {warning_count} warning(s) "
                f"({completed_chunks}/{total_chunks} chunk(s) completed); {flush_status(output_path)}"
            )
            write_merged_review(
                output_path,
                base,
                args.full,
                responses,
                runner_warnings,
                completed_chunks,
                total_chunks,
                False,
            )

    responses.sort(key=lambda item: (item[0], item[1]))
    payload = write_merged_review(
        output_path,
        base,
        args.full,
        responses,
        runner_warnings,
        completed_chunks,
        total_chunks,
        True,
    )
    for warning in runner_warnings:
        print(warning, file=sys.stderr)
    if output_path:
        print(payload["summary"])
        print(f"Wrote review JSON to {output_path}")
    else:
        print(json.dumps(merge_pass_responses(base, args.full, responses, runner_warnings), indent=2))
    mock_exit_code = os.environ.get("SPECIAL_RUST_RELEASE_REVIEW_MOCK_EXIT_CODE")
    if mock_exit_code is not None and os.environ.get(MOCK_ALLOW_ENV) == "1":
        return int(mock_exit_code)
    return 1 if runner_warnings else 0


if __name__ == "__main__":
    raise SystemExit(main())
