#!/usr/bin/env python3
"""Count Rust tests per package and optionally enforce minimum coverage."""

from __future__ import annotations

import argparse
import re
import subprocess
import sys
from pathlib import Path


TEST_LINE_RE = re.compile(r": test$")
PACKAGE_RE = re.compile(r'^name\s*=\s*"([^"]+)"\s*$')


def discover_packages(workspace_root: Path) -> list[str]:
    crates_dir = workspace_root / "crates"
    packages: list[str] = []
    for manifest in sorted(crates_dir.glob("*/Cargo.toml")):
        name = parse_package_name(manifest)
        if name:
            packages.append(name)
    return packages


def parse_package_name(manifest: Path) -> str | None:
    in_package = False
    for line in manifest.read_text(encoding="utf-8").splitlines():
        stripped = line.strip()
        if stripped == "[package]":
            in_package = True
            continue
        if stripped.startswith("[") and stripped != "[package]":
            in_package = False
        if not in_package:
            continue
        match = PACKAGE_RE.match(stripped)
        if match:
            return match.group(1)
    return None


def count_tests(pkg: str, workspace_root: Path) -> int:
    cmd = ["cargo", "test", "-p", pkg, "--", "--list"]
    proc = subprocess.run(
        cmd,
        cwd=workspace_root,
        text=True,
        capture_output=True,
        check=False,
    )
    if proc.returncode != 0:
        raise RuntimeError(
            f"failed to list tests for {pkg}\nstdout:\n{proc.stdout}\nstderr:\n{proc.stderr}"
        )
    return sum(1 for line in proc.stdout.splitlines() if TEST_LINE_RE.search(line))


def main() -> int:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--min", type=int, default=200, dest="minimum")
    parser.add_argument("--enforce", action="store_true")
    parser.add_argument("--package", action="append", default=[])
    args = parser.parse_args()

    workspace_root = Path(__file__).resolve().parent.parent
    packages = args.package or discover_packages(workspace_root)

    failures: list[str] = []
    print(f"{'Package':20} {'Tests':>8} {'Min':>8} {'Status':>8}")
    print("-" * 48)
    for pkg in packages:
        count = count_tests(pkg, workspace_root)
        status = "ok" if count >= args.minimum else "low"
        print(f"{pkg:20} {count:8} {args.minimum:8} {status:>8}")
        if status == "low":
            failures.append(pkg)

    if failures:
        print()
        print("Below minimum:", ", ".join(failures))
        if args.enforce:
            return 1
    return 0


if __name__ == "__main__":
    sys.exit(main())
