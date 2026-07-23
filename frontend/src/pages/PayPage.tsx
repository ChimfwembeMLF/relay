import { FormEvent, useEffect, useState } from "react";
import { useParams } from "react-router-dom";
import { fetchPayPage, PayPageData, submitPayPage } from "@/api";
import { BrandLogo } from "@/components/BrandLogo";
import { Button } from "@/components/ui/button";
import { Card, CardContent, CardDescription, CardHeader, CardTitle } from "@/components/ui/card";
import { Input } from "@/components/ui/input";
import { Label } from "@/components/ui/label";
import { Select } from "@/components/ui/select";

type ViewState = "loading" | "ready" | "paying" | "success" | "error" | "not_found";

function formatExpiry(iso: string | undefined): string {
  if (!iso) return "";
  try {
    return new Intl.DateTimeFormat(undefined, {
      dateStyle: "medium",
      timeStyle: "short",
    }).format(new Date(iso));
  } catch {
    return iso;
  }
}

function PageShell({ children }: { children: React.ReactNode }) {
  return (
    <div className="flex min-h-screen flex-col bg-background">
      <header className="flex h-16 items-center justify-between border-b border-border bg-background px-4 sm:px-6">
        <a href="/" className="flex items-center" aria-label="Relay">
          <BrandLogo imgClassName="h-8 w-auto" />
        </a>
        <span className="text-sm text-muted-foreground">Secure checkout</span>
      </header>
      <main className="mx-auto flex w-full max-w-md flex-1 items-start px-4 py-10 sm:px-6">
        {children}
      </main>
      <footer className="border-t border-border bg-muted/50 py-5 text-center text-sm text-muted-foreground">
        Powered by Relay · Mobile money payments
      </footer>
    </div>
  );
}

function StatusCard({
  title,
  body,
  amount,
  reference,
  tone = "default",
}: {
  title: string;
  body?: string | null;
  amount?: string | null;
  reference?: string;
  tone?: "default" | "success" | "error" | "dark";
}) {
  const shell =
    tone === "dark"
      ? "w-full border-0 bg-surface-dark text-white"
      : tone === "success"
        ? "w-full"
        : "w-full";

  return (
    <Card className={shell}>
      <CardContent className="space-y-4 pt-10 text-center">
        {amount && (
            <p
              className={
                tone === "dark"
                  ? "font-mono text-display-sm text-white"
                  : "font-mono text-display-sm text-foreground"
              }
            >
              {amount}
            </p>
          )}
          <h1
            className={
              tone === "dark" ? "text-title-lg text-white" : "text-title-lg text-foreground"
            }
          >
            {title}
          </h1>
        {body && (
          <p className={tone === "dark" ? "text-white/70" : "text-muted-foreground"}>{body}</p>
        )}
        {reference && (
          <p className={tone === "dark" ? "text-xs text-white/50" : "text-xs text-muted-foreground"}>
            Reference: {reference}
          </p>
        )}
      </CardContent>
    </Card>
  );
}

