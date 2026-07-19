# Payment Relay System - Simple Internal System Requirements

## Implementation (Rust)

The spec-driven implementation lives in this repository:

```bash
cp .env.example .env
docker compose up -d postgres   # or use local Postgres
cargo run
```

- **Feature 001 (payments)**: `specs/001-payment-relay/`
- **Feature 003 (invoice pay page)**: `specs/003-invoice-pay-page/`
- **API contracts**: `specs/001-payment-relay/contracts/openapi.yaml`, `specs/002-wallet-invoices-reports/contracts/openapi.yaml`
- **Validation guides**: `specs/001-payment-relay/quickstart.md`, `specs/002-wallet-invoices-reports/quickstart.md`
- **Tests**: `DATABASE_URL=postgres://relay:relay@localhost:5432/payment_relay cargo test`

### Feature 002 endpoints

| Endpoint | Description |
|----------|-------------|
| `POST /systems` | Registers a system and auto-seeds wallets from `config/wallet_seed_defaults.json` (optional `wallet_seeds` overrides) |
| `POST /invoices` | Creates a pay-in invoice with QR URL and base64 PNG |
| `GET /invoices` | Lists invoices (optional `?status=open`) |
| `GET /invoices/{reference}` | Lookup invoice by reference |
| `POST /invoices/{id}/collect` | Collect payment via pawaPay deposit; credits wallet |
| `POST /invoices/{id}/cancel` | Cancel an open invoice |
| `GET /reports/transactions` | Transaction summary by date range (`?format=csv&detail=true` for CSV) |
| `GET /reports/wallets` | Wallet balances and period deltas (`?format=csv`) |
| `GET /reports/invoices` | Invoice summary by date range (`?format=csv`) |
| `GET /pay/{reference}` | Public HTML pay page (from invoice QR links) |
| `POST /pay/{reference}` | Submit mobile-money payment from pay page |

Configure wallet seed defaults via `WALLET_SEED_DEFAULTS_PATH` or `WALLET_SEED_DEFAULTS_JSON`, set `INVOICE_PAY_BASE_URL` for QR payment links, and `PAY_PAGE_RATE_LIMIT` for POST rate limiting.

---

Got it! You want a **lightweight internal payment relay** for your own systems, not a multi-tenant SaaS. Let me strip it down to **just what you need**.

---

## 🎯 Core Concept

**One central payment relay** that your internal systems call to process payments. Each system gets its own:
- ✅ Wallet (per country)
- ✅ External ID prefix
- ✅ Configurable webhooks

**NO**:
- ❌ Multi-tenancy complexity
- ❌ Customer dashboards
- ❌ User management
- ❌ Billing/plans
- ❌ SaaS features

---

## 📋 Simplified Requirements

### 1. System Registration (Internal)

| Requirement | Description |
|-------------|-------------|
| **System ID** | UUID (auto-generated) |
| **System Name** | Human-readable name (e.g., "E-Commerce", "Subscription") |
| **System Prefix** | Configurable prefix for external IDs (e.g., "ECO", "SUB") |
| **Enabled Countries** | Which countries this system operates in |
| **Webhook URL** | Where to send payment notifications |
| **API Key** | Auto-generated for authentication |

```json
// Example system config
{
  "id": "550e8400-e29b-41d4-a716-446655440000",
  "name": "E-Commerce Platform",
  "prefix": "ECO",
  "enabled_countries": ["US", "GB", "CA"],
  "webhook_url": "https://api.myapp.com/payments/webhook",
  "api_key": "sk_live_abc123..."
}
```

### 2. Wallet System

| Requirement | Description |
|-------------|-------------|
| **Auto-create** | Wallet automatically created when first payment is made |
| **Country-specific** | Separate wallet per country |
| **Currency** | Each wallet has a base currency |
| **Balance** | Available balance in cents (integer) |
| **Transactions** | Simple transaction history |

```sql
-- Simple wallet table
CREATE TABLE wallets (
    id UUID PRIMARY KEY,
    system_id UUID NOT NULL,
    country TEXT NOT NULL,
    currency TEXT NOT NULL DEFAULT 'USD',
    balance BIGINT NOT NULL DEFAULT 0, -- in cents
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    UNIQUE(system_id, country, currency)
);
```

### 3. Payment Processing

| Requirement | Description |
|-------------|-------------|
| **Simple API** | One endpoint to process payments |
| **Relay Logic** | Forward to configured payment gateway |
| **Retry** | Automatic retry on failure (3 attempts) |
| **Idempotency** | Prevent duplicate payments |
| **Webhooks** | Notify system when payment completes |

