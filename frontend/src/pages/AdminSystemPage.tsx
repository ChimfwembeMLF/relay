import { useEffect, useState } from "react";
import { Link, useParams } from "react-router-dom";
import {
  adminGetSystem,
  ApiError,
  formatMoney,
  type AdminSystemDetail,
} from "@/api";
import { Badge, statusBadgeVariant } from "@/components/ui/badge";
import { Button } from "@/components/ui/button";
import { Card, CardContent, CardHeader, CardTitle } from "@/components/ui/card";
import {
  Table,
  TableBody,
  TableCell,
  TableHead,
  TableHeader,
  TableRow,
} from "@/components/ui/table";

const STORAGE_KEY = "relay_admin_token";

export function AdminSystemPage() {
  const { id } = useParams<{ id: string }>();
  const adminToken = sessionStorage.getItem(STORAGE_KEY) ?? "";
  const [detail, setDetail] = useState<AdminSystemDetail | null>(null);
  const [error, setError] = useState<string | null>(null);

  useEffect(() => {
    if (!id || !adminToken) return;
    let cancelled = false;
    adminGetSystem(adminToken, id)
      .then((d) => {
        if (!cancelled) setDetail(d);
      })
      .catch((e) => {
        if (!cancelled) {
          setError(e instanceof ApiError ? e.message : "Failed to load system");
        }
      });
    return () => {
      cancelled = true;
    };
  }, [id, adminToken]);

  if (!adminToken) {
    return (
      <Card className="mx-auto max-w-md">
        <CardContent className="pt-6">
          <p className="text-muted-foreground">
            <Link to="/admin" className="text-primary underline-offset-4 hover:underline">
              Sign in to backoffice
            </Link>{" "}
            to view this system.
          </p>
        </CardContent>
      </Card>
    );
  }

  if (error) {
    return (
      <div className="space-y-4">
        <Button variant="ghost" asChild>
          <Link to="/admin">← All systems</Link>
        </Button>
        <p className="text-sm text-destructive">{error}</p>
      </div>
    );
  }

  if (!detail) {
    return <p className="text-sm text-muted-foreground">Loading…</p>;
  }

  const { system, wallets, webhook_endpoints } = detail;

  return (
    <div className="space-y-8">
      <Button variant="ghost" asChild>
        <Link to="/admin">← All systems</Link>
      </Button>

      <div>
        <h1 className="text-title-lg text-foreground">{system.name}</h1>
        <p className="mt-1 text-muted-foreground">
          Prefix <code>{system.prefix}</code> · ID <code>{system.id}</code>
        </p>
      </div>

      <Card>
        <CardHeader>
          <CardTitle>Countries</CardTitle>
        </CardHeader>
        <CardContent>
          <p>{system.enabled_countries.join(", ") || "—"}</p>
        </CardContent>
      </Card>

      <Card className="overflow-hidden p-0">
        <CardHeader className="p-6 sm:p-8">
          <CardTitle>Wallets</CardTitle>
        </CardHeader>
        <CardContent className="p-0">
          <Table>
            <TableHeader>
              <TableRow>
                <TableHead>Country</TableHead>
                <TableHead>Currency</TableHead>
                <TableHead>Balance</TableHead>
              </TableRow>
            </TableHeader>
            <TableBody>
              {wallets.map((w) => (
                <TableRow key={w.id}>
                  <TableCell>{w.country}</TableCell>
                  <TableCell>{w.currency}</TableCell>
                  <TableCell className="font-mono">{formatMoney(w.balance, w.currency)}</TableCell>
                </TableRow>
              ))}
              {wallets.length === 0 && (
                <TableRow>
                  <TableCell colSpan={3} className="text-muted-foreground">
                    No wallets
                  </TableCell>
                </TableRow>
              )}
            </TableBody>
          </Table>
        </CardContent>
      </Card>

      <Card className="overflow-hidden p-0">
        <CardHeader className="p-6 sm:p-8">
          <CardTitle>Webhook endpoints</CardTitle>
        </CardHeader>
        <CardContent className="p-0">
          <Table>
            <TableHeader>
              <TableRow>
                <TableHead>Label</TableHead>
                <TableHead>URL</TableHead>
                <TableHead>Status</TableHead>
              </TableRow>
            </TableHeader>
            <TableBody>
              {webhook_endpoints.map((ep) => (
                <TableRow key={ep.id}>
                  <TableCell>{ep.label || "—"}</TableCell>
                  <TableCell>
                    <code className="text-xs">{ep.url}</code>
                  </TableCell>
                  <TableCell>
                    <Badge variant={statusBadgeVariant(ep.enabled ? "enabled" : "disabled")}>
                      {ep.enabled ? "Enabled" : "Disabled"}
                    </Badge>
                  </TableCell>
                </TableRow>
              ))}
              {webhook_endpoints.length === 0 && (
                <TableRow>
                  <TableCell colSpan={3} className="text-muted-foreground">
                    No endpoints
                  </TableCell>
                </TableRow>
              )}
            </TableBody>
          </Table>
        </CardContent>
      </Card>
    </div>
  );
}
