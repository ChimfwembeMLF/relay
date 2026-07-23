# Quickstart: 004 Modernize Merchant & Pay UI

Validate that the SPA matches **`frontend/DESIGN.md` literally** — including the **sidebar** — with no invented theme.

## Prerequisites

- Node 20+
- Relay serving `frontend/dist` (or Vite `:5173` proxy to `:8080`)
- Merchant username/password (or register)

## Setup

```bash
cd frontend
npm install
npm run build
```

Restart payment-relay so SPA assets refresh. For UI iteration: `npm run dev`.

## Validation scenarios

### 1. Token + SoT smoke

1. Open `/` and `/login`.
2. Primary CTAs = Coinbase Blue `#0052ff` (not ERP `#0D3A69` / mauve).
3. Canvas white; body cool gray; hairlines `#dee1e6`-class light gray.
4. Confirm no decorative gradients/glow that are not DESIGN.md recipes.

**Pass**: Matches [contracts/ui-design-system.md](./contracts/ui-design-system.md).

### 2. Sidebar + shell (required)

1. Sign in; open Overview.
2. Expand/collapse sidebar; confirm BrandMark when collapsed.
3. Active nav uses primary voltage; inactive uses muted/body; dividers are hairlines on white canvas.
4. On ≤390px, open mobile drawer; CTAs remain usable.

**Pass**: Shell is restyled to DESIGN.md tokens; collapse behavior still works.

### 3. Merchant workspace

1. Walk Overview → Invoices → Payouts → Transactions → Webhooks → Reports.
2. Check page title/lede, pill primary CTAs, asset-row tables, empty states.

**Pass**: Same token set as shell; no orphaned ERP chrome.

### 4. Pay page

1. Create invoice; open `/pay/{reference}`.
2. Amount dominant; Pay CTA pill primary; exercise success + not-found.

**Pass**: Usable states; DESIGN.md styling.

### 5. Admin

1. `/admin` login → systems list → detail.

**Pass**: Same tokens as merchant app.

### 6. Build gate

```bash
cd frontend && npm run build
```

**Pass**: Exit 0.

## Implementation validation (004 — fidelity pass)

Executed during `/speckit-implement` (2026-07-23):

| Check | Result |
|-------|--------|
| DESIGN.md tokens + type scale utilities | Pass |
| Sidebar `nav-link` + `top-nav-light` height | Pass |
| Invented radial gradient removed from home | Pass |
| ERP brand hex absent from `frontend/src` | Pass |
| `npm run build` / `pay_page_test` | Run at implement time |
| Manual ≤390 / ≥1280 browser | Operator confirm |

## Out of scope

- Visual regression CI
- Replacing Tekrem logo assets
- Inventing colors/layouts outside DESIGN.md
