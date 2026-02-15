# Modding Docs Hub

This is the single entry for modding authoring and engine integration.

## Start Here

- New mod authors: `docs/modding/quickstart.md`
- Daily workflow and tooling: `docs/modding/tooling.md`
- Adaptive mixin writing guide: `docs/modding/mixins.md`
- Hardcoded behavior audit list: `docs/modding/hardcoded_behavior.md`
- Runtime/hook and integration details: `docs/modding_develop.md`
- Effect/condition reference: `docs/content_effects.md`

## Recommended Flow

1. Scaffold a mod:
   - `./tools/python tools/moddev.py init my_mod --template lua`
2. Validate the mod:
   - `./tools/python tools/moddev.py validate mods/my_mod`
3. Inspect load order/dependencies:
   - `./tools/python tools/moddev.py inspect mods`
4. Audit hardcoded behavior anchors:
   - `./tools/python tools/moddev.py hardcoded --root .`
5. Run and iterate:
   - `cargo run -p rulatro-cli`

## Scope Notes

- Lua runtime is supported.
- Wasm runtime is scaffolded but currently unavailable.
- Content packs use the same DSL/JSON schema as base assets.
