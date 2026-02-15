#!/usr/bin/env python3
"""Mod development helper for Rulatro.

Commands:
  init      scaffold a new mod folder
  validate  validate one mod folder or a mods root
  inspect   print a compact summary for mods in load order
  hardcoded audit hardcoded core behavior anchors
"""

from __future__ import annotations

import argparse
import json
import re
import sys
from dataclasses import dataclass, field
from pathlib import Path
from typing import Dict, List, Optional, Sequence, Tuple

ID_RE = re.compile(r"^[A-Za-z0-9_-]+$")
VALID_ENTRY_EXT = {"lua", "wasm"}
CONSUMABLE_KIND_BY_FILE = {
    "tarots.json": "Tarot",
    "planets.json": "Planet",
    "spectrals.json": "Spectral",
}
CONSUMABLE_MIXINS_FILE = "consumable_mixins.json"
NAMED_EFFECT_MIXINS_FILE = "named_effect_mixins.json"
NAMED_DSL_FILES = {
    "jokers.dsl": "joker",
    "tags.dsl": "tag",
    "bosses.dsl": "boss",
}
SEVERITY_ORDER = {"high": 0, "medium": 1, "low": 2}
HARDCODED_CHECKS = [
    {
        "id": "last_consumable_the_fool",
        "severity": "high",
        "path": "crates/core/src/run/hand.rs",
        "pattern": '!def.id.eq_ignore_ascii_case("the_fool")',
        "message": "special-case ID gate: the_fool bypasses last_consumable tracking",
    },
    {
        "id": "effectop_runtime_match",
        "severity": "high",
        "path": "crates/core/src/run/hand.rs",
        "pattern": "match effect {",
        "message": "EffectOp execution is still branch-based in core runtime",
    },
    {
        "id": "actionop_runtime_match",
        "severity": "high",
        "path": "crates/core/src/run/joker.rs",
        "pattern": "match action.op {",
        "message": "ActionOp execution is still branch-based in core runtime",
    },
    {
        "id": "hookpoint_runtime_match",
        "severity": "medium",
        "path": "crates/core/src/run/hooks.rs",
        "pattern": "fn activation_for(point: HookPoint)",
        "message": "hook dispatch still uses explicit core mapping",
    },
    {
        "id": "pack_kind_keyword_mapping",
        "severity": "medium",
        "path": "crates/core/src/run/joker.rs",
        "pattern": "fn parse_pack_target",
        "message": "pack kind parsing uses hardcoded keywords",
    },
]


@dataclass
class Issue:
    level: str
    message: str
    path: Optional[Path] = None


@dataclass
class ModSummary:
    root: Path
    mod_id: str
    name: str
    version: str
    load_order: int
    dependencies: List[str] = field(default_factory=list)
    entry: Optional[str] = None
    content_root: Optional[str] = None


@dataclass
class ValidationResult:
    issues: List[Issue] = field(default_factory=list)
    summary: Optional[ModSummary] = None

    def add(self, level: str, message: str, path: Optional[Path] = None) -> None:
        self.issues.append(Issue(level=level, message=message, path=path))

    @property
    def has_errors(self) -> bool:
        return any(item.level == "error" for item in self.issues)


def is_safe_relative(value: str) -> bool:
    path = Path(value)
    if path.is_absolute():
        return False
    parts = path.parts
    if not parts:
        return False
    return all(part not in ("..", "") for part in parts)


def load_json(path: Path) -> Tuple[Optional[object], Optional[str]]:
    try:
        raw = path.read_text(encoding="utf-8")
    except OSError as exc:
        return None, f"read error: {exc}"
    try:
        return json.loads(raw), None
    except json.JSONDecodeError as exc:
        return None, f"json parse error: {exc}"


