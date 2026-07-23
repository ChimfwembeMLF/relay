# Research: 005 PawaPay Wallets, Country Catalog & Forms

## Decision: Static shared catalog (Rust + TS), not live PawaPay availability API in v1

**Rationale**: User provided a fixed market/MNO list; live `/v2/availability` needs account-specific config and adds latency/failure modes. Static catalog from [PawaPay Using the API — Correspondents](https://docs.pawapay.io/using_the_api) is enough for dropdowns and validation. Optional later: refresh from availability endpoint.

**Alternatives considered**:
- Call PawaPay availability on every form load — deferred
- DB-backed catalog table — unnecessary for v1 static list

## Decision: Keep wallet `country` as ISO-2 (`ZM`); map to PawaPay ISO-3 only at gateway edge

**Rationale**: Existing schema, seeds, invoices, and UI use ISO-2. PawaPay request bodies use ISO-3 (`ZMB`) + correspondents (`MTN_MOMO_ZMB`). Catalog stores both `iso2` and `iso3`; gateway layer already (or will) map as needed. Avoids a breaking migration.

**Alternatives considered**:
- Migrate all countries to ISO-3 — high blast radius; rejected for v1

## Decision: Register hard-defaults to Zambia; hide country UI

**Rationale**: User request. Frontend omits field; backend ignores client `enabled_countries` override on public register OR forces `["ZM"]` server-side (prefer **server-side force** so API clients cannot register arbitrary countries without a future admin path).

**Alternatives considered**:
- Trust client-sent `["ZM"]` only — weaker; rejected
- Multi-country picker on register — rejected by user

## Decision: Expand catalog to user-listed markets with PawaPay correspondent codes

| Country | ISO-2 | ISO-3 | Currency | MNOs (label → correspondent) |
|---------|-------|-------|----------|------------------------------|
| Benin | BJ | BEN | XOF | Moov → `MOOV_BEN`; MTN → `MTN_MOMO_BEN` |
| Cameroon | CM | CMR | XAF | MTN → `MTN_MOMO_CMR` (+ Orange `ORANGE_CMR` if enabled later) |
| Côte d'Ivoire | CI | CIV | XOF | MTN → `MTN_MOMO_CIV`; Orange → `ORANGE_CIV` |
| DRC | CD | COD | CDF, USD | Vodacom → `VODACOM_MPESA_COD`; Airtel → `AIRTEL_COD`; Orange → `ORANGE_COD` |
| Gabon | GA | GAB | XAF | Airtel → `AIRTEL_GAB` |
| Kenya | KE | KEN | KES | Safaricom M-Pesa → `MPESA_KEN` |
| Congo (Republic) | CG | COG | XAF | Airtel → `AIRTEL_COG`; MTN → `MTN_MOMO_COG` |
| Rwanda | RW | RWA | RWF | Airtel → `AIRTEL_RWA`; MTN → `MTN_MOMO_RWA` |
| Senegal | SN | SEN | XOF | Free → `FREE_SEN`; Orange → `ORANGE_SEN` |
| Sierra Leone | SL | SLE | SLE | Orange → `ORANGE_SLE` |
| Uganda | UG | UGA | UGX | Airtel → `AIRTEL_OAPI_UGA`; MTN → `MTN_MOMO_UGA` |
| Zambia | ZM | ZMB | ZMW | Airtel → `AIRTEL_OAPI_ZMB`; MTN → `MTN_MOMO_ZMB`; Zamtel → `ZAMTEL_ZMB` |

**Rationale**: Matches user list + official PawaPay correspondent table. Cameroon user list showed MTN only — catalog may still include Orange as available MNO for that market (docs); UI filters by catalog. Prefer full doc MNOs for listed countries.

**Alternatives considered**:
- Only exact user MNO subset without Orange CMR — acceptable tweak at implement if product wants strict subset

## Decision: Dual-currency UI only for DRC

**Rationale**: Only COD has CDF+USD in PawaPay docs for these markets. Other countries: currency read-only from catalog.

**Alternatives considered**:
- Always show editable currency — rejected (error-prone)

## Decision: Friendly provider labels; send correspondent codes to API

**Rationale**: User must not type `MTN_MOMO_ZMB`. Dropdown shows “MTN”; value submitted is correspondent code.

**Alternatives considered**:
- Predict provider from MSISDN via PawaPay — nice-to-have later

## Decision: Overview shows balance + MNO labels (not separate MNO balances)

**Rationale**: Relay wallets are per country/currency, not per MNO. “Cross-check” means see which operators can move that wallet’s funds, not per-MNO ledger balances (gateway doesn’t expose that in Relay today).

**Alternatives considered**:
- Fake per-MNO balances — contradicts ledger model; rejected

## Decision: Reports restyle with DESIGN.md / shadcn only

**Rationale**: Feature 004 established SoT; Reports still sparse — apply same page chrome, cards, tables, pill CTA.

**Alternatives considered**:
- New chart library — out of scope

## Decision: Expand `wallet_seed_defaults.json` for Zambia (keep ZM); drop US from product catalog path

**Rationale**: US/`STRIPE_US` is not PawaPay. Seed defaults should center on ZM/ZMW; other countries can seed at 0 when enabled later via admin/API.

**Alternatives considered**:
- Seed all catalog countries at register — rejected (user: Zambia default only)
