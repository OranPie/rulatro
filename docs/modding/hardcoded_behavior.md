# Hardcoded Behavior Audit (List First)

This list tracks behavior still fixed in `core` instead of fully data-defined paths.
Priority is ordered High -> Medium.

## High

1. `the_fool` special-case in last-consumable tracking  
   - `crates/core/src/run/hand.rs`
2. `EffectOp` runtime execution uses explicit match branches  
   - `crates/core/src/run/hand.rs`
3. `ActionOp` runtime execution uses explicit match branches  
   - `crates/core/src/run/joker.rs`

## Medium

1. Hook dispatch mapping is explicit in runtime  
   - `crates/core/src/run/hooks.rs`
2. Pack kind parsing uses hardcoded keywords  
   - `crates/core/src/run/joker.rs`

## How To Check

Run:

```bash
./tools/python tools/moddev.py hardcoded --root .
```

Strict mode (non-zero exit when anchors are found):

```bash
./tools/python tools/moddev.py hardcoded --root . --strict
```

## Migration Direction

- Prefer data package composition (`mixins`, DSL effects, content rules).
- Keep `core` focused on generic execution engine APIs.
- Convert one hardcoded branch family at a time, with tests per slice.