def strip_comments(line: str) -> str:
    in_string = False
    escaped = False
    out: List[str] = []
    for idx, ch in enumerate(line):
        if ch == '"' and not escaped:
            in_string = not in_string
            out.append(ch)
            escaped = False
            continue
        if not in_string and ch == "#":
            break
        if not in_string and ch == "/" and idx + 1 < len(line) and line[idx + 1] == "/":
            break
        out.append(ch)
        escaped = (ch == "\\") and not escaped
    return "".join(out)


def brace_delta(line: str) -> int:
    in_string = False
    escaped = False
    delta = 0
    for ch in line:
        if ch == '"' and not escaped:
            in_string = not in_string
            escaped = False
            continue
        if in_string:
            escaped = (ch == "\\") and not escaped
            continue
        if ch == "{":
            delta += 1
        elif ch == "}":
            delta -= 1
        escaped = False
    return delta


def normalize_named_kind(raw: str) -> Optional[str]:
    value = raw.strip().lower()
    if value in {"joker", "jokers"}:
        return "joker"
    if value in {"tag", "tags"}:
        return "tag"
    if value in {"boss", "bosses"}:
        return "boss"
    return None


def parse_mixin_line(line: str) -> Optional[List[str]]:
    trimmed = line.strip()
    if not (trimmed.startswith("mixin ") or trimmed.startswith("mixins ")):
        return None
    _, _, tail = trimmed.partition(" ")
    normalized = tail.replace(",", " ").replace("{", " ").replace("}", " ")
    refs = [token.strip().strip(";") for token in normalized.split() if token.strip().strip(";")]
    return refs


def collect_named_dsl_mixin_refs(path: Path, keyword: str) -> Tuple[Dict[str, List[str]], List[str]]:
    refs_by_id: Dict[str, List[str]] = {}
    errors: List[str] = []
    try:
        raw = path.read_text(encoding="utf-8")
    except OSError as exc:
        return refs_by_id, [f"read error: {exc}"]

    in_block = False
    current_id = ""
    depth = 0
    for lineno, raw_line in enumerate(raw.splitlines(), start=1):
        line = strip_comments(raw_line)
        trimmed = line.strip()
        if not in_block:
            if not trimmed.startswith(f"{keyword} "):
                continue
            header = trimmed.split("{", 1)[0].strip()
            parts = header.split()
            if len(parts) < 2:
                errors.append(f"line {lineno}: {keyword} id is missing")
                continue
            current_id = parts[1].strip()
            refs_by_id.setdefault(current_id, [])
            in_block = True
            depth = max(1, brace_delta(trimmed))
            if "{" in trimmed:
                body_part = trimmed.split("{", 1)[1]
                parsed = parse_mixin_line(body_part)
                if parsed is not None:
                    if parsed:
                        refs_by_id[current_id].extend(parsed)
                    else:
                        errors.append(f"line {lineno}: mixin line missing id")
            if depth <= 0:
                in_block = False
            continue

        parsed = parse_mixin_line(trimmed)
        if parsed is not None:
            if parsed:
                refs_by_id[current_id].extend(parsed)
            else:
                errors.append(f"line {lineno}: mixin line missing id")
        depth += brace_delta(trimmed)
        if depth <= 0:
            in_block = False
            current_id = ""
            depth = 0

    return refs_by_id, errors


def detect_dependency_cycles(graph: Dict[str, List[str]]) -> List[List[str]]:
    visited: set[str] = set()
    stack: List[str] = []
    stack_set: set[str] = set()
    seen_cycles: set[Tuple[str, ...]] = set()
    cycles: List[List[str]] = []

    def dfs(node: str) -> None:
        if node in stack_set:
            idx = stack.index(node)
            cycle = stack[idx:] + [node]
            key = tuple(cycle)
            if key not in seen_cycles:
                seen_cycles.add(key)
                cycles.append(cycle)
            return
        if node in visited:
            return
        visited.add(node)
        stack.append(node)
        stack_set.add(node)
        for dep in graph.get(node, []):
            if dep in graph:
                dfs(dep)
        stack.pop()
        stack_set.remove(node)

    for node in graph:
        dfs(node)
    return cycles


