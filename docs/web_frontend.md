# Web Frontend (API Connected)

> Status: Active
> Audience: Web UI developers, players, QA
> Last Reviewed: 2026-02-15
> Doc Type: Guide

This frontend talks to the Rust game core via a local HTTP API. No mock logic.

## 1) Run

From repo root:

```
cargo run -p rulatro-web
```

Run with Simplified Chinese UI/content names:

```
cargo run -p rulatro-web -- --lang zh_CN
```

Then open:

```
http://localhost:7878
```

## 2) Notes

- The server serves the static UI from `web/` and exposes JSON endpoints:
  - `GET /api/state`
  - `POST /api/action`
- API responses include `locale` so the frontend can localize static/dynamic text.
- The UI buttons send API actions such as `deal`, `play`, `enter_shop`, `buy_card`, `pick_pack`, etc.
- Selecting hand cards, jokers, and consumables in the UI determines indices sent to actions.
- The UI supports browser-local run save/restore (action-log replay):
  - `Save Local` stores current run progression + run seed + UI prefs to `localStorage`.
  - `Load Local` verifies content/mod signature, then resets with saved seed and replays saved actions.
  - `Clear Local Save` removes browser save data.
