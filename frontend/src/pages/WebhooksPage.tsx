import { FormEvent, useEffect, useState } from "react";
import {
  ApiError,
  createWebhookEndpoint,
  deleteWebhookEndpoint,
  listWebhookEndpoints,
  updateWebhookEndpoint,
  type WebhookEndpoint,
} from "@/api";
import { useAuth } from "@/auth";
import { EmptyState } from "@/components/EmptyState";
import { Badge, statusBadgeVariant } from "@/components/ui/badge";
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

export function WebhooksPage() {
  const { session } = useAuth();
  const [endpoints, setEndpoints] = useState<WebhookEndpoint[]>([]);
  const [error, setError] = useState<string | null>(null);
  const [url, setUrl] = useState("");
  const [label, setLabel] = useState("");
  const [busy, setBusy] = useState(false);

  async function refresh() {
    if (!session) return;
    const rows = await listWebhookEndpoints(session.sessionToken);
    setEndpoints(rows);
  }

  useEffect(() => {
    if (!session) return;
    let cancelled = false;
    listWebhookEndpoints(session.sessionToken)
      .then((rows) => {
        if (!cancelled) setEndpoints(rows);
      })
      .catch((e) => {
        if (!cancelled) setError(e instanceof ApiError ? e.message : "Failed to load");
      });
    return () => {
      cancelled = true;
    };
  }, [session]);

  async function onCreate(e: FormEvent) {
    e.preventDefault();
    if (!session) return;
    setBusy(true);
    setError(null);
    try {
      await createWebhookEndpoint(session.sessionToken, {
        url: url.trim(),
        label: label.trim() || undefined,
      });
      setUrl("");
      setLabel("");
      await refresh();
    } catch (err) {
      setError(err instanceof ApiError ? err.message : "Could not create endpoint");
    } finally {
      setBusy(false);
    }
  }

  async function toggleEnabled(ep: WebhookEndpoint) {
    if (!session) return;
    setError(null);
    try {
      await updateWebhookEndpoint(session.sessionToken, ep.id, { enabled: !ep.enabled });
      await refresh();
    } catch (err) {
      setError(err instanceof ApiError ? err.message : "Could not update endpoint");
    }
  }

  async function onDelete(id: string) {
    if (!session) return;
    if (!window.confirm("Remove this webhook endpoint?")) return;
    setError(null);
    try {
      await deleteWebhookEndpoint(session.sessionToken, id);
      await refresh();
    } catch (err) {
      setError(err instanceof ApiError ? err.message : "Could not delete endpoint");
    }
  }

  return (
    <div className="space-y-8">
      <div>
        <h1 className="text-title-lg text-foreground">Webhooks</h1>
        <p className="mt-1 text-muted-foreground">
          Receive signed payment and invoice events at your HTTPS endpoints.
        </p>
      </div>

      <Card>
        <CardHeader>
          <CardTitle>Add endpoint</CardTitle>
        </CardHeader>
        <CardContent>
          <form className="space-y-5" onSubmit={onCreate}>
            <div className="space-y-2">
              <Label htmlFor="url">URL</Label>
              <Input
                id="url"
                type="url"
                value={url}
                onChange={(e) => setUrl(e.target.value)}
                required
                placeholder="https://example.com/hooks/relay"
              />
            </div>
            <div className="space-y-2">
              <Label htmlFor="label">Label (optional)</Label>
              <Input
                id="label"
                value={label}
                onChange={(e) => setLabel(e.target.value)}
                placeholder="Production"
              />
            </div>
            <Button type="submit" disabled={busy}>
              {busy ? "Adding…" : "Add webhook"}
            </Button>
          </form>
        </CardContent>
      </Card>

      {error && <p className="text-sm text-destructive">{error}</p>}

      {endpoints.length === 0 ? (
        <EmptyState
          title="No webhook endpoints yet"
          description="Add an HTTPS URL to receive signed Relay events."
        />
      ) : (
        <Card className="overflow-hidden p-0">
          <Table>
            <TableHeader>
              <TableRow>
                <TableHead>Label</TableHead>
                <TableHead>URL</TableHead>
                <TableHead>Status</TableHead>
                <TableHead />
              </TableRow>
            </TableHeader>
            <TableBody>
              {endpoints.map((ep) => (
                <TableRow key={ep.id}>
                  <TableCell>{ep.label || "—"}</TableCell>
                  <TableCell>
                    <code className="text-xs">{ep.url}</code>
                  </TableCell>
                  <TableCell>
                    <Button type="button" variant="ghost" size="sm" onClick={() => toggleEnabled(ep)}>
                      <Badge variant={statusBadgeVariant(ep.enabled ? "enabled" : "disabled")}>
                        {ep.enabled ? "Enabled" : "Disabled"}
                      </Badge>
                    </Button>
                  </TableCell>
                  <TableCell>
                    <Button type="button" variant="ghost" size="sm" onClick={() => onDelete(ep.id)}>
                      Remove
                    </Button>
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
