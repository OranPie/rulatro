# Exit Handover (出境文档)

> Status: Active
> Audience: Maintainers and release owners
> Last Reviewed: 2026-02-15
> Doc Type: Checklist

Use this before release handover or when passing ownership to another developer.

## 1) Code Exit Checklist

- [ ] all local changes committed/pushed
- [ ] no dirty workspace
- [ ] key risks documented
- [ ] unresolved TODOs linked to issues

## 2) Knowledge Handover

- [ ] changed behavior documented in `docs/`
- [ ] migration notes for mod/content changes
- [ ] runtime compatibility notes (Lua/Wasm/API)
- [ ] known edge cases and reproduction steps

## 3) Operational Exit

- [ ] release checklist completed (`docs/release_engineering.md`)
- [ ] test strategy checks green (`docs/testing_strategy.md`)
- [ ] docs `Last Reviewed` date updated

## 4) Minimal Handover Template

```md
## Handover Summary
- Scope:
- Major commits:
- Risk areas:
- Follow-up tasks:
```

## 5) Related Docs

- `docs/onboarding_entry.md`
- `docs/release_engineering.md`
