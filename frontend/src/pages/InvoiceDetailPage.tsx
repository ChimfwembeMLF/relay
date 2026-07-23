import { useEffect, useState } from "react";
import { Link, useParams } from "react-router-dom";
import { ApiError, cancelInvoice, formatMoney, getInvoice, type Invoice } from "@/api";
import { useAuth } from "@/auth";
import { Badge, statusBadgeVariant } from "@/components/ui/badge";
import { Button } from "@/components/ui/button";
import { Card, CardContent, CardDescription, CardHeader, CardTitle } from "@/components/ui/card";

export function InvoiceDetailPage() {
  const { reference } = useParams<{ reference: string }>();
  const { session } = useAuth();
  const [invoice, setInvoice] = useState<Invoice | null>(null);
  const [error, setError] = useState<string | null>(null);
  const [busy, setBusy] = useState(false);

  useEffect(() => {
    if (!session || !reference) return;
    let cancelled = false;
    getInvoice(session.sessionToken, reference)
      .then((row) => {
        if (!cancelled) setInvoice(row);
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

  if (!invoice && !error) return <p className="text-sm text-muted-foreground">Loading…</p>;
  if (!invoice) return <p className="text-sm text-destructive">{error}</p>;

  const payUrl = invoice.qr_url || `${window.location.origin}/pay/${invoice.reference}`;

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
          </p>
        </div>
        <Button variant="outline" asChild>
          <Link to="/invoices">All invoices</Link>
        </Button>
      </div>

      {error && <p className="text-sm text-destructive">{error}</p>}
      {invoice.description && <p className="text-muted-foreground">{invoice.description}</p>}

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

      {invoice.status === "open" && (
        <Button type="button" variant="ghost" disabled={busy} onClick={onCancel}>
          {busy ? "Cancelling…" : "Cancel invoice"}
        </Button>
      )}
    </div>
  );
}
