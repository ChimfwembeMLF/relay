import { FormEvent, useEffect, useMemo, useState } from "react";
import { ApiError, listWallets, processPayment } from "@/api";
import { useAuth } from "@/auth";
import { CountrySelect } from "@/components/CountrySelect";
import { ProviderSelect } from "@/components/ProviderSelect";
import { Button } from "@/components/ui/button";
import { Card, CardContent, CardDescription, CardHeader, CardTitle } from "@/components/ui/card";
import { Input } from "@/components/ui/input";
import { Label } from "@/components/ui/label";
import { Select } from "@/components/ui/select";
import { countriesForEnabled, countryByIso2 } from "@/lib/catalog";

export function PaymentsPage() {
  const { session } = useAuth();
  const [enabledIso2, setEnabledIso2] = useState<string[]>(["ZM"]);
  const [amount, setAmount] = useState("25.00");
  const [country, setCountry] = useState("ZM");
  const [currency, setCurrency] = useState("ZMW");
  const [phone, setPhone] = useState("260763456789");
  const [provider, setProvider] = useState(
    () => countryByIso2("ZM")?.mnos[0]?.correspondent ?? "",
  );
  const [externalId, setExternalId] = useState("");
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
        }
      })
      .catch(() => {
        /* keep defaults */
      });
    return () => {
      cancelled = true;
    };
  }, [session]);

  async function onSubmit(e: FormEvent) {
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

  return (
    <div className="space-y-8">
      <div>
        <h1 className="text-title-lg text-foreground">Payouts</h1>
        <p className="mt-1 text-muted-foreground">Debit your wallet and send mobile money.</p>
      </div>
      <Card className="mx-auto max-w-lg">
        <CardHeader>
          <CardTitle className="text-2xl">Send payout</CardTitle>
          <CardDescription>
            Choose country and network — Relay maps them to the correct gateway codes.
          </CardDescription>
        </CardHeader>
        <CardContent>
          <form className="space-y-5" onSubmit={onSubmit}>
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
                placeholder={`${entry?.dialingPrefix ?? ""}…`}
              />
            </div>
            <div className="space-y-2">
              <Label htmlFor="external">External ID (optional)</Label>
              <Input
                id="external"
                value={externalId}
                onChange={(e) => setExternalId(e.target.value)}
                placeholder="SHOP_20260723_ABC12345"
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
    </div>
  );
}
