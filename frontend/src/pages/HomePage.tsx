import { Link, Navigate } from "react-router-dom";
import { useAuth } from "@/auth";
import { BrandLogo } from "@/components/BrandLogo";
import { Button } from "@/components/ui/button";

export function HomePage() {
  const { session } = useAuth();
  if (session) return <Navigate to="/dashboard" replace />;

  return (
    <div className="space-y-0">
      {/* DESIGN.md hero-band-dark */}
      <section className="bg-surface-dark px-6 py-16 text-white sm:px-12 sm:py-24">
        <div className="mx-auto max-w-xl space-y-6">
          <BrandLogo imgClassName="h-10 w-auto brightness-0 invert sm:h-12" />
          <h1 className="text-display-sm text-white sm:text-display-md">Relay</h1>
          <p className="text-lg text-white/70">
            Register your system, manage wallets and invoices, and collect mobile money payments.
          </p>
          <div className="flex flex-wrap gap-3 pt-2">
            <Button asChild size="lg">
              <Link to="/register">Register system</Link>
            </Button>
            <Button
              asChild
              variant="outline"
              size="lg"
              className="border-white/30 bg-transparent text-white hover:bg-white/10 hover:text-white"
            >
              <Link to="/login">Sign in</Link>
            </Button>
          </div>
        </div>
      </section>

      {/* Soft elevation band — surface-soft */}
      <section className="bg-muted px-6 py-16 sm:px-12 sm:py-20">
        <div className="mx-auto max-w-xl">
          <h2 className="text-title-lg text-foreground">Built for merchants</h2>
          <p className="mt-3 text-muted-foreground">
            Create invoices, share pay links, track payouts, and wire webhooks — one calm workspace.
          </p>
        </div>
      </section>
    </div>
  );
}
