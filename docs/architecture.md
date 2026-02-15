# System Architecture

> Status: Active
> Audience: Engine/UI/modding developers
> Last Reviewed: 2026-02-15
> Doc Type: Reference

This document describes package boundaries and runtime responsibilities.

## 1) Workspace Packages

- `crates/core`: deterministic gameplay engine, rules, scoring, state transitions.
- `crates/data`: asset loading, content merge, mod content composition.
- `crates/modding`: mod loader + script runtimes (Lua/Wasm).
- `crates/cli`: command-driven text frontend.
- `crates/cui`: panel-style terminal frontend.
- `crates/web`: local HTTP server + web UI bridge.

## 2) Dependency Direction

Recommended direction is:

`core <- data <- (cli | cui | web)` and `core <- modding <- (cli | cui | web)`

Rules:

- `core` must stay IO-free and host-agnostic.
- `data` owns content shape + merge rules.
- `modding` owns runtime adapters and ABI bridges.
- frontends own input/output, save host behavior, and UX concerns.

## 3) Runtime Flow

1. Frontend loads config/content from `data`.
2. Frontend initializes runtime manager from `modding`.
3. Frontend creates `RunState` from `core`.
4. Actions are applied via `core` APIs.
5. Events and snapshots are rendered by frontend.

## 4) Boundary Guardrails

- No frontend logic in `core`.
- No asset path assumptions in `core`.
- No hardcoded gameplay IDs in host frontends.
- Prefer data-defined behavior + mixin composition over Rust branch growth.

## 5) Related Docs

- `docs/rules.md`
- `docs/content_effects.md`
- `docs/modding_develop.md`
