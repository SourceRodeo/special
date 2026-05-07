#!/usr/bin/env python3
# Validates shipped skill templates against Special's supported command surface.
# @fileimplements SPECIAL.DISTRIBUTION.RELEASE_FLOW

from __future__ import annotations

import re
import shlex
import sys
from dataclasses import dataclass
from pathlib import Path

ROOT = Path(__file__).resolve().parent.parent
SKILL_ROOTS = [
    ROOT / "templates" / "skills",
    ROOT / "codex-plugin" / "special" / "skills",
]

COMMAND_OPTIONS = {
    "": {"--help", "--version"},
    "arch": {"--current", "--html", "--json", "--metrics", "--planned", "--unimplemented", "--verbose"},
    "docs": {"--json", "--metrics", "--output", "--target", "--verbose"},
    "health": {"--html", "--json", "--metrics", "--symbol", "--target", "--verbose"},
    "init": set(),
    "lint": set(),
    "mcp": set(),
    "patterns": {"--html", "--json", "--metrics", "--symbol", "--target", "--verbose"},
    "skills": {"--destination", "--force"},
    "specs": {
        "--current",
        "--deprecated",
        "--html",
        "--json",
        "--metrics",
        "--planned",
        "--proofs",
        "--unverified",
        "--unsupported",
        "--verbose",
    },
}

OPTIONS_WITH_VALUES = {"--destination", "--symbol", "--target"}
OPTIONS_WITH_OPTIONAL_VALUES = {"--output"}
POSITIONAL_COMMANDS = {"arch", "patterns", "specs"}
DOCS_BUILD_OPTIONS = {"--output", "--target"}


@dataclass(frozen=True)
class CommandExample:
    path: Path
    line_number: int
    text: str


def main() -> int:
    errors: list[str] = []
    for path in skill_markdown_files():
        text = path.read_text(encoding="utf-8")
        if re.search(r"\bspecial\s+modules?\b", text):
            errors.append(f"{display(path)}: uses stale `special modules` wording")
        if re.search(r"\bspecial\s+repo\b", text):
            errors.append(f"{display(path)}: uses unsupported `special repo` command wording")
        if re.search(r"\bSPECIAL\.", text):
            errors.append(f"{display(path)}: public skill examples must not use Special's own `SPECIAL.*` ids")
        for example in command_examples(path, text):
            errors.extend(validate_command_example(example))

    if errors:
        for error in errors:
            print(error, file=sys.stderr)
        return 1
    print("Skill template command examples are current.")
    return 0


def skill_markdown_files() -> list[Path]:
    files: list[Path] = []
    for root in SKILL_ROOTS:
        if root.exists():
            files.extend(sorted(root.rglob("*.md")))
    return files


def command_examples(path: Path, text: str) -> list[CommandExample]:
    examples: list[CommandExample] = []
    in_fence = False
    for index, line in enumerate(text.splitlines(), start=1):
        stripped = line.strip()
        if stripped.startswith("```"):
            in_fence = not in_fence
            continue
        if in_fence and stripped.startswith("special"):
            examples.append(CommandExample(path, index, stripped))
        for match in re.finditer(r"`(special(?:\s+[^`]+)?)`", line):
            candidate = match.group(1).strip()
            if should_validate_inline_command(candidate):
                examples.append(CommandExample(path, index, candidate))
    return examples


def should_validate_inline_command(candidate: str) -> bool:
    tokens = split_command(candidate)
    if not tokens or tokens[0] != "special":
        return False
    if len(tokens) <= 2:
        return True
    subcommand = tokens[1]
    if subcommand not in COMMAND_OPTIONS:
        return True
    if subcommand in POSITIONAL_COMMANDS:
        return all(token_is_command_like(token) for token in tokens[2:])
    if subcommand == "skills":
        return len(tokens) == 3 and tokens[2] == "install"
    return all(token.startswith("--") for token in tokens[2:])


def token_is_command_like(token: str) -> bool:
    return (
        token.startswith("--")
        or "." in token
        or "/" in token
        or token.isupper()
        or re.fullmatch(r"[A-Z][A-Z0-9_]*(?:\.[A-Z0-9_]+)*", token) is not None
    )


def validate_command_example(example: CommandExample) -> list[str]:
    tokens = split_command(example.text)
    if not tokens or tokens[0] != "special":
        return []
    if len(tokens) == 1:
        return []

    errors: list[str] = []
    subcommand = tokens[1]
    if subcommand.startswith("--"):
        subcommand = ""
        index = 1
    else:
        index = 2
    if subcommand not in COMMAND_OPTIONS:
        return [f"{location(example)}: unknown `special {subcommand}` command in `{example.text}`"]

    if subcommand == "skills" and index < len(tokens) and tokens[index] == "install":
        index += 1
    docs_build = subcommand == "docs" and index < len(tokens) and tokens[index] == "build"
    if docs_build:
        index += 1
    active_options = DOCS_BUILD_OPTIONS if docs_build else COMMAND_OPTIONS[subcommand]

    while index < len(tokens):
        token = tokens[index]
        if token.startswith("--"):
            if token not in active_options:
                errors.append(f"{location(example)}: unsupported option `{token}` in `{example.text}`")
            if token in OPTIONS_WITH_VALUES:
                index += 1
                if index >= len(tokens) or tokens[index].startswith("--"):
                    errors.append(f"{location(example)}: option `{token}` needs a value in `{example.text}`")
            elif (
                token in OPTIONS_WITH_OPTIONAL_VALUES
                and index + 1 < len(tokens)
                and not tokens[index + 1].startswith("--")
            ):
                index += 1
            index += 1
            continue

        if subcommand in POSITIONAL_COMMANDS:
            index += 1
            continue
        if docs_build:
            index += 1
            continue
        if subcommand == "skills" and re.fullmatch(r"[a-z0-9]+(?:-[a-z0-9]+)*", token):
            index += 1
            continue
        errors.append(f"{location(example)}: unsupported positional `{token}` in `{example.text}`")
        index += 1

    return errors


def split_command(command: str) -> list[str]:
    try:
        return shlex.split(command)
    except ValueError:
        return command.split()


def location(example: CommandExample) -> str:
    return f"{display(example.path)}:{example.line_number}"


def display(path: Path) -> str:
    return path.relative_to(ROOT).as_posix()


if __name__ == "__main__":
    raise SystemExit(main())
