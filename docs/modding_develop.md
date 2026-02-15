# Modding Development Guide

> Status: Active
> Audience: Engine contributors, advanced mod authors
> Last Reviewed: 2026-02-15
> Doc Type: Reference

This document is for mod authors and engine contributors implementing mod support.
It describes the data model, runtime hooks, and expected mod layout.

Recommended docs entrypoint:
- `docs/modding/index.md`

Roadmap for flow improvements:
- `docs/modding/roadmap.md`

## 1) Goals

- Data-first content: most gameplay rules live in DSL/JSON, not Rust.
- Script hooks: mods can inject effects via the same EffectBlock pipeline.
- Deterministic load order and explicit override rules.

## 2) Mod Layout

```
mods/<id>/
  mod.json
  content/            # optional
    jokers.dsl
    tags.dsl
    bosses.dsl
    named_effect_mixins.json
    consumable_mixins.json
    tarots.json
    planets.json
    spectrals.json
  scripts/            # optional
    main.lua
    main.wasm
```

Notes:
- `mod.json` is required.
- Missing content files are treated as empty.
- `entry` is optional for data-only mods.

## 3) Manifest (mod.json)

```
{
  "meta": { "id": "example", "name": "Example Mod", "version": "0.1.0" },
  "entry": "scripts/main.lua",
  "content": { "root": "content" },
  "overrides": ["joker:base_joker_id"],
  "dependencies": [{ "id": "other_mod", "version": ">=0.1.0" }],
  "load_order": 0
}
```

Validation rules:
- `meta.id` must match folder name and be `[a-zA-Z0-9_-]+`.
- `entry` and `content.root` must be safe relative paths.
- If `overrides` are used, the base entry must exist and be from base content.
- Any conflict without a matching override is a hard error.

## 4) Data Content Packs

Data mods use the same formats as core content:
- `jokers.dsl`, `tags.dsl`, `bosses.dsl` (DSL)
- `tarots.json`, `planets.json`, `spectrals.json` (JSON)
- `named_effect_mixins.json` (optional reusable Joker/Tag/Boss DSL effect snippets)
- `consumable_mixins.json` (optional reusable effect blocks)

Merge behavior:
- By default, any duplicate ID is rejected.
- Overrides require `overrides` entries like `joker:blueprint`.

Consumable mixin behavior:
- Consumables can include `mixins: ["id"]`.
- Mixins are loaded from `consumable_mixins.json`.
- Mixin dependencies (`requires`) are expanded first.
- Final effect order is: mixin effects -> consumable effects.

Named mixin behavior:
- Jokers/Tags/Bosses can reference DSL mixins inside block bodies:
  - `mixin base_bonus`
  - `mixins base_bonus, other_bonus`
- Mixins are loaded from `named_effect_mixins.json`.
- `kinds` can limit a mixin to `joker`, `tag`, `boss`.
- Final effect order is: named mixin effects -> block effects.
- Design note: mixins should stay adaptive (conditions/expressions), avoid ID-specific branches.

Hardcoded behavior audit command:
- `./tools/python tools/moddev.py hardcoded --root .`

## 5) Script Runtime (Lua)

Lua runtime registers hooks and returns effects:

```
rulatro.log("loaded")

rulatro.register_hook("OnShopEnter", function(ctx)
  return {
    cancel_core = false,
    effects = {
      {
        block = {
          trigger = "OnShopEnter",
          conditions = { "Always" },
          effects = { { AddMoney = 1 } }
        }
      }
    }
  }
end)
```

Lifecycle phases:
- `rulatro.register_hook(trigger, fn)` => post-core phase
- `rulatro.register_hook(trigger, fn, "pre")` => pre-core phase
- In pre-core phase, return `{ cancel_core = true }` to skip core Boss/Tag/Joker rule hooks for this trigger.

Return value formats:
- `nil` => no action
- `{ stop = true, cancel_core = false, effects = { ... } }`
- `{ block = ..., selected = { ... } }` (single ModEffectBlock)

Notes:
- `stop` stops later hooks in the current hook pipeline.
- Save/load remains host-side command behavior (CLI/CUI), not DSL/mod hook op.

Effect blocks are the same structure as core consumables:
- `trigger` uses `ActivationType` enum names (e.g., `OnPlayed`, `OnUse`).
- `conditions` use `Condition` enum names (e.g., `Always`, `HandKind`).
- `effects` are `EffectOp` entries encoded as `{ OpName = value }`.

If an effect requires card selection, include `selected` indices in the
ModEffectBlock. Indices refer to the current hand.

## 6) Wasm Runtime

Wasm runtime is available (experimental ABI).

Expected guest exports:
- `memory`
- `alloc(len: i32) -> i32`
- `on_hook(ptr: i32, len: i32) -> i64` (packs output pointer/length)
- optional `dealloc(ptr: i32, len: i32)`

Hook payloads are JSON-serialized `ModHookContext`, and return payload is
`ModHookResult` or a single `ModEffectBlock`.

## 7) Engine Integration

Hooks are executed after core rules (Boss/Tag/Joker DSL) at HookPriority::Post.
The runtime receives a read-only context and can return EffectBlocks that are
applied through the same `apply_effect_blocks` pipeline as core content.

Relevant code:
- Mod API: `crates/core/src/modding.rs`
- Hook integration: `crates/core/src/run/hooks.rs`
- Loader + merge: `crates/data/src/load.rs`
- Runtime: `crates/modding/src/runtime/`
