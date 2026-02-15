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
- `--max-steps <u32>`
- `--max-play-candidates <usize>`
- `--max-discard-candidates <usize>`
- `--max-shop-candidates <usize>`
- `--exploration-c <f64>`
- `--rollout-depth <u32>`

## 4) Output

JSON trace includes per-step:

- state snapshot before/after action
- chosen action
- MCTS stats (sim count, elapsed, selected visits/value)
- event count and blind outcome

Text output summarizes final status and step stream for quick reading.

## 5) Crate API

Primary API:

- `run_autoplay(factory, request) -> AutoplayResult`
- `Simulator` with typed `AutoAction`
- `write_json` / `write_text`

`factory` builds a fresh simulator instance used for deterministic replay-based search.
