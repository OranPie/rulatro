# Release Engineering Checklist

> Status: Active
> Audience: Maintainers, release owners
> Last Reviewed: 2026-02-15
> Doc Type: Checklist

Use this checklist before tagging or announcing a release.

## 1) Pre-Release Validation

- [ ] `cargo fmt` clean
- [ ] `cargo test -q` pass
- [ ] `./tools/python tools/test_counts.py --min 200 --enforce` pass
- [ ] docs reviewed date updated where behavior changed

## 2) Modding / Content Validation

- [ ] `./tools/python tools/moddev.py validate mods`
- [ ] `./tools/python tools/moddev.py hardcoded --root .`
- [ ] verify mixin and hook behavior changes are documented

## 3) Frontend Smoke Checks

- [ ] CLI run path works
- [ ] CUI run path works (`--lang zh_CN`)
- [ ] Web server starts and `/api/state` responds

## 4) Release Exit (出境)

- [ ] changelog summary prepared
- [ ] known risks and mitigations listed
- [ ] handover notes updated in `docs/offboarding_exit.md`

## 5) Related Docs

- `docs/offboarding_exit.md`
- `docs/testing_strategy.md`