def find_pattern_lines(path: Path, pattern: str) -> Tuple[Optional[List[int]], Optional[str]]:
    try:
        raw = path.read_text(encoding="utf-8")
    except OSError as exc:
        return None, f"read error: {exc}"
    lines: List[int] = []
    for idx, line in enumerate(raw.splitlines(), start=1):
        if pattern in line:
            lines.append(idx)
    return lines, None


def validate_manifest(mod_dir: Path) -> ValidationResult:
    result = ValidationResult()
    manifest_path = mod_dir / "mod.json"
    if not manifest_path.exists():
        result.add("error", "missing mod.json", manifest_path)
        return result

    manifest_data, err = load_json(manifest_path)
    if err:
        result.add("error", err, manifest_path)
        return result
    if not isinstance(manifest_data, dict):
        result.add("error", "manifest root must be a JSON object", manifest_path)
        return result

    meta = manifest_data.get("meta")
    if not isinstance(meta, dict):
        result.add("error", "manifest.meta must be an object", manifest_path)
        return result

    mod_id = str(meta.get("id", "")).strip()
    name = str(meta.get("name", "")).strip()
    version = str(meta.get("version", "")).strip()
    if not mod_id:
        result.add("error", "meta.id is required", manifest_path)
    elif not ID_RE.fullmatch(mod_id):
        result.add("error", "meta.id must match [A-Za-z0-9_-]+", manifest_path)
    if not name:
        result.add("error", "meta.name is required", manifest_path)
    if not version:
        result.add("error", "meta.version is required", manifest_path)

    if mod_id and mod_dir.name != mod_id:
        result.add(
            "error",
            f"directory name '{mod_dir.name}' does not match meta.id '{mod_id}'",
            manifest_path,
        )

    load_order = manifest_data.get("load_order", 0)
    if not isinstance(load_order, int):
        result.add("error", "load_order must be an integer", manifest_path)
        load_order = 0

    dependencies: List[str] = []
    raw_deps = manifest_data.get("dependencies", [])
    if raw_deps is None:
        raw_deps = []
    if not isinstance(raw_deps, list):
        result.add("error", "dependencies must be an array", manifest_path)
    else:
        for idx, dep in enumerate(raw_deps):
            if not isinstance(dep, dict):
                result.add("error", f"dependencies[{idx}] must be an object", manifest_path)
                continue
            dep_id = str(dep.get("id", "")).strip()
            if not dep_id:
                result.add("error", f"dependencies[{idx}].id is required", manifest_path)
                continue
            dependencies.append(dep_id)

    entry = manifest_data.get("entry")
    if entry is not None:
        if not isinstance(entry, str) or not entry.strip():
            result.add("error", "entry must be a non-empty string when provided", manifest_path)
        else:
            entry = entry.strip()
            if not is_safe_relative(entry):
                result.add("error", "entry must be a safe relative path", manifest_path)
            else:
                entry_path = mod_dir / entry
                if not entry_path.exists():
                    result.add("error", f"entry file not found: {entry}", entry_path)
                ext = entry.rsplit(".", 1)[-1].lower() if "." in entry else ""
                if ext not in VALID_ENTRY_EXT:
                    result.add(
                        "warning",
                        f"entry extension '{ext}' is unusual (expected one of {sorted(VALID_ENTRY_EXT)})",
                        manifest_path,
                    )
                if ext == "wasm":
                    result.add(
                        "warning",
                        "wasm runtime is scaffolded but currently unavailable",
                        manifest_path,
                    )

    content_root: Optional[str] = None
    content = manifest_data.get("content")
    if content is not None:
        if not isinstance(content, dict):
            result.add("error", "content must be an object", manifest_path)
        else:
            root = content.get("root")
            if not isinstance(root, str) or not root.strip():
                result.add("error", "content.root must be a non-empty string", manifest_path)
            else:
                content_root = root.strip()
                if not is_safe_relative(content_root):
                    result.add("error", "content.root must be a safe relative path", manifest_path)
                else:
                    content_path = mod_dir / content_root
                    if not content_path.exists() or not content_path.is_dir():
                        result.add("error", f"content root not found: {content_root}", content_path)

    if content_root:
        validate_content_files(mod_dir / content_root, result)

    overrides = manifest_data.get("overrides", [])
    if overrides is None:
        overrides = []
    if not isinstance(overrides, list):
        result.add("error", "overrides must be an array", manifest_path)
    else:
        for idx, value in enumerate(overrides):
            if not isinstance(value, str) or ":" not in value:
                result.add(
                    "warning",
                    f"overrides[{idx}] should use '<kind>:<id>' format",
                    manifest_path,
                )

    result.summary = ModSummary(
        root=mod_dir,
        mod_id=mod_id or mod_dir.name,
        name=name or mod_dir.name,
        version=version or "0.0.0",
        load_order=int(load_order),
        dependencies=dependencies,
        entry=entry if isinstance(entry, str) else None,
        content_root=content_root,
    )
    return result


