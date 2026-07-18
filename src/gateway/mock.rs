use async_trait::async_trait;
use uuid::Uuid;

use crate::error::AppError;
use crate::gateway::{GatewayPaymentRequest, GatewayResponse, PaymentGateway};

#[derive(Clone, Default)]
pub struct MockGateway {
    pub succeed: bool,
    pub reference: Option<String>,
}

impl MockGateway {
    pub fn success() -> Self {
        Self {
            succeed: true,
            reference: Some(Uuid::new_v4().to_string()),
        }
    }

    pub fn failure() -> Self {
        Self {
            succeed: false,
            reference: None,
        }
    }
}

#[async_trait]
impl PaymentGateway for MockGateway {
    async fn process_payment(&self, request: GatewayPaymentRequest) -> Result<GatewayResponse, AppError> {
        if self.succeed {
            Ok(GatewayResponse {
                reference: self
                    .reference
                    .clone()
                    .or(Some(request.payout_id.to_string())),
                status: "ACCEPTED".into(),
                success: true,
                error: None,
            })
        } else {
            Ok(GatewayResponse {
                reference: None,
                status: "REJECTED".into(),
                success: false,
                error: Some("mock gateway failure".into()),
            })
        }
    }
}
