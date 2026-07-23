from __future__ import annotations

import hashlib
import hmac
import json
from typing import Any, Literal, TypedDict, Union


class PaymentStatusChangedWebhook(TypedDict):
    event: Literal["payment.status_changed"]
    payment_id: str
    system_id: str
    external_id: str
    status: Literal["completed", "failed"]
    amount: int
    currency: str
    country: str
    gateway_reference: str | None
    error: str | None
    timestamp: str


class InvoicePaidWebhook(TypedDict):
    event: Literal["invoice.paid"]
    invoice_id: str
    system_id: str
    reference: str
    amount: int
    currency: str
    country: str
    status: Literal["paid"]
    transaction_id: str
    timestamp: str


WebhookEvent = Union[PaymentStatusChangedWebhook, InvoicePaidWebhook]

SIGNATURE_PREFIX = "sha256="


def verify_webhook_signature(
    raw_body: str | bytes,
    signature_header: str | None,
    secret: str,
) -> bool:
    if not signature_header or not signature_header.startswith(SIGNATURE_PREFIX):
        return False

    if isinstance(raw_body, str):
        raw_body = raw_body.encode("utf-8")

    expected = hmac.new(secret.encode("utf-8"), raw_body, hashlib.sha256).hexdigest()
    provided = signature_header[len(SIGNATURE_PREFIX) :]

    return hmac.compare_digest(expected, provided)


def parse_webhook_event(raw_body: str | bytes) -> WebhookEvent:
    if isinstance(raw_body, bytes):
        raw_body = raw_body.decode("utf-8")
    event: dict[str, Any] = json.loads(raw_body)
    if event.get("event") in ("payment.status_changed", "invoice.paid"):
        return event  # type: ignore[return-value]
    raise ValueError(f"Unknown webhook event: {event.get('event')}")


def is_payment_status_changed(event: WebhookEvent) -> bool:
    return event.get("event") == "payment.status_changed"


def is_invoice_paid(event: WebhookEvent) -> bool:
    return event.get("event") == "invoice.paid"
