import type { ApiErrorBody } from "./types.js";

export class RelayError extends Error {
  readonly status: number;
  readonly code: string;

  constructor(status: number, code: string, message: string) {
    super(message);
    this.name = "RelayError";
    this.status = status;
    this.code = code;
  }

  static async fromResponse(status: number, body: string): Promise<RelayError> {
    try {
      const parsed = JSON.parse(body) as ApiErrorBody;
      return new RelayError(status, parsed.error ?? "unknown_error", parsed.message ?? body);
    } catch {
      return new RelayError(status, "unknown_error", body || `HTTP ${status}`);
    }
  }
}
