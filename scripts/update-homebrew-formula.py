#!/usr/bin/env python3
# @module SPECIAL.DISTRIBUTION.HOMEBREW_UPDATE
# Homebrew formula update in `scripts/update-homebrew-formula.py`.
# @fileimplements SPECIAL.DISTRIBUTION.HOMEBREW_UPDATE

from __future__ import annotations

import sys

sys.dont_write_bytecode = True

import base64
import json
import subprocess
from pathlib import Path

SCRIPT_DIR = Path(__file__).resolve().parent
if str(SCRIPT_DIR) not in sys.path:
    sys.path.insert(0, str(SCRIPT_DIR))

from release_tooling import normalize_tag, package_version, run_checked

REPOSITORY = "sourcerodeo/special"
TAP_REPOSITORY = "sourcerodeo/homebrew-tap"
FORMULA_PATH = "Formula/special.rb"
FORMULA_DESC = "Repo-native semantic spec tool"
FORMULA_HOMEPAGE = "https://github.com/sourcerodeo/special"
ARCHIVE_ORDER = [
    "special-cli-x86_64-apple-darwin.tar.xz",
    "special-cli-aarch64-apple-darwin.tar.xz",
    "special-cli-x86_64-unknown-linux-gnu.tar.xz",
    "special-cli-aarch64-unknown-linux-gnu.tar.xz",
]


def repo_root() -> Path:
    return Path(__file__).resolve().parent.parent


def release_assets(root: Path, tag: str) -> dict[str, dict]:
    payload = json.loads(
        run_checked(
            root,
            ["gh", "release", "view", tag, "--repo", REPOSITORY, "--json", "assets"],
        )
    )
    return {asset["name"]: asset for asset in payload["assets"]}


def formula_file(root: Path) -> tuple[str | None, str | None]:
    result = subprocess.run(
        ["gh", "api", f"repos/{TAP_REPOSITORY}/contents/{FORMULA_PATH}"],
        cwd=root,
        capture_output=True,
        text=True,
    )
    if result.returncode != 0:
        if "HTTP 404" in result.stderr or "Not Found" in result.stderr:
            return None, None
        sys.stderr.write(result.stderr)
        raise SystemExit(result.returncode)
    payload = json.loads(result.stdout)
    content = base64.b64decode(payload["content"]).decode()
    return content, str(payload["sha"])


def release_asset_names(root: Path) -> list[str]:
    payload = json.loads(root.joinpath("scripts/release-assets.json").read_text())
    return list(payload["homebrew_formula_archives"])


def asset_sha256(asset: dict) -> str:
    digest = str(asset["digest"])
    if not digest.startswith("sha256:"):
        raise SystemExit(f"release asset digest is not sha256: {asset}")
    return digest.removeprefix("sha256:")


def build_formula(version: str, assets: dict[str, dict]) -> str:
    def archive_url(name: str) -> str:
        return str(assets[name]["url"])

    def archive_sha(name: str) -> str:
        return asset_sha256(assets[name])

    return f"""# typed: false
# frozen_string_literal: true

class Special < Formula
  desc "{FORMULA_DESC}"
  homepage "{FORMULA_HOMEPAGE}"
  version "{version}"

  archive = on_system_conditional(
    macos: on_arch_conditional(
      arm: "special-cli-aarch64-apple-darwin.tar.xz",
      intel: "special-cli-x86_64-apple-darwin.tar.xz",
    ),
    linux: on_arch_conditional(
      arm: "special-cli-aarch64-unknown-linux-gnu.tar.xz",
      intel: "special-cli-x86_64-unknown-linux-gnu.tar.xz",
    ),
  )
  url "https://github.com/sourcerodeo/special/releases/download/v{version}/#{{archive}}"
  sha256 on_system_conditional(
    macos: on_arch_conditional(
      arm: "{archive_sha('special-cli-aarch64-apple-darwin.tar.xz')}",
      intel: "{archive_sha('special-cli-x86_64-apple-darwin.tar.xz')}",
    ),
    linux: on_arch_conditional(
      arm: "{archive_sha('special-cli-aarch64-unknown-linux-gnu.tar.xz')}",
      intel: "{archive_sha('special-cli-x86_64-unknown-linux-gnu.tar.xz')}",
    ),
  )

  def install
    bin.install "special"
  end

  test do
    system "#{{bin}}/special", "--help"
  end
end
"""


def main() -> int:
    root = repo_root()
    version = package_version(root)
    tag = normalize_tag(version)
    assets = release_assets(root, tag)
    required = release_asset_names(root)
    missing = sorted(set(required) - assets.keys())
    if missing:
        raise SystemExit(f"missing required release assets for Homebrew formula: {missing}")

    _, sha = formula_file(root)
    formula = build_formula(version, assets)
    encoded = base64.b64encode(formula.encode()).decode()
    command = [
        "gh",
        "api",
        f"repos/{TAP_REPOSITORY}/contents/{FORMULA_PATH}",
        "--method",
        "PUT",
        "-f",
        f"message=Update Formula/special.rb for {tag}",
        "-f",
        f"content={encoded}",
        "-f",
        "branch=main",
    ]
    if sha is not None:
        command.extend(["-f", f"sha={sha}"])
    run_checked(root, command)
    print(f"Updated {TAP_REPOSITORY}/{FORMULA_PATH} for {tag}")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
