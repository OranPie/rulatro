# Early Access CUI (Playable Prototype)

This CLI provides a simple command-line interface (CUI) to play through the core Balatro loop:
blinds, hands, scoring, shop, and packs. It is designed for fast iteration and debugging.

## Run

From repo root:

```
cargo run -p rulatro-cli
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
- Some actions require a specific phase; the CLI prints errors when used at the wrong time.
- The output includes events to help validate gameplay flow.
