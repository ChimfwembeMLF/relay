# Quickstart: 006 Batch Payouts, Refunds & Full-Country Registration

Validate full-catalog registration, batch disbursements, and invoice refunds.

## Prerequisites

- Postgres + Relay running
- Frontend built or `npm run dev`
- Mock or sandbox gateway

```bash
export SPECIFY_FEATURE=006-batch-payouts-refunds
cd frontend && npm run build
# restart payment-relay
```

## 1. Register → all catalog wallets

1. Open `/register` (still no country picker).
2. Create a system.
3. Overview: wallets for all catalog countries/currencies (incl. CD CDF + CD USD).

**Pass**: SC-001 / US1.

## 2. Batch payout

1. Fund a ZM wallet (admin seed / override / prior deposit).
2. Open Payouts → Batch mode; add ≥3 lines (one intentionally invalid).
3. Submit; confirm mixed per-line results; wallet debited only for successes.
4. Retry same idempotency key → identical result, no extra debit.

**Pass**: SC-002 / SC-003 / US2. See [contracts/batch-payouts.md](./contracts/batch-payouts.md).

## 3. Invoice refund

1. Create + pay an invoice (pay page).
2. Invoice detail shows refundable amount and refund action.
3. Partial refund → `refunded_amount` increases; status stays `paid`.
4. Over-refund rejected; replay same idempotency key safe.

**Pass**: SC-004 / SC-005 / US3. See [contracts/invoice-refunds.md](./contracts/invoice-refunds.md).

## 4. Automated checks

```bash
cargo test --test seed_on_register_test
cargo test --test batch_payout_test
cargo test --test invoice_refund_test
cd frontend && npm run build
```

## Validation notes (implement)

| Scenario | Result |
|----------|--------|
| 1 Register full catalog wallets | Pass |
| 2 Batch partial + idempotency | Pass (`batch_payout_test`) |
| 3 Invoice refund partial/guards | Pass (`invoice_refund_test`) |
| 4 `npm run build` | Pass |

## Out of scope

- Auto-migrating legacy systems to full catalog
- Async batch job queue
- Separate gateway “refund” API
