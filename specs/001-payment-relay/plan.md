# Implementation Plan: Internal Payment Relay

**Branch**: `001-payment-relay` | **Date**: 2026-07-18 | **Spec**: [spec.md](./spec.md)

**Input**: Feature specification from `/specs/001-payment-relay/spec.md`

**Note**: This template is filled in by the `/speckit-plan` command; its definition describes the execution workflow.

## Summary

Build a single Rust HTTP service that registers internal systems, authenticates them via API keys,
processes idempotent outbound payments through a pluggable gateway adapter (pawaPay v2 primary),
maintains per-system country/currency wallets, and notifies consumers via signed webhooks.
PostgreSQL stores all state; sqlx migrations version the schema.

## Technical Context

**Language/Version**: Rust 1.75+ (2021 edition)

**Primary Dependencies**: axum 0.7, tokio, sqlx 0.7, reqwest 0.11, serde/serde_json, uuid,
chrono, tracing, sha2, tower-http (CORS)

**Storage**: PostgreSQL 15+ (systems, wallets, transactions, webhook_delivery_attempts)

**Testing**: cargo test, integration tests with testcontainers or docker-compose Postgres;
HTTP contract tests against axum TestClient; gateway mocked via trait

**Target Platform**: Linux server (Docker); local dev via docker-compose

**Project Type**: web-service (REST API, no UI)

**Performance Goals**: 100 concurrent payment requests; p95 relay latency < 2s excluding gateway
time; webhook dispatch started within 5s of terminal status

**Constraints**: Integer minor-unit amounts only; API keys hashed at rest; no PAN storage;
3 gateway retries on transient errors; single-region v1

**Scale/Scope**: Tens of internal systems, thousands of transactions/day, 3 core tables + webhook
audit table, 6 REST endpoints, 1 primary gateway adapter (pawaPay)

## Constitution Check

*GATE: Must pass before Phase 0 research. Re-check after Phase 1 design.*

| Principle | Pre-Design | Post-Design |
|-----------|------------|-------------|
| I. Internal-First Simplicity | PASS — single binary, no SaaS UI | PASS — REST-only scope |
| II. System Isolation | PASS — API key + system_id scoping on all queries | PASS — DB FK + auth middleware enforce isolation |
| III. Idempotent Payments | PASS — unique idempotency_key constraint | PASS — lookup-before-create + body hash comparison |
| IV. Reliable External Relay | PASS — retry policy in gateway adapter | PASS — 3 retries + persisted terminal status |
| V. Observability | PASS — structured tracing fields defined | PASS — correlation IDs in logs + webhook audit table |
| Security (API keys hashed, HTTPS, no PAN) | PASS | PASS — HMAC webhook signing, env-based secrets |

**Gate result**: PASS — proceed to implementation tasks.

## Project Structure

### Documentation (this feature)

```text
specs/001-payment-relay/
├── plan.md              # This file
├── research.md          # Phase 0 output
├── data-model.md        # Phase 1 output
├── quickstart.md        # Phase 1 output
├── contracts/           # Phase 1 output
│   ├── openapi.yaml
│   └── webhook-payload.json
└── tasks.md             # Phase 2 output (/speckit-tasks)
```

### Source Code (repository root)

```text
payment-relay/
├── Cargo.toml
├── .env.example
├── docker-compose.yml
├── migrations/
│   └── 001_initial.sql
├── src/
│   ├── main.rs
│   ├── config.rs
│   ├── error.rs
│   ├── models.rs
│   ├── api/
│   │   ├── mod.rs
│   │   ├── routes.rs
│   │   ├── systems.rs
│   │   ├── payments.rs
│   │   └── wallets.rs
│   ├── auth.rs
│   ├── db/
│   │   ├── mod.rs
│   │   └── queries.rs
│   ├── gateway/
│   │   ├── mod.rs
│   │   ├── traits.rs
│   │   └── pawapay.rs
│   └── webhook/
│       ├── mod.rs
│       └── sender.rs
└── tests/
    ├── integration/
    │   ├── systems_test.rs
    │   ├── payments_test.rs
    │   └── idempotency_test.rs
    └── common/
        └── mod.rs
```

**Structure Decision**: Single Rust binary (Option 1). Gateway adapters live behind a trait in
`src/gateway/` so pawaPay is v1 default while Stripe/Adyen remain future adapters without
 restructuring.

## Complexity Tracking

> No constitution violations requiring justification.

| Violation | Why Needed | Simpler Alternative Rejected Because |
|-----------|------------|-------------------------------------|
| — | — | — |
