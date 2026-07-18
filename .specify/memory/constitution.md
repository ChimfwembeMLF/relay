<!--
Sync Impact Report
- Version change: (unversioned template) → 1.0.0
- Modified principles: All placeholders replaced with Payment Relay principles
- Added sections: Security & Compliance, Development Workflow
- Removed sections: None
- Templates requiring updates:
  - .specify/templates/plan-template.md ✅ aligned (Constitution Check gate present)
  - .specify/templates/spec-template.md ✅ aligned
  - .specify/templates/tasks-template.md ✅ aligned
- Follow-up TODOs: None
-->

# Payment Relay Constitution

## Core Principles

### I. Internal-First Simplicity

The system MUST serve internal systems only. Multi-tenant SaaS features (customer dashboards,
user management, billing plans) are out of scope unless explicitly added by amendment. Every
feature MUST justify its complexity against the goal of a lightweight payment relay.

### II. System Isolation

Each registered internal system MUST have its own wallet(s), external ID prefix, API key, and
webhook configuration. Data and payment flows MUST NOT leak across system boundaries. Country-
specific wallets MUST remain scoped to the owning system.

### III. Idempotent Payment Processing (NON-NEGOTIABLE)

All payment operations MUST be idempotent. Duplicate requests with the same idempotency key
MUST return the original result without creating duplicate charges or ledger entries. Retries
MUST be safe and observable.

### IV. Reliable External Relay

Payment requests MUST be forwarded to configured external gateways with automatic retry (minimum
three attempts on transient failure). Final status MUST be persisted and communicated to the
originating system via webhook when configured.

### V. Observability & Auditability

All payment lifecycle events MUST be logged with correlation identifiers (system ID, external
ID, idempotency key). Transaction history MUST be queryable for reconciliation and support.
Structured logging is required for production operation.

## Security & Compliance Requirements

- API keys MUST be generated securely, stored hashed at rest, and transmitted only over HTTPS.
- Secrets (gateway keys, webhook signing secrets) MUST NOT be committed to version control.
- Webhook payloads MUST be signed or otherwise verifiable by receiving systems where supported.
- Amounts MUST be stored and transmitted as integer minor units (e.g., cents) to avoid floating-
  point errors.
- PCI-sensitive card data MUST NOT be stored by the relay; tokenized payment methods only.

## Development Workflow

- Spec-driven development: features start in `specs/` before implementation planning.
- Constitution Check MUST pass in plan.md before Phase 0 research and again after Phase 1 design.
- Integration tests MUST cover payment flows, idempotency, wallet updates, and webhook delivery.
- Database schema changes MUST be versioned via migrations.
- Breaking API changes require a version bump and migration notes for internal consumers.

## Governance

This constitution supersedes ad-hoc implementation choices. Amendments require updating this
file, bumping the version per semantic versioning, and verifying dependent templates remain
aligned. All feature plans and pull requests MUST verify compliance with Core Principles and
Security requirements. Complexity beyond these principles MUST be documented with explicit
rationale in the feature plan.

**Version**: 1.0.0 | **Ratified**: 2026-07-18 | **Last Amended**: 2026-07-18
