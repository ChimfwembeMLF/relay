export interface RelayClientConfig {
  baseUrl: string;
  apiKey?: string;
  fetch?: typeof fetch;
}

export interface ApiErrorBody {
  error: string;
  message: string;
}

export interface WalletSeedOverride {
  country: string;
  currency: string;
  amount: number;
}

export interface CreateSystemRequest {
  name: string;
  prefix: string;
  enabled_countries: string[];
  webhook_url?: string;
  wallet_seeds?: WalletSeedOverride[];
}

export interface CreateSystemResponse {
  id: string;
  name: string;
  prefix: string;
  api_key: string;
  wallets_seeded: number;
}

export interface SystemPublic {
  id: string;
  name: string;
  prefix: string;
  enabled_countries: string[];
  webhook_url?: string | null;
}

export interface PaymentMethod {
  type: string;
  details: Record<string, unknown>;
}

export interface ProcessPaymentRequest {
  system_id: string;
  external_id: string;
  amount: number;
  currency: string;
  country: string;
  payment_method: PaymentMethod;
  idempotency_key: string;
}

export interface ProcessPaymentResponse {
  id: string;
  external_id: string;
  status: string;
  gateway_reference?: string | null;
  error?: string | null;
}

export interface Wallet {
  id: string;
  system_id: string;
  country: string;
  currency: string;
  balance: number;
  created_at?: string;
  updated_at?: string;
}

export interface Transaction {
  id: string;
  system_id: string;
  wallet_id: string;
  external_id: string;
  idempotency_key: string;
  amount: number;
  currency: string;
  country: string;
  status: string;
  gateway: string;
  gateway_reference?: string | null;
  gateway_status?: string | null;
  error?: string | null;
  invoice_id?: string | null;
  direction: string;
  created_at: string;
  updated_at: string;
}

export interface CreateInvoiceRequest {
  amount: number;
  currency: string;
  country: string;
  description?: string;
  expires_in_hours?: number;
}

export interface InvoiceResponse {
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

export interface CollectInvoiceRequest {
  payment_method: PaymentMethod;
  idempotency_key: string;
}

export interface StatusSummary {
  count: number;
  amount: number;
}

export interface ReportSummary {
  from: string;
  to: string;
  total_count: number;
  total_amount: number;
  by_status: Record<string, StatusSummary>;
}

export interface WalletReportRow {
  country: string;
  currency: string;
  current_balance: number;
  period_deposits: number;
  period_payouts: number;
  net_change: number;
}

export interface WalletsReport {
  from: string;
  to: string;
  wallets: WalletReportRow[];
}

export interface ProviderOption {
  value: string;
  label: string;
}

export interface PayPageResponse {
  reference: string;
  amount: number;
  amount_display: string;
  currency: string;
  country: string;
  description?: string | null;
  status: string;
  expires_at: string;
  payable: boolean;
  form_token?: string;
  idempotency_key?: string;
  providers: ProviderOption[];
}

export interface PaySubmitRequest {
  phone: string;
  provider: string;
  idempotency_key: string;
  form_token: string;
}

export interface PaySubmitResponse {
  status: string;
  message: string;
  amount_display: string;
  reference: string;
}

export interface PaymentStatusChangedWebhook {
  event: "payment.status_changed";
  payment_id: string;
  system_id: string;
  external_id: string;
  status: "completed" | "failed";
  amount: number;
  currency: string;
  country: string;
  gateway_reference?: string | null;
  error?: string | null;
  timestamp: string;
}

export interface InvoicePaidWebhook {
  event: "invoice.paid";
  invoice_id: string;
  system_id: string;
  reference: string;
  amount: number;
  currency: string;
  country: string;
  status: "paid";
  transaction_id: string;
  timestamp: string;
}

export type RelayWebhookEvent = PaymentStatusChangedWebhook | InvoicePaidWebhook;

export interface ListInvoicesParams {
  status?: "open" | "paid" | "expired" | "cancelled";
  limit?: number;
}

export interface ListTransactionsParams {
  external_id?: string;
  limit?: number;
}

export interface ReportParams {
  from: string;
  to: string;
  format?: "json" | "csv";
  status?: string;
  detail?: boolean;
}
