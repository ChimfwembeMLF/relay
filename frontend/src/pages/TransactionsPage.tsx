import { useEffect, useState } from "react";
import { ApiError, formatMoney, listTransactions, type Transaction } from "@/api";
import { useAuth } from "@/auth";
import { EmptyState } from "@/components/EmptyState";
import { Badge, statusBadgeVariant } from "@/components/ui/badge";
import { Card } from "@/components/ui/card";
import {
  Table,
  TableBody,
  TableCell,
  TableHead,
  TableHeader,
  TableRow,
} from "@/components/ui/table";

export function TransactionsPage() {
  const { session } = useAuth();
  const [rows, setRows] = useState<Transaction[]>([]);
  const [error, setError] = useState<string | null>(null);

  useEffect(() => {
    if (!session) return;
    let cancelled = false;
    listTransactions(session.systemId, session.sessionToken, { limit: 50 })
      .then((data) => {
        if (!cancelled) setRows(data);
      })
      .catch((e) => {
        if (!cancelled) setError(e instanceof ApiError ? e.message : "Failed to load");
      });
    return () => {
      cancelled = true;
    };
  }, [session]);

  return (
    <div className="space-y-8">
      <div>
        <h1 className="text-title-lg text-foreground">Transactions</h1>
        <p className="mt-1 text-muted-foreground">Recent deposits and payouts for this system.</p>
      </div>
      {error && <p className="text-sm text-destructive">{error}</p>}
      {rows.length === 0 && !error ? (
        <EmptyState
          title="No transactions yet"
          description="Deposits and payouts will show up here once payments start flowing."
        />
      ) : (
        <Card className="overflow-hidden p-0">
          <Table>
            <TableHeader>
              <TableRow>
                <TableHead>External ID</TableHead>
                <TableHead>Amount</TableHead>
                <TableHead>Status</TableHead>
                <TableHead>Direction</TableHead>
                <TableHead>Created</TableHead>
              </TableRow>
            </TableHeader>
            <TableBody>
              {rows.map((tx) => (
                <TableRow key={tx.id}>
                  <TableCell className="font-mono text-xs sm:text-sm">{tx.external_id}</TableCell>
                  <TableCell className="font-mono">{formatMoney(tx.amount, tx.currency)}</TableCell>
                  <TableCell>
                    <Badge variant={statusBadgeVariant(tx.status)}>{tx.status}</Badge>
                  </TableCell>
                  <TableCell>{tx.direction ?? "—"}</TableCell>
                  <TableCell>
                    {tx.created_at ? new Date(tx.created_at).toLocaleString() : "—"}
                  </TableCell>
                </TableRow>
              ))}
            </TableBody>
          </Table>
        </Card>
      )}
    </div>
  );
}
