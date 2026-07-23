import { useEffect, useState } from "react";
import { Link } from "react-router-dom";
import { ApiError, formatMoney, listInvoices, listWallets, type Invoice, type Wallet } from "@/api";
import { useAuth } from "@/auth";
import { EmptyState } from "@/components/EmptyState";
import { Badge, statusBadgeVariant } from "@/components/ui/badge";
import { Button } from "@/components/ui/button";
import { Card, CardContent, CardDescription, CardHeader, CardTitle } from "@/components/ui/card";
import {
  Table,
  TableBody,
  TableCell,
  TableHead,
  TableHeader,
  TableRow,
} from "@/components/ui/table";
import { countryByIso2, mnoLabels } from "@/lib/catalog";

export function DashboardPage() {
  const { session } = useAuth();
  const [wallets, setWallets] = useState<Wallet[]>([]);
  const [openInvoices, setOpenInvoices] = useState<Invoice[]>([]);
  const [error, setError] = useState<string | null>(null);

  useEffect(() => {
    if (!session) return;
    let cancelled = false;
    Promise.all([
      listWallets(session.systemId, session.sessionToken),
      listInvoices(session.sessionToken, { status: "open", limit: 5 }),
    ])
      .then(([w, inv]) => {
        if (cancelled) return;
        setWallets(w);
        setOpenInvoices(inv);
      })
      .catch((e) => {
        if (!cancelled) setError(e instanceof ApiError ? e.message : "Failed to load");
      });
    return () => {
      cancelled = true;
    };
  }, [session]);

  if (!session) return null;

  return (
    <div className="space-y-8">
      <div className="flex flex-col gap-4 sm:flex-row sm:items-start sm:justify-between">
        <div>
          <h1 className="text-title-lg text-foreground">Overview</h1>
          <p className="mt-1 text-muted-foreground">
            {session.name} · @{session.username} · prefix {session.prefix}
          </p>
        </div>
        <Button asChild>
          <Link to="/invoices/new">New invoice</Link>
        </Button>
      </div>

      {error && <p className="text-sm text-destructive">{error}</p>}

      <section className="space-y-4">
        <h2 className="text-title-md text-foreground">Wallets</h2>
        {wallets.length === 0 ? (
          <EmptyState title="No wallets yet" description="Wallets appear after registration seeds countries." />
        ) : (
          <Card className="overflow-hidden p-0">
            <Table>
              <TableHeader>
                <TableRow>
                  <TableHead>Country</TableHead>
                  <TableHead>Currency</TableHead>
                  <TableHead>Balance</TableHead>
                  <TableHead>Networks</TableHead>
                </TableRow>
              </TableHeader>
              <TableBody>
                {wallets.map((w) => {
                  const meta = countryByIso2(w.country);
                  const labels = mnoLabels(w.country);
                  return (
                    <TableRow key={w.id}>
                      <TableCell>
                        {meta ? (
                          <span>
                            {meta.flag} {meta.name}
                          </span>
                        ) : (
                          w.country
                        )}
                      </TableCell>
                      <TableCell>{w.currency}</TableCell>
                      <TableCell className="font-mono text-[15px]">
                        {formatMoney(w.balance, w.currency)}
                      </TableCell>
                      <TableCell>
                        <div className="flex flex-wrap gap-1.5">
                          {labels.length === 0 ? (
                            <span className="text-muted-foreground">—</span>
                          ) : (
                            labels.map((label) => (
                              <Badge key={label} variant="secondary">
                                {label}
                              </Badge>
                            ))
                          )}
                        </div>
                      </TableCell>
                    </TableRow>
                  );
                })}
              </TableBody>
            </Table>
          </Card>
        )}
      </section>

      <section className="space-y-4">
        <h2 className="text-title-md text-foreground">Open invoices</h2>
        {openInvoices.length === 0 ? (
          <EmptyState
            title="No open invoices"
            description="Create an invoice to share a pay link with customers."
            actionLabel="New invoice"
            actionTo="/invoices/new"
          />
        ) : (
          <Card className="overflow-hidden p-0">
            <Table>
              <TableHeader>
                <TableRow>
                  <TableHead>Reference</TableHead>
                  <TableHead>Amount</TableHead>
                  <TableHead>Status</TableHead>
                </TableRow>
              </TableHeader>
              <TableBody>
                {openInvoices.map((inv) => (
                  <TableRow key={inv.id}>
                    <TableCell>
                      <Link
                        to={`/invoices/${inv.reference}`}
                        className="font-medium text-primary underline-offset-4 hover:underline"
                      >
                        {inv.reference}
                      </Link>
                    </TableCell>
                    <TableCell className="font-mono">{formatMoney(inv.amount, inv.currency)}</TableCell>
                    <TableCell>
                      <Badge variant={statusBadgeVariant(inv.status)}>{inv.status}</Badge>
                    </TableCell>
                  </TableRow>
                ))}
              </TableBody>
            </Table>
          </Card>
        )}
      </section>

      <Card>
        <CardHeader>
          <CardTitle>API access</CardTitle>
          <CardDescription>
            Use an API key from registration in SDKs as <code>X-API-Key</code>. The dashboard uses
            your login session.
          </CardDescription>
        </CardHeader>
        <CardContent className="space-y-2">
          {session.apiKey ? (
            <div className="rounded-md border border-border bg-muted p-4 font-mono text-sm">
              {session.apiKey.slice(0, 12)}…{session.apiKey.slice(-4)}
            </div>
          ) : (
            <p className="text-sm text-muted-foreground">
              API key is only shown once at registration.
            </p>
          )}
          <p className="text-sm text-muted-foreground">System ID: {session.systemId}</p>
        </CardContent>
      </Card>
    </div>
  );
}
