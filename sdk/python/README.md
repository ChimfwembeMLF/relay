# relay-sdk

Python client for the Payment Relay merchant API.

## Install

```bash
pip install -e sdk/python
```

## Usage

```python
import os
from relay_sdk import RelayClient, RelayError

relay = RelayClient(
    base_url="http://localhost:8080",
    api_key=os.environ["RELAY_API_KEY"],
)

invoice = relay.invoices.create(
    amount=5000,
    currency="ZMW",
    country="ZM",
    description="Order #42",
)

print(invoice["qr_url"])
```

## Webhooks (Flask example)

```python
from flask import Flask, request
from relay_sdk import parse_webhook_event, verify_webhook_signature

app = Flask(__name__)

@app.post("/webhooks/relay")
def relay_webhook():
    raw = request.get_data()
    signature = request.headers.get("X-Relay-Signature")

    if not verify_webhook_signature(raw, signature, os.environ["WEBHOOK_SIGNING_SECRET"]):
        return {"error": "invalid signature"}, 401

    event = parse_webhook_event(raw)
    # handle event["event"] == "payment.status_changed" or "invoice.paid"
    return {"ok": True}
```

## Resources

| Property | Methods |
|----------|---------|
| `relay.systems` | `create(**fields)`, `get(id)` |
| `relay.payments` | `process(**fields)`, `get(id)` |
| `relay.wallets` | `list(system_id)` |
| `relay.transactions` | `list(system_id, **params)` |
| `relay.invoices` | `create`, `list`, `get`, `collect`, `cancel` |
| `relay.reports` | `transactions`, `wallets`, `invoices` |
| `relay.pay` | `get(reference)`, `submit(reference, **fields)` |

See [integration guides](../../docs/integration/README.md) for full workflows.
