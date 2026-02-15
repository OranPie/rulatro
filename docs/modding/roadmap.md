# Modding System Improvement Roadmap

> Status: Active
> Audience: Engine/modding maintainers
> Last Reviewed: 2026-02-15
> Doc Type: Roadmap

This roadmap focuses on better author/developer flow.

## 1) Phase 1 (Now) - Workflow Baseline

- Single docs entrypoint (`docs/modding/index.md`)
- Quickstart + tooling docs
- `tools/moddev.py` with:
  - `init` (scaffold)
  - `validate` (manifest/content/dependency checks)
  - `inspect` (load-order/dependency summary)

## 2) Phase 2 - CLI Integration

- Add first-class CLI subcommands:
  - `rulatro-cli mod init`
  - `rulatro-cli mod validate`
  - `rulatro-cli mod inspect`
- Keep output format compatible with `tools/moddev.py`.
- Add mixin-aware diagnostics in CLI output (resolved chain + source file).

## 3) Phase 3 - Debug Ergonomics

- Add mod trace mode:
  - hook trigger, mod id, effect output, elapsed time
- Add error format normalization:
  - `mod_id + file + line + field + reason`
- Add optional strict/warn conflict mode for override handling.
- Extend mixin system beyond consumables (jokers/tags/bosses internal definitions). (baseline done: `named_effect_mixins.json`)
- Keep an explicit hardcoded-behavior audit list + checker command. (baseline done: `moddev.py hardcoded`)

## 4) Phase 4 - Packaging/Publishing

- Add `mod pack` command for reproducible releases
- Add template repo + CI checks
- Add compatibility matrix and API stability policy
- Continue extracting behavior logic from core into data-defined execution paths.

## 5) Acceptance Criteria

- New author can scaffold + validate + run first mod in <= 30 minutes
- Typical config/schema issues are caught before runtime
- Mod load order and dependencies are easy to inspect
