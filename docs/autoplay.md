# Autoplay MCTS

> Status: Active
> Audience: Gameplay developers, tooling users
> Last Reviewed: 2026-02-15
> Doc Type: Guide

`rulatro-autoplay` provides seeded MCTS-based autoplay search with configurable targets.

## 1) Goals

- Deterministic replay for a fixed `seed + config`.
- Configurable target conditions (`score`, `ante`, `money`).
- Full gameplay action space (battle + shop + pack + consumable + sell).
- Output both machine-readable trace (`json`) and quick report (`txt`).

## 2) CLI Usage

```bash
cargo run -p rulatro-cli -- autoplay \
  --seed 12345 \
  --target-ante 4 \
  --target-money 60 \
  --target-score 300000 \
  --time-ms 120 \
  --max-sims 800 \
  --max-steps 500 \
  --trace-json traces/autoplay.json \
  --trace-text traces/autoplay.txt
```

If trace paths are omitted, defaults are generated under `traces/`.

## 3) Main Options

- `--seed <u64>`
- `--target-score <i64>`
- `--target-ante <u8>`
- `--target-money <i64>`
- `--weight-score <f64>`
- `--weight-ante <f64>`
- `--weight-money <f64>`
- `--weight-survival <f64>`
- `--weight-steps <f64>`
- `--time-ms <u64>`
- `--max-sims <u32>`
- `--min-sims <u32>`
- `--max-steps <u32>`
- `--max-play-candidates <usize>`
- `--max-discard-candidates <usize>`
- `--max-shop-candidates <usize>`
- `--exploration-c <f64>`
- `--rollout-depth <u32>`
- `--rollout-top-k <usize>`
- `--action-retries <u32>`
- `--tactical-finish-margin <i64>`
- `--tactical-force-min-sims <u32>`
- `--tactical-max-step-share <f64>`
- `--skip-blind-deficit-penalty <f64>`
- `--endgame-exact-lookahead <bool>`
- `--no-endgame-exact-lookahead`
- `--desperation-discard-boost <f64>`

## 4) Output

JSON trace includes per-step:

- state snapshot before/after action
- chosen action
- MCTS stats (sim count, elapsed, selected visits/value)
- event count and blind outcome

Text output now includes richer per-step context:

- bilingual labels (`English/中文`) for quick cross-language reading
- real selected card faces (`AS`, `TD`, etc.) instead of only action indices
- play scoring details (score delta + score-trace effect chain)
- buy details with concrete shop item names/effects (joker/tarot/planet/voucher/pack)
- blind/ante transition details and target values
- per-ante target table (`small`, `big`, `boss`) at report top

Search now includes invalid-action recovery:

- root actions are validated before expansion
- rollout retries alternate candidates when action application fails
- execution retries with action blacklisting instead of aborting immediately
- near-target tactical finish lookahead to prioritize clearing the current blind
- tactical + MCTS hybrid execution with forced minimum sims in pressure states
- endgame exact lookahead for last-hand play/discard closeout choices

## 5) Crate API

Primary API:

- `run_autoplay(factory, request) -> AutoplayResult`
- `Simulator` with typed `AutoAction`
- `write_json` / `write_text`

`factory` builds a fresh simulator instance used for deterministic replay-based search.
