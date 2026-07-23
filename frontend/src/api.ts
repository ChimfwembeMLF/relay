export class ApiError extends Error {
  status: number;
  code: string;

  constructor(status: number, code: string, message: string) {
    super(message);
    this.status = status;
    this.code = code;
  }
}

async function parseError(res: Response): Promise<never> {
  const text = await res.text();
  try {
    const body = JSON.parse(text) as { error?: string; message?: string };
    throw new ApiError(res.status, body.error ?? "error", body.message ?? text);
  } catch (e) {
    if (e instanceof ApiError) throw e;
    throw new ApiError(res.status, "error", text || res.statusText);
  }
}

export async function apiFetch<T>(
  path: string,
  options: {
    method?: string;
    apiKey?: string | null;
    sessionToken?: string | null;
    adminToken?: string | null;
    body?: unknown;
    query?: Record<string, string | number | boolean | undefined>;
  } = {},
): Promise<T> {
  const url = new URL(path, window.location.origin);
  if (options.query) {
    for (const [k, v] of Object.entries(options.query)) {
      if (v !== undefined) url.searchParams.set(k, String(v));
    }
  }

  const headers: Record<string, string> = { Accept: "application/json" };
  if (options.body !== undefined) headers["Content-Type"] = "application/json";
  if (options.apiKey) headers["X-API-Key"] = options.apiKey;
  if (options.sessionToken) headers["X-Session-Token"] = options.sessionToken;
  if (options.adminToken) headers["X-Admin-Token"] = options.adminToken;

  const res = await fetch(url.pathname + url.search, {
    method: options.method ?? (options.body ? "POST" : "GET"),
    headers,
    body: options.body !== undefined ? JSON.stringify(options.body) : undefined,
  });

  if (!res.ok) await parseError(res);

  const text = await res.text();
  if (!text) return undefined as T;
  const ct = res.headers.get("content-type") ?? "";
  if (ct.includes("application/json")) return JSON.parse(text) as T;
  return text as T;
}

/* —— Pay page —— */
export interface ProviderOption {
  value: string;
  label: string;
}

export interface PayPageData {
  reference: string;
  amount: number;
  amount_display: string;
  currency: string;
  country: string;
  description?: string | null;
  status: string;
  expires_at: string;
  payable: boolean;
  form_token?: string | null;
  idempotency_key?: string | null;
  providers: ProviderOption[];
}

export interface PaySubmitResponse {
  status: string;
  message: string;
  amount_display: string;
  reference: string;
}

export function fetchPayPage(reference: string): Promise<PayPageData> {
  return apiFetch(`/api/pay/${encodeURIComponent(reference)}`).catch((e) => {
    if (e instanceof ApiError && e.status === 404) throw new Error("not_found");
    throw new Error("load_failed");
  });
}

export function submitPayPage(
  reference: string,
  body: {
    phone: string;
    provider: string;
    idempotency_key: string;
    form_token: string;
  },
): Promise<PaySubmitResponse> {
  return apiFetch(`/api/pay/${encodeURIComponent(reference)}`, { method: "POST", body });
}

/* —— Merchant —— */
export interface CreateSystemResponse {
  id: string;
  name: string;
  prefix: string;
  username: string;
  api_key: string;
  session_token: string;
  wallets_seeded: number;
}

export interface SystemPublic {
  id: string;
  name: string;
  prefix: string;
  enabled_countries: string[];
  webhook_url?: string | null;
}

export interface Wallet {
  id: string;
  system_id: string;
  country: string;
  currency: string;
  balance: number;
}

export interface Invoice {
  id: string;
  reference: string;
  system_id: string;
  amount: number;
  currency: string;
  country: string;
  status: string;
  description?: string | null;
  expires_at: string;
  paid_at?: string | null;
  transaction_id?: string | null;
  qr_url: string;
  qr_code_png_base64: string;
}

export interface Transaction {
  id: string;
  system_id: string;
  external_id: string;
  amount: number;
  currency: string;
  country: string;
  status: string;
  gateway?: string;
  direction?: string;
  created_at?: string;
  gateway_reference?: string | null;
  error?: string | null;
}

export interface ProcessPaymentResponse {
  id: string;
  external_id: string;
  status: string;
  gateway_reference?: string | null;
  error?: string | null;
}

export interface ReportSummary {
  from: string;
  to: string;
  total_count: number;
  total_amount: number;
  by_status: Record<string, { count: number; amount: number }>;
}

export interface WalletsReport {
  from: string;
  to: string;
  wallets: Array<{
    country: string;
    currency: string;
    current_balance: number;
    period_deposits: number;
    period_payouts: number;
    net_change: number;
  }>;
}

export function registerSystem(body: {
  name: string;
  prefix: string;
  username: string;
  password: string;
  webhook_url?: string;
  enabled_countries?: string[];
}): Promise<CreateSystemResponse> {
  return apiFetch("/systems", { method: "POST", body });
}

export interface MerchantLoginResponse {
  token: string;
  username: string;
  expires_at: string;
  system_id: string;
  name: string;
  prefix: string;
}

