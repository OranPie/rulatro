# Player Quick Start

> Status: Active
> Audience: Players
> Last Reviewed: 2026-02-15
> Doc Type: Guide

## 1) Pick a Mode

CLI mode:

```bash
cargo run -p rulatro-cli
```

CUI mode (recommended for daily play):

```bash
cargo run -p rulatro-cui -- --lang zh_CN
```

Web mode:

```bash
cargo run -p rulatro-web -- --lang zh_CN
```

Then open `http://localhost:7878`.

## 2) First 5 Minutes

1. Start blind and deal.
2. Try one play path and one discard path.
3. Enter shop after clearing blind.
4. Buy one low-risk value card first.
5. Continue to next blind and compare score growth.

## 3) Useful Options

- Fixed seed for repeatable practice: `--seed 12345`
- Chinese UI: `--lang zh_CN`
- CLI guided menu: `cargo run -p rulatro-cli -- --menu`

## 4) Related Docs

- `docs/player/controls.md`
- `docs/player/play_tips.md`
