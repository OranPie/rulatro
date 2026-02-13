# Web Frontend (API Connected)

This frontend talks to the Rust game core via a local HTTP API. No mock logic.

## Run

From repo root:

```
cargo run -p rulatro-web
```

Then open:

```
http://localhost:7878
```

## Notes

- The server serves the static UI from `web/` and exposes JSON endpoints:
  - `GET /api/state`
  - `POST /api/action`
- The UI buttons send API actions such as `deal`, `play`, `enter_shop`, `buy_card`, `pick_pack`, etc.
- Selecting hand cards, jokers, and consumables in the UI determines indices sent to actions.
