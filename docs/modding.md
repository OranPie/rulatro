# Modding Overview

This project supports data-first mods (content packs) and script hooks.

## Folder Layout

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

## Lua Hooks

Lua mods can register hooks and return effects.

```
rulatro.log("mod loaded")

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

Wasm support is scaffolded but not implemented yet.
