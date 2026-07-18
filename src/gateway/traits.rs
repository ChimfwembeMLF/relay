use async_trait::async_trait;
use uuid::Uuid;

use crate::error::AppError;
use crate::models::PaymentMethod;

#[derive(Debug, Clone)]
pub struct GatewayResponse {
    pub reference: Option<String>,
    pub status: String,
    pub success: bool,
    pub error: Option<String>,
}

#[derive(Debug, Clone)]
pub struct GatewayPaymentRequest {
    pub payout_id: Uuid,
    pub amount: i64,
    pub currency: String,
    pub payment_method: PaymentMethod,
}

#[async_trait]
pub trait PaymentGateway: Send + Sync {
    async fn process_payment(&self, request: GatewayPaymentRequest) -> Result<GatewayResponse, AppError>;
}
