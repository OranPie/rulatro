# Documentation Standards

> Status: Active
> Audience: All contributors
> Last Reviewed: 2026-02-15
> Doc Type: Standard

This file defines the required format and quality bar for all docs under `docs/`.

## 1) Required Header

Every document should start with:

```md
# <Title>

> Status: Active | Draft | Deprecated
> Audience: <who should read this>
> Last Reviewed: YYYY-MM-DD
> Doc Type: Index | Guide | Reference | Runbook | Checklist | Standard | Roadmap
```

## 2) Required Structure

Use a consistent top-level flow:

1. Purpose / Scope
2. How to use this doc
3. Main content
4. Verification or examples
5. Related docs

## 3) Formatting Rules

- Use numbered top-level sections (`## 1) ...`).
- Keep commands in fenced code blocks.
- Keep file paths in backticks.
- Prefer short bullet lists over dense paragraphs.
- Clearly mark unstable/experimental behavior.

## 4) Coverage Rules

When adding a feature, update at least:

- one user-facing guide (how to use),
- one developer/reference doc (how it works),
- one workflow/checklist doc (how to maintain/test/release).

## 5) Doc Types (Taxonomy)

- **Index**: entry hub + navigation.
- **Guide**: task-oriented, step-by-step.
- **Reference**: enums, APIs, exact behavior.
- **Runbook**: incident/ops procedures.
- **Checklist**: repeatable release/handover steps.
- **Standard**: formatting and policy constraints.
- **Roadmap**: future milestones and acceptance criteria.

## 6) Review Policy

- Review all `Active` docs before release cut.
- Convert stale docs to `Deprecated` with a replacement link.
- Keep `Last Reviewed` date updated when semantics change.
