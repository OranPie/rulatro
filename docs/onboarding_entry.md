# Entry Onboarding (入境文档)

> Status: Active
> Audience: New contributors
> Last Reviewed: 2026-02-15
> Doc Type: Guide

Start here when you join the project or return after a long break.

## 1) Environment Setup

```bash
cargo --version
./tools/python --version
```

Then verify workspace:

```bash
cargo test -q
```

## 2) Read Order

1. `docs/README.md`
2. `docs/architecture.md`
3. your target area doc (rules/modding/frontend)

## 3) First Contribution Flow

1. Pick a small issue.
2. Reproduce behavior (CLI/CUI/Web).
3. Add/adjust tests.
4. Update relevant docs.
5. Run full checks.

## 4) Ready-to-Commit Checklist

- [ ] `cargo fmt`
- [ ] `cargo test -q`
- [ ] `./tools/python tools/test_counts.py --min 200 --enforce`

## 5) Related Docs

- `docs/offboarding_exit.md`
- `docs/testing_strategy.md`
