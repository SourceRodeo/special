# @module SPECIAL.RELEASE_REVIEW.SWARM
# DeepSeek swarm prompt assembly and parallel invocation for release review.
# @fileimplements SPECIAL.RELEASE_REVIEW.SWARM
from __future__ import annotations

import concurrent.futures
import math
import json
import sys
import time
from pathlib import Path

from release_review_contract import validate_review_preview
from release_review_invoke import (
    SWARM_MODEL,
    CodexInvocationError,
    invoke_opencode,
    swarm_invocation_config,
)
from release_review_merge import merge_pass_responses

DEFAULT_SWARM_COUNT = 3
MAX_SWARM_COUNT = 12
MAX_SWARM_PROMPT_CHARS = 900_000
SWARM_HEARTBEAT_SECONDS = 30.0
SWARM_PROMPT_HEADROOM_CHARS = 100_000


def recommended_swarm_count(root: Path, files: list[str]) -> int:
    total_size = sum(file_review_size(root, relative) for relative in files)
    effective_budget = max(1, MAX_SWARM_PROMPT_CHARS - SWARM_PROMPT_HEADROOM_CHARS)
    size_count = math.ceil(total_size / effective_budget) if total_size else 1
    return min(MAX_SWARM_COUNT, max(DEFAULT_SWARM_COUNT, size_count))


def build_swarm_assignments(root: Path, files: list[str], count: int) -> list[list[str]]:
    assignments = [[] for _ in range(count)]
    assignment_sizes = [0 for _ in range(count)]
    sized_files = sorted(
        ((file_review_size(root, relative), relative) for relative in files),
        key=lambda item: (-item[0], item[1]),
    )
    for size, relative in sized_files:
        index = min(range(count), key=lambda candidate: (assignment_sizes[candidate], candidate))
        assignments[index].append(relative)
        assignment_sizes[index] += size
    return [sorted(assigned) for assigned in assignments]


def file_review_size(root: Path, relative: str) -> int:
    path = root / relative
    if not path.is_file():
        return 0
    try:
        return path.stat().st_size
    except OSError:
        return 0


def repo_manifest(files: list[str], assigned: set[str]) -> str:
    lines = ["Repo file manifest. Files marked `assigned` are included below."]
    for relative in files:
        marker = "assigned" if relative in assigned else "context-only"
        lines.append(f"- [{marker}] {relative}")
    return "\n".join(lines)


def build_swarm_prompt(
    root: Path,
    version: str,
    backend: str,
    head: str,
    agent_index: int,
    agent_count: int,
    files: list[str],
    assigned_files: list[str],
) -> tuple[str, list[str]]:
    warnings: list[str] = []
    assigned = set(assigned_files)
    header = f"""You are DeepSeek V4 Flash reviewer {agent_index} of {agent_count} for Special {version}.

Perform an independent implementation-quality release review over your assigned repo slice.
Focus on correctness, behavioral regressions, parser and graph-model bugs, CLI/MCP/plugin boundary mismatches, release risks, test honesty, and maintainability problems.
Use the repo manifest to understand where your slice sits, but only make findings anchored in file contents included in this prompt.
Do not perform a product strategy, spec-authoring, or architecture-preference review.
Do not recommend changing intended Special semantics unless the implementation contradicts an existing claim, command behavior, or documented contract.

Return only JSON matching this schema shape:
{{
  "baseline": null,
  "full_scan": true,
  "summary": "short summary",
  "warnings": [
    {{
      "id": "temporary-id",
      "category": "api-design|type-design|state-model|error-handling|layering|test-quality|maintainability|rust-idioms|release-risk",
      "severity": "warn",
      "title": "specific issue",
      "why_it_matters": "why this could hurt release quality",
      "evidence": [{{"path": "repo/path", "line": 1, "detail": "anchored detail"}}],
      "recommendation": "concrete fix"
    }}
  ]
}}

Repository backend: {backend}
Review head: {head}
Assigned files: {len(assigned_files)} of {len(files)}

{repo_manifest(files, assigned)}
"""
    parts = [header]
    current_size = len(header)
    included = 0
    for relative in assigned_files:
        path = root / relative
        if not path.is_file():
            continue
        content = path.read_text(encoding="utf-8", errors="replace")
        block = f"\n--- FILE: {relative} ---\n{content}\n--- END FILE: {relative} ---\n"
        if current_size + len(block) > MAX_SWARM_PROMPT_CHARS:
            warnings.append(
                f"swarm prompt {agent_index} omitted {len(assigned_files) - included} assigned file(s) after reaching {MAX_SWARM_PROMPT_CHARS} char budget"
            )
            break
        parts.append(block)
        current_size += len(block)
        included += 1
    return ("".join(parts), warnings)


