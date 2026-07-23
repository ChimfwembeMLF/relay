class RelayError(Exception):
    def __init__(self, status: int, code: str, message: str):
        super().__init__(message)
        self.status = status
        self.code = code
        self.message = message

    @classmethod
    def from_response(cls, status: int, body: str) -> "RelayError":
        try:
            import json

            parsed = json.loads(body)
            return cls(
                status,
                parsed.get("error", "unknown_error"),
                parsed.get("message", body),
            )
        except Exception:
            return cls(status, "unknown_error", body or f"HTTP {status}")
