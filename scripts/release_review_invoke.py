# @module SPECIAL.RELEASE_REVIEW.INVOKE
# Model-runner invocation policy and runtime error handling for local release review. This module does not choose review passes or merge warning payloads.
# @fileimplements SPECIAL.RELEASE_REVIEW.INVOKE
from __future__ import annotations

import json
import os
import subprocess
from pathlib import Path


DEFAULT_MODEL = "gpt-5.3-codex"
FAST_MODEL = "gpt-5.3-codex-spark"
SMART_MODEL = "gpt-5.4"
SWARM_MODEL = "deepseek/deepseek-v4-flash"
SWARM_HARNESS = "opencode"
OPENCODE_SWARM_PERMISSION = {
    "*": "deny",
    "read": "allow",
    "glob": "allow",
    "grep": "allow",
    "list": "allow",
    "edit": "deny",
    "bash": "deny",
    "task": "deny",
    "todoread": "allow",
    "todowrite": "deny",
    "webfetch": "deny",
    "websearch": "deny",
    "codesearch": "deny",
    "external_directory": "deny",
    "lsp": "deny",
}
PERMISSIONS_PROFILE = "release_review"
MOCK_ALLOW_ENV = "SPECIAL_RUST_RELEASE_REVIEW_ALLOW_MOCK"
MOCK_OUTPUT_ENV = "SPECIAL_RUST_RELEASE_REVIEW_MOCK_OUTPUT"
QUOTA_ERROR_MARKERS = (
    "quota",
    "rate limit",
    "rate_limit",
    "rate-limit",
    "usage limit",
    "usage_limit",
    "insufficient_quota",
    "quota_exceeded",
    "too many requests",
    "429",
    "5hr",
    "7day",
)


class CodexInvocationError(RuntimeError):
    pass


def quota_guidance(model_mode: str) -> str | None:
    if model_mode == "fast":
        return (
            f"{FAST_MODEL} appears quota-limited. Rerun without --fast for the default "
            f"{DEFAULT_MODEL} review, or use --smart for {SMART_MODEL}."
        )
    if model_mode == "smart":
        return (
            f"{SMART_MODEL} appears quota-limited. Rerun without --smart for the default "
            f"{DEFAULT_MODEL} review, or use --fast if Spark quota is available."
        )
    return None


def quota_guidance_for_error(stderr: str, model_mode: str) -> str | None:
    lower = stderr.lower()
    if not any(marker in lower for marker in QUOTA_ERROR_MARKERS):
        return None
    return quota_guidance(model_mode)


def codex_invocation_config(model: str) -> dict[str, object]:
    return {
        "model": model,
        "sandbox_mode": "read-only",
        "web_search": "disabled",
        "default_permissions": PERMISSIONS_PROFILE,
        "filesystem_permissions": {
            ":project_roots": {
                ".": "read",
            }
        },
    }


def swarm_invocation_config() -> dict[str, object]:
    return {
        "harness": SWARM_HARNESS,
        "model": SWARM_MODEL,
        "sandbox_mode": "opencode-read-only-permissions",
        "web_search": "denied",
        "default_permissions": "opencode_swarm_read_only",
        "filesystem_permissions": {
            "opencode": OPENCODE_SWARM_PERMISSION,
        },
        "format": "default",
        "prompt_transport": "stdin",
        "permission": OPENCODE_SWARM_PERMISSION,
        "auth_source": "~/.local/share/opencode/auth.json",
    }


def opencode_run_command(model: str, root: Path, title: str) -> list[str]:
    return [
        "opencode",
        "run",
        "--model",
        model,
        "--dir",
        str(root),
        "--title",
        title,
    ]


def opencode_permission_config() -> str:
    return json.dumps(
        {
            "$schema": "https://opencode.ai/config.json",
            "permission": OPENCODE_SWARM_PERMISSION,
        },
        separators=(",", ":"),
    )


def invoke_opencode(
    root: Path,
    prompt: str,
    model: str,
    validate_response_shape,
) -> dict:
    mocked = os.environ.get(MOCK_OUTPUT_ENV)
    if mocked and os.environ.get(MOCK_ALLOW_ENV) == "1":
        try:
            return validate_response_shape(json.loads(mocked))
        except (json.JSONDecodeError, SystemExit) as err:
            raise CodexInvocationError(f"mocked review output was invalid: {err}") from err

    result = subprocess.run(
        opencode_run_command(model, root, "Special release swarm review"),
        cwd=root,
        input=prompt,
        capture_output=True,
        text=True,
        env={
            **os.environ,
            "OPENCODE_CONFIG_CONTENT": opencode_permission_config(),
        },
    )
    if result.returncode != 0:
        stderr = result.stderr.strip()
        raise CodexInvocationError(stderr or f"opencode exited with status {result.returncode}")

    try:
        return validate_response_shape(json.loads(extract_json_text(opencode_final_text(result.stdout))))
    except (json.JSONDecodeError, SystemExit) as err:
        raise CodexInvocationError(f"opencode returned invalid structured output: {err}") from err


