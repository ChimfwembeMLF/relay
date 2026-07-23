import { RelayError } from "./errors.js";
import type {
  CollectInvoiceRequest,
  CreateInvoiceRequest,
  CreateSystemRequest,
  CreateSystemResponse,
  InvoiceResponse,
  ListInvoicesParams,
  ListTransactionsParams,
  PayPageResponse,
  PaySubmitRequest,
  PaySubmitResponse,
  ProcessPaymentRequest,
  ProcessPaymentResponse,
  RelayClientConfig,
  ReportParams,
  ReportSummary,
  SystemPublic,
  Transaction,
  Wallet,
  WalletsReport,
} from "./types.js";

type HttpMethod = "GET" | "POST";

export class RelayClient {
  private readonly baseUrl: string;
  private readonly apiKey?: string;
  private readonly fetchImpl: typeof fetch;

  readonly systems: SystemsResource;
  readonly payments: PaymentsResource;
  readonly wallets: WalletsResource;
  readonly transactions: TransactionsResource;
  readonly invoices: InvoicesResource;
  readonly reports: ReportsResource;
  readonly pay: PayPageResource;

  constructor(config: RelayClientConfig) {
    this.baseUrl = config.baseUrl.replace(/\/$/, "");
    this.apiKey = config.apiKey;
    this.fetchImpl = config.fetch ?? fetch;

    this.systems = new SystemsResource(this);
    this.payments = new PaymentsResource(this);
    this.wallets = new WalletsResource(this);
    this.transactions = new TransactionsResource(this);
    this.invoices = new InvoicesResource(this);
    this.reports = new ReportsResource(this);
    this.pay = new PayPageResource(this);
  }

  async request<T>(
    method: HttpMethod,
    path: string,
    options: {
      auth?: boolean;
      body?: unknown;
      query?: Record<string, string | number | boolean | undefined> | object;
    } = {},
  ): Promise<T> {
    const { auth = true, body, query } = options;
    const url = new URL(path, this.baseUrl);

    if (query) {
      for (const [key, value] of Object.entries(query as Record<string, unknown>)) {
        if (value !== undefined) {
          url.searchParams.set(key, String(value));
        }
      }
    }

    const headers: Record<string, string> = {
      Accept: "application/json",
    };

    if (body !== undefined) {
      headers["Content-Type"] = "application/json";
    }

    if (auth) {
      if (!this.apiKey) {
        throw new RelayError(401, "missing_api_key", "API key required for this request");
      }
      headers["X-API-Key"] = this.apiKey;
    }

    const response = await this.fetchImpl(url, {
      method,
      headers,
      body: body !== undefined ? JSON.stringify(body) : undefined,
    });

    const text = await response.text();

    if (!response.ok) {
      throw await RelayError.fromResponse(response.status, text);
    }

    const contentType = response.headers.get("content-type") ?? "";
    if (contentType.includes("application/json")) {
      return JSON.parse(text) as T;
    }

    return text as T;
  }
}

class SystemsResource {
  constructor(private readonly client: RelayClient) {}

  create(body: CreateSystemRequest): Promise<CreateSystemResponse> {
    return this.client.request("POST", "/systems", { auth: false, body });
  }

  get(id: string): Promise<SystemPublic> {
    return this.client.request("GET", `/systems/${id}`, { auth: false });
  }
}

class PaymentsResource {
  constructor(private readonly client: RelayClient) {}

  process(body: ProcessPaymentRequest): Promise<ProcessPaymentResponse> {
    return this.client.request("POST", "/payments", { body });
  }

  get(id: string): Promise<ProcessPaymentResponse> {
    return this.client.request("GET", `/payments/${id}`);
  }
}

class WalletsResource {
  constructor(private readonly client: RelayClient) {}

  list(systemId: string): Promise<Wallet[]> {
    return this.client.request("GET", `/wallets/${systemId}`);
  }
}

class TransactionsResource {
  constructor(private readonly client: RelayClient) {}

  list(systemId: string, params: ListTransactionsParams = {}): Promise<Transaction[]> {
    return this.client.request("GET", `/transactions/${systemId}`, { query: params });
  }
}

class InvoicesResource {
  constructor(private readonly client: RelayClient) {}

  create(body: CreateInvoiceRequest): Promise<InvoiceResponse> {
    return this.client.request("POST", "/invoices", { body });
  }

  list(params: ListInvoicesParams = {}): Promise<InvoiceResponse[]> {
    return this.client.request("GET", "/invoices", { query: params });
  }

  get(reference: string): Promise<InvoiceResponse> {
    return this.client.request("GET", `/invoices/reference/${reference}`);
  }

  collect(id: string, body: CollectInvoiceRequest): Promise<ProcessPaymentResponse> {
    return this.client.request("POST", `/invoices/${id}/collect`, { body });
  }

  cancel(id: string): Promise<void> {
    return this.client.request("POST", `/invoices/${id}/cancel`);
  }
}

class ReportsResource {
  constructor(private readonly client: RelayClient) {}

  transactions(params: ReportParams): Promise<ReportSummary | string> {
    return this.fetchReport("/reports/transactions", params);
  }

  wallets(params: ReportParams): Promise<WalletsReport | string> {
    return this.fetchReport("/reports/wallets", params);
  }

  invoices(params: ReportParams): Promise<ReportSummary | string> {
    return this.fetchReport("/reports/invoices", params);
  }

  private fetchReport<T>(path: string, params: ReportParams): Promise<T | string> {
    const format = params.format ?? "json";
    return this.client.request<T | string>("GET", path, {
      query: {
        from: params.from,
        to: params.to,
        format,
        status: params.status,
        detail: params.detail,
      },
    }).then((result) => {
      if (format === "csv" && typeof result === "string") {
        return result;
      }
      return result;
    });
  }
}

class PayPageResource {
  constructor(private readonly client: RelayClient) {}

  get(reference: string): Promise<PayPageResponse> {
    return this.client.request("GET", `/api/pay/${reference}`, { auth: false });
  }

  submit(reference: string, body: PaySubmitRequest): Promise<PaySubmitResponse> {
    return this.client.request("POST", `/api/pay/${reference}`, { auth: false, body });
  }
}