def validate_content_files(content_dir: Path, result: ValidationResult) -> None:
    mixin_defs: Dict[str, Dict[str, object]] = {}
    mixins_path = content_dir / CONSUMABLE_MIXINS_FILE
    if mixins_path.exists():
        mixins_data, err = load_json(mixins_path)
        if err:
            result.add("error", err, mixins_path)
        elif not isinstance(mixins_data, list):
            result.add("error", "consumable_mixins.json must be a JSON array", mixins_path)
        else:
            for idx, item in enumerate(mixins_data):
                prefix = f"{CONSUMABLE_MIXINS_FILE}[{idx}]"
                if not isinstance(item, dict):
                    result.add("error", f"{prefix} must be an object", mixins_path)
                    continue
                mixin_id = str(item.get("id", "")).strip()
                if not mixin_id:
                    result.add("error", f"{prefix}.id is required", mixins_path)
                    continue
                if mixin_id in mixin_defs:
                    result.add("error", f"duplicate mixin id '{mixin_id}'", mixins_path)
                    continue
                kinds = item.get("kinds", [])
                if kinds is None:
                    kinds = []
                if not isinstance(kinds, list) or not all(isinstance(value, str) for value in kinds):
                    result.add("error", f"{prefix}.kinds must be an array of strings", mixins_path)
                    continue
                requires = item.get("requires", [])
                if requires is None:
                    requires = []
                if not isinstance(requires, list) or not all(
                    isinstance(value, str) and value.strip() for value in requires
                ):
                    result.add("error", f"{prefix}.requires must be an array of non-empty strings", mixins_path)
                    continue
                effects = item.get("effects", [])
                if not isinstance(effects, list):
                    result.add("error", f"{prefix}.effects must be an array", mixins_path)
                    continue
                mixin_defs[mixin_id] = {
                    "kinds": list(kinds),
                    "requires": list(requires),
                }

            for mixin_id, item in mixin_defs.items():
                for dep in item["requires"]:
                    if dep not in mixin_defs:
                        result.add(
                            "error",
                            f"mixin '{mixin_id}' requires unknown mixin '{dep}'",
                            mixins_path,
                        )
            graph = {
                mixin_id: [dep for dep in item["requires"] if dep in mixin_defs]
                for mixin_id, item in mixin_defs.items()
            }
            for cycle in detect_dependency_cycles(graph):
                result.add(
                    "error",
                    f"mixin dependency cycle: {' -> '.join(cycle)}",
                    mixins_path,
                )

    for file_name, expected_kind in CONSUMABLE_KIND_BY_FILE.items():
        file_path = content_dir / file_name
        if not file_path.exists():
            continue
        data, err = load_json(file_path)
        if err:
            result.add("error", err, file_path)
            continue
        if not isinstance(data, list):
            result.add("error", "file must be a JSON array", file_path)
            continue
        seen_ids: set[str] = set()
        for idx, item in enumerate(data):
            prefix = f"{file_name}[{idx}]"
            if not isinstance(item, dict):
                result.add("error", f"{prefix} must be an object", file_path)
                continue
            item_id = str(item.get("id", "")).strip()
            if not item_id:
                result.add("error", f"{prefix}.id is required", file_path)
            elif item_id in seen_ids:
                result.add("error", f"duplicate id '{item_id}' in {file_name}", file_path)
            else:
                seen_ids.add(item_id)
            if not str(item.get("name", "")).strip():
                result.add("error", f"{prefix}.name is required", file_path)
            kind = str(item.get("kind", "")).strip()
            if kind != expected_kind:
                result.add(
                    "error",
                    f"{prefix}.kind must be '{expected_kind}' (got '{kind or '-'}')",
                    file_path,
                )
            effects = item.get("effects")
            if not isinstance(effects, list) or not effects:
                result.add("warning", f"{prefix}.effects is empty", file_path)
            mixins = item.get("mixins", [])
            if mixins is None:
                mixins = []
            if not isinstance(mixins, list) or not all(isinstance(value, str) for value in mixins):
                result.add("error", f"{prefix}.mixins must be an array of strings", file_path)
                continue
            for mixin_id in mixins:
                if mixin_id not in mixin_defs:
                    result.add(
                        "error",
                        f"{prefix} references unknown mixin '{mixin_id}'",
                        file_path,
                    )
                    continue
                allowed = mixin_defs[mixin_id]["kinds"]
                if allowed and expected_kind not in allowed:
                    result.add(
                        "error",
                        f"{prefix} kind '{expected_kind}' is not allowed by mixin '{mixin_id}' ({allowed})",
                        file_path,
                    )

    named_mixin_defs: Dict[str, Dict[str, object]] = {}
    named_mixins_path = content_dir / NAMED_EFFECT_MIXINS_FILE
    if named_mixins_path.exists():
        named_data, err = load_json(named_mixins_path)
        if err:
            result.add("error", err, named_mixins_path)
        elif not isinstance(named_data, list):
            result.add("error", f"{NAMED_EFFECT_MIXINS_FILE} must be a JSON array", named_mixins_path)
        else:
            for idx, item in enumerate(named_data):
                prefix = f"{NAMED_EFFECT_MIXINS_FILE}[{idx}]"
                if not isinstance(item, dict):
                    result.add("error", f"{prefix} must be an object", named_mixins_path)
                    continue
                mixin_id = str(item.get("id", "")).strip()
                if not mixin_id:
                    result.add("error", f"{prefix}.id is required", named_mixins_path)
                    continue
                if mixin_id in named_mixin_defs:
                    result.add("error", f"duplicate named mixin id '{mixin_id}'", named_mixins_path)
                    continue
                raw_kinds = item.get("kinds", [])
                if raw_kinds is None:
                    raw_kinds = []
                if not isinstance(raw_kinds, list) or not all(
                    isinstance(value, str) for value in raw_kinds
                ):
                    result.add("error", f"{prefix}.kinds must be an array of strings", named_mixins_path)
                    continue
                kinds: List[str] = []
                invalid_kind = False
                for raw_kind in raw_kinds:
                    kind = normalize_named_kind(raw_kind)
                    if kind is None:
                        result.add(
                            "error",
                            f"{prefix}.kinds has invalid value '{raw_kind}' (allowed: joker/tag/boss)",
                            named_mixins_path,
                        )
                        invalid_kind = True
                        continue
                    if kind not in kinds:
                        kinds.append(kind)
                if invalid_kind:
                    continue
                requires = item.get("requires", [])
                if requires is None:
                    requires = []
                if not isinstance(requires, list) or not all(
                    isinstance(value, str) and value.strip() for value in requires
                ):
                    result.add(
                        "error",
                        f"{prefix}.requires must be an array of non-empty strings",
                        named_mixins_path,
                    )
                    continue
                effects = item.get("effects", [])
                if not isinstance(effects, list) or not all(
                    isinstance(value, str) and value.strip() for value in effects
                ):
                    result.add(
                        "error",
                        f"{prefix}.effects must be an array of non-empty strings",
                        named_mixins_path,
                    )
                    continue
                named_mixin_defs[mixin_id] = {
                    "kinds": kinds,
                    "requires": list(requires),
                }

            for mixin_id, item in named_mixin_defs.items():
                for dep in item["requires"]:
                    if dep not in named_mixin_defs:
                        result.add(
                            "error",
                            f"named mixin '{mixin_id}' requires unknown mixin '{dep}'",
                            named_mixins_path,
                        )
            graph = {
                mixin_id: [dep for dep in item["requires"] if dep in named_mixin_defs]
                for mixin_id, item in named_mixin_defs.items()
            }
            for cycle in detect_dependency_cycles(graph):
                result.add(
                    "error",
                    f"named mixin dependency cycle: {' -> '.join(cycle)}",
                    named_mixins_path,
                )

    for file_name, expected_kind in NAMED_DSL_FILES.items():
        file_path = content_dir / file_name
        if not file_path.exists():
            continue
        refs_by_id, parse_errors = collect_named_dsl_mixin_refs(file_path, expected_kind)
        for message in parse_errors:
            result.add("error", message, file_path)
        for def_id, refs in refs_by_id.items():
            for mixin_id in refs:
                if mixin_id not in named_mixin_defs:
                    result.add(
                        "error",
                        f"{expected_kind} '{def_id}' references unknown named mixin '{mixin_id}'",
                        file_path,
                    )
                    continue
                allowed = named_mixin_defs[mixin_id]["kinds"]
                if allowed and expected_kind not in allowed:
                    result.add(
                        "error",
                        f"{expected_kind} '{def_id}' is not allowed by named mixin '{mixin_id}' ({allowed})",
                        file_path,
                    )


