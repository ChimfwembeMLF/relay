import { useEffect, useState } from "react";
import { Link, NavLink, Outlet, useLocation } from "react-router-dom";
import {
  FileText,
  LayoutDashboard,
  Link2,
  LogOut,
  Menu,
  PanelLeftClose,
  PanelLeftOpen,
  Receipt,
  Send,
  X,
  BookOpen,
  Shield,
} from "lucide-react";
import { useAuth } from "@/auth";
import { BrandLogo, BrandMark } from "@/components/BrandLogo";
import { Button } from "@/components/ui/button";
import { cn } from "@/lib/utils";
import { merchantLogout } from "@/api";

const STORAGE_KEY = "relay_sidebar_collapsed";

type NavItem = {
  to: string;
  label: string;
  icon: React.ComponentType<{ className?: string }>;
  end?: boolean;
};

const merchantNav: NavItem[] = [
  { to: "/dashboard", label: "Overview", icon: LayoutDashboard },
  { to: "/invoices", label: "Invoices", icon: FileText },
  { to: "/payments", label: "Payouts", icon: Send },
  { to: "/transactions", label: "Transactions", icon: Receipt },
  { to: "/webhooks", label: "Webhooks", icon: Link2 },
  { to: "/reports", label: "Reports", icon: BookOpen },
];

function NavItemLink({
  item,
  collapsed,
  onNavigate,
}: {
  item: NavItem;
  collapsed: boolean;
  onNavigate?: () => void;
}) {
  const Icon = item.icon;
  return (
    <NavLink
      to={item.to}
      end={item.end}
      onClick={onNavigate}
      title={collapsed ? item.label : undefined}
      className={({ isActive }) =>
        cn(
          "flex items-center gap-3 rounded-md px-3 py-2 text-nav-link transition-colors",
          collapsed && "justify-center px-2",
          isActive
            ? "bg-secondary text-primary"
            : "text-muted-foreground hover:bg-muted hover:text-foreground",
        )
      }
    >
      <Icon className="h-4 w-4 shrink-0" />
      {!collapsed && <span className="truncate">{item.label}</span>}
    </NavLink>
  );
}

function SidebarBody({
  collapsed,
  onNavigate,
  onToggleCollapse,
  showCollapseToggle,
}: {
  collapsed: boolean;
  onNavigate?: () => void;
  onToggleCollapse?: () => void;
  showCollapseToggle?: boolean;
}) {
  const { session, signOut } = useAuth();

  async function handleSignOut() {
    if (session?.sessionToken) {
      try {
        await merchantLogout(session.sessionToken);
      } catch {
        /* ignore */
      }
    }
    signOut();
  }

  return (
    <div className="flex h-full flex-col bg-background">
      <div
        className={cn(
          "flex h-16 items-center border-b border-border px-3",
          collapsed ? "justify-center" : "justify-between gap-2",
        )}
      >
        <Link
          to="/dashboard"
          onClick={onNavigate}
          className="flex min-w-0 items-center"
          aria-label="Relay home"
        >
          {collapsed ? (
            <BrandMark className="h-8 w-8" />
          ) : (
            <BrandLogo imgClassName="h-8 w-auto max-w-[140px]" />
          )}
        </Link>
        {showCollapseToggle && onToggleCollapse && (
          <Button
            type="button"
            variant="ghost"
            size="icon"
            className="hidden h-9 w-9 shrink-0 md:inline-flex"
            onClick={onToggleCollapse}
            aria-label={collapsed ? "Expand sidebar" : "Collapse sidebar"}
          >
            {collapsed ? (
              <PanelLeftOpen className="h-4 w-4" />
            ) : (
              <PanelLeftClose className="h-4 w-4" />
            )}
          </Button>
        )}
      </div>

      <nav className="flex-1 space-y-0.5 overflow-y-auto p-3">
        {merchantNav.map((item) => (
          <NavItemLink
            key={item.to}
            item={item}
            collapsed={collapsed}
            onNavigate={onNavigate}
          />
        ))}
      </nav>

      <div className="space-y-1 border-t border-border bg-background p-3">
        {session && !collapsed && (
          <p className="truncate px-3 pb-2 text-xs text-muted-foreground">
            {session.name} · {session.prefix}
          </p>
        )}

        <NavItemLink
          item={{ to: "/admin", label: "Admin", icon: Shield }}
          collapsed={collapsed}
          onNavigate={onNavigate}
        />

        <button
          type="button"
          title={collapsed ? "Sign out" : undefined}
          onClick={() => {
            void handleSignOut();
            onNavigate?.();
          }}
          className={cn(
            "flex w-full items-center gap-3 rounded-md px-3 py-2 text-nav-link text-muted-foreground transition-colors hover:bg-muted hover:text-foreground",
            collapsed && "justify-center px-2",
          )}
        >
          <LogOut className="h-4 w-4 shrink-0" />
          {!collapsed && <span>Sign out</span>}
        </button>
      </div>
    </div>
  );
}

