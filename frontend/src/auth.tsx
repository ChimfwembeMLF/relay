import { createContext, useContext, useMemo, useState, type ReactNode } from "react";

const STORAGE_KEY = "relay.session";

export interface Session {
  systemId: string;
  sessionToken: string;
  username: string;
  name: string;
  prefix: string;
  /** Present only right after registration — copy for SDKs. */
  apiKey?: string;
}

interface AuthContextValue {
  session: Session | null;
  signIn: (session: Session) => void;
  signOut: () => void;
}

const AuthContext = createContext<AuthContextValue | null>(null);

function loadSession(): Session | null {
  try {
    const raw = localStorage.getItem(STORAGE_KEY);
    if (!raw) return null;
    const parsed = JSON.parse(raw) as Session & { apiKey?: string };
    // Migrate legacy API-key sessions — force re-login
    if (!parsed.sessionToken) return null;
    return parsed;
  } catch {
    return null;
  }
}

export function AuthProvider({ children }: { children: ReactNode }) {
  const [session, setSession] = useState<Session | null>(() => loadSession());

  const value = useMemo<AuthContextValue>(
    () => ({
      session,
      signIn: (next) => {
        localStorage.setItem(STORAGE_KEY, JSON.stringify(next));
        setSession(next);
      },
      signOut: () => {
        localStorage.removeItem(STORAGE_KEY);
        setSession(null);
      },
    }),
    [session],
  );

  return <AuthContext.Provider value={value}>{children}</AuthContext.Provider>;
}

export function useAuth(): AuthContextValue {
  const ctx = useContext(AuthContext);
  if (!ctx) throw new Error("useAuth must be used within AuthProvider");
  return ctx;
}
