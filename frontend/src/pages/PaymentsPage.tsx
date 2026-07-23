import { FormEvent, useEffect, useMemo, useState } from "react";
import {
  ApiError,
  createBatch,
  listWallets,
  processPayment,
  type BatchLineResult,
} from "@/api";
import { useAuth } from "@/auth";
import { CountrySelect } from "@/components/CountrySelect";
import { ProviderSelect } from "@/components/ProviderSelect";
import { Button } from "@/components/ui/button";
import { Card, CardContent, CardDescription, CardHeader, CardTitle } from "@/components/ui/card";
import { Input } from "@/components/ui/input";
import { Label } from "@/components/ui/label";
import { Select } from "@/components/ui/select";
import { countriesForEnabled, countryByIso2 } from "@/lib/catalog";

type BatchRow = {
  id: string;
  amount: string;
  country: string;
  currency: string;
  provider: string;
  phone: string;
};

function emptyRow(country = "ZM"): BatchRow {
  const entry = countryByIso2(country);
  return {
    id: crypto.randomUUID(),
    amount: "10.00",
    country,
    currency: entry?.defaultCurrency ?? "ZMW",
    provider: entry?.mnos[0]?.correspondent ?? "",
    phone: entry?.dialingPrefix ?? "",
  };
}

export function PaymentsPage() {
  const { session } = useAuth();
  const [mode, setMode] = useState<"single" | "batch">("single");
  const [enabledIso2, setEnabledIso2] = useState<string[]>(["ZM"]);
  const [amount, setAmount] = useState("25.00");
  const [country, setCountry] = useState("ZM");
  const [currency, setCurrency] = useState("ZMW");
  const [phone, setPhone] = useState("260763456789");
  const [provider, setProvider] = useState(
    () => countryByIso2("ZM")?.mnos[0]?.correspondent ?? "",
  );
  const [externalId, setExternalId] = useState("");
  const [rows, setRows] = useState<BatchRow[]>(() => [emptyRow()]);
  const [paste, setPaste] = useState("");
  const [batchResults, setBatchResults] = useState<BatchLineResult[] | null>(null);
  const [result, setResult] = useState<string | null>(null);
  const [error, setError] = useState<string | null>(null);
  const [busy, setBusy] = useState(false);

  const countryOptions = useMemo(() => countriesForEnabled(enabledIso2), [enabledIso2]);
  const entry = countryByIso2(country);
  const multiCurrency = (entry?.currencies.length ?? 0) > 1;

  function applyCountry(iso2: string) {
    setCountry(iso2);
    const next = countryByIso2(iso2);
    if (next) {
      setCurrency(next.defaultCurrency);
      setProvider(next.mnos[0]?.correspondent ?? "");
      setPhone(next.dialingPrefix);
    }
  }

  useEffect(() => {
    if (!session) return;
    let cancelled = false;
    listWallets(session.systemId, session.sessionToken)
      .then((wallets) => {
        if (cancelled) return;
        const codes = [...new Set(wallets.map((w) => w.country.toUpperCase()))];
        if (codes.length) {
          setEnabledIso2(codes);
          const first = codes.includes("ZM") ? "ZM" : codes[0];
          applyCountry(first);
          setRows([emptyRow(first)]);
        }
      })
      .catch(() => {
        /* keep defaults */
      });
    return () => {
      cancelled = true;
    };
  }, [session]);

  function updateRow(id: string, patch: Partial<BatchRow>) {
    setRows((prev) =>
      prev.map((r) => {
        if (r.id !== id) return r;
        const next = { ...r, ...patch };
        if (patch.country) {
          const meta = countryByIso2(patch.country);
          if (meta) {
            next.currency = meta.defaultCurrency;
            next.provider = meta.mnos[0]?.correspondent ?? "";
            if (!patch.phone) next.phone = meta.dialingPrefix;
          }
        }
        return next;
      }),
    );
  }

  function applyPaste() {
    const lines = paste
      .trim()
      .split(/\n/)
      .map((l) => l.trim())
      .filter(Boolean);
    if (!lines.length) return;
    const parsed: BatchRow[] = [];
    for (const line of lines) {
      const parts = line.split(/[,\t]/).map((p) => p.trim());
      if (parts.length < 3) continue;
      const [phoneVal, amountVal, countryVal, providerVal] = parts;
      const iso = (countryVal || "ZM").toUpperCase();
      const meta = countryByIso2(iso);
      parsed.push({
        id: crypto.randomUUID(),
        phone: phoneVal,
        amount: amountVal,
        country: iso,
        currency: meta?.defaultCurrency ?? "ZMW",
        provider: providerVal || meta?.mnos[0]?.correspondent || "",
      });
    }
    if (parsed.length) setRows(parsed);
  }

  async function onSubmitSingle(e: FormEvent) {
    e.preventDefault();
    if (!session) return;
    if (!provider) {
      setError("Select a mobile money provider");
      return;
    }
    if (!phone.trim()) {
      setError("Enter a recipient phone number");
      return;
    }
    setBusy(true);
    setError(null);
    setResult(null);
    try {
      const major = Number.parseFloat(amount);
      const minor = Math.round(major * 100);
      const ext =
        externalId.trim() ||
        `${session.prefix}_${new Date().toISOString().slice(0, 10).replace(/-/g, "")}_${crypto.randomUUID().slice(0, 8)}`;
      const payment = await processPayment(session.sessionToken, {
        system_id: session.systemId,
        external_id: ext,
        amount: minor,
        currency: currency.toUpperCase(),
        country: country.toUpperCase(),
        payment_method: {
          type: "mmo",
          details: { phone: phone.trim(), provider },
        },
        idempotency_key: crypto.randomUUID(),
      });
      setResult(`Payment ${payment.status} · id ${payment.id}`);
    } catch (err) {
      setError(err instanceof ApiError ? err.message : "Payment failed");
    } finally {
      setBusy(false);
    }
  }

  async function onSubmitBatch(e: FormEvent) {
    e.preventDefault();
    if (!session) return;
    setBusy(true);
    setError(null);
    setResult(null);
    setBatchResults(null);
    try {
      const lines = rows.map((r) => ({
        amount: Math.round(Number.parseFloat(r.amount) * 100),
        currency: r.currency.toUpperCase(),
        country: r.country.toUpperCase(),
        payment_method: {
          type: "mmo",
          details: { phone: r.phone.trim(), provider: r.provider },
        },
      }));
      const batch = await createBatch(session.sessionToken, {
        system_id: session.systemId,
        idempotency_key: crypto.randomUUID(),
        lines,
      });
      setBatchResults(batch.lines);
      setResult(
        `Batch ${batch.status} · ${batch.success_count} ok · ${batch.failure_count} failed`,
      );
    } catch (err) {
      setError(err instanceof ApiError ? err.message : "Batch failed");
    } finally {
      setBusy(false);
    }
  }

  return (
    <div className="space-y-8">
      <div className="flex flex-col gap-4 sm:flex-row sm:items-end sm:justify-between">
        <div>
          <h1 className="text-title-lg text-foreground">Payouts</h1>
          <p className="mt-1 text-muted-foreground">Debit your wallet and send mobile money.</p>
        </div>
        <div className="flex gap-2">
          <Button
            type="button"
            variant={mode === "single" ? "default" : "outline"}
            onClick={() => setMode("single")}
          >
            Single
          </Button>
          <Button
            type="button"
            variant={mode === "batch" ? "default" : "outline"}
            onClick={() => setMode("batch")}
          >
            Batch
          </Button>
        </div>
      </div>

      {mode === "single" ? (
        <Card className="mx-auto max-w-lg">
          <CardHeader>
            <CardTitle className="text-2xl">Send payout</CardTitle>
            <CardDescription>
              Choose country and network — Relay maps them to the correct gateway codes.
            </CardDescription>
          </CardHeader>
          <CardContent>
            <form className="space-y-5" onSubmit={onSubmitSingle}>
              <div className="space-y-2">
                <Label htmlFor="amount">Amount</Label>
                <Input
                  id="amount"
                  value={amount}
                  onChange={(e) => setAmount(e.target.value)}
                  required
                />
              </div>
              <div className="space-y-2">
                <Label htmlFor="country">Country</Label>
                <CountrySelect
                  id="country"
                  value={country}
                  onChange={applyCountry}
                  options={countryOptions}
                  required
                />
              </div>
              <div className="space-y-2">
                <Label htmlFor="currency">Currency</Label>
                {multiCurrency ? (
                  <Select
                    id="currency"
                    value={currency}
                    onChange={(e) => setCurrency(e.target.value)}
                    required
                  >
                    {entry?.currencies.map((c) => (
                      <option key={c} value={c}>
                        {c}
                      </option>
                    ))}
                  </Select>
                ) : (
                  <Input id="currency" value={currency} readOnly className="bg-muted" />
                )}
              </div>
              <div className="space-y-2">
                <Label htmlFor="provider">Mobile network</Label>
                <ProviderSelect
                  id="provider"
                  countryIso2={country}
                  value={provider}
                  onChange={setProvider}
                  required
                />
              </div>
              <div className="space-y-2">
                <Label htmlFor="phone">Recipient phone</Label>
                <Input
                  id="phone"
                  value={phone}
                  onChange={(e) => setPhone(e.target.value)}
                  required
                  inputMode="tel"
                />
              </div>
              <div className="space-y-2">
                <Label htmlFor="external">External ID (optional)</Label>
                <Input
                  id="external"
                  value={externalId}
                  onChange={(e) => setExternalId(e.target.value)}
                />
              </div>
              {error && <p className="text-sm text-destructive">{error}</p>}
              {result && <p className="text-sm font-medium text-success">{result}</p>}
              <Button type="submit" disabled={busy}>
                {busy ? "Sending…" : "Send payout"}
              </Button>
            </form>
          </CardContent>
        </Card>
      ) : (
        <Card className="mx-auto max-w-3xl">
          <CardHeader>
            <CardTitle className="text-2xl">Batch payout</CardTitle>
            <CardDescription>
              Add rows or paste CSV/TSV: phone, amount, country[, provider]. Max 100 lines.
            </CardDescription>
          </CardHeader>
          <CardContent>
            <form className="space-y-5" onSubmit={onSubmitBatch}>
              <div className="space-y-2">
                <Label htmlFor="paste">Paste lines (optional)</Label>
                <textarea
                  id="paste"
                  className="min-h-[88px] w-full rounded-md border border-border bg-background px-3 py-2 text-sm"
                  value={paste}
                  onChange={(e) => setPaste(e.target.value)}
                  placeholder={"2607…,10.00,ZM,MTN_MOMO_ZMB"}
                />
                <Button type="button" variant="outline" onClick={applyPaste}>
                  Apply paste
                </Button>
              </div>

              <div className="space-y-4">
                {rows.map((row, i) => {
                  const meta = countryByIso2(row.country);
                  const multi = (meta?.currencies.length ?? 0) > 1;
                  return (
                    <div
                      key={row.id}
                      className="grid gap-3 rounded-md border border-border p-4 sm:grid-cols-2"
                    >
                      <p className="sm:col-span-2 text-sm font-medium text-muted-foreground">
                        Line {i + 1}
                      </p>
                      <div className="space-y-2">
                        <Label>Amount</Label>
                        <Input
                          value={row.amount}
                          onChange={(e) => updateRow(row.id, { amount: e.target.value })}
                          required
                        />
                      </div>
                      <div className="space-y-2">
                        <Label>Country</Label>
                        <CountrySelect
                          value={row.country}
                          onChange={(c) => updateRow(row.id, { country: c })}
                          options={countryOptions}
                          required
                        />
                      </div>
                      <div className="space-y-2">
                        <Label>Currency</Label>
                        {multi ? (
                          <Select
                            value={row.currency}
                            onChange={(e) => updateRow(row.id, { currency: e.target.value })}
                          >
                            {meta?.currencies.map((c) => (
                              <option key={c} value={c}>
                                {c}
                              </option>
                            ))}
                          </Select>
                        ) : (
                          <Input value={row.currency} readOnly className="bg-muted" />
                        )}
                      </div>
                      <div className="space-y-2">
                        <Label>Network</Label>
                        <ProviderSelect
                          countryIso2={row.country}
                          value={row.provider}
                          onChange={(p) => updateRow(row.id, { provider: p })}
                          required
                        />
                      </div>
                      <div className="space-y-2 sm:col-span-2">
                        <Label>Phone</Label>
                        <Input
                          value={row.phone}
                          onChange={(e) => updateRow(row.id, { phone: e.target.value })}
                          required
                        />
                      </div>
                      {rows.length > 1 && (
                        <Button
                          type="button"
                          variant="ghost"
                          onClick={() => setRows((r) => r.filter((x) => x.id !== row.id))}
                        >
                          Remove row
                        </Button>
                      )}
                    </div>
                  );
                })}
              </div>

              <div className="flex flex-wrap gap-2">
                <Button
                  type="button"
                  variant="outline"
                  onClick={() => setRows((r) => [...r, emptyRow(r[0]?.country ?? "ZM")])}
                >
                  Add row
                </Button>
                <Button type="submit" disabled={busy}>
                  {busy ? "Sending…" : "Submit batch"}
                </Button>
              </div>

              {error && <p className="text-sm text-destructive">{error}</p>}
              {result && <p className="text-sm font-medium text-success">{result}</p>}
              {batchResults && (
                <ul className="space-y-1 text-sm text-muted-foreground">
                  {batchResults.map((l) => (
                    <li key={l.line_index} className="flex justify-between gap-4 border-b border-border py-2">
                      <span>
                        #{l.line_index} {l.external_id}
                      </span>
                      <span className={l.status === "completed" ? "text-success" : "text-destructive"}>
                        {l.status}
                        {l.error ? ` · ${l.error}` : ""}
                      </span>
                    </li>
                  ))}
                </ul>
              )}
            </form>
          </CardContent>
        </Card>
      )}
    </div>
  );
}
