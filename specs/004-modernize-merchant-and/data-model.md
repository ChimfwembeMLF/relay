# Data Model: 004 Modernize Merchant & Pay UI

No persistent domain entities. This feature models **design-system entities** taken **verbatim** from `frontend/DESIGN.md`.

## Fidelity rule

Every token and recipe below MUST resolve to a named entry in DESIGN.md. Implementers MUST NOT introduce colors, radii, or component looks that are not defined there.

## DesignTokenSet

| Token role | DESIGN.md source | Hex / value |
|------------|------------------|-------------|
| primary | `colors.primary` | `#0052ff` |
| primary-active | `colors.primary-active` | `#003ecc` |
| primary-disabled | `colors.primary-disabled` | `#a8b8cc` |
| ink / foreground | `colors.ink` | `#0a0b0d` |
| body / muted-foreground | `colors.body` / `colors.muted` | `#5b616e` / `#7c828a` |
| canvas / background | `colors.canvas` | `#ffffff` |
| surface-soft / muted | `colors.surface-soft` | `#f7f7f7` |
| surface-strong / accent-secondary | `colors.surface-strong` | `#eef0f3` |
| hairline / border | `colors.hairline` | `#dee1e6` |
| semantic-up | `colors.semantic-up` | `#05b169` |
| semantic-down / destructive | `colors.semantic-down` | `#cf202f` |
| surface-dark | `colors.surface-dark` | `#0a0b0d` |
| surface-dark-elevated | `colors.surface-dark-elevated` | `#16181c` |
| radius-md | `rounded.md` | `12px` |
| radius-xl | `rounded.xl` | `24px` |
| radius-pill | `rounded.pill` | `100px` |

**Validation**: No ERP brand voltage (`#0D3A69`, `#E394AF`) as primary/secondary.

## UISurface (full inventory — sidebar included)

| Surface | Path / component | DESIGN.md grounding |
|---------|------------------|---------------------|
| **Shell sidebar + top bar** | `AppLayout.tsx` | `top-nav-light`, `nav-link`, canvas, hairline, primary active |
| Home | `/` | `hero-band-dark` or `hero-band-light`, `button-primary` / outline |
| Auth | `/login`, `/register` | `product-ui-card-light`, `text-input`, `button-primary` |
| Overview | `/dashboard` | titles + `asset-row` tables + empty state |
| Invoices | `/invoices*` | list/form/detail recipes |
| Payouts | `/payments` | form recipes |
| Transactions | `/transactions` | `asset-row` |
| Webhooks | `/webhooks` | form + table |
| Reports | `/reports` | filters + summary cards |
| Admin | `/admin*` | same tokens as merchant |
| Pay | `/pay/:reference` | amount / number-display, `button-primary` or `button-pill-cta`, status cards, `legal-band` footer |

## ComponentRecipe

| DESIGN.md recipe | Implementation target |
|------------------|----------------------|
| `top-nav-light` | AppLayout header (+ sidebar chrome token roles) |
| `nav-link` | Sidebar / header nav items |
| `button-primary` (+ active/disabled) | `Button` default |
| `button-secondary-light` | `Button` secondary |
| `button-outline-on-dark` | Outline on dark heroes |
| `button-pill-cta` | Hero / pay primary large CTA |
| `text-input` | `Input` / `Select` |
| `badge-pill` | `Badge` |
| `product-ui-card-light` | `Card` |
| `product-ui-card-dark` | Pay/home status accent (optional) |
| `asset-row` | `TableRow` |
| `hero-band-dark` / `hero-band-light` | Home hero |
| `legal-band` / `footer-light` | Pay footer |
| `price-up-cell` / `price-down-cell` | Semantic amount text |

## Relationships

```text
DESIGN.md (SoT)
  └── DesignTokenSet → CSS/Tailwind
        └── ComponentRecipe → shadcn ui/* + AppLayout
              └── UISurface (all routes including sidebar)
```

## State Transitions

N/A for domain. Pay view states (`loading` / `ready` / `paying` / `success` / `error` / `not_found`) unchanged; presentation only.
