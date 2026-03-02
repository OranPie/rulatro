# System Architecture

> Status: Active
> Audience: Engine/UI/modding developers
> Last Reviewed: 2026-03-02
> Doc Type: Reference

This document describes package boundaries, runtime responsibilities, and the data/code split model.

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
- Enforce hardcode policy via `tools/hardcoded_behavior_contract.json` + `moddev.py hardcoded`.

## 5) Data/Code Split

The engine uses a three-layer model. Each layer has different change costs:

```
Layer 1 — Content files   (zero Rust change)
  assets/content/jokers.dsl
  assets/content/bosses.dsl  / tags.dsl
  assets/content/tarots.json / planets.json / spectrals.json
  assets/vouchers.json
  assets/card_debuff_rules.json
  assets/card_draw_facedown_rules.json
  assets/*.json (economy, shop, antes, blinds, hands, ranks)

Layer 2 — Enum vocabulary  (Rust change required per new operation type)
  crates/core/src/effects.rs
    ActionOp enum      (51 variants, DSL keyword → enum via from_keyword())
    EffectOp enum      (27 variants, deserialized from JSON)
    ActivationType     (triggers)
    Condition          (consumable conditions)

Layer 3 — Dispatch        (Rust change required, but isolated)
  crates/core/src/run/action_handlers.rs
    dispatch_action()  (51-arm match, each arm delegates to handle_X())
  crates/core/src/run/hand.rs
    apply_pending_effects()   (6-arm match, post-scoring pending ops)
    apply_effect()            (~27-arm match, consumable op dispatch)
    validate_effect_selection() (12-arm match, selection count check)
```

### What is already fully data-driven (Layer 1 only)

| Feature | File | Loaded via |
|---|---|---|
| All joker/boss/tag effects | `assets/content/*.dsl` | joker_dsl.rs |
| Card modifiers (enhancement/edition/seal) | `crates/data/card_modifiers.json` | card_modifier_defs.rs |
| Voucher definitions (all 32) | `assets/vouchers.json` | voucher_defs.rs |
| Boss card debuff rules (6 rules) | `assets/card_debuff_rules.json` | card_conditional_rules.rs |
| Face-down draw rules | `assets/card_draw_facedown_rules.json` | card_conditional_rules.rs |
| Consumables (tarots/planets/spectrals) | `assets/content/*.json` | load.rs |
| Economy, shop, blinds, antes | `assets/*.json` | load.rs |

These can be changed — or extended — without any Rust changes. A mod that adds a new joker
using existing DSL keywords, or a new debuff condition, only touches JSON/DSL.

### What requires Layer 2 + Layer 3 changes

**Adding a new DSL action keyword** (e.g. `shuffle_deck`):
1. Add `ActionOp::ShuffleDeck` to `effects.rs`.
2. Add `"shuffle_deck" => Some(Self::ShuffleDeck)` to `from_keyword()`.
3. Add `ActionOp::ShuffleDeck => handle_shuffle_deck(...)` to `dispatch_action()`.
4. Implement `handle_shuffle_deck()` in `action_handlers.rs`.

**Adding a new consumable effect** (e.g. `CopyJoker`):
1. Add `EffectOp::CopyJoker { ... }` to `effects.rs`.
2. Add match arm in `apply_effect()` in `hand.rs`.
3. If selection is needed, add match arm in `validate_effect_selection()`.
4. Use it in a `tarots.json`/`spectrals.json` entry.

**Adding a new consumable condition** (e.g. `CardIsRank`):
1. Add `Condition::CardIsRank(Rank)` to `effects.rs`.
2. Add evaluation case in `eval.rs`.
3. Use it in consumable JSON.

### The `Custom` escape hatch

Both `ActionOp` and `EffectOp` provide an escape hatch for mods:

- `ActionOpKind::Custom(String)` — a mod-registered Lua/Wasm keyword; dispatched to the mod
  runtime instead of `dispatch_action()`. Mods can implement new joker behavior without Rust.
- `EffectOp::Custom { name, value }` — dispatched to the mod runtime's `apply_custom_effect`
  hook. Mods can implement new consumable effects without Rust.

This means mods can add fully novel behavior at Layer 1 (DSL/JSON) + their Lua/Wasm script,
without touching Layers 2 or 3.

### Phase 4: EffectOp registry (deferred)

The three `match effect { ... }` blocks in `hand.rs` are currently allowlisted in the contract.
The plan is to replace them with a handler registry once the pattern is proven with `ActionOp`.

Current challenge: `apply_effect()` requires a heterogeneous context
(`&mut RunState, selected: &[usize], score: &mut Score, money: &mut i64, ...`). A trait-based
registry would require boxing or function pointer tables with a fixed context struct.
`validate_effect_selection()` would be cleanest as a `requires_selection(&EffectOp) -> Option<usize>`
free function, separate from the dispatch table.

See `tools/hardcoded_behavior_contract.json` for the current allowlist status.

## 6) Related Docs

- `docs/rules.md`
- `docs/content_effects.md`
- `docs/modding_develop.md`
