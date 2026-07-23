import { FormEvent, useEffect, useState } from "react";
import { Link, useParams } from "react-router-dom";
import {
  ApiError,
  cancelInvoice,
  formatMoney,
  getInvoice,
  refundInvoice,
  type Invoice,
} from "@/api";
import { useAuth } from "@/auth";
import { ProviderSelect } from "@/components/ProviderSelect";
import { Badge, statusBadgeVariant } from "@/components/ui/badge";
import { Button } from "@/components/ui/button";
import { Card, CardContent, CardDescription, CardHeader, CardTitle } from "@/components/ui/card";
import { Input } from "@/components/ui/input";
import { Label } from "@/components/ui/label";

export function InvoiceDetailPage() {
  const { reference } = useParams<{ reference: string }>();
  const { session } = useAuth();
  const [invoice, setInvoice] = useState<Invoice | null>(null);
  const [error, setError] = useState<string | null>(null);
  const [busy, setBusy] = useState(false);
  const [refundAmount, setRefundAmount] = useState("");
  const [refundPhone, setRefundPhone] = useState("");
  const [refundProvider, setRefundProvider] = useState("");
  const [refundMsg, setRefundMsg] = useState<string | null>(null);

  useEffect(() => {
    if (!session || !reference) return;
    let cancelled = false;
    getInvoice(session.sessionToken, reference)
      .then((row) => {
        if (cancelled) return;
        setInvoice(row);
        const remaining = row.remaining_refundable ?? Math.max(0, row.amount - (row.refunded_amount ?? 0));
        setRefundAmount((remaining / 100).toFixed(2));
        setRefundPhone(row.payer_phone ?? "");
        setRefundProvider(row.payer_provider ?? "");
      })
      .catch((e) => {
        if (!cancelled) setError(e instanceof ApiError ? e.message : "Failed to load");
      });
    return () => {
      cancelled = true;
    };
  }, [session, reference]);

  async function onCancel() {
    if (!session || !invoice || invoice.status !== "open") return;
    setBusy(true);
    try {
      await cancelInvoice(session.sessionToken, invoice.id);
      setInvoice({ ...invoice, status: "cancelled" });
    } catch (e) {
      setError(e instanceof ApiError ? e.message : "Cancel failed");
    } finally {
      setBusy(false);
    }
  }

  async function onRefund(e: FormEvent) {
    e.preventDefault();
    if (!session || !invoice) return;
    setBusy(true);
    setError(null);
    setRefundMsg(null);
    try {
      const minor = Math.round(Number.parseFloat(refundAmount) * 100);
      const res = await refundInvoice(session.sessionToken, invoice.id, {
        amount: minor,
        idempotency_key: crypto.randomUUID(),
        phone: refundPhone.trim() || undefined,
        provider: refundProvider || undefined,
      });
      setInvoice({
        ...invoice,
        refunded_amount: res.invoice.refunded_amount,
        remaining_refundable: res.invoice.remaining_refundable,
        fully_refunded: res.invoice.fully_refunded,
        status: res.invoice.status,
      });
      setRefundAmount((res.invoice.remaining_refundable / 100).toFixed(2));
      setRefundMsg(`Refund ${res.status} · ${formatMoney(res.amount, invoice.currency)}`);
    } catch (err) {
      setError(err instanceof ApiError ? err.message : "Refund failed");
    } finally {
      setBusy(false);
    }
  }

  if (!invoice && !error) return <p className="text-sm text-muted-foreground">Loading…</p>;
  if (!invoice) return <p className="text-sm text-destructive">{error}</p>;

  const payUrl = invoice.qr_url || `${window.location.origin}/pay/${invoice.reference}`;
  const remaining =
    invoice.remaining_refundable ?? Math.max(0, invoice.amount - (invoice.refunded_amount ?? 0));
  const canRefund = invoice.status === "paid" && remaining > 0;

  return (
    <div className="space-y-8">
      <div className="flex flex-col gap-4 sm:flex-row sm:items-start sm:justify-between">
        <div className="space-y-2">
          <h1 className="text-title-lg text-foreground">{invoice.reference}</h1>
          <p className="flex flex-wrap items-center gap-3 text-muted-foreground">
            <span className="font-mono text-lg text-foreground">
              {formatMoney(invoice.amount, invoice.currency)}
            </span>
            <Badge variant={statusBadgeVariant(invoice.status)}>{invoice.status}</Badge>
            {invoice.fully_refunded && <Badge variant="secondary">fully refunded</Badge>}
          </p>
        </div>
        <Button variant="outline" asChild>
          <Link to="/invoices">All invoices</Link>
        </Button>
      </div>

      {error && <p className="text-sm text-destructive">{error}</p>}
      {invoice.description && <p className="text-muted-foreground">{invoice.description}</p>}

      {invoice.status === "paid" && (
        <p className="text-sm text-muted-foreground">
          Refunded {formatMoney(invoice.refunded_amount ?? 0, invoice.currency)} · Remaining{" "}
          {formatMoney(remaining, invoice.currency)}
        </p>
      )}

      <Card>
        <CardHeader>
          <CardTitle>Pay link</CardTitle>
          <CardDescription>Share this URL or QR with the customer.</CardDescription>
        </CardHeader>
        <CardContent className="space-y-5">
          <div className="break-all rounded-md border border-border bg-muted p-4 font-mono text-sm">
            <a
              href={payUrl}
              target="_blank"
              rel="noreferrer"
              className="text-primary underline-offset-4 hover:underline"
            >
              {payUrl}
            </a>
          </div>
          {invoice.qr_code_png_base64 && (
            <img
              className="h-48 w-48 rounded-xl border border-border bg-white p-2"
              src={
                invoice.qr_code_png_base64.startsWith("data:")
                  ? invoice.qr_code_png_base64
                  : `data:image/png;base64,${invoice.qr_code_png_base64}`
              }
              alt={`QR for ${invoice.reference}`}
            />
          )}
          <Button type="button" variant="secondary" onClick={() => navigator.clipboard.writeText(payUrl)}>
            Copy pay link
          </Button>
        </CardContent>
      </Card>

      {canRefund && (
        <Card className="mx-auto max-w-lg">
          <CardHeader>
            <CardTitle>Refund</CardTitle>
            <CardDescription>
              Debits your wallet and sends mobile money to the customer. Invoice stays paid.
            </CardDescription>
          </CardHeader>
          <CardContent>
            <form className="space-y-4" onSubmit={onRefund}>
              <div className="space-y-2">
                <Label htmlFor="refund-amount">Amount</Label>
                <Input
                  id="refund-amount"
                  value={refundAmount}
                  onChange={(e) => setRefundAmount(e.target.value)}
                  required
                />
              </div>
              <div className="space-y-2">
                <Label htmlFor="refund-phone">Destination phone</Label>
                <Input
                  id="refund-phone"
                  value={refundPhone}
                  onChange={(e) => setRefundPhone(e.target.value)}
                  required
                />
              </div>
              <div className="space-y-2">
                <Label htmlFor="refund-provider">Network</Label>
                <ProviderSelect
                  id="refund-provider"
                  countryIso2={invoice.country}
                  value={refundProvider}
                  onChange={setRefundProvider}
                  required
                />
              </div>
              {refundMsg && <p className="text-sm font-medium text-success">{refundMsg}</p>}
              <Button type="submit" disabled={busy}>
                {busy ? "Refunding…" : "Issue refund"}
              </Button>
            </form>
          </CardContent>
        </Card>
      )}

      {invoice.status === "open" && (
        <Button type="button" variant="ghost" disabled={busy} onClick={onCancel}>
          {busy ? "Cancelling…" : "Cancel invoice"}
        </Button>
      )}
    </div>
  );
}
