import { FormEvent, useEffect, useMemo, useState } from "react";
import { Link, useNavigate } from "react-router-dom";
import { ApiError, createInvoice, listWallets } from "@/api";
import { useAuth } from "@/auth";
import { CountrySelect } from "@/components/CountrySelect";
import { Button } from "@/components/ui/button";
import { Card, CardContent, CardDescription, CardHeader, CardTitle } from "@/components/ui/card";
import { Input } from "@/components/ui/input";
import { Label } from "@/components/ui/label";
import { Select } from "@/components/ui/select";
import { countriesForEnabled, countryByIso2 } from "@/lib/catalog";

export function NewInvoicePage() {
  const { session } = useAuth();
  const navigate = useNavigate();
  const [enabledIso2, setEnabledIso2] = useState<string[]>(["ZM"]);
  const [amount, setAmount] = useState("50.00");
  const [country, setCountry] = useState("ZM");
  const [currency, setCurrency] = useState("ZMW");
  const [description, setDescription] = useState("");
  const [error, setError] = useState<string | null>(null);
  const [busy, setBusy] = useState(false);

  const countryOptions = useMemo(() => countriesForEnabled(enabledIso2), [enabledIso2]);
  const entry = countryByIso2(country);
  const multiCurrency = (entry?.currencies.length ?? 0) > 1;

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
          setCountry(first);
          const cur = countryByIso2(first)?.defaultCurrency ?? "ZMW";
          setCurrency(cur);
        }
      })
      .catch(() => {
        /* keep Zambia default */
      });
    return () => {
      cancelled = true;
    };
  }, [session]);

  function onCountryChange(iso2: string) {
    setCountry(iso2);
    const next = countryByIso2(iso2);
    if (next) setCurrency(next.defaultCurrency);
  }

  async function onSubmit(e: FormEvent) {
    e.preventDefault();
    if (!session) return;
    setBusy(true);
    setError(null);
    try {
      const major = Number.parseFloat(amount);
      if (!Number.isFinite(major) || major <= 0) throw new Error("Enter a valid amount");
      const minor = Math.round(major * 100);
      const inv = await createInvoice(session.sessionToken, {
        amount: minor,
        currency: currency.toUpperCase(),
        country: country.toUpperCase(),
        description: description.trim() || undefined,
      });
      navigate(`/invoices/${inv.reference}`);
    } catch (err) {
      setError(err instanceof ApiError || err instanceof Error ? err.message : "Create failed");
    } finally {
      setBusy(false);
    }
  }

  return (
    <Card className="mx-auto max-w-lg">
      <CardHeader>
        <CardTitle className="text-title-lg">New invoice</CardTitle>
        <CardDescription>
          Amounts are entered in major units (e.g. 50.00 → 5000 cents). Currency follows the country.
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
              inputMode="decimal"
            />
          </div>
          <div className="space-y-2">
            <Label htmlFor="country">Country</Label>
            <CountrySelect
              id="country"
              value={country}
              onChange={onCountryChange}
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
            <Label htmlFor="description">Description</Label>
            <Input
              id="description"
              value={description}
              onChange={(e) => setDescription(e.target.value)}
              placeholder="Order #42"
            />
          </div>
          {error && <p className="text-sm text-destructive">{error}</p>}
          <div className="flex flex-wrap gap-2">
            <Button type="submit" disabled={busy}>
              {busy ? "Creating…" : "Create invoice"}
            </Button>
            <Button type="button" variant="ghost" asChild>
              <Link to="/invoices">Cancel</Link>
            </Button>
          </div>
        </form>
      </CardContent>
    </Card>
  );
}
