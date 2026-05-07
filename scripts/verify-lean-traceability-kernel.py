#!/usr/bin/env python3
# @fileimplements SPECIAL.DISTRIBUTION.RELEASE_FLOW
from __future__ import annotations

import json
import os
import subprocess
import sys
from pathlib import Path


ROOT = Path(__file__).resolve().parent.parent
LEAN_ROOT = ROOT / "lean"
LEAN_TOOLCHAIN = (LEAN_ROOT / "lean-toolchain").read_text().strip()


def mise_elan_lake(*args: str) -> list[str]:
    return [
        "mise",
        "exec",
        "--",
        "elan",
        "run",
        LEAN_TOOLCHAIN,
        "lake",
        *args,
    ]


def release_kernel_path() -> Path:
    suffix = ".exe" if sys.platform == "win32" else ""
    return LEAN_ROOT / ".lake" / "build" / "bin" / f"special_traceability_kernel{suffix}"


def build_release_kernel() -> Path:
    subprocess.run(
        mise_elan_lake("-Krelease", "build", "special_traceability_kernel"),
        cwd=LEAN_ROOT,
        check=True,
    )
    kernel = release_kernel_path()
    if not kernel.is_file():
        raise SystemExit(f"release Lean kernel was not built at {kernel}")
    return kernel


def run_kernel(
    kernel: Path,
    payload: str,
    *,
    check: bool = True,
) -> subprocess.CompletedProcess[str]:
    result = subprocess.run(
        [str(kernel)],
        cwd=LEAN_ROOT,
        input=payload,
        text=True,
        capture_output=True,
        check=False,
    )
    if check and result.returncode != 0:
        raise SystemExit(
            f"Lean kernel failed with exit code {result.returncode}:\n"
            f"stdout:\n{result.stdout}\n"
            f"stderr:\n{result.stderr}"
        )
    return result


def parse_kernel_output(result: subprocess.CompletedProcess[str]) -> dict:
    json_lines = [line for line in result.stdout.splitlines() if line.startswith("{")]
    if not json_lines:
        raise SystemExit(f"Lean kernel did not emit JSON output:\n{result.stdout}")
    return json.loads(json_lines[-1])


def run_kernel_json(kernel: Path, fixture: dict) -> dict:
    return parse_kernel_output(run_kernel(kernel, json.dumps(fixture)))


def run_kernel_fixture(kernel: Path) -> None:
    fixture = {
        "schema_version": 1,
        "projected_item_ids": ["app::live", "app::orphan"],
        "preserved_reverse_closure_target_ids": None,
        "edges": {
            "app::helper": ["app::live"],
            "tests::test_helper": ["app::helper"],
            "tests::test_live": ["app::live"],
        },
        "support_root_ids": ["tests::test_live"],
    }
    output = run_kernel_json(kernel, fixture)
    reference = output["reference"]
    contract = reference["contract"]
    closure = reference["exact_reverse_closure"]

    assert output["schema_version"] == 1
    assert set(contract["projected_item_ids"]) == {"app::live", "app::orphan"}
    assert set(contract["preserved_reverse_closure_target_ids"]) == {"app::live"}
    assert set(closure["target_ids"]) == {"app::live"}
    assert set(closure["node_ids"]) == {
        "app::live",
        "app::helper",
        "tests::test_helper",
        "tests::test_live",
    }
    assert set(closure["internal_edges"]["app::helper"]) == {"app::live"}
    assert set(closure["internal_edges"]["tests::test_helper"]) == {"app::helper"}
    assert set(closure["internal_edges"]["tests::test_live"]) == {"app::live"}


def run_kernel_protocol_error_cases(kernel: Path) -> None:
    valid_fixture = {
        "schema_version": 1,
        "projected_item_ids": ["app::live"],
        "preserved_reverse_closure_target_ids": None,
        "edges": {},
        "support_root_ids": [],
    }
    cases = [
        ("malformed JSON", "{", None),
        (
            "missing projected ids",
            json.dumps(
                {
                    key: value
                    for key, value in valid_fixture.items()
                    if key != "projected_item_ids"
                }
            ),
            None,
        ),
        (
            "wrong-typed projected ids",
            json.dumps({**valid_fixture, "projected_item_ids": "app::live"}),
            None,
        ),
        (
            "wrong-typed edge map",
            json.dumps({**valid_fixture, "edges": []}),
            None,
        ),
        (
            "wrong-typed support root entry",
            json.dumps({**valid_fixture, "support_root_ids": ["tests::ok", 7]}),
            None,
        ),
        (
            "unsupported schema version",
            json.dumps({**valid_fixture, "schema_version": 2}),
            "unsupported traceability kernel schema version 2",
        ),
    ]

    for name, payload, expected in cases:
        result = run_kernel(kernel, payload, check=False)
        if result.returncode == 0:
            raise SystemExit(f"{name} unexpectedly succeeded:\n{result.stdout}")
        if "special traceability kernel error:" not in result.stderr:
            raise SystemExit(f"{name} did not report a kernel error:\n{result.stderr}")
        if expected is not None and expected not in result.stderr:
            raise SystemExit(
                f"{name} did not include expected error `{expected}`:\n{result.stderr}"
            )


