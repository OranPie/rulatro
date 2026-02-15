# Troubleshooting Guide

> Status: Active
> Audience: Developers and advanced users
> Last Reviewed: 2026-02-15
> Doc Type: Runbook

Quick fixes for common failures.

## 1) Build/Test Failures

- Symptom: random compile failure after branch switch  
  Fix: run `cargo clean` then `cargo test -q`.

- Symptom: test-count gate fails  
  Fix: run `./tools/python tools/test_counts.py --min 200` and expand missing package tests.

## 2) Mod Load Issues

- Symptom: mod not loaded  
  Fix: check `mod.json` id/path, run `./tools/python tools/moddev.py validate mods/<id>`.

- Symptom: unknown mixin / cycle error  
  Fix: check `requires` chain and IDs in mixin files.

## 3) Runtime Hook Issues

- Symptom: Lua/Wasm hook has no effect  
  Fix: verify trigger name + phase (`pre`/`post`) and return payload shape.

- Symptom: selection-required effects fail  
  Fix: pass valid hand indices in `selected`.

## 4) Frontend Issues

- CLI/CUI input confusion: run help/menu and confirm phase-dependent commands.
- Web action errors: inspect `/api/action` payload and state phase from `/api/state`.

## 5) Related Docs

- `docs/modding/tooling.md`
- `docs/cui_early_access.md`
