# Game Design Patterns

> Status: Active
> Audience: Gameplay designers, content tuners, engine developers
> Last Reviewed: 2026-02-17
> Doc Type: Guide

This guide captures repeatable design patterns for balancing run variety, player agency, and deterministic simulation.

## 1) Purpose / Scope

- Provide a shared pattern language for new jokers, bosses, consumables, and economy rules.
- Keep content additions aligned with the core loop in `docs/rules.md`.
- Reduce balance drift by tying each pattern to concrete tuning knobs.

## 2) How to Use This Doc

1. Pick the player experience you want to create (stability, spike turns, greedy economy, etc.).
2. Select one primary pattern and one supporting pattern from section 3.
3. Implement the change in content first (`assets/content/*.dsl`, `assets/*.json`), then add runtime logic only if needed.
4. Validate with focused tests and at least one manual seeded run.

## 3) Main Patterns

### A) Decision Pressure + Recovery Window

- **Intent**: every blind should ask for tradeoffs, but players should have a way to recover.
- **Tuning knobs**: hand/discard limits, blind target scaling, shop reward pacing.
- **Typical levers**: `assets/antes.json`, `assets/blinds.json`, `assets/economy.json`.
- **Failure mode**: repeated dead turns where the player has no meaningful action.

### B) Build Commitment with Flexible Pivot Points

- **Intent**: reward committing to a strategy while allowing late pivots.
- **Tuning knobs**: joker pool breadth, consumable frequency, voucher/shop slot mix.
- **Typical levers**: `assets/content/jokers.dsl`, `assets/content/planets.json`, `assets/shop.json`.
- **Failure mode**: early RNG lock-in that makes the rest of the run scripted.

### C) Controlled Randomness

- **Intent**: keep runs surprising without feeling unfair.
- **Tuning knobs**: offer weights, reroll cost growth, pack composition.
- **Typical levers**: `assets/shop.json`, `assets/content/tarots.json`, `assets/content/spectrals.json`.
- **Failure mode**: high variance outcomes that ignore player skill.

### D) Power Curves with Soft Caps

- **Intent**: let strong combos happen while avoiding runaway inflation.
- **Tuning knobs**: interest cap, multiplicative sources, retrigger density.
- **Typical levers**: `assets/economy.json`, `assets/content/jokers.dsl`, activation order in `docs/rules.md`.
- **Failure mode**: one dominant scaling path invalidates most content.

### E) Readable Causality

- **Intent**: players should understand why a score/economy outcome happened.
- **Tuning knobs**: trigger complexity, stacked modifiers per card, UI feedback detail.
- **Typical levers**: event trace surfaces in CLI/CUI/Web and effect wording in content files.
- **Failure mode**: hidden interactions that feel random even when rules are deterministic.

## 4) Verification / Examples

- Add or update data-level behavior coverage in `crates/data/tests/behavior_alignment.rs`.
- Add runtime coverage for new trigger interactions in `crates/core/tests/effects_matrix.rs`.
- Run targeted checks after each pattern-level change:

```bash
cargo test -p rulatro-data --tests
cargo test -p rulatro-core --tests
```

- Manual playtest minimum:
  - one fixed-seed run (regression check),
  - one random-seed run (variance check),
  - one UI pass for readability (CLI, CUI, or Web).

## 5) Related Docs

- `docs/gameplay_engineering.md`
- `docs/rules.md`
- `docs/content_effects.md`
- `docs/testing_strategy.md`
