# Hardcoded Behavior Audit Contract

> Status: Active
> Audience: Engine contributors
> Last Reviewed: 2026-03-02
> Doc Type: Runbook

This runbook defines the contract for what must not be encoded in `core`, and how to track temporary exceptions during migration.

## 1) Contract Source of Truth

- Contract file: `tools/hardcoded_behavior_contract.json`
- It defines:
  - forbidden behavior categories in `core`,
  - allowed exception categories (engine scaffolding only),
  - concrete audit checks (`id`, `severity`, `path`, `pattern`, `message`),
  - temporary allowlist entries with mandatory reasons.

## 2) Current Anchors and Allowlist Policy

- `last_consumable_the_fool` is expected to stay **missing** (resolved regression guard).
- `actionop_runtime_match` is expected to stay **missing** (shop-related builtin actions now use registry dispatch).
- `hookpoint_runtime_match` is expected to stay **missing** (hookpoint mapping now uses declarative binding table + validation).
- The following anchors are currently allowlisted debt and must be removed over time:
  - `effectop_runtime_match`
- `pack_kind_keyword_mapping` is expected to stay **missing** (pack kind keyword mapping extracted from hardcoded match logic).
- Rule: add allowlist entries only for migration debt that already has test coverage and a tracked extraction todo.

## 3) How To Check

Run:

```bash
./tools/python tools/moddev.py hardcoded --root .
```

Strict mode fails only on non-allowlisted findings:

```bash
./tools/python tools/moddev.py hardcoded --root . --strict
```

Optional explicit contract path:

```bash
./tools/python tools/moddev.py hardcoded --root . --contract tools/hardcoded_behavior_contract.json
```

## 4) Migration Direction

- Keep `core` as deterministic, generic execution infrastructure.
- Move gameplay identifiers, keyword aliases, and tuning constants into content/config/mod definitions.
- Remove allowlist entries one slice at a time, with parity tests per slice.
