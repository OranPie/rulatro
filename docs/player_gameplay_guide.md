# Player Gameplay Guide

> Status: Active
> Audience: Players, QA playtesters
> Last Reviewed: 2026-02-15
> Doc Type: Guide

For the newer player docs directory, start here:
- `docs/player/index.md`

Quick guide for actually playing Rulatro (CLI/CUI/Web), with a focus on fun and flow.

## 1) Start Playing

CLI:

```bash
cargo run -p rulatro-cli
```

CUI:

```bash
cargo run -p rulatro-cui -- --lang zh_CN
```

Web:

```bash
cargo run -p rulatro-web -- --lang zh_CN
```

Open `http://localhost:7878`.

## 2) Core Loop

1. Start blind.
2. Deal cards.
3. Play/discard for score.
4. Clear blind and enter shop.
5. Buy cards/packs/vouchers.
6. Next blind.

## 3) New Player Priorities

- Keep enough money for reroll flexibility.
- Use discards to improve hand quality, not only to cycle fast.
- Track boss effects before committing to a line.
- Prefer stable value jokers early, high-roll builds later.

## 4) Reading the UI

- **Score breakdown**: why your score changed.
- **Event log**: effect trigger order and result.
- **Boss/voucher lines**: passive modifiers affecting decisions.
- **Consumable summaries**: immediate and conditional effects.

## 5) Better Playability Settings

- Use fixed seed to compare build choices.
- Use zh_CN for localized text during daily play.
- Use menu mode (`--menu`) if you do not want to memorize commands.

## 6) Related Docs

- `docs/cui_early_access.md`
- `docs/rules.md`
- `docs/gameplay_engineering.md`
