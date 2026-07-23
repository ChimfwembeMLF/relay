# Contract: Invoice Refunds

## Invoice fields (read models)

Invoice JSON gains:

| Field | Type | Notes |
|-------|------|-------|
| `refunded_amount` | integer | Minor units |
| `remaining_refundable` | integer | `amount - refunded_amount` when paid; else 0 |
| `fully_refunded` | boolean | paid && remaining == 0 |
| `payer_phone` | string \| null | From collect |
| `payer_provider` | string \| null | Correspondent from collect |

`status` remains `paid` after refunds.

## `POST /invoices/{id}/refund` (auth: API key or session)

### Request

```json
{
  "amount": 500,
  "idempotency_key": "string",
  "phone": "2607…",
  "provider": "MTN_MOMO_ZMB"
}
```

- `amount` required, minor units, `1 … remaining_refundable`
- `phone` / `provider` optional if invoice has payer fields; required if missing
- Country/currency taken from invoice

### Responses

| Status | Meaning |
|--------|---------|
| 200 | Refund completed (or idempotent replay) |
| 400 | Validation / not refundable state / over-refund |
| 402 | Insufficient wallet balance |
| 404 | Invoice not found |
| 409 | Idempotency conflict |

### Response body (shape)

```json
{
  "id": "uuid",
  "invoice_id": "uuid",
  "amount": 500,
  "status": "completed",
  "transaction_id": "uuid",
  "invoice": {
    "refunded_amount": 500,
    "remaining_refundable": 4500,
    "fully_refunded": false,
    "status": "paid"
  }
}
```

## Collect side-effect

Successful `POST /invoices/{id}/collect` and pay-page collect MUST persist `payer_phone` and `payer_provider` on the invoice.

## Webhooks

Refund payout completion uses existing payout transaction webhook (transaction has `refund_id` / `invoice_id` as applicable).
