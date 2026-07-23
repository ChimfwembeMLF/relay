# Contract: Country / MNO Catalog & Form Mapping

**Feature**: 005-pawapay-wallets-forms  
**Sources**: [PawaPay Correspondents](https://docs.pawapay.io/using_the_api), Relay ISO-2 wallets, `frontend/DESIGN.md` for Reports

## Non-goals

- No change to payment idempotency contracts
- No requirement to expose live PawaPay availability API in v1
- No per-MNO wallet balances

## Catalog contract

Implementers MUST provide a catalog (Rust + TS) where each entry includes:

```json
{
  "iso2": "ZM",
  "iso3": "ZMB",
  "name": "Zambia",
  "flag": "🇿🇲",
  "currencies": ["ZMW"],
  "default_currency": "ZMW",
  "mnos": [
    { "label": "Airtel", "correspondent": "AIRTEL_OAPI_ZMB" },
    { "label": "MTN", "correspondent": "MTN_MOMO_ZMB" },
    { "label": "Zamtel", "correspondent": "ZAMTEL_ZMB" }
  ]
}
```

Minimum countries: BJ, CM, CI, CD, GA, KE, CG, RW, SN, SL, UG, ZM (names as in research.md).

## Register API / UI

| Rule | Behavior |
|------|----------|
| UI | No country/wallet fields |
| Server | Force `enabled_countries = ["ZM"]` on public register; seed ZMW wallet |
| Client body | May omit `enabled_countries`; if present, server overrides to Zambia |

## Form mapping (invoice / payout)

| UI control | User sees | Submitted to API |
|------------|-----------|------------------|
| Country | Flag + name | `country` ISO-2 |
| Currency | Read-only or DRC select | `currency` code |
| Provider | MNO label | `payment_method.details.provider` = correspondent |

Validation MUST reject provider not in country’s MNO list and currency not in country’s currencies.

## Overview

Wallet list MUST include for each wallet: country display name, currency, formatted balance, and MNO labels from catalog for that `iso2`.

## Pay page

`providers` array returned by pay API MUST be built from catalog for invoice country (label + correspondent value).

## Reports UI

MUST use DESIGN.md tokens / shadcn: page `text-title-lg`, pill primary “Run reports”, card/table asset-row patterns — no free-form unstyled dump.

## Compatibility

- Existing systems with other `enabled_countries` remain readable
- Gateway continues to receive PawaPay correspondent strings unchanged in meaning
