import { FormEvent, useState } from "react";
import { Link, useNavigate } from "react-router-dom";
import { ApiError, registerSystem } from "@/api";
import logoBlue from "@/assets/logo-blue.png";
import { useAuth } from "@/auth";
import { FieldLabel } from "@/components/FieldLabel";
import { Button } from "@/components/ui/button";
import { Input } from "@/components/ui/input";
import { TooltipProvider } from "@/components/ui/tooltip";

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

  return (
    <TooltipProvider delayDuration={200}>
      <div className="grid min-h-screen lg:grid-cols-2">
        <aside className="relative hidden overflow-hidden bg-[#0a0b0d] lg:block">
          <img
            src="/register-hero.jpg"
            alt=""
            className="absolute inset-0 h-full w-full object-cover"
          />
          <div className="absolute inset-0 bg-gradient-to-t from-[#0a0b0d]/85 via-[#0a0b0d]/35 to-transparent" />
          <div className="relative z-10 flex h-full flex-col justify-end p-10 xl:p-14">
            <p className="text-display-lg text-white">Relay</p>
            <p className="mt-4 max-w-md text-lg text-[#a8acb3]">
              Collect and disburse across African mobile money — one system, every market we
              support.
            </p>
          </div>
        </aside>

        <div className="flex min-h-screen flex-col bg-background">
          <header className="flex h-16 items-center justify-between px-6 sm:px-10">
            <Link to="/" className="flex items-center" aria-label="Relay home">
              <img src={logoBlue} alt="Relay" className="h-8 w-auto" />
            </Link>
            <Button asChild variant="ghost" size="sm">
              <Link to="/login">Sign in</Link>
            </Button>
          </header>

          <div className="flex flex-1 items-center px-6 py-10 sm:px-10 lg:px-14">
            <div className="mx-auto w-full max-w-lg">
              {createdKey ? (
                <div className="space-y-6">
                  <div>
                    <h1 className="text-title-lg text-foreground">System registered</h1>
                    <p className="mt-2 text-muted-foreground">
                      You are signed in with all supported markets enabled. Copy your API key —
                      Relay only shows it once.
                    </p>
                  </div>
                  <div className="break-all rounded-xl border border-border bg-muted p-4 font-mono text-sm">
                    {createdKey}
                  </div>
                  <div className="flex flex-wrap gap-2">
                    <Button
                      type="button"
                      onClick={() => navigator.clipboard.writeText(createdKey)}
                    >
                      Copy API key
                    </Button>
                    <Button
                      type="button"
                      variant="outline"
                      onClick={() => navigate("/dashboard")}
                    >
                      Go to dashboard
                    </Button>
                  </div>
                </div>
              ) : (
                <>
                  <div className="mb-8">
                    <h1 className="text-title-lg text-foreground">Create your system</h1>
                    <p className="mt-2 text-muted-foreground">
                      Business login for the dashboard. All catalog countries are enabled
                      automatically.
                    </p>
                  </div>

                  <form className="space-y-5" onSubmit={onSubmit}>
                    <div className="grid gap-4 sm:grid-cols-2">
                      <div className="space-y-2">
                        <FieldLabel
                          htmlFor="name"
                          tip="Display name for your merchant system in the dashboard and reports."
                        >
                          Business name
                        </FieldLabel>
                        <Input
                          id="name"
                          value={name}
                          onChange={(e) => setName(e.target.value)}
                          required
                          placeholder="My Shop"
                        />
                      </div>
                      <div className="space-y-2">
                        <FieldLabel
                          htmlFor="prefix"
                          tip="Short unique code (2–8 letters/numbers) used to build external IDs for payouts, e.g. SHOP_20260723_abc."
                        >
                          Prefix
                        </FieldLabel>
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
                        <FieldLabel
                          htmlFor="username"
                          tip="Dashboard login username (3–64 characters). Letters, numbers, and _ - . only."
                        >
                          Username
                        </FieldLabel>
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
                        <FieldLabel
                          htmlFor="password"
                          tip="Dashboard password. Must be at least 8 characters."
                        >
                          Password
                        </FieldLabel>
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
                        <FieldLabel
                          htmlFor="webhook"
                          tip="HTTPS URL Relay will POST signed payment events to. You can add or change endpoints later in Webhooks."
                        >
                          Webhook URL
                        </FieldLabel>
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
                    <Button type="submit" className="w-full sm:w-auto" disabled={busy}>
                      {busy ? "Creating…" : "Create system"}
                    </Button>
                  </form>

                  <p className="mt-8 text-sm text-muted-foreground">
                    Already registered?{" "}
                    <Link
                      to="/login"
                      className="font-medium text-primary underline-offset-4 hover:underline"
                    >
                      Sign in
                    </Link>
                  </p>
                </>
              )}
            </div>
          </div>
        </div>
      </div>
    </TooltipProvider>
  );
}