export function merchantLogin(
  username: string,
  password: string,
): Promise<MerchantLoginResponse> {
  return apiFetch("/auth/login", { method: "POST", body: { username, password } });
}

export function merchantLogout(sessionToken: string): Promise<void> {
  return apiFetch("/auth/logout", { method: "POST", sessionToken });
}

export function getSystem(id: string): Promise<SystemPublic> {
  return apiFetch(`/systems/${id}`);
}

export function listWallets(systemId: string, sessionToken: string): Promise<Wallet[]> {
  return apiFetch(`/wallets/${systemId}`, { sessionToken });
}

export function listInvoices(
  sessionToken: string,
  params?: { status?: string; limit?: number },
): Promise<Invoice[]> {
  return apiFetch("/invoices", { sessionToken, query: params });
}

export function createInvoice(
  sessionToken: string,
  body: {
    amount: number;
    currency: string;
    country: string;
    description?: string;
    expires_in_hours?: number;
  },
): Promise<Invoice> {
  return apiFetch("/invoices", { method: "POST", sessionToken, body });
}

export function getInvoice(sessionToken: string, reference: string): Promise<Invoice> {
  return apiFetch(`/invoices/reference/${encodeURIComponent(reference)}`, { sessionToken });
}

export function cancelInvoice(sessionToken: string, id: string): Promise<void> {
  return apiFetch(`/invoices/${id}/cancel`, { method: "POST", sessionToken });
}

export function listTransactions(
  systemId: string,
  sessionToken: string,
  params?: { external_id?: string; limit?: number },
): Promise<Transaction[]> {
  return apiFetch(`/transactions/${systemId}`, { sessionToken, query: params });
}

export function processPayment(
  sessionToken: string,
  body: {
    system_id: string;
    external_id: string;
    amount: number;
    currency: string;
    country: string;
    payment_method: { type: string; details: Record<string, unknown> };
    idempotency_key: string;
  },
): Promise<ProcessPaymentResponse> {
  return apiFetch("/payments", { method: "POST", sessionToken, body });
}

export function reportTransactions(
  sessionToken: string,
  from: string,
  to: string,
): Promise<ReportSummary> {
  return apiFetch("/reports/transactions", { sessionToken, query: { from, to } });
}

export function reportWallets(sessionToken: string, from: string, to: string): Promise<WalletsReport> {
  return apiFetch("/reports/wallets", { sessionToken, query: { from, to } });
}

export function reportInvoices(sessionToken: string, from: string, to: string): Promise<ReportSummary> {
  return apiFetch("/reports/invoices", { sessionToken, query: { from, to } });
}

/* —— Webhooks (tenant) —— */
export interface WebhookEndpoint {
  id: string;
  system_id: string;
  url: string;
  label?: string | null;
  enabled: boolean;
  created_at: string;
  updated_at: string;
}

export function listWebhookEndpoints(sessionToken: string): Promise<WebhookEndpoint[]> {
  return apiFetch("/webhook-endpoints", { sessionToken });
}

export function createWebhookEndpoint(
  sessionToken: string,
  body: { url: string; label?: string },
): Promise<WebhookEndpoint> {
  return apiFetch("/webhook-endpoints", { method: "POST", sessionToken, body });
}

export function updateWebhookEndpoint(
  sessionToken: string,
  id: string,
  body: { url?: string; label?: string; enabled?: boolean },
): Promise<WebhookEndpoint> {
  return apiFetch(`/webhook-endpoints/${id}`, { method: "PATCH", sessionToken, body });
}

export function deleteWebhookEndpoint(sessionToken: string, id: string): Promise<void> {
  return apiFetch(`/webhook-endpoints/${id}`, { method: "DELETE", sessionToken });
}

/* —— Admin backoffice —— */
export interface AdminLoginResponse {
  token: string;
  username: string;
  expires_at: string;
}

export interface AdminSystemSummary {
  id: string;
  name: string;
  prefix: string;
  enabled_countries: string[];
  webhook_url?: string | null;
  webhook_endpoints: number;
  created_at: string;
}

export interface AdminSystemDetail {
  system: SystemPublic;
  wallets: Wallet[];
  webhook_endpoints: WebhookEndpoint[];
}

export function adminLogin(
  username: string,
  password: string,
): Promise<AdminLoginResponse> {
  return apiFetch("/admin/login", { method: "POST", body: { username, password } });
}

export function adminLogout(adminToken: string): Promise<void> {
  return apiFetch("/admin/logout", { method: "POST", adminToken });
}

export function adminListSystems(adminToken: string): Promise<AdminSystemSummary[]> {
  return apiFetch("/admin/systems", { adminToken });
}

export function adminGetSystem(adminToken: string, id: string): Promise<AdminSystemDetail> {
  return apiFetch(`/admin/systems/${id}`, { adminToken });
}

export function formatMoney(amount: number, currency: string): string {
  const major = amount / 100;
  return `${currency} ${major.toLocaleString(undefined, {
    minimumFractionDigits: 2,
    maximumFractionDigits: 2,
  })}`;
}