def run_kernel_unicode_and_duplicate_array_case(kernel: Path) -> None:
    fixture = {
        "schema_version": 1,
        "projected_item_ids": ["app::目标", "app::目标"],
        "preserved_reverse_closure_target_ids": ["app::目标", "app::目标"],
        "edges": {
            "app::helper": ["app::目标", "app::目标"],
            "tests::测试": ["app::helper"],
            "tests::noise": ["app::noise"],
        },
        "support_root_ids": ["tests::测试", "tests::测试"],
    }
    output = run_kernel_json(kernel, fixture)
    closure = output["reference"]["exact_reverse_closure"]
    contract = output["reference"]["contract"]

    assert contract["projected_item_ids"] == ["app::目标"]
    assert contract["preserved_reverse_closure_target_ids"] == ["app::目标"]
    assert closure["target_ids"] == ["app::目标"]
    assert set(closure["node_ids"]) == {"app::目标", "app::helper", "tests::测试"}
    assert len(closure["node_ids"]) == 3
    assert set(closure["internal_edges"]["app::helper"]) == {"app::目标"}
    assert set(closure["internal_edges"]["tests::测试"]) == {"app::helper"}


def run_kernel_large_boundary_fixture(kernel: Path) -> None:
    depth = 96
    edges = {
        f"app::node_{index + 1}": [f"app::node_{index}"]
        for index in range(depth)
    }
    edges["tests::deep"] = [f"app::node_{depth}"]
    edges.update(
        {
            f"noise::caller_{index}": [f"noise::callee_{index}"]
            for index in range(40)
        }
    )
    fixture = {
        "schema_version": 1,
        "projected_item_ids": ["app::node_0"],
        "preserved_reverse_closure_target_ids": None,
        "edges": edges,
        "support_root_ids": ["tests::deep"],
    }
    output = run_kernel_json(kernel, fixture)
    closure = output["reference"]["exact_reverse_closure"]
    expected_nodes = {"tests::deep"} | {f"app::node_{index}" for index in range(depth + 1)}

    assert set(closure["target_ids"]) == {"app::node_0"}
    assert set(closure["node_ids"]) == expected_nodes
    assert not any(node.startswith("noise::") for node in closure["node_ids"])
    assert set(closure["internal_edges"]["tests::deep"]) == {f"app::node_{depth}"}


def run_kernel_duplicate_object_key_boundary_case(kernel: Path) -> None:
    payload = (
        '{"schema_version":1,'
        '"projected_item_ids":["app::target"],'
        '"preserved_reverse_closure_target_ids":["app::target"],'
        '"edges":{'
        '"support::root":["app::target"],'
        '"support::root":["app::other"]'
        '},'
        '"support_root_ids":["support::root"]}'
    )
    output = parse_kernel_output(run_kernel(kernel, payload))
    closure = output["reference"]["exact_reverse_closure"]

    # Duplicate JSON object keys are a protocol/parser boundary, not part of the
    # graph theorem. Lean's JSON object semantics keep the later key here; the
    # release verifier records that behavior explicitly for direct raw JSON use.
    assert closure["node_ids"] == ["app::target"]
    assert closure["internal_edges"] == {}


def run_kernel_protocol_boundary_cases(kernel: Path) -> None:
    run_kernel_protocol_error_cases(kernel)
    run_kernel_unicode_and_duplicate_array_case(kernel)
    run_kernel_large_boundary_fixture(kernel)
    run_kernel_duplicate_object_key_boundary_case(kernel)


def run_rust_equivalence_tests(kernel: Path) -> None:
    env = os.environ.copy()
    env["SPECIAL_TRACEABILITY_KERNEL_EXE"] = str(kernel)
    env["SPECIAL_REQUIRE_LEAN_KERNEL_TESTS"] = "1"
    for test_name in [
        "projected_traceability_lean_kernel_matches_rust_reference_cases",
        "projected_traceability_kernel_default_uses_lean_when_available",
    ]:
        subprocess.run(
            [
                "mise",
                "exec",
                "--",
                "cargo",
                "test",
                test_name,
            ],
            cwd=ROOT,
            env=env,
            check=True,
        )


def main() -> int:
    kernel = build_release_kernel()
    run_kernel_fixture(kernel)
    run_kernel_protocol_boundary_cases(kernel)
    run_rust_equivalence_tests(kernel)
    print("Release Lean traceability kernel verified.")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
