# Player FAQ

> Status: Active
> Audience: Players
> Last Reviewed: 2026-02-15
> Doc Type: Guide

## 1) Why is my action rejected?

Most commands are phase-dependent (blind/play/shop/pack).  
Check current phase first (`state` in CLI, header in CUI/Web).

## 2) Why does score look different than expected?

Check the score breakdown and event steps.  
Card modifiers, joker triggers, and retriggers apply in strict order.

## 3) How can I practice consistently?

Use a fixed seed:

```bash
cargo run -p rulatro-cui -- --seed 12345
```

This lets you compare builds on the same run shape.

## 4) Where is my save file?

CLI/CUI default uses:

- `$RULATRO_SAVE`, or
- `~/.rulatro_cli_state.json`

Web mode uses browser local storage.

## 5) How to get Chinese UI?

Use:

```bash
--lang zh_CN
```

or environment variable:

```bash
RULATRO_LANG=zh_CN
```

## 6) Related Docs

- `docs/player/quickstart.md`
- `docs/player/controls.md`