```rust
// Simple payment request
struct PaymentRequest {
    system_id: UUID,
    external_id: String,     // System's own ID
    amount: i64,            // In cents
    currency: String,        // "USD"
    country: String,        // "US"
    payment_method: PaymentMethod, // Card, Bank, etc.
    idempotency_key: String,
}

// Simple payment response
struct PaymentResponse {
    id: UUID,
    status: PaymentStatus, // "completed", "failed", "pending"
    gateway_reference: String,
    external_id: String,
}
```

### 4. External ID Format

```
{PREFIX}_{SYSTEM_ID_SHORT}_{TIMESTAMP}_{RANDOM}
Example: ECO_550e_20240715_ABC123

Where:
- ECO = System prefix
- 550e = First 4 chars of system UUID
- 20240715 = Date (YYYYMMDD)
- ABC123 = Random alphanumeric
```

---

## 🏗️ Simplified Architecture

```
┌─────────────────────────────────────────────────┐
│         Your Internal Systems                   │
│  ┌──────────┐  ┌──────────┐  ┌──────────┐     │
│  │ E-Commerce│  │Subscript.│  │  Mobile  │     │
│  └──────────┘  └──────────┘  └──────────┘     │
└────────────────────┬────────────────────────────┘
                     │ HTTPS (API Key)
┌────────────────────▼────────────────────────────┐
│          Payment Relay (Simple)                 │
│  ┌──────────────────────────────────────────┐   │
│  │  - Authenticate (API Key)               │   │
│  │  - Route to gateway                     │   │
│  │  - Track wallets                        │   │
│  │  - Send webhooks                        │   │
│  └──────────────────────────────────────────┘   │
└────────────────────┬────────────────────────────┘
                     │
┌────────────────────▼────────────────────────────┐
│        PostgreSQL (Single DB)                    │
│  - systems   - wallets   - transactions          │
└──────────────────────────────────────────────────┘
                     │
┌────────────────────▼────────────────────────────┐
│    External Gateways (Stripe, Adyen, etc.)      │
└──────────────────────────────────────────────────┘
```

---

## 🦀 Simplified Rust Implementation

### 1. Minimal Cargo.toml

```toml
[package]
name = "payment-relay"
version = "0.1.0"
edition = "2021"

[dependencies]
# Web framework
axum = "0.7"
tokio = { version = "1.0", features = ["full"] }
tower-http = { version = "0.5", features = ["cors"] }

# Database
sqlx = { version = "0.7", features = ["runtime-tokio-native-tls", "postgres", "uuid", "chrono"] }
tokio-postgres = "0.7"

# Serialization
serde = { version = "1.0", features = ["derive"] }
serde_json = "1.0"

# Utilities
uuid = { version = "1.0", features = ["v4", "serde"] }
chrono = "0.4"
anyhow = "1.0"
thiserror = "1.0"
tracing = "0.1"
tracing-subscriber = "0.3"
dotenvy = "0.15"

# HTTP client
reqwest = { version = "0.11", features = ["json"] }

# Crypto
sha2 = "0.10"
```

### 2. Configuration (.env)

```env
# Database
DATABASE_URL=postgres://user:pass@localhost:5432/payment_relay

# Server
PORT=8080

# Payment Gateways
STRIPE_SECRET_KEY=sk_test_xxx
STRIPE_WEBHOOK_SECRET=whsec_xxx

ADYEN_API_KEY=AQE...
ADYEN_MERCHANT_ACCOUNT=MerchantAccount

# For simple demo
FALLBACK_GATEWAY=stripe
```

### 3. Core Models

```rust
// src/models.rs
use serde::{Deserialize, Serialize};
use uuid::Uuid;
use chrono::{DateTime, Utc};

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct System {
    pub id: Uuid,
    pub name: String,
    pub prefix: String,
    pub enabled_countries: Vec<String>,
    pub webhook_url: Option<String>,
    pub api_key: String, // Hashed
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Wallet {
    pub id: Uuid,
    pub system_id: Uuid,
    pub country: String,
    pub currency: String,
    pub balance: i64,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Transaction {
    pub id: Uuid,
    pub system_id: Uuid,
    pub wallet_id: Uuid,
    pub external_id: String,
    pub idempotency_key: String,
    pub amount: i64,
    pub currency: String,
    pub country: String,
    pub status: String, // "completed", "failed", "pending"
    pub gateway: String,
    pub gateway_reference: Option<String>,
    pub error: Option<String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}
```

### 4. Simple API Endpoints

