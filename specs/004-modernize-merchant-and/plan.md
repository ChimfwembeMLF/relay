# Implementation Plan: Modernize Merchant & Pay UI (Coinbase Design System)

**Branch**: `004-modernize-merchant-and` | **Date**: 2026-07-23 | **Spec**: [spec.md](./spec.md)

**Input**: Feature specification from `/specs/004-modernize-merchant-and/spec.md`  
**User clarification (2026-07-23)**: Redesign **includes everything, including the sidebar**. Do **not** invent a creative visual language — implement **`frontend/DESIGN.md` literally** (colors, type scale, radii, spacing, component recipes).

## Summary

Apply `frontend/DESIGN.md` as the **only** visual source of truth across the entire Relay SPA: home, auth, **retractable sidebar + top bar**, all merchant workspace pages, public pay page, and admin. Map DESIGN.md tokens and `components.*` recipes into Tailwind/CSS variables and shadcn primitives. Preserve routes, auth, and payment behavior. **No decorative invention** (no extra accent palettes, glow, multi-shadow stacks, or layouts that are not grounded in DESIGN.md recipes).

## Technical Context

**Language/Version**: TypeScript 5.7 + React 19 (frontend); Rust/axum unchanged for this feature

**Primary Dependencies**: Vite 6, Tailwind CSS 3, shadcn/ui (new-york), lucide-react, react-router-dom 7

**Storage**: N/A (no schema changes)

**Testing**: `npm run build`; manual DESIGN.md checklist in quickstart; existing `cargo test --test pay_page_test` SPA shell remains green

**Target Platform**: Modern browsers (desktop + mobile); SPA served from `frontend/dist`

**Project Type**: Web application (existing `frontend/` SPA)

**Performance Goals**: No first-load regression; no heavy animation libraries; motion only if DESIGN.md implies it, and respect `prefers-reduced-motion`

**Constraints**:
- SoT = `frontend/DESIGN.md` only (not ERP/Tekrem theme, not agent invention)
- **Full chrome in scope**: sidebar + top bar + every listed page
- Do not change API/auth/payment logic
- Keep retractable sidebar **behavior**; restyle with DESIGN.md surfaces/type/primary active
- Fonts: document open substitutes; preserve DESIGN.md size/weight/letter-spacing

**Scale/Scope**: AppLayout (sidebar + header) + ~14 pages + shared `components/ui/*` primitives

## Constitution Check

*GATE: Must pass before Phase 0 research. Re-check after Phase 1 design.*

| Principle | Status | Notes |
|-----------|--------|-------|
| I. Internal-First Simplicity | PASS | Restyle of **existing** SPA only; no new SaaS product surface |
| II. System Isolation | PASS | No cross-tenant UI/data changes |
| III. Idempotent Payment Processing | PASS | No payment path changes |
| IV. Reliable External Relay | PASS | No gateway/webhook changes |
| V. Observability & Auditability | PASS | N/A for pure presentation |
| Security & Compliance | PASS | No new secrets |
| Spec-driven workflow | PASS | Artifacts under `specs/004-modernize-merchant-and/` |

**Gate result**: PASS — proceed to Phase 0/1.

### Post-design Constitution Check

| Principle | Status | Notes |
|-----------|--------|-------|
| I–V + Security | PASS | Contracts are UI-token only; no API surface |
| Spec-driven | PASS | research / data-model / contracts / quickstart updated for fidelity + full-shell scope |

**Gate result (post Phase 1)**: PASS.

## Project Structure

### Documentation (this feature)

```text
specs/004-modernize-merchant-and/
├── plan.md
├── research.md
├── data-model.md
├── quickstart.md
├── contracts/
│   └── ui-design-system.md
└── tasks.md              # (/speckit-tasks — not created here)
```

### Source Code (repository root)

```text
frontend/
├── DESIGN.md                 # ONLY visual SoT (Coinbase analysis)
├── src/
│   ├── index.css             # CSS vars = DESIGN.md colors/radius
│   ├── components/
│   │   ├── AppLayout.tsx     # Sidebar + top bar — IN SCOPE, DESIGN.md chrome
│   │   ├── BrandLogo.tsx
│   │   ├── EmptyState.tsx
│   │   └── ui/               # button, input, card, table, badge, …
│   └── pages/                # home, auth, workspace, pay, admin — ALL in scope
├── tailwind.config.js
└── index.html                # Font substitutes only
```

**Structure Decision**: Single existing Vite SPA; no new apps/packages.

## Implementation fidelity rules (non-negotiable)

1. **Copy tokens**: Every color used for brand/chrome must exist in DESIGN.md `colors:`.
2. **Copy recipes**: Buttons, inputs, cards, badges, asset rows, nav/top chrome map to named `components.*` entries.
3. **Sidebar included**: Style `AppLayout` sidebar + sticky header with canvas/hairline/ink/body/`nav-link`/`primary` active — not left on the old ERP look.
4. **No creative extras**: Do not add gradients, purple/mauve accents, glow, or layout patterns absent from DESIGN.md.
5. **Dark bands only where DESIGN.md defines them**: `hero-band-dark` / `product-ui-card-dark` / `cta-band-dark` — optional on home/pay only, not invented dark themes elsewhere.

## Complexity Tracking

> No constitution violations requiring justification.
