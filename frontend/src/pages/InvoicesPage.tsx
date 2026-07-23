import { useEffect, useState } from "react";
import { Link } from "react-router-dom";
import { ApiError, formatMoney, listInvoices, type Invoice } from "@/api";
import { useAuth } from "@/auth";
import { EmptyState } from "@/components/EmptyState";
import { Badge, statusBadgeVariant } from "@/components/ui/badge";
import { Button } from "@/components/ui/button";
import { Card } from "@/components/ui/card";
import { Label } from "@/components/ui/label";
import { Select } from "@/components/ui/select";
import {
  Table,
  TableBody,
  TableCell,
  TableHead,
  TableHeader,
  TableRow,
} from "@/components/ui/table";

export function InvoicesPage() {
  const { session } = useAuth();
  const [status, setStatus] = useState<string>("");
  const [invoices, setInvoices] = useState<Invoice[]>([]);
  const [error, setError] = useState<string | null>(null);

  useEffect(() => {
    if (!session) return;
    let cancelled = false;
    listInvoices(session.sessionToken, {
      status: status || undefined,
      limit: 50,
    })
      .then((rows) => {
        if (!cancelled) setInvoices(rows);
      })
      .catch((e) => {
        if (!cancelled) setError(e instanceof ApiError ? e.message : "Failed to load");
      });
    return () => {
      cancelled = true;
    };
  }, [session, status]);

  return (
    <div className="space-y-8">
      <div className="flex flex-col gap-4 sm:flex-row sm:items-start sm:justify-between">
        <div>
          <h1 className="text-title-lg text-foreground">Invoices</h1>
          <p className="mt-1 text-muted-foreground">
            Create QR invoices and share pay links with customers.
          </p>
        </div>
        <Button asChild>
          <Link to="/invoices/new">New invoice</Link>
        </Button>
      </div>

      <div className="flex max-w-xs flex-col gap-2">
        <Label htmlFor="status">Status</Label>
        <Select id="status" value={status} onChange={(e) => setStatus(e.target.value)}>
          <option value="">All</option>
          <option value="open">Open</option>
          <option value="paid">Paid</option>
          <option value="expired">Expired</option>
          <option value="cancelled">Cancelled</option>
        </Select>
      </div>

      {error && <p className="text-sm text-destructive">{error}</p>}

      {invoices.length === 0 && !error ? (
        <EmptyState
          title="No invoices found"
          description="Create an invoice to get a pay link and QR code."
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
                <TableHead>Country</TableHead>
                <TableHead>Status</TableHead>
                <TableHead>Expires</TableHead>
              </TableRow>
            </TableHeader>
            <TableBody>
              {invoices.map((inv) => (
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
                  <TableCell>{inv.country}</TableCell>
                  <TableCell>
                    <Badge variant={statusBadgeVariant(inv.status)}>{inv.status}</Badge>
                  </TableCell>
                  <TableCell>{new Date(inv.expires_at).toLocaleString()}</TableCell>
                </TableRow>
              ))}
            </TableBody>
          </Table>
        </Card>
      )}
    </div>
  );
}