```rust
// src/api.rs
use axum::{
    Router,
    routing::{post, get},
    Json, extract::{State, Path},
    http::StatusCode,
};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

// === Request/Response Types ===

#[derive(Debug, Deserialize)]
pub struct CreateSystemRequest {
    pub name: String,
    pub prefix: String,
    pub enabled_countries: Vec<String>,
    pub webhook_url: Option<String>,
}

#[derive(Debug, Serialize)]
pub struct CreateSystemResponse {
    pub id: Uuid,
    pub name: String,
    pub prefix: String,
    pub api_key: String, // Returned once
}

#[derive(Debug, Deserialize)]
pub struct ProcessPaymentRequest {
    pub system_id: Uuid,
    pub external_id: String,
    pub amount: i64, // In cents
    pub currency: String,
    pub country: String,
    pub payment_method: PaymentMethod,
    pub idempotency_key: String,
}

#[derive(Debug, Deserialize)]
pub struct PaymentMethod {
    pub type_: String, // "card", "bank", "wallet"
    pub details: serde_json::Value,
}

#[derive(Debug, Serialize)]
pub struct ProcessPaymentResponse {
    pub id: Uuid,
    pub external_id: String,
    pub status: String,
    pub gateway_reference: String,
}

#[derive(Debug, Serialize)]
pub struct WalletResponse {
    pub id: Uuid,
    pub system_id: Uuid,
    pub country: String,
    pub currency: String,
    pub balance: i64,
}

// === API Routes ===

pub fn create_routes() -> Router {
    Router::new()
        .route("/systems", post(create_system))
        .route("/systems/:id", get(get_system))
        .route("/wallets/:system_id", get(get_wallets))
        .route("/payments", post(process_payment))
        .route("/payments/:id", get(get_payment))
        .route("/transactions/:system_id", get(get_transactions))
}

// === Handlers ===

async fn create_system(
    State(state): State<AppState>,
    Json(req): Json<CreateSystemRequest>,
) -> Result<Json<CreateSystemResponse>, AppError> {
    let api_key = generate_api_key();
    let hashed_key = hash_api_key(&api_key);
    
    let system = state.db.create_system(
        &req.name,
        &req.prefix,
        &req.enabled_countries,
        &req.webhook_url,
        &hashed_key,
    ).await?;
    
    Ok(Json(CreateSystemResponse {
        id: system.id,
        name: system.name,
        prefix: system.prefix,
        api_key,
    }))
}

async fn process_payment(
    State(state): State<AppState>,
    Json(req): Json<ProcessPaymentRequest>,
) -> Result<Json<ProcessPaymentResponse>, AppError> {
    // 1. Get system
    let system = state.db.get_system(req.system_id).await?;
    
    // 2. Check if country is enabled
    if !system.enabled_countries.contains(&req.country) {
        return Err(AppError::CountryNotEnabled);
    }
    
    // 3. Get or create wallet
    let wallet = state.db.get_or_create_wallet(
        req.system_id,
        &req.country,
        &req.currency,
    ).await?;
    
    // 4. Check balance
    if wallet.balance < req.amount {
        return Err(AppError::InsufficientBalance);
    }
    
    // 5. Check idempotency
    if let Some(existing) = state.db.get_transaction_by_idempotency(&req.idempotency_key).await? {
        return Ok(Json(ProcessPaymentResponse {
            id: existing.id,
            external_id: existing.external_id,
            status: existing.status,
            gateway_reference: existing.gateway_reference.unwrap_or_default(),
        }));
    }
    
    // 6. Process through gateway
    let gateway_response = state.gateway.process_payment(&req).await?;
    
    // 7. Record transaction
    let transaction = state.db.create_transaction(
        req.system_id,
        wallet.id,
        &req.external_id,
        &req.idempotency_key,
        req.amount,
        &req.currency,
        &req.country,
        &gateway_response,
    ).await?;
    
    // 8. Update wallet balance
    state.db.update_wallet_balance(wallet.id, -req.amount).await?;
    
    // 9. Send webhook (fire and forget)
    if let Some(url) = system.webhook_url {
        tokio::spawn(send_webhook(url, &transaction));
    }
    
    Ok(Json(ProcessPaymentResponse {
        id: transaction.id,
        external_id: transaction.external_id,
        status: transaction.status,
        gateway_reference: transaction.gateway_reference.unwrap_or_default(),
    }))
}

async fn get_wallets(
    State(state): State<AppState>,
    Path(system_id): Path<Uuid>,
) -> Result<Json<Vec<WalletResponse>>, AppError> {
    let wallets = state.db.get_wallets(system_id).await?;
    Ok(Json(wallets))
}

async fn get_transactions(
    State(state): State<AppState>,
    Path(system_id): Path<Uuid>,
) -> Result<Json<Vec<Transaction>>, AppError> {
    let transactions = state.db.get_transactions(system_id).await?;
    Ok(Json(transactions))
}
```