/** Guest shell: DESIGN.md top-nav-light only — no sidebar. */
function GuestLayout() {
  return (
    <div className="min-h-screen bg-background">
      <header className="sticky top-0 z-20 flex h-16 items-center gap-4 border-b border-border bg-background px-4 md:px-6">
        <Link to="/" className="flex items-center" aria-label="Relay home">
          <BrandLogo imgClassName="h-8 w-auto" />
        </Link>
        <nav className="ml-auto flex items-center gap-2 sm:gap-3">
          <Button asChild variant="ghost" size="sm">
            <Link to="/login">Sign in</Link>
          </Button>
          <Button asChild size="sm">
            <Link to="/register">Register</Link>
          </Button>
        </nav>
      </header>
      <main className="mx-auto w-full max-w-6xl flex-1 px-4 py-6 sm:px-6 md:py-10">
        <Outlet />
      </main>
    </div>
  );
}

/** Signed-in merchant shell: retractable sidebar. */
function MerchantLayout() {
  const { session } = useAuth();
  const location = useLocation();
  const [collapsed, setCollapsed] = useState(() => {
    try {
      return localStorage.getItem(STORAGE_KEY) === "1";
    } catch {
      return false;
    }
  });
  const [mobileOpen, setMobileOpen] = useState(false);

  useEffect(() => {
    try {
      localStorage.setItem(STORAGE_KEY, collapsed ? "1" : "0");
    } catch {
      /* ignore */
    }
  }, [collapsed]);

  useEffect(() => {
    setMobileOpen(false);
  }, [location.pathname]);

  function toggleCollapse() {
    setCollapsed((v) => !v);
  }

  return (
    <div className="min-h-screen bg-background">
      <aside
        className={cn(
          "fixed inset-y-0 left-0 z-30 hidden border-r border-border bg-background transition-[width] duration-200 ease-in-out md:flex md:flex-col",
          collapsed ? "w-16" : "w-64",
        )}
      >
        <SidebarBody
          collapsed={collapsed}
          onToggleCollapse={toggleCollapse}
          showCollapseToggle
        />
      </aside>

      {mobileOpen && (
        <div className="fixed inset-0 z-40 md:hidden">
          <button
            type="button"
            className="absolute inset-0 bg-foreground/40"
            aria-label="Close menu"
            onClick={() => setMobileOpen(false)}
          />
          <aside className="absolute inset-y-0 left-0 flex w-[280px] flex-col border-r border-border bg-background shadow-none animate-in slide-in-from-left duration-200">
            <div className="absolute right-2 top-3">
              <Button
                type="button"
                variant="ghost"
                size="icon"
                className="h-9 w-9"
                onClick={() => setMobileOpen(false)}
                aria-label="Close sidebar"
              >
                <X className="h-4 w-4" />
              </Button>
            </div>
            <SidebarBody collapsed={false} onNavigate={() => setMobileOpen(false)} />
          </aside>
        </div>
      )}

      <div
        className={cn(
          "flex min-h-screen flex-col transition-[padding] duration-200 ease-in-out",
          collapsed ? "md:pl-16" : "md:pl-64",
        )}
      >
        <header className="sticky top-0 z-20 flex h-16 items-center gap-3 border-b border-border bg-background px-4 md:px-6">
          <Button
            type="button"
            variant="ghost"
            size="icon"
            className="md:hidden"
            onClick={() => setMobileOpen(true)}
            aria-label="Open sidebar"
          >
            <Menu className="h-5 w-5" />
          </Button>

          <div className="md:hidden">
            <BrandLogo imgClassName="h-7 w-auto" />
          </div>

          <div className="ml-auto flex items-center gap-2">
            {session && (
              <span className="hidden text-sm text-muted-foreground sm:inline md:hidden">
                {session.name}
              </span>
            )}
            <Button
              type="button"
              variant="ghost"
              size="icon"
              className="hidden md:inline-flex"
              onClick={toggleCollapse}
              aria-label={collapsed ? "Expand sidebar" : "Collapse sidebar"}
            >
              {collapsed ? (
                <PanelLeftOpen className="h-4 w-4" />
              ) : (
                <PanelLeftClose className="h-4 w-4" />
              )}
            </Button>
          </div>
        </header>

        <main className="mx-auto w-full max-w-6xl flex-1 px-4 py-6 sm:px-6 md:py-10">
          <Outlet />
        </main>
      </div>
    </div>
  );
}

export function AppLayout() {
  const { session } = useAuth();
  return session ? <MerchantLayout /> : <GuestLayout />;
}
