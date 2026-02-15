# Gameplay Engineering

> Status: Active
> Audience: Gameplay developers, content tuners
> Last Reviewed: 2026-02-15
> Doc Type: Guide

This guide defines how to improve playability while keeping deterministic behavior.

## 1) Playability Goals

- Clear decision points each blind.
- Multiple viable lines (play, discard, shop routing).
- Low-friction feedback (events, score trace, effect summaries).
- Stable progression pacing across ante levels.

## 2) Tuning Loop

1. Adjust content definitions (`assets/content/*.dsl`, `*.json`).
2. Validate behavior with automated tests.
3. Run manual play sessions in CLI/CUI/Web.
4. Record pain points (too swingy, too slow, unclear UI feedback).
5. Iterate and re-check.

## 3) Playtest Matrix

Run at least:

- seed-stable run (`--seed` fixed),
- random run (default seed),
- zh_CN locale run (text fit + clarity),
- mod-enabled run (custom content interaction).

## 4) UX Signals to Keep

- visible hand value and scoring breakdown,
- explicit boss effects and voucher effects,
- compact but readable consumable effect summaries,
- command/key discoverability (`help`, menu, quick keys).

## 5) Regression Prevention

- Add focused tests for every new rule branch.
- Keep each package test count above quality gate threshold.
- Prefer data migration tests when changing content schema.

## 6) Related Docs

- `docs/player_gameplay_guide.md`
- `docs/rules.md`
- `docs/testing_strategy.md`
