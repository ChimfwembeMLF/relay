import { FormEvent, useState } from "react";
import { Link, useNavigate } from "react-router-dom";
import { ApiError, registerSystem } from "@/api";
import { useAuth } from "@/auth";
import { Button } from "@/components/ui/button";
import { Card, CardContent, CardDescription, CardHeader, CardTitle } from "@/components/ui/card";
import { Input } from "@/components/ui/input";
import { Label } from "@/components/ui/label";

export function RegisterPage() {
  const { signIn } = useAuth();
  const navigate = useNavigate();
  const [name, setName] = useState("");
  const [prefix, setPrefix] = useState("");
  const [username, setUsername] = useState("");
  const [password, setPassword] = useState("");
  const [webhookUrl, setWebhookUrl] = useState("");
  const [error, setError] = useState<string | null>(null);
  const [busy, setBusy] = useState(false);
  const [createdKey, setCreatedKey] = useState<string | null>(null);

  async function onSubmit(e: FormEvent) {
    e.preventDefault();
    setBusy(true);
    setError(null);
    try {
      const result = await registerSystem({
        name: name.trim(),
        prefix: prefix.trim().toUpperCase(),
        username: username.trim(),
        password,
        webhook_url: webhookUrl.trim() || undefined,
      });
      setCreatedKey(result.api_key);
      signIn({
        systemId: result.id,
        sessionToken: result.session_token,
        username: result.username,
        name: result.name,
        prefix: result.prefix,
        apiKey: result.api_key,
      });
    } catch (err) {
      setError(err instanceof ApiError ? err.message : "Registration failed");
    } finally {
      setBusy(false);
    }
  }

  if (createdKey) {
    return (
      <Card className="mx-auto max-w-lg">
        <CardHeader>
          <CardTitle className="text-title-lg">System registered</CardTitle>
          <CardDescription>
            You are signed in with Zambia enabled by default. Copy your API key for SDKs — Relay only
            shows it once.
          </CardDescription>
        </CardHeader>
        <CardContent className="space-y-5">
          <div className="break-all rounded-md border border-border bg-muted p-4 font-mono text-sm">
            {createdKey}
          </div>
          <div className="flex flex-wrap gap-2">
            <Button type="button" onClick={() => navigator.clipboard.writeText(createdKey)}>
              Copy API key
            </Button>
            <Button type="button" variant="outline" onClick={() => navigate("/dashboard")}>
              Go to dashboard
            </Button>
          </div>
        </CardContent>
      </Card>
    );
  }

  return (
    <Card className="mx-auto max-w-2xl">
      <CardHeader>
        <CardTitle className="text-title-lg">Register system</CardTitle>
        <CardDescription>
          Creates a merchant system and dashboard login. Zambia (ZMW) is enabled automatically.
        </CardDescription>
      </CardHeader>
      <CardContent>
        <form className="space-y-5" onSubmit={onSubmit}>
          <div className="grid gap-4 sm:grid-cols-2">
            <div className="space-y-2">
              <Label htmlFor="name">Business name</Label>
              <Input
                id="name"
                value={name}
                onChange={(e) => setName(e.target.value)}
                required
                placeholder="My Shop"
              />
            </div>
            <div className="space-y-2">
              <Label htmlFor="prefix">Prefix (2–8 chars, A–Z / 0–9)</Label>
              <Input
                id="prefix"
                value={prefix}
                onChange={(e) => setPrefix(e.target.value.toUpperCase())}
                required
                minLength={2}
                maxLength={8}
                pattern="[A-Z0-9]{2,8}"
                placeholder="SHOP"
              />
            </div>
            <div className="space-y-2">
              <Label htmlFor="username">Username</Label>
              <Input
                id="username"
                value={username}
                onChange={(e) => setUsername(e.target.value)}
                required
                minLength={3}
                autoComplete="username"
                placeholder="shop_admin"
              />
            </div>
            <div className="space-y-2">
              <Label htmlFor="password">Password</Label>
              <Input
                id="password"
                type="password"
                value={password}
                onChange={(e) => setPassword(e.target.value)}
                required
                minLength={8}
                autoComplete="new-password"
              />
            </div>
            <div className="space-y-2 sm:col-span-2">
              <Label htmlFor="webhook">Webhook URL (optional, HTTPS)</Label>
              <Input
                id="webhook"
                type="url"
                value={webhookUrl}
                onChange={(e) => setWebhookUrl(e.target.value)}
                placeholder="https://example.com/webhooks/relay"
              />
            </div>
          </div>
          {error && <p className="text-sm text-destructive">{error}</p>}
          <Button type="submit" disabled={busy}>
            {busy ? "Creating…" : "Create system"}
          </Button>
        </form>
        <p className="mt-5 text-sm text-muted-foreground">
          Already registered?{" "}
          <Link to="/login" className="font-medium text-primary underline-offset-4 hover:underline">
            Sign in
          </Link>
        </p>
      </CardContent>
    </Card>
  );
}
