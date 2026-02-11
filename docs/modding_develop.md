# Modding Development Guide

This document is for mod authors and engine contributors implementing mod support.
It describes the data model, runtime hooks, and expected mod layout.

## Goals

- Data-first content: most gameplay rules live in DSL/JSON, not Rust.
- Script hooks: mods can inject effects via the same EffectBlock pipeline.
- Deterministic load order and explicit override rules.

## Mod Layout

```
mods/<id>/
  mod.json
  content/            # optional
    jokers.dsl
    tags.dsl
    bosses.dsl
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

## Manifest (mod.json)

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

## Data Content Packs

Data mods use the same formats as core content:
- `jokers.dsl`, `tags.dsl`, `bosses.dsl` (DSL)
- `tarots.json`, `planets.json`, `spectrals.json` (JSON)

Merge behavior:
- By default, any duplicate ID is rejected.
- Overrides require `overrides` entries like `joker:blueprint`.

## Script Runtime (Lua)

Lua runtime registers hooks and returns effects:

```
rulatro.log("loaded")

rulatro.register_hook("OnShopEnter", function(ctx)
  return {
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

Return value formats:
- `nil` => no action
- `{ stop = true, effects = { ... } }`
- `{ block = ..., selected = { ... } }` (single ModEffectBlock)

Effect blocks are the same structure as core consumables:
- `trigger` uses `ActivationType` enum names (e.g., `OnPlayed`, `OnUse`).
- `conditions` use `Condition` enum names (e.g., `Always`, `HandKind`).
- `effects` are `EffectOp` entries encoded as `{ OpName = value }`.

If an effect requires card selection, include `selected` indices in the
ModEffectBlock. Indices refer to the current hand.

## Wasm Runtime

Wasm is scaffolded but not implemented yet. `.wasm` entries currently fail
with a runtime unavailable error.

## Engine Integration

Hooks are executed after core rules (Boss/Tag/Joker DSL) at HookPriority::Post.
The runtime receives a read-only context and can return EffectBlocks that are
applied through the same `apply_effect_blocks` pipeline as core content.

Relevant code:
- Mod API: `crates/core/src/modding.rs`
- Hook integration: `crates/core/src/run/hooks.rs`
- Loader + merge: `crates/data/src/load.rs`
- Runtime: `crates/modding/src/runtime/`