export function PayPage() {
  const { reference: routeRef } = useParams<{ reference: string }>();
  const reference = routeRef ?? window.location.pathname.split("/").pop() ?? "";

  const [view, setView] = useState<ViewState>("loading");
  const [invoice, setInvoice] = useState<PayPageData | null>(null);
  const [phone, setPhone] = useState("");
  const [provider, setProvider] = useState("");
  const [error, setError] = useState<string | null>(null);
  const [successMsg, setSuccessMsg] = useState<string | null>(null);

  useEffect(() => {
    if (!reference || reference === "pay") {
      setView("not_found");
      return;
    }

    let cancelled = false;
    fetchPayPage(reference)
      .then((data) => {
        if (cancelled) return;
        setInvoice(data);
        setProvider(data.providers[0]?.value ?? "");
        if (data.status === "paid") {
          setView("success");
          setSuccessMsg("Payment successful");
        } else {
          setView("ready");
        }
      })
      .catch((e: Error) => {
        if (cancelled) return;
        setView(e.message === "not_found" ? "not_found" : "error");
        setError(
          e.message === "not_found"
            ? "This payment link is invalid."
            : "Could not load invoice.",
        );
      });

    return () => {
      cancelled = true;
    };
  }, [reference]);

  async function onSubmit(e: FormEvent) {
    e.preventDefault();
    if (!invoice?.payable || !invoice.form_token || !invoice.idempotency_key) return;

    setView("paying");
    setError(null);
    try {
      const result = await submitPayPage(reference, {
        phone,
        provider,
        idempotency_key: invoice.idempotency_key,
        form_token: invoice.form_token,
      });
      setSuccessMsg(result.message);
      setView("success");
    } catch (err) {
      setView("ready");
      setError(err instanceof Error ? err.message : "Payment failed");
    }
  }

  if (view === "loading") {
    return (
      <PageShell>
        <Card className="w-full">
          <CardContent className="py-12 text-center text-muted-foreground">
            Loading invoice…
          </CardContent>
        </Card>
      </PageShell>
    );
  }

  if (view === "not_found") {
    return (
      <PageShell>
        <StatusCard
          title="Link not found"
          body="The payment link is invalid or has expired."
          tone="dark"
        />
      </PageShell>
    );
  }

  if (view === "error") {
    return (
      <PageShell>
        <StatusCard title="Something went wrong" body={error} tone="error" />
      </PageShell>
    );
  }

  if (view === "success") {
    return (
      <PageShell>
        <StatusCard
          title="Payment complete"
          body={successMsg}
          amount={invoice?.amount_display}
          reference={reference}
          tone="success"
        />
      </PageShell>
    );
  }

  const statusLabel =
    invoice?.status === "expired"
      ? "Invoice expired"
      : invoice?.status === "cancelled"
        ? "Invoice cancelled"
        : null;

  if (invoice && !invoice.payable) {
    return (
      <PageShell>
        <StatusCard
          title={statusLabel ?? "Not payable"}
          body={
            invoice.status === "expired"
              ? "This invoice is no longer payable. Request a new one from the merchant."
              : invoice.status === "cancelled"
                ? "This invoice has been cancelled."
                : undefined
          }
          reference={reference}
          tone="dark"
        />
      </PageShell>
    );
  }

  return (
    <PageShell>
      <Card className="w-full">
        <CardHeader>
          <CardTitle className="text-2xl">Pay invoice</CardTitle>
          <CardDescription>Enter your mobile money details to complete payment</CardDescription>
        </CardHeader>
        <CardContent className="space-y-6">
          <div>
            <p className="font-mono text-display-sm text-foreground sm:text-display-md">
              {invoice?.amount_display}
            </p>
            {invoice?.description && (
              <p className="mt-2 text-sm text-muted-foreground">{invoice.description}</p>
            )}
          </div>

          <div className="flex flex-wrap gap-4 text-sm text-muted-foreground">
            <p>
              <span className="font-medium text-foreground">Country</span> · {invoice?.country}
            </p>
            <p>
              <span className="font-medium text-foreground">Expires</span> ·{" "}
              {formatExpiry(invoice?.expires_at)}
            </p>
          </div>

          <form onSubmit={onSubmit} className="space-y-5">
            <div className="space-y-2">
              <Label htmlFor="phone">Mobile number</Label>
              <Input
                id="phone"
                type="tel"
                value={phone}
                onChange={(e) => setPhone(e.target.value)}
                placeholder="260763456789"
                required
                disabled={view === "paying"}
                autoComplete="tel"
              />
            </div>

            <div className="space-y-2">
              <Label htmlFor="provider">Provider</Label>
              <Select
                id="provider"
                value={provider}
                onChange={(e) => setProvider(e.target.value)}
                required
                disabled={view === "paying"}
              >
                {invoice?.providers.map((p) => (
                  <option key={p.value} value={p.value}>
                    {p.label}
                  </option>
                ))}
              </Select>
            </div>

            {error && <p className="text-sm text-destructive">{error}</p>}

            <Button type="submit" className="w-full" size="lg" disabled={view === "paying"}>
              {view === "paying" ? "Processing…" : "Pay now"}
            </Button>
          </form>

          <p className="text-center text-xs text-muted-foreground">Reference: {reference}</p>
        </CardContent>
      </Card>
    </PageShell>
  );
}
