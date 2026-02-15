# Hardcoded Behavior Audit (List First)

> Status: Active
> Audience: Engine contributors
> Last Reviewed: 2026-02-15
> Doc Type: Runbook

This list tracks behavior still fixed in `core` instead of fully data-defined paths.
Priority is ordered High -> Medium.

## 1) High

1. `the_fool` special-case in last-consumable tracking  
   - `crates/core/src/run/hand.rs`
2. `EffectOp` runtime execution uses explicit match branches  
   - `crates/core/src/run/hand.rs`
3. `ActionOp` runtime execution uses explicit match branches  
   - `crates/core/src/run/joker.rs`

## 2) Medium

1. Hook dispatch mapping is explicit in runtime  
   - `crates/core/src/run/hooks.rs`
2. Pack kind parsing uses hardcoded keywords  
   - `crates/core/src/run/joker.rs`

## 3) How To Check

Run:

```bash
./tools/python tools/moddev.py hardcoded --root .
```

Strict mode (non-zero exit when anchors are found):

```bash
./tools/python tools/moddev.py hardcoded --root . --strict
```

## 4) Migration Direction

- Prefer data package composition (`mixins`, DSL effects, content rules).
- Keep `core` focused on generic execution engine APIs.
- Convert one hardcoded branch family at a time, with tests per slice.
