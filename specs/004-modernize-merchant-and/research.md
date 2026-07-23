# Research: 004 Modernize Merchant & Pay UI

## Decision: `frontend/DESIGN.md` is the sole visual SoT — no creative redesign

**Rationale**: User clarified (2026-07-23): redesign includes **everything including the sidebar**, and implementers must **not be creative** — use DESIGN.md colors, typography, radii, spacing, and `components.*` recipes only.

**Alternatives considered**:
- Agent-invented “modern” layout (gradients, decorative cards, new accent colors) — **rejected**
- Partial restyle (pages only, leave sidebar ERP-styled) — **rejected**; shell is in scope
- Keep Tekrem/ERP `#0D3A69` / `#E394AF` — **rejected**

## Decision: Full surface inventory (including sidebar)

**Rationale**: Spec FR-005 + user clarification. In-scope chrome:
- Retractable sidebar (`AppLayout`)
- Sticky top bar / mobile drawer
- Home, login, register, dashboard
- Invoices (list/new/detail), payouts, transactions, webhooks, reports
- Pay page shell + states
- Admin login + systems list/detail

**Alternatives considered**:
- Marketing pages only — rejected
- Pay page only — rejected

## Decision: Map sidebar chrome to DESIGN.md nav/surface recipes (keep structure)

**Rationale**: DESIGN.md documents `top-nav-light` (canvas bg, ink text, `nav-link` type, height 64px), hairlines, primary for active/emphasis, and soft surfaces. Relay keeps the **existing retractable sidebar structure** (merchant app density) but applies those **exact token roles** to sidebar + header — not a new invented shell language, and not marketing top-nav-only.

**Alternatives considered**:
- Replace sidebar with Coinbase marketing top-nav only — rejected (breaks merchant density / FR-005 behavior)
- Leave sidebar on old theme — rejected (user: includes sidebar)

## Decision: Remap Tailwind/CSS variables; update shadcn recipes to DESIGN.md

**Rationale**: Existing stack. Tokens and classNames change to match DESIGN.md; no UI framework swap.

**Alternatives considered**:
- New design system package — rejected (complexity vs Internal-First)

## Decision: Font substitutes only where Coinbase fonts unavailable

**Rationale**: Licensed Coinbase Display/Sans/Mono not bundled. Documented substitutes:
- Display → Inter @ 400 (letter-spacing/size from DESIGN.md scale)
- Sans/UI → Inter
- Mono / number-display → IBM Plex Mono

Do not invent a third type personality.

**Alternatives considered**:
- Unlicensed font files — rejected
- Keep Figtree/Gilmer ERP pairing — rejected

## Decision: Use DESIGN.md dark recipes only where named

**Rationale**: `hero-band-dark`, `product-ui-card-dark`, `cta-band-dark` are defined accents. Apply on home/pay when a dark band is needed; do not invent global dark mode or random dark panels.

**Alternatives considered**:
- Full product dark mode — deferred / not requested

## Decision: Semantic up/down as text color only

**Rationale**: DESIGN.md: semantic-up/down are text colors, never loud background fills. Status badges stay text-forward (soft tint at most).

**Alternatives considered**:
- Solid green/red chip fills — rejected by DESIGN.md

## Decision: No backend / OpenAPI / auth changes

**Rationale**: FR-009.

**Alternatives considered**: None.

## Decision: Validate with DESIGN.md checklist + build

**Rationale**: Visual fidelity against DESIGN.md tokens; `npm run build`; pay_page SPA shell tests.

**Alternatives considered**:
- Visual regression CI — optional later