def swarm_dry_run_preview(
    root: Path,
    version: str,
    backend: str,
    head: str,
    count: int,
    files: list[str],
    schema_path: Path,
) -> dict:
    prompts = []
    runner_warnings: list[str] = []
    assignments = build_swarm_assignments(root, files, count)
    for index, assigned_files in enumerate(assignments, start=1):
        prompt, warnings = build_swarm_prompt(
            root, version, backend, head, index, count, files, assigned_files
        )
        runner_warnings.extend(warnings)
        prompts.append(
            {
                "chunk_index": index,
                "chunk_count": count,
                "estimated_chars": len(prompt),
                "files": assigned_files,
                "file_contexts": [],
                "prompt": prompt,
            }
        )
    return validate_review_preview(
        {
            "model": SWARM_MODEL,
            "review_mode": "swarm",
            "codex_invocation": swarm_invocation_config(),
            "schema_path": str(schema_path),
            "backend": backend,
            "baseline": None,
            "head": head,
            "full_scan": True,
            "changed_files": files,
            "runner_warnings": runner_warnings,
            "review_passes": [
                {
                    "name": "deepseek_swarm",
                    "focus": [
                        "parallel full-repository implementation-quality review using DeepSeek V4 Flash"
                    ],
                    "files": files,
                    "chunks": prompts,
                }
            ],
        },
        subject="review swarm dry-run preview",
    )


def run_swarm_review(
    root: Path,
    version: str,
    backend: str,
    head: str,
    count: int,
    files: list[str],
    validate_response_shape,
) -> tuple[dict, list[str], int]:
    responses: list[tuple[str, int, dict]] = []
    runner_warnings: list[str] = []
    print(f"swarm: preparing {count} DeepSeek review agent(s)", file=sys.stderr, flush=True)
    assignments = build_swarm_assignments(root, files, count)
    with concurrent.futures.ThreadPoolExecutor(max_workers=count) as executor:
        future_to_index = {}
        future_started_at = {}
        future_last_report = {}
        for index, assigned_files in enumerate(assignments, start=1):
            prompt, prompt_warnings = build_swarm_prompt(
                root, version, backend, head, index, count, files, assigned_files
            )
            runner_warnings.extend(prompt_warnings)
            print(
                f"swarm: agent {index}/{count} starting OpenCode "
                f"with {len(assigned_files)} assigned file(s) ({len(prompt)} prompt chars)",
                file=sys.stderr,
                flush=True,
            )
            future = executor.submit(
                invoke_opencode,
                root,
                prompt,
                SWARM_MODEL,
                validate_response_shape,
            )
            future_to_index[future] = index
            future_started_at[future] = time.monotonic()
            future_last_report[future] = future_started_at[future]
        pending = set(future_to_index)
        completed = 0
        while pending:
            done, pending = concurrent.futures.wait(
                pending,
                timeout=SWARM_HEARTBEAT_SECONDS,
                return_when=concurrent.futures.FIRST_COMPLETED,
            )
            now = time.monotonic()
            for future in sorted(done, key=lambda item: future_to_index[item]):
                index = future_to_index[future]
                elapsed = now - future_started_at[future]
                completed += 1
                print(
                    f"swarm: agent {index}/{count} completed after {elapsed:.0f}s "
                    f"({completed}/{count})",
                    file=sys.stderr,
                    flush=True,
                )
                try:
                    responses.append(("deepseek_swarm", index, future.result()))
                except CodexInvocationError as error:
                    runner_warnings.append(f"review swarm agent {index} failed: {error}")
                except Exception as error:
                    runner_warnings.append(f"review swarm agent {index} crashed: {error}")
            for future in sorted(pending, key=lambda item: future_to_index[item]):
                if now - future_last_report[future] >= SWARM_HEARTBEAT_SECONDS:
                    index = future_to_index[future]
                    elapsed = now - future_started_at[future]
                    print(
                        f"swarm: agent {index}/{count} still running after {elapsed:.0f}s",
                        file=sys.stderr,
                        flush=True,
                    )
                    future_last_report[future] = now
    responses.sort(key=lambda item: item[1])
    return merge_pass_responses(None, True, responses, runner_warnings), runner_warnings, count
