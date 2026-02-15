# Modding Quickstart

This guide gets you from zero to a runnable mod quickly.

## 1) Create a Mod

Lua hook template:

```bash
./tools/python tools/moddev.py init my_first_mod --template lua
```

Data-only template:

```bash
./tools/python tools/moddev.py init my_data_mod --template data
```

Wasm template:

```bash
./tools/python tools/moddev.py init my_wasm_mod --template wasm
```

## 2) Validate

```bash
./tools/python tools/moddev.py validate mods/my_first_mod
```

If you are validating all local mods:

```bash
./tools/python tools/moddev.py validate mods
```

## 3) Run

```bash
cargo run -p rulatro-cli
```

The CLI loads `mods/` automatically.

## 4) Iterate

- Edit `mod.json`, `content/*.json` / `*.dsl`, and/or `scripts/main.lua`.
- For complex consumables, add reusable blocks in `content/consumable_mixins.json`
  and reference them via `mixins` in consumable entries.
- For complex Joker/Tag/Boss effects, add reusable snippets in
  `content/named_effect_mixins.json` and reference them with `mixin ...` in DSL blocks.
- Re-run validate after each change.
- Optionally audit hardcoded runtime anchors before engine refactors:
  - `./tools/python tools/moddev.py hardcoded --root .`
- Re-run the game and check runtime output.

## Common Mistakes

- `meta.id` does not match folder name.
- `entry` or `content.root` uses unsafe path (`..`, absolute path).
- Consumable JSON `kind` does not match file (`tarots.json => Tarot`, etc.).
- `mixin` references in DSL point to missing/invalid named mixin ids.
- Missing dependency mod id.
