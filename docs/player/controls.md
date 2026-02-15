# Player Controls

> Status: Active
> Audience: Players
> Last Reviewed: 2026-02-15
> Doc Type: Reference

## 1) CUI Keys (Recommended)

- `Tab` / `Shift+Tab`: switch focus pane
- `Up` / `Down` or `k` / `j`: move cursor
- `Space`: select/unselect
- `Enter`: execute primary action
- `d`: deal
- `p`: play selected
- `x`: discard selected
- `s`: enter/leave shop
- `b`: buy selected offer
- `r`: reroll shop
- `u`: use consumable
- `v`: sell joker
- `n`: next blind
- `0-9`: quick number selection
- `Ctrl+S` / `Ctrl+L`: save/load
- `q`: quit

## 2) CLI Core Commands

- `help`: show command list
- `deal`, `play <idx..>`, `discard <idx..>`
- `shop`, `buy card|pack|voucher <idx>`, `reroll`, `leave`
- `pack`, `pick <idx..>`, `skip`
- `use <idx> [sel..]`, `sell <idx>`
- `state`, `summary`, `inv`, `deck`, `levels`, `tags`
- `save [path]`, `load [path]`

## 3) Web Actions

- Use buttons in UI for deal/play/discard/shop loop.
- Card and item selections control indices sent to backend.
- Use browser local save/restore for session continuity.

## 4) Related Docs

- `docs/player/quickstart.md`
- `docs/cui_early_access.md`