def collect_mod_dirs(path: Path) -> List[Path]:
    if (path / "mod.json").exists():
        return [path]
    if not path.exists() or not path.is_dir():
        return []
    out: List[Path] = []
    for child in sorted(path.iterdir()):
        if child.is_dir() and (child / "mod.json").exists():
            out.append(child)
    return out


def check_cross_dependencies(
    summaries: Sequence[ModSummary],
    results: Dict[Path, ValidationResult],
) -> None:
    ids = {item.mod_id for item in summaries}
    for summary in summaries:
        for dep in summary.dependencies:
            if dep not in ids:
                results[summary.root].add(
                    "error",
                    f"missing dependency '{dep}'",
                    summary.root / "mod.json",
                )


def print_validation(results: Sequence[ValidationResult]) -> None:
    for item in results:
        if item.summary:
            print(f"[{item.summary.mod_id}] {item.summary.root}")
        for issue in item.issues:
            prefix = issue.level.upper()
            if issue.path:
                print(f"  - {prefix}: {issue.message} ({issue.path})")
            else:
                print(f"  - {prefix}: {issue.message}")
        if not item.issues:
            print("  - OK")


def cmd_validate(args: argparse.Namespace) -> int:
    target = Path(args.path).resolve()
    mod_dirs = collect_mod_dirs(target)
    if not mod_dirs:
        print(f"no mods found at {target}", file=sys.stderr)
        return 2

    results: Dict[Path, ValidationResult] = {}
    summaries: List[ModSummary] = []
    for mod_dir in mod_dirs:
        res = validate_manifest(mod_dir)
        results[mod_dir] = res
        if res.summary:
            summaries.append(res.summary)

    check_cross_dependencies(summaries, results)

    ordered = [results[path] for path in mod_dirs]
    print_validation(ordered)

    errors = sum(1 for res in ordered for issue in res.issues if issue.level == "error")
    warnings = sum(1 for res in ordered for issue in res.issues if issue.level == "warning")
    print(f"\nvalidation complete: {len(mod_dirs)} mod(s), {errors} error(s), {warnings} warning(s)")
    return 1 if errors else 0


