# Contract: Batch Payouts

## `POST /batches` (auth: API key or session)

Create and process a payout batch synchronously.

### Request

```json
{
  "system_id": "uuid",
  "idempotency_key": "string",
  "lines": [
    {
      "amount": 1000,
      "currency": "ZMW",
      "country": "ZM",
      "external_id": "SHOP_20260723_abc",
      "payment_method": {
        "type": "mmo",
        "details": { "phone": "2607…", "provider": "MTN_MOMO_ZMB" }
      }
    }
  ]
}
```

- `lines`: 1–100 items
- Same commercial fields as `POST /payments` per line
- `system_id` must match authenticated system

### Responses

| Status | Meaning |
|--------|---------|
| 200 | Batch processed (may be partial); body includes batch + lines |
| 400 | Validation (empty lines, bad catalog combo, >100) |
| 401/403 | Auth |
| 409 | Idempotency key reused with different body |

### Response body (shape)

```json
{
  "id": "uuid",
  "status": "partial",
  "success_count": 2,
  "failure_count": 1,
  "lines": [
    {
      "line_index": 0,
      "status": "completed",
      "transaction_id": "uuid",
      "error": null
    },
    {
      "line_index": 1,
      "status": "failed",
      "transaction_id": null,
      "error": "insufficient_balance"
    }
  ]
}
```

## `GET /batches/{id}` (auth)

Return stored batch + lines for the authenticated system.

## Idempotency

- Unique `(system_id, idempotency_key)`
- Successful first processing persists batch; replay returns same payload

## Notes

- Single `POST /payments` remains unchanged
- Webhooks: each successful line emits existing payout transaction webhook semantics
