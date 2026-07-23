# Reports Integration

Export transaction, wallet, and invoice summaries for reconciliation and accounting.

## Common parameters

All report endpoints require a date range:

| Parameter | Format | Example |
|-----------|--------|---------|
| `from` | ISO 8601 datetime | `2026-07-01T00:00:00Z` |
| `to` | ISO 8601 datetime | `2026-07-31T23:59:59Z` |
| `format` | `json` (default) or `csv` | `csv` |

All require `X-API-Key`.

## Transaction report

**JSON summary**

```bash
curl -s "http://localhost:8080/reports/transactions?from=2026-07-01T00:00:00Z&to=2026-07-31T23:59:59Z" \
  -H "X-API-Key: $RELAY_API_KEY" | jq .
```

```json
{
  "from": "2026-07-01T00:00:00Z",
  "to": "2026-07-31T23:59:59Z",
  "total_count": 42,
  "total_amount": 125000,
  "by_status": {
    "completed": { "count": 40, "amount": 120000 },
    "failed": { "count": 2, "amount": 5000 }
  }
}
```

**TypeScript**

```typescript
const report = await relay.reports.transactions({
  from: "2026-07-01T00:00:00Z",
  to: "2026-07-31T23:59:59Z",
});
```

**CSV detail export**

```typescript
const csv = await relay.reports.transactions({
  from: "2026-07-01T00:00:00Z",
  to: "2026-07-31T23:59:59Z",
  format: "csv",
  detail: true,
});

fs.writeFileSync("transactions-july.csv", csv as string);
```

**Python**

```python
csv = relay.reports.transactions(
    from_date="2026-07-01T00:00:00Z",
    to_date="2026-07-31T23:59:59Z",
    format="csv",
    detail=True,
)
Path("transactions-july.csv").write_text(csv)
```

Optional filters: `status` (query param), `detail=true` for row-level CSV.

## Wallet report

Shows current balances and period activity per country/currency.

```typescript
const report = await relay.reports.wallets({
  from: "2026-07-01T00:00:00Z",
  to: "2026-07-31T23:59:59Z",
});
```

```json
{
  "from": "2026-07-01T00:00:00Z",
  "to": "2026-07-31T23:59:59Z",
  "wallets": [
    {
      "country": "ZM",
      "currency": "ZMW",
      "current_balance": 102500,
      "period_deposits": 50000,
      "period_payouts": 25000,
      "net_change": 25000
    }
  ]
}
```

**Python**

```python
report = relay.reports.wallets(
    from_date="2026-07-01T00:00:00Z",
    to_date="2026-07-31T23:59:59Z",
)
```

## Invoice report

Summary of invoices created in the date range.

```typescript
const report = await relay.reports.invoices({
  from: "2026-07-01T00:00:00Z",
  to: "2026-07-31T23:59:59Z",
  status: "paid",
});
```

Filter by `status`: `open`, `paid`, `expired`, or `cancelled`.

## CSV format

Set `format=csv` to receive `text/csv` instead of JSON. The SDK returns a raw string.

```bash
curl -s "http://localhost:8080/reports/wallets?from=2026-07-01T00:00:00Z&to=2026-07-31T23:59:59Z&format=csv" \
  -H "X-API-Key: $RELAY_API_KEY" -o wallets-july.csv
```

## Tips

- Use UTC datetimes for consistent boundaries
- Run wallet reports after month-end before accounting close
- Combine with [webhooks](webhooks.md) for real-time updates; use reports for reconciliation
