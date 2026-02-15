# Early Access CUI (Playable Prototype)

This CLI provides a simple command-line interface (CUI) to play through the core Balatro loop:
blinds, hands, scoring, shop, and packs. It is designed for fast iteration and debugging.

## Run

From repo root:

```
cargo run -p rulatro-cli
```

Improved interactive CUI package (panel layout + keyboard actions):

```
cargo run -p rulatro-cui
```

Launch the CUI through the CLI entrypoint:

```
cargo run -p rulatro-cli -- --cui
```

With locale and seed:

```
cargo run -p rulatro-cui -- --lang zh_CN --seed 12345
```

Menu-driven CUI mode (numbered actions, no command memorization):

```
cargo run -p rulatro-cli -- --menu
```

Run in Simplified Chinese UI/content names:

```
cargo run -p rulatro-cli -- --lang zh_CN
```

For an automated demo (no input):

```
cargo run -p rulatro-cli -- --auto
```

## Core Flow

1. Start in a blind (ante 1, small).
2. `deal` to draw into hand.
3. `play` cards to score and progress the blind.
4. When blind is cleared, `shop` to enter shop.
5. `buy` cards/packs/vouchers or `reroll`.
6. `leave` shop and `next` to start the next blind.

## Commands

- `help` — show command list
- `save [path]` — save run progression to local JSON (default path from `$RULATRO_SAVE` or `~/.rulatro_cli_state.json`)
- `load [path]` — load local JSON and replay actions
- `state` — print run state
- `hand` — show current hand with indices
- `deck` — draw/discard sizes
- `levels` — show current hand levels
- `tags` — list active tags
- `inv` — inventory (jokers, consumables)
- `reward` — estimate clear reward with interest
- `summary` — compact status summary
- `deal` — prepare/draw to hand (phase Deal)
- `play <idx..>` — play cards by index (max 5)
- `discard <idx..>` — discard cards by index (max 5)
- `shop` — enter shop (requires blind cleared)
- `reroll` — reroll shop offers
- `buy card|pack|voucher <idx>` — buy from shop
- `pack` — reprint open pack options
- `pick <idx..>` — choose pack options
- `skip` — skip open pack
- `peek draw|discard [n]` — show top cards (debug)
- `use <idx> [sel..]` — use consumable (optional selected indices)
- `sell <idx>` — sell joker by index
- `leave` — leave shop and return to blind flow
- `next` — advance to next blind
- `quit` — exit

## Notes

- The CUI uses current content/config and the hook system, so any DSL changes are reflected.
- `rulatro-cui` uses a panel interface with focus switching (`Tab`), selection (`Space`), and context action (`Enter`).
- Shortcut highlights: `d` deal, `p` play, `x` discard, `s` shop, `b` buy, `r` reroll, `u` use consumable, `v` sell joker, `n` next blind.
- `rulatro-cui` quick persistence: `Shift+S` / `Ctrl+S` save and `Shift+L` / `Ctrl+L` load open a path prompt (empty input uses `$RULATRO_SAVE` or `~/.rulatro_cli_state.json`).
- Locale can be set via `--lang zh_CN` (or `RULATRO_LANG=zh_CN`).
- `rulatro-cui` now shows richer detail in-pane (card value/detail, expanded shop/inventory rows, score breakdown lines in event log).
- Tarot / Planet / Spectral entries now show compact effect summaries in CUI shop/inventory/pack panels and CLI `shop`/`inv`/`pack` output.
- Shop voucher rows now show voucher name/effect from the wiki-aligned voucher catalog (with partial core effect support).
- Number keys `0-9` can quick-select by row index in the focused pane (hand/pack toggles selection; shop/inventory moves focus).
- `play` now appends detailed effect trace steps (source + effect + before/after chips/mult) to the Events panel.
- In `zh_CN`, CUI pane titles/help/status/event messages are localized for daily play.
- Interactive input supports command history (`↑`/`↓`) and completion (`Tab` for commands, plus `buy`/`peek` subcommands).
- `--menu` enables a guided CUI menu that prints context-aware numbered actions and still accepts direct commands.
- In `zh_CN`, command feedback/events/status tables are localized for daily play (commands/DSL keywords remain English by design).
- Save file schema uses action-log replay (deterministic seed), so it remains robust across minor internal state changes.
- Save file includes run seed; `load` restores the same seed before replaying actions.
- Save file includes content signature (assets/mods fingerprint); `load` verifies signature before replay.
- DSL name localization is supported in `jokers.dsl`, `bosses.dsl`, and `tags.dsl`:
  - `i18n zh_CN "名称"`
  - `name.zh_CN "名称"`
- JSON consumables (`tarots.json`, `planets.json`, `spectrals.json`) support:
  - `"names": { "zh_CN": "名称" }`
- Some actions require a specific phase; the CLI prints errors when used at the wrong time.
- The output includes events to help validate gameplay flow.
