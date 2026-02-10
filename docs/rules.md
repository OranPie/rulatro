# Core Flow (v1 skeleton)

This document mirrors the minimal flow implemented in `rulatro-core` and is
intended to be filled out as more mechanics are added.

## Flow

1. **RunInit**
   - Initialize RNG with seed.
   - Build standard 52-card deck and shuffle.
2. **BlindStart**
   - Select ante + blind kind (Small/Big/Boss).
   - Resolve blind rule (hands/discards, multiplier).
   - Resolve base target from `antes.json`, apply blind multiplier.
   - Blind order is Small → Big → Boss → next Ante.
3. **HandLoop**
   - Draw to `hand_size` (default 8).
   - Player chooses play or discard (discard refills to `hand_size`).
   - Hands and discards are limited per blind.
4. **Scoring Pipeline**
   - Evaluate hand kind (pairs/straights/flush).
   - Determine scoring cards.
   - Base chips/mult from `hands.json` (adjusted by the current hand level).
   - Rank chips from `ranks.json`.
   - Total = (base chips + rank chips) × mult.
5. **BlindEnd**
   - If `blind_score >= target`, blind cleared.
   - If `hands_left == 0` and target not met, blind failed.
   - On clear, award base reward (by blind) + per-hand reward + interest (see `economy.json`).
   - Interest is capped by the configured interest cap.
6. **Shop**
   - Generate offers from `shop.json` (cards, boosters, voucher slots).
   - Players can buy offers or reroll cards (reroll cost increases by step).
   - Buying a pack opens options and allows `picks` selections.
   - Arcana/Celestial/Spectral pack choices are used immediately.

## Activation Order (Reference)

Based on the wiki activation sequence, scoring follows this order:
1. Boss blind effects
2. On Played jokers
3. Scored cards, left-to-right:
   - Base card scoring
   - Card modifiers (enhancement → seal → edition)
   - On Scored jokers
   - Retriggers (red seal first, then retriggering jokers left-to-right)
4. Held-in-hand effects, left-to-right:
   - Card modifiers (enhancement → seal → edition)
   - On Held jokers
   - Retriggers (red seal first, then retriggering jokers left-to-right)
5. Joker editions and independent jokers:
   - Foil/Holographic apply before the joker effect
   - Polychrome applies after the joker effect

Activation types (for effect definitions):
- On Played: triggers before scoring.
- On Scored: triggers per scored card.
- On Held: triggers per card in hand after scoring.
- Independent: triggers after scoring completes.
- On Discard: triggers per discarded card.
- On Discard Batch: triggers once per discard action (batch of cards).
- On Card Destroyed: triggers when a card is destroyed during scoring.
- On Round End: triggers after the blind ends.
- On Blind Start: triggers when a blind is selected.
- On Shop Enter: triggers when the shop is entered.
- On Shop Reroll: triggers after a shop reroll.
- On Pack Opened: triggers when a booster pack is opened.
- On Pack Skipped: triggers when a booster pack is skipped.
- On Sell: triggers when a joker is sold (joker-specific).
- On Any Sell: triggers when any joker is sold.
- On Acquire: triggers when a joker is acquired.
- On Use: triggers when a consumable is used (tarot/planet/spectral).

## Card Modifiers (Baseline)

Enhancements applied on scoring unless noted otherwise:
- Bonus: +30 chips when scored.
- Mult: +4 mult when scored.
- Glass: x2 mult when scored; 1 in 4 chance to break.
- Stone: +50 chips when scored; does not contribute rank/suit.
- Lucky: 1 in 5 chance +20 mult; 1 in 15 chance +$20.
- Steel: x1.5 mult when held in hand.
- Gold: +$3 at end of round if held in hand.
- Wild: counts as any suit (hand evaluation TODO).

Seals:
- Red: retriggers scored/held card once.
- Gold: +$3 when scored.
- Blue: create Planet for the final hand if held at end of round (requires space).
- Purple: create Tarot when discarded (requires space).

Editions:
- Foil: +50 chips.
- Holographic: +10 mult.
- Polychrome: x1.5 mult.
- Negative: +1 Joker slot (handled on acquisition).

## Hand Levels (Baseline)

Hands start at level 1 each run. Level ups increase base chips/mult by the
per-hand increments in `hands.json` (`level_chips`, `level_mult`). Jokers and
consumables can trigger level upgrades during play.

## Data Files

See `assets/README.md`. Joker effects are defined in `assets/content/jokers.dsl`.
