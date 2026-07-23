import { createHmac, timingSafeEqual } from "node:crypto";
import type { InvoicePaidWebhook, PaymentStatusChangedWebhook, RelayWebhookEvent } from "./types.js";

const SIGNATURE_PREFIX = "sha256=";

/** Verify `X-Relay-Signature` header against raw webhook body bytes. */
export function verifyWebhookSignature(
  rawBody: string | Buffer,
  signatureHeader: string | null | undefined,
  secret: string,
): boolean {
  if (!signatureHeader?.startsWith(SIGNATURE_PREFIX)) {
    return false;
  }

  const expected = createHmac("sha256", secret)
    .update(rawBody)
    .digest("hex");
  const provided = signatureHeader.slice(SIGNATURE_PREFIX.length);

  try {
    return timingSafeEqual(Buffer.from(expected, "hex"), Buffer.from(provided, "hex"));
  } catch {
    return false;
  }
}

/** Parse webhook JSON after signature verification. */
export function parseWebhookEvent(rawBody: string): RelayWebhookEvent {
  const event = JSON.parse(rawBody) as RelayWebhookEvent;
  if (event.event === "payment.status_changed" || event.event === "invoice.paid") {
    return event;
  }
  throw new Error(`Unknown webhook event: ${(event as { event?: string }).event}`);
}

export function isPaymentStatusChanged(
  event: RelayWebhookEvent,
): event is PaymentStatusChangedWebhook {
  return event.event === "payment.status_changed";
}

export function isInvoicePaid(event: RelayWebhookEvent): event is InvoicePaidWebhook {
  return event.event === "invoice.paid";
}
