# Testing Strategy

> Status: Active
> Audience: All developers
> Last Reviewed: 2026-02-15
> Doc Type: Runbook

This strategy keeps regression risk low across engine, modding, and frontends.

## 1) Quality Gate

Per-package tests must stay above the threshold (currently `>= 200`):

```bash
./tools/python tools/test_counts.py --min 200 --enforce
```

## 2) Test Layers

- Unit: parser helpers, formatting, pure utility rules.
- Behavior: action/hook pipelines and edge conditions.
- Integration: content load + runtime pipeline + frontend action flow.
- Regression: bugs reproduced by deterministic tests.

## 3) Required Verification

Before pushing:

```bash
cargo fmt
cargo test -q
./tools/python tools/test_counts.py --min 200 --enforce
```

## 4) Coverage Expectations

- `core`: rule correctness + state transitions.
- `data`: schema/merge/override behavior.
- `modding`: loader validation + hook runtime behavior.
- `cli/cui/web`: input mapping, helper behavior, API-level flows.

## 5) Related Docs

- `docs/release_engineering.md`
- `docs/troubleshooting.md`
