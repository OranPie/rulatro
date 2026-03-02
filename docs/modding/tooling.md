# Modding Tooling

> Status: Active
> Audience: Mod authors, tooling maintainers
> Last Reviewed: 2026-02-15
> Doc Type: Reference

Development helper script: `tools/moddev.py`

Run via:

```bash
./tools/python tools/moddev.py <command> ...
```

## 1) Commands

## `init`

Create a mod scaffold.

```bash
./tools/python tools/moddev.py init my_mod --template lua
```

Options:
- `--template lua|data|wasm` (default: `lua`)
- `--root <mods-root>` (default: `mods`)
- `--force` (allow using non-empty target dir)

## `validate`

Validate a single mod or all mods under a root.

```bash
./tools/python tools/moddev.py validate mods/my_mod
./tools/python tools/moddev.py validate mods
```

Checks include:
- Manifest required fields and id format
- Safe relative paths (`entry`, `content.root`)
- Entry/content path existence
- Dependency existence within the validated set
- Consumable JSON shape, file-kind consistency, and mixin references
- Named mixin file shape (`named_effect_mixins.json`) and Joker/Tag/Boss DSL `mixin` references

## `inspect`

Print compact load order/dependency summary.

```bash
./tools/python tools/moddev.py inspect mods
```

## `hardcoded`

Audit hardcoded behavior against the contract file:
- `tools/hardcoded_behavior_contract.json`

```bash
./tools/python tools/moddev.py hardcoded --root .
./tools/python tools/moddev.py hardcoded --root . --strict
./tools/python tools/moddev.py hardcoded --root . --contract tools/hardcoded_behavior_contract.json
```

Notes:
- `--strict` fails only on non-allowlisted findings.
- Allowlist reasons live in the contract JSON and are treated as temporary migration debt.

## 2) Suggested Inner Loop

1. `validate`
2. `cargo run -p rulatro-cli`
3. adjust content/scripts
4. repeat
