from __future__ import annotations

import json
import urllib.error
import urllib.parse
import urllib.request
from typing import Any

from relay_sdk.errors import RelayError


class _Resource:
    def __init__(self, client: "RelayClient"):
        self._client = client


class SystemsResource(_Resource):
    def create(self, **body: Any) -> dict[str, Any]:
        return self._client.request("POST", "/systems", auth=False, body=body)

    def get(self, system_id: str) -> dict[str, Any]:
        return self._client.request("GET", f"/systems/{system_id}", auth=False)


class PaymentsResource(_Resource):
    def process(self, **body: Any) -> dict[str, Any]:
        return self._client.request("POST", "/payments", body=body)

    def get(self, payment_id: str) -> dict[str, Any]:
        return self._client.request("GET", f"/payments/{payment_id}")


class WalletsResource(_Resource):
    def list(self, system_id: str) -> list[dict[str, Any]]:
        return self._client.request("GET", f"/wallets/{system_id}")


class TransactionsResource(_Resource):
    def list(self, system_id: str, **params: Any) -> list[dict[str, Any]]:
        return self._client.request("GET", f"/transactions/{system_id}", params=params)


class InvoicesResource(_Resource):
    def create(self, **body: Any) -> dict[str, Any]:
        return self._client.request("POST", "/invoices", body=body)

    def list(self, **params: Any) -> list[dict[str, Any]]:
        return self._client.request("GET", "/invoices", params=params)

    def get(self, reference: str) -> dict[str, Any]:
        return self._client.request("GET", f"/invoices/reference/{reference}")

    def collect(self, invoice_id: str, **body: Any) -> dict[str, Any]:
        return self._client.request("POST", f"/invoices/{invoice_id}/collect", body=body)

    def cancel(self, invoice_id: str) -> None:
        self._client.request("POST", f"/invoices/{invoice_id}/cancel")


class ReportsResource(_Resource):
    def transactions(
        self,
        *,
        from_date: str,
        to_date: str,
        format: str = "json",
        status: str | None = None,
        detail: bool | None = None,
    ) -> dict[str, Any] | str:
        return self._client.request(
            "GET",
            "/reports/transactions",
            params={
                "from": from_date,
                "to": to_date,
                "format": format,
                "status": status,
                "detail": detail,
            },
        )

    def wallets(
        self,
        *,
        from_date: str,
        to_date: str,
        format: str = "json",
    ) -> dict[str, Any] | str:
        return self._client.request(
            "GET",
            "/reports/wallets",
            params={"from": from_date, "to": to_date, "format": format},
        )

    def invoices(
        self,
        *,
        from_date: str,
        to_date: str,
        format: str = "json",
        status: str | None = None,
    ) -> dict[str, Any] | str:
        return self._client.request(
            "GET",
            "/reports/invoices",
            params={
                "from": from_date,
                "to": to_date,
                "format": format,
                "status": status,
            },
        )


class PayPageResource(_Resource):
    def get(self, reference: str) -> dict[str, Any]:
        return self._client.request("GET", f"/api/pay/{reference}", auth=False)

    def submit(self, reference: str, **body: Any) -> dict[str, Any]:
        return self._client.request("POST", f"/api/pay/{reference}", auth=False, body=body)


class RelayClient:
    """Payment Relay merchant API client."""

    def __init__(self, base_url: str, api_key: str | None = None, timeout: float = 30.0):
        self.base_url = base_url.rstrip("/")
        self.api_key = api_key
        self.timeout = timeout

        self.systems = SystemsResource(self)
        self.payments = PaymentsResource(self)
        self.wallets = WalletsResource(self)
        self.transactions = TransactionsResource(self)
        self.invoices = InvoicesResource(self)
        self.reports = ReportsResource(self)
        self.pay = PayPageResource(self)

    def request(
        self,
        method: str,
        path: str,
        *,
        auth: bool = True,
        body: dict[str, Any] | None = None,
        params: dict[str, Any] | None = None,
    ) -> Any:
        if auth and not self.api_key:
            raise RelayError(401, "missing_api_key", "API key required for this request")

        url = f"{self.base_url}{path}"
        if params:
            filtered = {k: v for k, v in params.items() if v is not None}
            if filtered:
                url = f"{url}?{urllib.parse.urlencode(filtered)}"

        data = None
        headers = {"Accept": "application/json"}
        if body is not None:
            data = json.dumps(body).encode("utf-8")
            headers["Content-Type"] = "application/json"
        if auth and self.api_key:
            headers["X-API-Key"] = self.api_key

        req = urllib.request.Request(url, data=data, headers=headers, method=method)

        try:
            with urllib.request.urlopen(req, timeout=self.timeout) as resp:
                raw = resp.read().decode("utf-8")
                content_type = resp.headers.get("Content-Type", "")
                if not raw:
                    return None
                if "application/json" in content_type:
                    return json.loads(raw)
                return raw
        except urllib.error.HTTPError as exc:
            body_text = exc.read().decode("utf-8") if exc.fp else ""
            raise RelayError.from_response(exc.code, body_text) from exc
