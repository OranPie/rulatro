# Adaptive Mixin Writing Guide

Mixins are for shared behavior with local overrides. Keep them adaptive:
- put common logic in mixins
- keep card-specific tuning in the concrete definition
- prefer conditions/expressions over fixed IDs

## Mixin Types

- Consumable mixins: `content/consumable_mixins.json`
- Named effect mixins (Joker/Tag/Boss): `content/named_effect_mixins.json`

## Named Mixins (Joker/Tag/Boss)

Schema:

```json
[
  {
    "id": "shop_bonus",
    "kinds": ["joker", "tag"],
    "requires": [],
    "effects": ["on shop_enter { add_money 2 }"]
  }
]
```

Reference in DSL:

```dsl
joker sample "Sample" Common {
  mixin shop_bonus
  on independent { add_mult 4 }
}
```

Also supported:
- `mixins a, b, c`

## Consumable Mixins

Schema:

```json
[
  {
    "id": "money_small",
    "kinds": ["Tarot", "Spectral"],
    "requires": [],
    "effects": [
      {
        "trigger": "OnUse",
        "conditions": ["Always"],
        "effects": [{ "AddMoney": 3 }]
      }
    ]
  }
]
```

Reference in consumables (`tarots.json`/`planets.json`/`spectrals.json`):

```json
{
  "id": "sample_card",
  "name": "Sample",
  "kind": "Tarot",
  "mixins": ["money_small"],
  "effects": []
}
```

## Composition Order

- Resolved mixin dependencies first (`requires` chain)
- Then the card/block's own effects

Use this to keep base behavior reusable while preserving local specialization.

## Validation

Use the tooling each edit cycle:

```bash
./tools/python tools/moddev.py validate mods/<your_mod>
```

Validation covers:
- unknown mixin references
- kind mismatch
- duplicate mixin IDs
- missing `requires`
- dependency cycles
