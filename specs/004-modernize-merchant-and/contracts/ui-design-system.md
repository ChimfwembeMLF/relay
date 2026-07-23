# UI Design System Contract

**Feature**: 004-modernize-merchant-and  
**Source of truth**: [`frontend/DESIGN.md`](../../../frontend/DESIGN.md) — **literal mapping only**  
**Stack**: React SPA + Tailwind CSS variables + shadcn/ui

## Non-goals

- No new HTTP API endpoints / OpenAPI changes
- No auth header changes (`X-Session-Token`, `X-API-Key`, `X-Admin-Token`)
- No invented visual language (gradients, glow, ERP mauve/blue, decorative multi-shadows)
- No removing retractable sidebar **behavior** (structure stays; chrome tokens change)

## Fidelity MUST

1. Colors used for brand/chrome MUST appear in DESIGN.md `colors:`.
2. CTAs MUST use `rounded.pill` + `colors.primary` per `button-primary` / `button-pill-cta`.
3. Cards MUST use `rounded.xl` (~24px), hairline border, minimal/no decorative shadow (`product-ui-card-light`).
4. Inputs MUST match `text-input` (~48px height, `rounded.md` ~12px).
5. Semantic up/down MUST be text-forward (`price-up-cell` / `price-down-cell` spirit), not solid loud chips.
6. **Sidebar + top bar are in contract scope** and MUST use canvas/hairline/ink/body/`nav-link`/primary-active — not residual ERP chrome.

## Token contract

| Name | Hex | Usage |
|------|-----|--------|
| `--primary` | `#0052ff` | Primary CTAs, active nav, brand links |
| `--primary-active` | `#003ecc` | Pressed primary |
| `--primary-disabled` | `#a8b8cc` | Disabled primary fill |
| `--foreground` / ink | `#0a0b0d` | Headings, strong text, nav ink |
| `--muted-foreground` | `#5b616e`–`#7c828a` | Body / muted |
| `--background` | `#ffffff` | Page + sidebar canvas |
| `--muted` / surface-soft | `#f7f7f7` | Soft bands / soft chrome |
| `--accent` / surface-strong | `#eef0f3` | Secondary fills |
| `--border` | `#dee1e6` | Hairlines (sidebar, header, tables) |
| `--destructive` | `#cf202f` | Errors / semantic-down |
| `--success` | `#05b169` | Semantic-up text |

Former ERP voltages (`#0D3A69`, `#E394AF`) MUST NOT be brand primary/secondary.

## Shell contract (sidebar + top bar)

| Element | DESIGN.md grounding | Required look |
|---------|---------------------|---------------|
| Sidebar background | `colors.canvas` | White canvas |
| Sidebar / header dividers | `colors.hairline` | 1px hairline |
| Nav item type | `typography.nav-link` | ~14px / weight 500 |
| Inactive nav | `colors.body` / muted | Cool gray |
| Active nav | primary text + soft primary/surface treatment | Coinbase Blue voltage, scarce |
| Top bar height | `top-nav-light.height` | ~64px |
| Collapsed mark | existing BrandMark | Readable icons; no ERP color dependency |

Behavior preserved: collapse/expand persistence, mobile drawer.

## Component contracts

### Primary button (`button-primary`)

- Height ≈ 44px (56px allowed for `button-pill-cta`)
- Radius: pill (`100px` / `rounded-full`)
- Background: primary; hover/active: primary-active; disabled: primary-disabled (not only opacity if practical)

### Input (`text-input`)

- Height ≈ 48px; radius md (~12px); hairline border; focus ring primary

### Card (`product-ui-card-light`)

- Radius xl (~24px); hairline; canvas background; no heavy shadows

### Table / asset row

- Vertical padding ~16px; hairline separators; soft hover fill

### Badge (`badge-pill`)

- Pill; caption-strong; status via semantic text colors

## Surface inventory checklist

Every surface in [data-model.md](../data-model.md) — **including AppLayout sidebar** — MUST pass a visual token check against this contract.

## Compatibility

- Retractable sidebar behavior remains required
- Pay page states and field names unchanged
- Tekrem logo assets MAY remain; **color system** is DESIGN.md only