### 5. Payment Gateway Integration (Simple)

```rust
// src/gateway.rs
use async_trait::async_trait;
use serde_json::json;

#[async_trait]
pub trait PaymentGateway {
    async fn process_payment(&self, request: &ProcessPaymentRequest) -> Result<GatewayResponse, GatewayError>;
}

pub struct StripeGateway {
    client: reqwest::Client,
    secret_key: String,
}

#[async_trait]
impl PaymentGateway for StripeGateway {
    async fn process_payment(&self, request: &ProcessPaymentRequest) -> Result<GatewayResponse, GatewayError> {
        let response = self.client
            .post("https://api.stripe.com/v1/payment_intents")
            .header("Authorization", format!("Bearer {}", self.secret_key))
            .form(&[
                ("amount", request.amount.to_string()),
                ("currency", request.currency.to_lowercase()),
                ("payment_method_types[]", "card".to_string()),
                ("metadata[system_id]", request.system_id.to_string()),
                ("metadata[external_id]", request.external_id.clone()),
            ])
            .send()
            .await?;

        let data: serde_json::Value = response.json().await?;
        
        Ok(GatewayResponse {
            gateway_id: data["id"].as_str().unwrap_or("").to_string(),
            status: match data["status"].as_str() {
                Some("succeeded") => "completed",
                Some("requires_payment_method") => "pending",
                _ => "pending",
            }.to_string(),
        })
    }
}
```

### 6. Simple Webhook Sender

```rust
// src/webhook.rs
use reqwest::Client;
use serde_json::json;

pub async fn send_webhook(url: String, transaction: &Transaction) {
    let client = Client::new();
    
    let payload = json!({
        "event": "payment.completed",
        "transaction_id": transaction.id,
        "external_id": transaction.external_id,
        "amount": transaction.amount,
        "currency": transaction.currency,
        "status": transaction.status,
        "gateway_reference": transaction.gateway_reference,
        "timestamp": transaction.updated_at,
    });
    
    // Retry up to 3 times
    for attempt in 1..=3 {
        match client.post(&url).json(&payload).send().await {
            Ok(res) if res.status().is_success() => {
                tracing::info!("Webhook sent successfully to {}", url);
                return;
            }
            Ok(res) => {
                tracing::warn!("Webhook failed (attempt {}): {}", attempt, res.status());
                tokio::time::sleep(tokio::time::Duration::from_millis(100 * attempt)).await;
            }
            Err(e) => {
                tracing::error!("Webhook error (attempt {}): {}", attempt, e);
                tokio::time::sleep(tokio::time::Duration::from_millis(100 * attempt)).await;
            }
        }
    }
}
```

---

## 🗄️ Simple Database Schema

```sql
-- Simple schema for internal use

CREATE TABLE systems (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    name TEXT NOT NULL,
    prefix TEXT NOT NULL UNIQUE,
    enabled_countries TEXT[] NOT NULL DEFAULT '{}',
    webhook_url TEXT,
    api_key_hash TEXT NOT NULL UNIQUE,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE TABLE wallets (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    system_id UUID NOT NULL REFERENCES systems(id) ON DELETE CASCADE,
    country TEXT NOT NULL,
    currency TEXT NOT NULL DEFAULT 'USD',
    balance BIGINT NOT NULL DEFAULT 0,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    UNIQUE(system_id, country, currency)
);

CREATE TABLE transactions (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    system_id UUID NOT NULL REFERENCES systems(id),
    wallet_id UUID NOT NULL REFERENCES wallets(id),
    external_id TEXT NOT NULL,
    idempotency_key TEXT NOT NULL UNIQUE,
    amount BIGINT NOT NULL,
    currency TEXT NOT NULL,
    country TEXT NOT NULL,
    status TEXT NOT NULL, -- 'pending', 'completed', 'failed'
    gateway TEXT NOT NULL,
    gateway_reference TEXT,
    error TEXT,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

-- Indexes for performance
CREATE INDEX idx_transactions_system_id ON transactions(system_id);
CREATE INDEX idx_transactions_external_id ON transactions(external_id);
CREATE INDEX idx_wallets_system_id ON wallets(system_id);
```

---

## 🚀 How to Use It

### 1. Start the Relay

