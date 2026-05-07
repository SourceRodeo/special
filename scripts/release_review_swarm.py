# @module SPECIAL.RELEASE_REVIEW.SWARM
# DeepSeek swarm prompt assembly and parallel invocation for release review.
# @fileimplements SPECIAL.RELEASE_REVIEW.SWARM
from __future__ import annotations

import concurrent.futures
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


def build_swarm_prompt(
    root: Path,
    version: str,
    backend: str,
    head: str,
    agent_index: int,
    agent_count: int,
    files: list[str],
) -> tuple[str, list[str]]:
    warnings: list[str] = []
    header = f"""You are DeepSeek V4 Flash reviewer {agent_index} of {agent_count} for Special {version}.

Perform an independent full-repository implementation-quality release review.
Focus on correctness, behavioral regressions, parser and graph-model bugs, CLI/MCP/plugin boundary mismatches, release risks, test honesty, and maintainability problems.
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

"""
    parts = [header]
    current_size = len(header)
    included = 0
    for relative in files:
        path = root / relative
        if not path.is_file():
            continue
        content = path.read_text(encoding="utf-8", errors="replace")
        block = f"\n--- FILE: {relative} ---\n{content}\n--- END FILE: {relative} ---\n"
        if current_size + len(block) > MAX_SWARM_PROMPT_CHARS:
            warnings.append(
                f"swarm prompt {agent_index} omitted {len(files) - included} file(s) after reaching {MAX_SWARM_PROMPT_CHARS} char budget"
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
    for index in range(1, count + 1):
        prompt, warnings = build_swarm_prompt(root, version, backend, head, index, count, files)
        runner_warnings.extend(warnings)
        prompts.append(
            {
                "chunk_index": index,
                "chunk_count": count,
                "estimated_chars": len(prompt),
                "files": files,
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
    with concurrent.futures.ThreadPoolExecutor(max_workers=count) as executor:
        future_to_index = {}
        future_started_at = {}
        future_last_report = {}
        for index in range(1, count + 1):
            prompt, prompt_warnings = build_swarm_prompt(
                root, version, backend, head, index, count, files
            )
            runner_warnings.extend(prompt_warnings)
            print(
                f"swarm: agent {index}/{count} starting OpenCode "
                f"({len(prompt)} prompt chars)",
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
