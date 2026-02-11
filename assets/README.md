Data files for core flow and scoring.

- `hands.json`: base hand chips/mult, per-level increments, and ranking order.
- `ranks.json`: base chip values for ranks.
- `blinds.json`: blind multipliers and per-blind hand/discard counts.
- `antes.json`: base targets per ante (Small blind). Big/Boss use multipliers.
- `economy.json`: blind rewards and interest rules.
- `shop.json`: shop slot counts, weighted offer pools, and price placeholders.
- `content/`: starter content definitions (jokers/consumables); currently a stub set.
  - `content/jokers.dsl`: Joker definitions in DSL form (templates + per-joker blocks). The DSL
    supports expressions, deck/hand counts, per-joker variables, hand upgrades, rule flags,
    card mutation actions, and deck mutation actions, along with scored_pre / discard batch /
    destroy / sell / use triggers and shop enter / reroll / pack opened / pack skipped / acquire
    / any-sell triggers.
  - `content/bosses.dsl`: Boss blind effect definitions (DSL).
  - `content/tags.dsl`: Tag effect definitions (DSL).

Base targets currently match the legacy (pre-2026) table; confirm if you want
to target a different version.

Economy and shop values are aligned to legacy rules (base rewards, interest,
reroll pricing, pack sizes, base price ranges). Pack weights are scaled by 100
from the wiki tables for integer weighting.

Legendary Joker rates are not modeled in shop weights (they do not appear in
the shop in the base rules).

Content files are minimal placeholders to exercise the execution system. They
should be replaced with full data from the target version.