def cmd_inspect(args: argparse.Namespace) -> int:
    target = Path(args.path).resolve()
    mod_dirs = collect_mod_dirs(target)
    if not mod_dirs:
        print(f"no mods found at {target}", file=sys.stderr)
        return 2

    summaries: List[ModSummary] = []
    for mod_dir in mod_dirs:
        res = validate_manifest(mod_dir)
        if res.summary is not None:
            summaries.append(res.summary)

    summaries.sort(key=lambda x: (x.load_order, x.mod_id))

    print("load_order  id                 version   deps  entry")
    print("----------  -----------------  -------   ----  ------------------------")
    for item in summaries:
        deps = ",".join(item.dependencies) if item.dependencies else "-"
        entry = item.entry or "-"
        print(f"{item.load_order:>10}  {item.mod_id:<17}  {item.version:<7}   {deps:<4}  {entry}")

    duplicate_ids: Dict[str, int] = {}
    for item in summaries:
        duplicate_ids[item.mod_id] = duplicate_ids.get(item.mod_id, 0) + 1
    duplicates = [mod_id for mod_id, count in duplicate_ids.items() if count > 1]
    if duplicates:
        print("\nwarning: duplicate mod ids detected:", ", ".join(sorted(duplicates)))

    return 0


def cmd_hardcoded(args: argparse.Namespace) -> int:
    root = Path(args.root).resolve()
    checks = sorted(
        HARDCODED_CHECKS,
        key=lambda item: (SEVERITY_ORDER.get(str(item["severity"]), 99), str(item["id"])),
    )
    found = 0
    missing = 0
    errors = 0

    print("hardcoded behavior audit (priority first):")
    for item in checks:
        file_path = root / str(item["path"])
        lines, err = find_pattern_lines(file_path, str(item["pattern"]))
        severity = str(item["severity"]).upper()
        if err:
            print(f"- [{severity}] {item['id']}: {item['message']}")
            print(f"  read error: {err} ({file_path})")
            errors += 1
            continue
        if not lines:
            print(f"- [{severity}] {item['id']}: not found ({file_path})")
            missing += 1
            continue
        line_str = ", ".join(str(value) for value in lines[:3])
        if len(lines) > 3:
            line_str += ", ..."
        print(f"- [{severity}] {item['id']}: {item['message']}")
        print(f"  {file_path}:{line_str}")
        found += 1

    print(
        f"\naudit complete: {found} found, {missing} missing, {errors} read error(s), {len(checks)} rule(s)"
    )
    if args.strict and found > 0:
        return 1
    if errors > 0:
        return 2
    return 0


