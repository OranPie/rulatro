# Modding Overview

This project supports data-first mods (content packs) and script hooks.

For the maintained docs entrypoint, start here:
- `docs/modding/index.md`
- Adaptive mixin guide: `docs/modding/mixins.md`
- Hardcoded behavior audit: `docs/modding/hardcoded_behavior.md`

## Folder Layout

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

Only files you include are loaded. Missing content files are treated as empty.

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

Notes:
- `entry` is optional for data-only mods.
- `content.root` is optional for script-only mods.
- `overrides` must reference existing base content; conflicts are errors by default.

## Consumable Mixins (Data-side composition)

For complex consumable behavior, define reusable effect blocks in:
- `content/consumable_mixins.json`

Then reference them from `tarots.json` / `planets.json` / `spectrals.json` with
`mixins: ["mixin_id"]`.

Composition order:
1. resolved mixin effects (dependencies first)
2. consumable's own `effects`

If a mixin sets `kinds`, only matching consumable kinds can use it.

## Named Effect Mixins (Joker/Tag/Boss)

For complex Joker/Tag/Boss composition, define reusable effect snippets in:
- `content/named_effect_mixins.json`

Example:

```json
[
  {
    "id": "shop_bonus",
    "kinds": ["joker", "tag"],
    "requires": [],
    "effects": ["on shop_enter { add_money 2 }"]
  }
]
```

Reference mixins directly in DSL blocks:

```dsl
joker sample "Sample" Common {
  mixin shop_bonus
  on independent { add_mult 4 }
}
```

Composition order:
1. resolved named mixin effects (dependencies first)
2. block's own `on ...` effects

## Lua Hooks

Lua mods can register hooks and return effects.

```
rulatro.log("mod loaded")

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

Hook phase (lifecycle):
- Default is `post` (after core rules).
- You can register a pre-core hook with:
  - `rulatro.register_hook("OnShopEnter", fn, "pre")`
- In `pre` phase, returning `{ cancel_core = true }` skips core Boss/Tag/Joker rule hooks for that trigger.

Effect blocks follow the same schema as consumables:
- `trigger` is an `ActivationType` string (e.g., `OnPlayed`, `OnUse`).
- `conditions` is a list of `Condition` values.
- `effects` is a list of `EffectOp` objects.

If an effect needs card selection, include `selected` indices in the block:

```
{
  block = { trigger = "OnUse", conditions = { "Always" }, effects = { { DestroySelected = { count = 1 } } } },
  selected = { 0 }
}
```

Card indices refer to the current hand.

## Wasm Hooks

Wasm support is available via runtime ABI (experimental).
Expected exports:
- `memory`
- `alloc(len: i32) -> i32`
- `on_hook(ptr: i32, len: i32) -> i64` (`(ptr << 32) | len`)
- optional `dealloc(ptr: i32, len: i32)`

## Dev Tooling

Use the helper script for scaffold/validate/inspect:

```bash
./tools/python tools/moddev.py init my_mod --template lua
./tools/python tools/moddev.py validate mods/my_mod
./tools/python tools/moddev.py inspect mods
./tools/python tools/moddev.py hardcoded --root .
```
