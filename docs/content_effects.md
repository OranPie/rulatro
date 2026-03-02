# Content Effects Reference

> Status: Active
> Audience: Engine developers, content authors, mod authors
> Last Reviewed: 2026-03-02
> Doc Type: Reference

This document summarizes enhancement data/effects, Joker DSL conditions, and
consumable JSON conditions/effects. It reflects the current engine behavior.

## 1) Enhancements, Seals, Editions (Core Effects)

Applied by the scoring pipeline and round-end hooks (see `rulatro-core`).

Enhancements:
- Bonus: +30 chips when scored.
- Mult: +4 mult when scored.
- Glass: x2 mult when scored; 1-in-4 chance to break (destroy card).
- Stone: +50 chips when scored; does not contribute rank/suit.
- Lucky: 1-in-5 chance +20 mult; 1-in-15 chance +$20.
- Steel: x1.5 mult when held in hand (held scoring).
- Gold: +$3 at end of round if held in hand.
- Wild: counts as any suit in hand evaluation.

Seals:
- Red: retriggers scored/held once (extra pass).
- Gold: +$3 when scored.
- Blue: create Planet for the final hand if held at end of round (requires space).
- Purple: create Tarot when discarded (requires space).

Editions:
- Foil: +50 chips when scored.
- Holographic: +10 mult when scored.
- Polychrome: x1.5 mult when scored.
- Negative: +1 Joker slot (handled on acquisition).

Notes:
- Debuffed cards ignore enhancements/editions/seals and do not trigger per-card hooks.
- Face-down cards are a visibility flag only.

## 2) Joker DSL (Conditions + Actions)

Jokers, Tags, and Bosses share the same DSL and trigger pipeline.

### Basic Syntax

```
joker some_id "Some Name" common {
  on scored when card.is_face { add_chips 10 }
  on independent when contains(hand, Straight) { mul_mult 2 }
}
```

Each effect line is:

```
on <trigger> [when <expr>] { <action> [; <action> ...] }
```

If `when` is omitted, it defaults to `true`.

### Triggers (ActivationType)

Supported `on` triggers (case-insensitive aliases are accepted):

- played, scored_pre, scored, held, independent
- discard, discard_batch
- card_destroyed, card_added
- round_end, hand_end
- blind_start, blind_failed
- shop_enter, shop_reroll, shop_exit
- pack_opened, pack_skipped
- use, sell, any_sell, acquire
- passive, other_jokers

### Conditions (`when` expression)

`when` is a boolean expression (`Expr`) evaluated in the current hook context.
Operators: `!`, `-`, `||`, `&&`, `==`, `!=`, `<`, `<=`, `>`, `>=`, `+`, `-`, `*`, `/`.

Common identifiers (see `eval.rs` for full list):
- Run/hand: `hand`, `hand_id`, `hand_level`, `most_played_hand`, `hand_size`
- Blind: `blind`, `ante`, `blind_score`, `target`, `is_boss_blind`, `boss_disabled`
- Counts: `played_count`, `scoring_count`, `held_count`, `deck_count`
- Limits: `hands_left`, `hands_max`, `discards_left`, `discards_max`
- Economy: `money`
- Jokers: `joker_count`, `joker_slots`, `empty_joker_slots`
- Per-hand count: `hand_play_count`
- Cards (when card context exists): `card.rank`, `card.rank_id`, `card.suit`, `card.suit_id`,
  `card.enhancement`, `card.edition`, `card.seal`, `card.lucky_triggers`,
  `card.is_face`, `card.is_odd`, `card.is_even`, `card.is_stone`, `card.is_wild`
- Consumable context: `consumable.kind`, `consumable.id`

Common functions:
- `contains(hand, HandKind)` -> bool
- `count(scope, target)` -> number
  - scope: `played|scoring|held|discarded|deck`
  - target: rank/suit/enhancement/edition/seal, or `face|odd|even|wild|stone|enhanced`
- `count_joker("joker_id"|"Joker Name")` -> number
- `count_rarity(common|uncommon|rare|legendary)` -> number
- `suit_match(suit|id)` -> bool (honors wild/smeared)
- `hand_count(HandKind)` -> number of times hand played
- `var("key")` -> per-joker variable
- `lowest_rank(scope)` -> lowest rank chips in scope
- `roll(sides)` -> bool (1-in-N)
- `rand(min, max)` -> number (inclusive)
- `min(...)`, `max(...)`, `floor(x)`, `ceil(x)`, `pow(a,b)`

### Actions

Actions are parsed from keywords and map to `ActionOp` (see `effects.rs`).
Examples:

```
on independent { add_chips 50; add_mult 4 }
on scored when card.is_face { add_mult 2 }
on shop_enter { add_money 5 }
on independent { set_rule splash 1 }
```

Actions that require a target take the target as the first argument:

```
set_rule splash 1
add_tag coupon
add_pack arcana 1
add_shop_joker rare
```

See `crates/core/src/effects.rs` for the full action list.

## 3) Consumables (JSON Conditions + Effects)

Consumables are data-defined in JSON (`tarots.json`, `planets.json`, `spectrals.json`).

### Schema

```
{
  "id": "the_fool",
  "name": "The Fool",
  "kind": "Tarot",
  "hand": null,              // Optional, used by planets
  "effects": [
    {
      "trigger": "OnUse",
      "conditions": ["Always"],
      "effects": [
        { "CreateLastConsumable": { "exclude": "the_fool" } }
      ]
    }
  ]
}
```