def write_json(path: Path, value: object) -> None:
    path.write_text(json.dumps(value, indent=2) + "\n", encoding="utf-8")


def scaffold_mod(mod_dir: Path, mod_id: str, template: str) -> None:
    mod_dir.mkdir(parents=True, exist_ok=True)
    content_dir = mod_dir / "content"
    scripts_dir = mod_dir / "scripts"
    content_dir.mkdir(exist_ok=True)

    manifest = {
        "meta": {"id": mod_id, "name": mod_id.replace("_", " ").title(), "version": "0.1.0"},
        "content": {"root": "content"},
        "load_order": 0,
    }

    if template == "lua":
        scripts_dir.mkdir(exist_ok=True)
        manifest["entry"] = "scripts/main.lua"
        (scripts_dir / "main.lua").write_text(
            "mod_meta = {\n"
            f"  id = \"{mod_id}\",\n"
            f"  name = \"{mod_id.replace('_', ' ').title()}\",\n"
            "  version = \"0.1.0\"\n"
            "}\n\n"
            "rulatro.log(\"mod loaded\")\n\n"
            "rulatro.register_hook(\"OnShopEnter\", function(ctx)\n"
            "  return {\n"
            "    effects = {\n"
            "      {\n"
            "        block = {\n"
            "          trigger = \"OnShopEnter\",\n"
            "          conditions = { \"Always\" },\n"
            "          effects = { { AddMoney = 1 } }\n"
            "        }\n"
            "      }\n"
            "    }\n"
            "  }\n"
            "end)\n",
            encoding="utf-8",
        )

    if template == "data":
        sample_tarot = [
            {
                "id": f"{mod_id}_gift",
                "name": "Gift",
                "kind": "Tarot",
                "effects": [
                    {
                        "trigger": "OnUse",
                        "conditions": ["Always"],
                        "effects": [{"AddMoney": 3}],
                    }
                ],
            }
        ]
        write_json(content_dir / "tarots.json", sample_tarot)

    write_json(mod_dir / "mod.json", manifest)


