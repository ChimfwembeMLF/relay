import { FormEvent, useEffect, useState } from "react";
import { Link } from "react-router-dom";
import {
  adminListSystems,
  adminLogin,
  adminLogout,
  ApiError,
  type AdminSystemSummary,
} from "@/api";
import { EmptyState } from "@/components/EmptyState";
import { Button } from "@/components/ui/button";
import { Card, CardContent, CardDescription, CardHeader, CardTitle } from "@/components/ui/card";
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

const TOKEN_KEY = "relay_admin_token";
const USER_KEY = "relay_admin_username";

export function AdminPage() {
  const [token, setToken] = useState(() => sessionStorage.getItem(TOKEN_KEY) ?? "");
  const [username, setUsername] = useState(() => sessionStorage.getItem(USER_KEY) ?? "");
  const [draftUser, setDraftUser] = useState("");
  const [draftPass, setDraftPass] = useState("");
  const [systems, setSystems] = useState<AdminSystemSummary[]>([]);
  const [error, setError] = useState<string | null>(null);
  const [busy, setBusy] = useState(false);

  useEffect(() => {
    if (!token) {
      setSystems([]);
      return;
    }
    let cancelled = false;
    setBusy(true);
    adminListSystems(token)
      .then((rows) => {
        if (!cancelled) {
          setSystems(rows);
          setError(null);
        }
      })
      .catch((e) => {
        if (!cancelled) {
          setSystems([]);
          if (e instanceof ApiError && e.status === 401) {
            sessionStorage.removeItem(TOKEN_KEY);
            sessionStorage.removeItem(USER_KEY);
            setToken("");
            setUsername("");
            setError("Session expired. Sign in again.");
          } else {
            setError(e instanceof ApiError ? e.message : "Failed to load systems");
          }
        }
      })
      .finally(() => {
        if (!cancelled) setBusy(false);
      });
    return () => {
      cancelled = true;
    };
  }, [token]);

  async function onLogin(e: FormEvent) {
    e.preventDefault();
    setBusy(true);
    setError(null);
    try {
      const res = await adminLogin(draftUser.trim(), draftPass);
      sessionStorage.setItem(TOKEN_KEY, res.token);
      sessionStorage.setItem(USER_KEY, res.username);
      setToken(res.token);
      setUsername(res.username);
      setDraftPass("");
    } catch (err) {
      setError(
        err instanceof ApiError
          ? err.status === 401
            ? "Invalid username or password"
            : err.message
          : "Sign in failed",
      );
    } finally {
      setBusy(false);
    }
  }

  async function signOutAdmin() {
    try {
      if (token) await adminLogout(token);
    } catch {
      /* ignore */
    }
    sessionStorage.removeItem(TOKEN_KEY);
    sessionStorage.removeItem(USER_KEY);
    setToken("");
    setUsername("");
    setSystems([]);
  }

  if (!token) {
    return (
      <Card className="mx-auto max-w-md">
        <CardHeader>
          <CardTitle className="text-title-lg">Backoffice</CardTitle>
          <CardDescription>Sign in with your platform admin credentials.</CardDescription>
        </CardHeader>
        <CardContent>
          <form className="space-y-5" onSubmit={onLogin}>
            <div className="space-y-2">
              <Label htmlFor="username">Username</Label>
              <Input
                id="username"
                value={draftUser}
                onChange={(e) => setDraftUser(e.target.value)}
                required
                autoComplete="username"
                placeholder="admin"
              />
            </div>
            <div className="space-y-2">
              <Label htmlFor="password">Password</Label>
              <Input
                id="password"
                type="password"
                value={draftPass}
                onChange={(e) => setDraftPass(e.target.value)}
                required
                autoComplete="current-password"
              />
            </div>
            {error && <p className="text-sm text-destructive">{error}</p>}
            <Button type="submit" className="w-full" disabled={busy}>
              {busy ? "Signing in…" : "Sign in"}
            </Button>
          </form>
        </CardContent>
      </Card>
    );
  }

  return (
    <div className="space-y-8">
      <div className="flex flex-col gap-4 sm:flex-row sm:items-start sm:justify-between">
        <div>
          <h1 className="text-title-lg text-foreground">Backoffice</h1>
          <p className="mt-1 text-muted-foreground">
            Signed in as <strong className="font-medium text-foreground">{username}</strong> · all
            tenant systems
          </p>
        </div>
        <Button type="button" variant="outline" onClick={signOutAdmin}>
          Sign out
        </Button>
      </div>

      {error && <p className="text-sm text-destructive">{error}</p>}
      {busy && !systems.length && !error && (
        <p className="text-sm text-muted-foreground">Loading…</p>
      )}

      {!busy && systems.length === 0 && !error ? (
        <EmptyState
          title="No systems registered yet"
          description="Merchant systems appear here after registration."
        />
      ) : (
        <Card className="overflow-hidden p-0">
          <Table>
            <TableHeader>
              <TableRow>
                <TableHead>Name</TableHead>
                <TableHead>Prefix</TableHead>
                <TableHead>Countries</TableHead>
                <TableHead>Webhooks</TableHead>
                <TableHead>Created</TableHead>
              </TableRow>
            </TableHeader>
            <TableBody>
              {systems.map((s) => (
                <TableRow key={s.id}>
                  <TableCell>
                    <Link
                      to={`/admin/systems/${s.id}`}
                      className="font-medium text-primary underline-offset-4 hover:underline"
                    >
                      {s.name}
                    </Link>
                  </TableCell>
                  <TableCell>
                    <code>{s.prefix}</code>
                  </TableCell>
                  <TableCell>{s.enabled_countries.join(", ")}</TableCell>
                  <TableCell>{s.webhook_endpoints}</TableCell>
                  <TableCell>{new Date(s.created_at).toLocaleString()}</TableCell>
                </TableRow>
              ))}
            </TableBody>
          </Table>
        </Card>
      )}
    </div>
  );
}
