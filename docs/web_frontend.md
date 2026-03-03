# Web Frontend (API Connected)

> Status: Active
> Audience: Web UI developers, players, QA
> Last Reviewed: 2026-02-15
> Doc Type: Guide

This frontend talks to the Rust game core via a local HTTP API. No mock logic.
The interactive UI is now a React + Vite app (`web/vite/`) with a legacy static fallback (`web/`).

## 1) Run

Backend (state + API, also serves frontend assets):

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

Frontend build (served by the backend from `web/vite/dist`):

```
cd web/vite
npm install
npm run build
```

Frontend dev server (hot reload, API proxied to `:7878`):

```
cd web/vite
npm install
npm run dev
```

Then open:

```
http://localhost:5173
```

## 2) Notes

- The backend always exposes JSON endpoints:
  - `GET /api/state`
  - `POST /api/action`
- Static serving order:
  1. `web/vite/dist` (if built)
  2. `web/` legacy static assets (fallback)
- API responses include `locale` so the frontend can localize static/dynamic text.
- The UI buttons send API actions such as `deal`, `play`, `enter_shop`, `buy_card`, `pick_pack`, etc.
- Selecting hand cards, jokers, and consumables in the UI determines indices sent to actions.
- The UI supports browser-local run save/restore (action-log replay):
  - `Save Local` stores current run progression + run seed + UI prefs to `localStorage`.
  - `Load Local` verifies content/mod signature, then resets with saved seed and replays saved actions.
  - `Clear Local Save` removes browser save data.