### Conditions (Condition enum)

- Always
- HandKind(HandKind)
- BlindKind(BlindKind)
- CardSuit(Suit)
- CardRank(Rank)
- CardIsFace
- CardIsOdd
- CardIsEven
- CardHasEnhancement(Enhancement)
- CardHasEdition(Edition)
- CardHasSeal(Seal)
- CardIsStone
- CardIsWild
- IsBossBlind
- IsScoringCard
- IsHeldCard
- IsPlayedCard

### Effects (EffectOp enum)

Key effect ops used by consumables:
- Score(RuleEffect)              // score modifications (chips/mult)
- AddMoney / SetMoney / DoubleMoney / AddMoneyFromJokers
- AddHandSize
- UpgradeHand / UpgradeAllHands
- AddRandomConsumable(kind, count)
- AddJoker(rarity, count) / AddRandomJoker(count)
- RandomJokerEdition / SetRandomJokerEdition / SetRandomJokerEditionDestroyOthers
- DuplicateRandomJokerDestroyOthers
- EnhanceSelected / AddEditionToSelected / AddSealToSelected
- ConvertSelectedSuit / IncreaseSelectedRank
- DestroySelected / DestroyRandomInHand / CopySelected
- ConvertLeftIntoRight / ConvertHandToRandomRank / ConvertHandToRandomSuit
- AddRandomEnhancedCards
- CreateLastConsumable
- RetriggerScored / RetriggerHeld

### Use + Selection Rules

- `use` removes the consumable first, then applies `OnUse` blocks.
- Effects that require card selection enforce a count and reject empty selections.
- Selection indices refer to the current hand.
- If an effect does not require selection, it runs without a selection list.

Selection-required effects:
- EnhanceSelected / AddEditionToSelected / AddSealToSelected
- ConvertSelectedSuit / IncreaseSelectedRank
- DestroySelected / CopySelected
- ConvertLeftIntoRight (requires exactly 2 cards)

See `crates/core/src/effects.rs` and `crates/core/src/run/hand.rs` for full details.

## 4) Vouchers (JSON)

Vouchers are defined in `assets/vouchers.json` and loaded into `Content.vouchers` by
`rulatro-data`. Use `Content::voucher_by_id(id)` at runtime; do not access any static array.

### Schema

```json
{
  "id": "overstock_norm",
  "name_en": "Overstock",
  "name_zh": "过量库存",
  "effect_en": "+1 card slot in shop",
  "effect_zh": "商店增加1个卡位",
  "effect": { "AddShopCardSlots": 1 }
}
```

### VoucherEffect variants

- `None`
- `AddShopCardSlots(u8)` — extra card offer slots
- `AddConsumableSlots(u8)` — extra consumable inventory slots
- `AddJokerSlots(u8)` — extra joker inventory slots
- `AddHandsPerRound(u8)` — extra hands per blind
- `AddDiscardsPerRound(u8)` — extra discards per blind
- `AddHandSizeBase(u8)` — larger starting hand size
- `AddTarotWeight(u32)` — increased tarot offer weight
- `AddPlanetWeight(u32)` — increased planet offer weight
- `ReduceRerollBase(i64)` — cheaper base reroll cost
- `SetShopDiscountPercent(u8)` — percentage discount on all shop items

### Extending vouchers

To add a new voucher using an existing `VoucherEffect` variant, only edit `assets/vouchers.json`.
To add a new *kind* of voucher effect, add a variant to `VoucherEffect` in `vouchers.rs` and
handle it in the relevant `run/shop.rs` methods.

## 5) Card Conditional Rules

Two data tables drive per-card flag + predicate checks without hardcoded Rust branches.
Both are loaded into `Content` and evaluated via `CardConditionalRule::condition.matches()`.

### `assets/card_debuff_rules.json`

Controls which cards are debuffed when a boss rule flag is set.
Loaded into `Content.debuff_rules`. Evaluated in `is_card_debuffed()`.

```json
[
  { "key": "debuff_face",          "condition": "is_face" },
  { "key": "debuff_suit_spades",   "condition": { "is_suit": "Spades" } },
  { "key": "debuff_suit_hearts",   "condition": { "is_suit": "Hearts" } },
  { "key": "debuff_suit_clubs",    "condition": { "is_suit": "Clubs" } },
  { "key": "debuff_suit_diamonds", "condition": { "is_suit": "Diamonds" } },
  { "key": "debuff_played_ante",   "condition": "played_this_ante" }
]
```

### `assets/card_draw_facedown_rules.json`

Controls which cards are drawn face-down via a rule flag + card predicate.
Loaded into `Content.draw_facedown_rules`. Evaluated in `should_draw_face_down()`.

```json
[
  { "key": "draw_face_down_face", "condition": "is_face" }
]
```

### `CardCondition` variants

- `"is_face"` — card rank is Jack, Queen, or King
- `{ "is_suit": "<Suit>" }` — card suit equals Spades/Hearts/Clubs/Diamonds
- `"played_this_ante"` — card ID appears in `state.played_card_ids_ante`

To add a new debuff type (e.g. debuff Aces), add an entry to `card_debuff_rules.json` and a
matching `set_rule debuff_ace 1` in the relevant boss DSL block. No Rust change is needed
unless the condition type itself is new — in which case add a `CardCondition` variant.

