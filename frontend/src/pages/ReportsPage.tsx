import { FormEvent, useState } from "react";
import {
  ApiError,
  formatMoney,
  reportInvoices,
  reportTransactions,
  reportWallets,
  type ReportSummary,
  type WalletsReport,
} from "@/api";
import { useAuth } from "@/auth";
import { Button } from "@/components/ui/button";
import { Card, CardContent, CardHeader, CardTitle } from "@/components/ui/card";
import { Input } from "@/components/ui/input";
import { Label } from "@/components/ui/label";
import {
  Table,
  TableBody,
  TableCell,
  TableHead,
  TableHeader,
  TableRow,
} from "@/components/ui/table";
import { countryByIso2 } from "@/lib/catalog";

function monthRange(): { from: string; to: string } {
  const now = new Date();
  const from = new Date(Date.UTC(now.getUTCFullYear(), now.getUTCMonth(), 1));
  return { from: from.toISOString(), to: now.toISOString() };
}

export function ReportsPage() {
  const { session } = useAuth();
  const defaults = monthRange();
  const [from, setFrom] = useState(defaults.from.slice(0, 16));
  const [to, setTo] = useState(defaults.to.slice(0, 16));
  const [tx, setTx] = useState<ReportSummary | null>(null);
  const [inv, setInv] = useState<ReportSummary | null>(null);
  const [wallets, setWallets] = useState<WalletsReport | null>(null);
  const [error, setError] = useState<string | null>(null);
  const [busy, setBusy] = useState(false);

  async function onSubmit(e: FormEvent) {
    e.preventDefault();
    if (!session) return;
    setBusy(true);
    setError(null);
    try {
      const fromIso = new Date(from).toISOString();
      const toIso = new Date(to).toISOString();
      const [t, i, w] = await Promise.all([
        reportTransactions(session.sessionToken, fromIso, toIso),
        reportInvoices(session.sessionToken, fromIso, toIso),
        reportWallets(session.sessionToken, fromIso, toIso),
      ]);
      setTx(t);
      setInv(i);
      setWallets(w);
    } catch (err) {
      setError(err instanceof ApiError ? err.message : "Report failed");
    } finally {
      setBusy(false);
    }
  }

  return (
    <div className="space-y-8">
      <div>
        <h1 className="text-title-lg text-foreground">Reports</h1>
        <p className="mt-1 text-muted-foreground">Summaries for reconciliation and accounting.</p>
      </div>

      <Card>
        <CardHeader>
          <CardTitle>Date range</CardTitle>
        </CardHeader>
        <CardContent>
          <form className="flex flex-col gap-4 sm:flex-row sm:items-end" onSubmit={onSubmit}>
            <div className="space-y-2">
              <Label htmlFor="from">From</Label>
              <Input
                id="from"
                type="datetime-local"
                value={from}
                onChange={(e) => setFrom(e.target.value)}
                required
              />
            </div>
            <div className="space-y-2">
              <Label htmlFor="to">To</Label>
              <Input
                id="to"
                type="datetime-local"
                value={to}
                onChange={(e) => setTo(e.target.value)}
                required
              />
            </div>
            <Button type="submit" disabled={busy}>
              {busy ? "Loading…" : "Run reports"}
            </Button>
          </form>
        </CardContent>
      </Card>

      {error && <p className="text-sm text-destructive">{error}</p>}

      {(tx || inv) && (
        <div className="grid gap-4 sm:grid-cols-2">
          {tx && (
            <Card>
              <CardHeader>
                <CardTitle>Transactions</CardTitle>
              </CardHeader>
              <CardContent className="space-y-3">
                <p className="text-number-display text-foreground">
                  {tx.total_count} txs · {tx.total_amount}
                </p>
                <ul className="space-y-1.5 text-sm text-muted-foreground">
                  {Object.entries(tx.by_status).map(([status, s]) => (
                    <li key={status} className="flex justify-between gap-4 border-b border-border py-2 last:border-0">
                      <span className="capitalize">{status}</span>
                      <span className="font-mono text-foreground">
                        {s.count} · {s.amount}
                      </span>
                    </li>
                  ))}
                </ul>
              </CardContent>
            </Card>
          )}
          {inv && (
            <Card>
              <CardHeader>
                <CardTitle>Invoices</CardTitle>
              </CardHeader>
              <CardContent className="space-y-3">
                <p className="text-number-display text-foreground">
                  {inv.total_count} invoices · {inv.total_amount}
                </p>
                <ul className="space-y-1.5 text-sm text-muted-foreground">
                  {Object.entries(inv.by_status).map(([status, s]) => (
                    <li key={status} className="flex justify-between gap-4 border-b border-border py-2 last:border-0">
                      <span className="capitalize">{status}</span>
                      <span className="font-mono text-foreground">
                        {s.count} · {s.amount}
                      </span>
                    </li>
                  ))}
                </ul>
              </CardContent>
            </Card>
          )}
        </div>
      )}

      {wallets && (
        <Card className="overflow-hidden p-0">
          <CardHeader className="p-6 sm:p-8">
            <CardTitle>Wallets</CardTitle>
          </CardHeader>
          <CardContent className="p-0">
            <Table>
              <TableHeader>
                <TableRow>
                  <TableHead>Country</TableHead>
                  <TableHead>Balance</TableHead>
                  <TableHead>Deposits</TableHead>
                  <TableHead>Payouts</TableHead>
                  <TableHead>Net</TableHead>
                </TableRow>
              </TableHeader>
              <TableBody>
                {wallets.wallets.map((w) => {
                  const meta = countryByIso2(w.country);
                  return (
                    <TableRow key={`${w.country}-${w.currency}`}>
                      <TableCell>
                        {meta ? `${meta.flag} ${meta.name}` : w.country}/{w.currency}
                      </TableCell>
                      <TableCell className="font-mono">
                        {formatMoney(w.current_balance, w.currency)}
                      </TableCell>
                      <TableCell className="font-mono text-success">
                        {formatMoney(w.period_deposits, w.currency)}
                      </TableCell>
                      <TableCell className="font-mono text-destructive">
                        {formatMoney(w.period_payouts, w.currency)}
                      </TableCell>
                      <TableCell className="font-mono">
                        {formatMoney(w.net_change, w.currency)}
                      </TableCell>
                    </TableRow>
                  );
                })}
              </TableBody>
            </Table>
          </CardContent>
        </Card>
      )}
    </div>
  );
}
