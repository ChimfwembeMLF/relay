# Data Model: 005 PawaPay Wallets, Country Catalog & Forms

No new persistent tables required for v1. Catalog is reference data; wallets/systems remain existing entities.

## CountryCatalogEntry

| Field | Type | Notes |
|-------|------|-------|
| iso2 | string | Relay storage key (`ZM`) |
| iso3 | string | PawaPay country (`ZMB`) |
| name | string | Display (“Zambia”) |
| flag_emoji or flag_code | string | UI flag (emoji or `flag-icons` code) |
| dialing_prefix | string | e.g. `260` (optional validation aid) |
| currencies | string[] | Usually one; DRC `["CDF","USD"]` |
| default_currency | string | First/default for forms |
| mnos | MnoProvider[] | Operators for this country |

**Validation**: `iso2` unique; every `mnos[].correspondent` unique globally; currencies non-empty.

## MnoProvider

| Field | Type | Notes |
|-------|------|-------|
| id / correspondent | string | PawaPay code e.g. `MTN_MOMO_ZMB` |
| label | string | Human “MTN” / “Safaricom” |
| country_iso2 | string | Parent country |
| currencies | string[] | Subset if MNO differs by currency (DRC) |

## System (existing)

| Field | Change |
|-------|--------|
| enabled_countries | Register forces `["ZM"]`; may expand later via admin/API |

## Wallet (existing)

| Field | Notes |
|-------|-------|
| country | ISO-2 |
| currency | From catalog |
| balance | Minor units; Overview displays major + MNO labels from catalog |

## FormSelection (ephemeral UI)

| Field | Notes |
|-------|-------|
| country_iso2 | From CountrySelect |
| currency | Derived / DRC picker |
| provider_correspondent | Hidden from user label; submitted to API |
| phone | MSISDN |

## ReportSurface (UI only)

Filters (`from`, `to`) + summary cards/tables — no new persisted entity.

## Relationships

```text
CountryCatalogEntry 1──* MnoProvider
System 1──* Wallet (country/currency)
Wallet.country ──> CountryCatalogEntry.iso2
Invoice/Payment.country + provider ──> catalog validation
```

## State Transitions

N/A for catalog. Register: create System → seed Wallet(s) for default Zambia.