def invoke_opencode_text(root: Path, prompt: str, model: str) -> str:
    mocked = os.environ.get(MOCK_OUTPUT_ENV)
    if mocked and os.environ.get(MOCK_ALLOW_ENV) == "1":
        return mocked

    result = subprocess.run(
        opencode_run_command(model, root, "Special release swarm review"),
        cwd=root,
        input=prompt,
        capture_output=True,
        text=True,
        env={
            **os.environ,
            "OPENCODE_CONFIG_CONTENT": opencode_permission_config(),
        },
    )
    if result.returncode != 0:
        stderr = result.stderr.strip()
        raise CodexInvocationError(stderr or f"opencode exited with status {result.returncode}")
    return opencode_final_text(result.stdout)


def opencode_final_text(output: str) -> str:
    parts: list[str] = []
    errors: list[str] = []
    saw_json_event = False
    for line in output.splitlines():
        if not line.strip():
            continue
        try:
            event = json.loads(line)
        except json.JSONDecodeError:
            continue
        if "type" not in event:
            continue
        saw_json_event = True
        if event.get("type") == "text":
            text = event.get("text")
            if isinstance(text, str):
                parts.append(text)
            part = event.get("part")
            if isinstance(part, dict) and isinstance(part.get("text"), str):
                parts.append(part["text"])
        if event.get("type") == "error":
            error = event.get("error")
            if isinstance(error, dict):
                data = error.get("data")
                message = error.get("message")
                if not isinstance(message, str) and isinstance(data, dict):
                    message = data.get("message")
                if isinstance(message, str):
                    errors.append(message)
    if parts:
        return "".join(parts).strip()
    if errors:
        raise CodexInvocationError("; ".join(errors))
    if saw_json_event:
        raise CodexInvocationError("opencode produced no text output")
    plain = output.strip()
    if plain:
        return plain
    raise CodexInvocationError("opencode produced no text output")


def extract_json_text(text: str) -> str:
    stripped = text.strip()
    if stripped.startswith("```"):
        lines = stripped.splitlines()
        if lines and lines[0].startswith("```"):
            lines = lines[1:]
        if lines and lines[-1].strip() == "```":
            lines = lines[:-1]
        stripped = "\n".join(lines).strip()
    return stripped


def codex_exec_command(model: str, schema_path: Path) -> list[str]:
    config = codex_invocation_config(model)
    filesystem_toml = '{":project_roots"={"."="read"}}'
    return [
        "codex",
        "exec",
        "--ephemeral",
        "--sandbox",
        str(config["sandbox_mode"]),
        "-c",
        f'web_search="{config["web_search"]}"',
        "-c",
        f'default_permissions="{config["default_permissions"]}"',
        "-c",
        f"permissions.{config['default_permissions']}.filesystem={filesystem_toml}",
        "--skip-git-repo-check",
        "--model",
        model,
        "--output-schema",
        str(schema_path),
        "-",
    ]


def invoke_codex(
    root: Path,
    prompt: str,
    model: str,
    model_mode: str,
    schema_path: Path,
    validate_response_shape,
) -> dict:
    mocked = os.environ.get(MOCK_OUTPUT_ENV)
    if mocked and os.environ.get(MOCK_ALLOW_ENV) == "1":
        try:
            return validate_response_shape(json.loads(mocked))
        except (json.JSONDecodeError, SystemExit) as err:
            raise CodexInvocationError(f"mocked review output was invalid: {err}") from err

    result = subprocess.run(
        codex_exec_command(model, schema_path),
        cwd=root,
        input=prompt,
        capture_output=True,
        text=True,
    )
    if result.returncode != 0:
        stderr = result.stderr.strip()
        guidance = quota_guidance_for_error(stderr, model_mode)
        if guidance:
            raise CodexInvocationError(f"{stderr}\n{guidance}")
        raise CodexInvocationError(stderr or f"codex exited with status {result.returncode}")

    try:
        return validate_response_shape(json.loads(result.stdout))
    except (json.JSONDecodeError, SystemExit) as err:
        raise CodexInvocationError(f"codex returned invalid structured output: {err}") from err