```bash
# Clone and build
git clone https://github.com/yourcompany/payment-relay
cd payment-relay
cargo build --release

# Configure
cp .env.example .env
# Edit .env with your database and gateway keys

# Run
cargo run --release
```

### 2. Register a System

```bash
curl -X POST http://localhost:8080/systems \
  -H "Content-Type: application/json" \
  -d '{
    "name": "E-Commerce",
    "prefix": "ECO",
    "enabled_countries": ["US", "GB", "CA"],
    "webhook_url": "https://api.myapp.com/webhook"
  }'

# Response:
{
  "id": "550e8400-e29b-41d4-a716-446655440000",
  "name": "E-Commerce",
  "prefix": "ECO",
  "api_key": "sk_live_abc123xyz789"
}
```

### 3. Process a Payment

```bash
curl -X POST http://localhost:8080/payments \
  -H "Content-Type: application/json" \
  -H "X-API-Key: sk_live_abc123xyz789" \
  -d '{
    "system_id": "550e8400-e29b-41d4-a716-446655440000",
    "external_id": "order_12345",
    "amount": 1999,
    "currency": "USD",
    "country": "US",
    "idempotency_key": "550e8400-e29b-41d4-a716-446655440000_order_12345",
    "payment_method": {
      "type": "card",
      "details": {
        "token": "pm_card_visa"
      }
    }
  }'

# Response:
{
  "id": "6ba7b810-9dad-11d1-80b4-00c04fd430c8",
  "external_id": "order_12345",
  "status": "completed",
  "gateway_reference": "pi_3JxKXXX"
}
```

### 4. Check Wallet Balance

```bash
curl http://localhost:8080/wallets/550e8400-e29b-41d4-a716-446655440000 \
  -H "X-API-Key: sk_live_abc123xyz789"

# Response:
[
  {
    "id": "6ba7b810-9dad-11d1-80b4-00c04fd430c8",
    "system_id": "550e8400-e29b-41d4-a716-446655440000",
    "country": "US",
    "currency": "USD",
    "balance": 198201
  }
]
```

---

## 📁 Folder Structure (Minimal)

```
payment-relay/
├── Cargo.toml
├── .env
├── .gitignore
├── src/
│   ├── main.rs           # Entry point
│   ├── config.rs         # Config loading
│   ├── models.rs         # Data models
│   ├── api.rs            # API routes & handlers
│   ├── db.rs             # Database operations
│   ├── gateway.rs        # Payment gateway integration
│   ├── webhook.rs        # Webhook sender
│   └── error.rs          # Error handling
├── migrations/
│   └── 001_initial.sql   # Database schema
├── tests/
│   └── integration.rs    # Basic integration tests
└── docker-compose.yml    # For local development
```

---

## 🐳 Simple Docker Setup

```yaml
# docker-compose.yml
version: '3.8'
services:
  postgres:
    image: postgres:15
    environment:
      POSTGRES_USER: relay
      POSTGRES_PASSWORD: relay123
      POSTGRES_DB: payment_relay
    ports:
      - "5432:5432"
    volumes:
      - pg_data:/var/lib/postgresql/data

  payment-relay:
    build: .
    ports:
      - "8080:8080"
    environment:
      DATABASE_URL: postgres://relay:relay123@postgres:5432/payment_relay
      STRIPE_SECRET_KEY: ${STRIPE_SECRET_KEY}
    depends_on:
      - postgres
    volumes:
      - ./src:/app/src

volumes:
  pg_data:
```

---

## ✅ What You Get (Simple Version)

| Feature | Status |
|---------|--------|
| Multi-system support | ✅ |
| Per-system wallets | ✅ |
| Per-country wallets | ✅ |
| Configurable prefixes | ✅ |
| Payment relay to gateways | ✅ |
| Webhook notifications | ✅ |
| Idempotency | ✅ |
| Balance tracking | ✅ |
| Simple API | ✅ |
| Easy setup | ✅ |

## ❌ What You DON'T Get

| Feature | Reason |
|---------|--------|
| User management | Not needed (internal) |
| Admin dashboard | Not needed (direct API) |
| Multi-tenancy | Not needed (internal) |
| Billing | Not needed (internal) |
| SaaS features | Not needed (internal) |
| Complex reporting | Not needed (DB queries) |

---

This is a **no-nonsense, internal payment relay** that does exactly what you need and nothing more. The entire codebase will be **under 1000 lines of Rust** and **easy to maintain**.

Want me to:
1. **Generate the full code** for this simple version?
2. **Add more payment gateways** (Braintree, PayPal)?
3. **Create a simple admin CLI** for management?
4. **Add basic monitoring** (Prometheus metrics)?