def cmd_init(args: argparse.Namespace) -> int:
    mod_id = args.mod_id.strip()
    if not ID_RE.fullmatch(mod_id):
        print("mod id must match [A-Za-z0-9_-]+", file=sys.stderr)
        return 2

    root = Path(args.root).resolve()
    mod_dir = root / mod_id
    if mod_dir.exists() and any(mod_dir.iterdir()) and not args.force:
        print(f"target exists and is not empty: {mod_dir} (use --force to continue)", file=sys.stderr)
        return 2

    scaffold_mod(mod_dir, mod_id, args.template)
    print(f"created mod scaffold: {mod_dir}")
    print("next steps:")
    print("  1) ./tools/python tools/moddev.py validate " + str(mod_dir))
    print("  2) cargo run -p rulatro-cli")
    return 0


def build_parser() -> argparse.ArgumentParser:
    parser = argparse.ArgumentParser(description="Rulatro mod development helper")
    sub = parser.add_subparsers(dest="command", required=True)

    init_p = sub.add_parser("init", help="create a mod scaffold")
    init_p.add_argument("mod_id", help="mod id / folder name")
    init_p.add_argument("--root", default="mods", help="mods root (default: mods)")
    init_p.add_argument(
        "--template",
        choices=["lua", "data"],
        default="lua",
        help="scaffold template",
    )
    init_p.add_argument("--force", action="store_true", help="overwrite in existing folder")
    init_p.set_defaults(func=cmd_init)

    val_p = sub.add_parser("validate", help="validate mod manifest/content")
    val_p.add_argument("path", nargs="?", default="mods", help="mod dir or mods root")
    val_p.set_defaults(func=cmd_validate)

    inspect_p = sub.add_parser("inspect", help="inspect mods in load order")
    inspect_p.add_argument("path", nargs="?", default="mods", help="mod dir or mods root")
    inspect_p.set_defaults(func=cmd_inspect)

    hardcoded_p = sub.add_parser("hardcoded", help="audit core hardcoded behavior anchors")
    hardcoded_p.add_argument(
        "--root",
        default=".",
        help="repository root used for file lookup (default: current directory)",
    )
    hardcoded_p.add_argument(
        "--strict",
        action="store_true",
        help="return exit code 1 when hardcoded anchors are found",
    )
    hardcoded_p.set_defaults(func=cmd_hardcoded)

    return parser


def main(argv: Optional[Sequence[str]] = None) -> int:
    parser = build_parser()
    args = parser.parse_args(argv)
    return int(args.func(args))


if __name__ == "__main__":
    raise SystemExit(main())
