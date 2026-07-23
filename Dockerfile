# syntax=docker/dockerfile:1

# —— React SPA ——
FROM node:22-bookworm-slim AS frontend
WORKDIR /frontend
COPY frontend/package.json frontend/package-lock.json ./
RUN npm ci
COPY frontend/ ./
RUN npm run build

# —— Rust API ——
FROM rust:1-bookworm AS builder
WORKDIR /app
COPY Cargo.toml Cargo.lock build.rs ./
COPY src ./src
COPY migrations ./migrations
COPY config ./config
COPY openapi ./openapi
COPY --from=frontend /frontend/dist ./frontend/dist
# Frontend already built above; skip build.rs npm step
ENV SKIP_FRONTEND_BUILD=1
RUN cargo build --release

# —— Runtime ——
FROM debian:bookworm-slim AS runtime
RUN apt-get update \
    && apt-get install -y --no-install-recommends ca-certificates curl \
    && rm -rf /var/lib/apt/lists/* \
    && useradd --system --uid 10001 --create-home relay

WORKDIR /app
COPY --from=builder /app/target/release/payment-relay /app/payment-relay
COPY --from=frontend /frontend/dist ./frontend/dist
COPY config ./config

LABEL org.opencontainers.image.title="payment-relay" \
      org.opencontainers.image.description="Relay API + merchant/pay SPA"

ENV WALLET_SEED_DEFAULTS_PATH=config/wallet_seed_defaults.json \
    PORT=8080 \
    RUST_LOG=payment_relay=info,tower_http=info

USER relay
EXPOSE 8080
HEALTHCHECK --interval=30s --timeout=5s --start-period=20s --retries=3 \
    CMD curl -fsS "http://127.0.0.1:${PORT}/swagger-ui/" >/dev/null || exit 1

CMD ["./payment-relay"]
