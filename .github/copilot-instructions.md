# Copilot Instructions

## Build, Test, and Lint

```bash
# Format
cargo fmt

# Run all tests
cargo test -q

# Run tests for a single crate
cargo test -p rulatro-core --tests
cargo test -p rulatro-data --tests

# Run a single test by name
cargo test -p rulatro-core <test_name>

# Enforce minimum test count (>= 200 per package)
./tools/python tools/test_counts.py --min 200 --enforce

# Run CLI / web frontends
cargo run -p rulatro-cli
cargo run -p rulatro-web              # then open http://localhost:7878
cargo run -p rulatro-web -- --lang zh_CN
```

Use `./tools/python` (not bare `python`) for all Python tooling in `tools/`.

Before committing: `cargo fmt && cargo test -q && ./tools/python tools/test_counts.py --min 200 --enforce`.

## Architecture

Rulatro is a Rust workspace implementing a deterministic card-game engine (Balatro-inspired) with multiple frontends.

**Crates and dependency direction:**
```
core <- data <- cli
              <- cui
              <- web
core <- modding <- cli | cui | web
crates/autoplay (MCTS autoplay, used by cli)
```

- **`rulatro-core`**: deterministic engine — scoring pipeline, state transitions, rules, effects, joker DSL evaluation. **Must stay IO-free and host-agnostic** (no asset paths, no platform code).
- **`rulatro-data`**: asset loading, JSON/DSL content parsing, mod content merge. Owns content schema and override rules.
- **`rulatro-modding`**: mod loader, Lua and Wasm script runtimes, ABI bridges.
- **`rulatro-cli`**: command-line text frontend; entry point for normal play and autoplay.
- **`rulatro-cui`**: panel-style terminal (TUI) frontend.
- **`rulatro-web`**: local HTTP server (`localhost:7878`) + JSON API (`GET /api/state`, `POST /api/action`); serves static UI from `web/`.
- **`rulatro-autoplay`**: MCTS-based seeded autoplay; primary API is `run_autoplay(factory, request)`.

## Content and Data Model

Game rules live in **data files, not Rust**. Add or change gameplay behavior here first:

- `assets/content/jokers.dsl` — joker effect definitions (custom DSL)
- `assets/content/bosses.dsl`, `assets/content/tags.dsl` — boss/tag effects (same DSL)
- `assets/content/tarots.json`, `planets.json`, `spectrals.json` — consumables (JSON)
- `assets/*.json` — economy, shop, blinds, antes, hands, ranks

**Joker DSL syntax:**
```
joker some_id "Some Name" common {
  on scored when card.is_face { add_chips 10 }
  on independent when contains(hand, Straight) { mul_mult 2 }
}
```
Each effect line: `on <trigger> [when <expr>] { <action> [; <action>] }`.
See `docs/content_effects.md` for the full trigger/condition/action reference.

**DSL keywords are backed by core Rust types** — adding a new action keyword, trigger, or condition also requires changes to `crates/core/src/effects.rs`:
- New action keyword → new `ActionOp` variant + entry in `ActionOp::from_keyword()`
- New trigger → new `ActivationType` variant
- New condition → new `Condition` variant

The DSL parser (`crates/data/src/joker_dsl.rs`) calls `ActionOp::from_keyword()` at load time; unknown keywords are a hard error. Pure content changes (tuning existing jokers using already-registered keywords) do not require any Rust changes.

**Named mixins** can be shared across jokers/tags/bosses via `named_effect_mixins.json`:
```
mixin base_bonus
```
Keep mixins condition-based; avoid ID-specific branches in mixins.

## Modding

Mods live in `mods/<id>/` with a required `mod.json`. Data mods use the same DSL/JSON formats as core content. Duplicate IDs are rejected unless listed in `overrides`. Lua hooks:
```lua
rulatro.register_hook("OnShopEnter", function(ctx)
  return { effects = { { block = { trigger = "OnShopEnter", conditions = {"Always"}, effects = { {AddMoney=1} } } } } }
end)
```
Validate mods: `./tools/python tools/moddev.py validate mods`  
Audit hardcoded behavior: `./tools/python tools/moddev.py hardcoded --root .`

## Key Conventions

- **No frontend logic in `core`**; no hardcoded gameplay IDs in frontends.
- **Prefer data-defined behavior** over adding Rust branches. When adding a new joker/boss/tag using *existing* DSL keywords, only edit `assets/content/`. When you need a genuinely new action, trigger, or condition, add the `ActionOp`/`ActivationType`/`Condition` variant in `crates/core/src/effects.rs` first, then use it in the DSL.
- **Scoring activation order** (matters when adding effects): played jokers → scored cards (base → enhancement → seal → edition → on-scored jokers → retriggers) → held-in-hand effects → joker editions → independent jokers. Full order in `docs/rules.md §2`.
- **Determinism**: `core` uses a seeded RNG (`crates/core/src/rng.rs`). Never introduce non-determinism in `core` or `data`.
- **Test coverage target**: keep each package above 200 tests. Add regression tests for any bug fix.
- **Documentation format**: every doc under `docs/` requires a status header (`Status`, `Audience`, `Last Reviewed`, `Doc Type`) and numbered top-level sections (`## 1) ...`). When adding a feature, update one user-facing guide, one reference doc, and one workflow/checklist doc.
- **Save/load** is a host-side frontend concern (CLI/CUI action-log replay), not a core DSL op.

## Useful Entry Points

| Goal | Start here |
|------|-----------|
| Adding/changing a joker | `assets/content/jokers.dsl`, `docs/content_effects.md` |
| Scoring pipeline bug | `crates/core/src/scoring.rs`, `crates/core/src/run/hand.rs` |
| Economy/blind tuning | `assets/economy.json`, `assets/antes.json`, `assets/blinds.json` |
| Mod system | `crates/core/src/modding.rs`, `crates/data/src/load.rs`, `crates/modding/src/runtime/` |
| Web API | `crates/web/src/`, `web/`, `docs/web_frontend.md` |
| Autoplay search | `crates/autoplay/`, `docs/autoplay.md` |
| Behavior regression tests | `crates/data/tests/behavior_alignment.rs`, `crates/core/tests/effects_matrix.rs` |
