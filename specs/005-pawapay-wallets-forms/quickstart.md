# Quickstart: 005 PawaPay Wallets, Country Catalog & Forms

Validate catalog-driven register defaults, Overview MNOs, form dropdowns, and Reports restyle.

## Prerequisites

- Relay API + Postgres running
- Frontend built or `npm run dev`
- PawaPay sandbox optional (forms validate without live payout)

## Setup

```bash
export SPECIFY_FEATURE=005-pawapay-wallets-forms
cd frontend && npm install && npm run build
# restart payment-relay
```

## Validation scenarios

### 1. Register defaults to Zambia

1. Open `/register` — confirm **no** country/wallet field.
2. Register a system.
3. Sign in → Overview: Zambia wallet present (ZMW); MNOs show Airtel / MTN / Zamtel labels.

**Pass**: SC-001 / SC-002.

### 2. Invoice form catalog UX

1. `/invoices/new` — country dropdown shows flag + name for enabled countries (ZM).
2. Currency auto-fills ZMW (not free-typed).
3. Create invoice successfully.

**Pass**: SC-003.

### 3. Payout form providers

1. `/payments` — select country; provider dropdown shows friendly names only.
2. Inspect request (network): `provider` is correspondent e.g. `MTN_MOMO_ZMB`.

**Pass**: SC-003 / FR-005.

### 4. Validation

1. Attempt submit with empty phone / missing provider — blocked with error text.
2. (If dual-currency DRC enabled later) switching currency keeps providers coherent.

**Pass**: SC-004.

### 5. Reports DESIGN.md

1. Open `/reports`.
2. Confirm title/lede, pill Run CTA, card/table chrome match DESIGN.md (not sparse plain form).

**Pass**: SC-005.

### 6. Build / tests

```bash
cd frontend && npm run build
cargo test --test pay_page_test
# plus any new register/catalog unit tests added in implement
```

**Pass**: SC-006.

## Validation notes (implement)

| Scenario | Result |
|----------|--------|
| 1 Register Zambia / no country field | Pass (server forces `ZM`; seed amount `0`) |
| 2 Invoice CountrySelect + currency | Pass |
| 3 Payout ProviderSelect → correspondent | Pass |
| 4 Client validation empty phone/provider | Pass |
| 5 Reports DESIGN.md cards/table | Pass |
| 6 `npm run build` + `cargo test --test pay_page_test` (+ seed register tests) | Pass |

## Out of scope

- Enabling all catalog countries on every new merchant
- Live PawaPay availability polling
- Per-MNO balance ledgers
