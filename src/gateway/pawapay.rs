use std::time::Duration;

use async_trait::async_trait;
use reqwest::Client;
use serde::{Deserialize, Serialize};

use crate::error::AppError;
use crate::gateway::{GatewayDepositRequest, GatewayPaymentRequest, GatewayResponse, PaymentGateway};

#[derive(Clone)]
pub struct PawapayGateway {
    client: Client,
    base_url: String,
    api_token: String,
}

impl PawapayGateway {
    pub fn new(base_url: String, api_token: String) -> Self {
        Self {
            client: Client::builder()
                .timeout(Duration::from_secs(30))
                .build()
                .expect("failed to build HTTP client"),
            base_url,
            api_token,
        }
    }

    fn is_retryable(status: reqwest::StatusCode, body: &PawapayResponse) -> bool {
        if status.is_server_error() {
            return true;
        }
        if let Some(reason) = &body.failure_reason {
            return reason.failure_code == "PROVIDER_TEMPORARILY_UNAVAILABLE";
        }
        false
    }
}

#[derive(Serialize)]
struct PawapayPayoutRequest {
    amount: String,
    currency: String,
    #[serde(rename = "payoutId")]
    payout_id: String,
    recipient: serde_json::Value,
}

#[derive(Debug, Deserialize)]
struct PawapayResponse {
    #[serde(rename = "payoutId")]
    payout_id: Option<String>,
    #[serde(rename = "depositId")]
    deposit_id: Option<String>,
    status: String,
    #[serde(rename = "failureReason")]
    failure_reason: Option<PawapayFailure>,
}

#[derive(Debug, Deserialize)]
struct PawapayFailure {
    #[serde(rename = "failureCode")]
    failure_code: String,
    #[serde(rename = "failureMessage")]
    failure_message: String,
}

#[async_trait]
impl PaymentGateway for PawapayGateway {
    async fn process_payment(&self, request: GatewayPaymentRequest) -> Result<GatewayResponse, AppError> {
        let recipient = build_recipient(&request.payment_method)?;

        let payload = PawapayPayoutRequest {
            amount: format_pawapay_amount(request.amount),
            currency: request.currency.clone(),
            payout_id: request.payout_id.to_string(),
            recipient,
        };

        let url = format!("{}/v2/payouts", self.base_url.trim_end_matches('/'));
        let mut last_error = None;

        for attempt in 0..3 {
            if attempt > 0 {
                let delay = Duration::from_millis(100 * 4_u64.pow(attempt - 1));
                tokio::time::sleep(delay).await;
            }

            let response = self
                .client
                .post(&url)
                .bearer_auth(&self.api_token)
                .json(&payload)
                .send()
                .await;

            match response {
                Ok(resp) => {
                    let status_code = resp.status();
                    let body: PawapayResponse = resp
                        .json()
                        .await
                        .map_err(|e| AppError::Gateway(format!("invalid gateway response: {e}")))?;

                    let success = matches!(
                        body.status.as_str(),
                        "ACCEPTED" | "COMPLETED" | "DUPLICATE_IGNORED"
                    );

                    if success {
                        return Ok(GatewayResponse {
                            reference: body.payout_id.or(Some(request.payout_id.to_string())),
                            status: body.status,
                            success: true,
                            error: None,
                        });
                    }

                    if Self::is_retryable(status_code, &body) && attempt < 2 {
                        last_error = body.failure_reason.map(|f| f.failure_message);
                        continue;
                    }

                    return Ok(GatewayResponse {
                        reference: body.payout_id,
                        status: body.status,
                        success: false,
                        error: body
                            .failure_reason
                            .map(|f| format!("{}: {}", f.failure_code, f.failure_message)),
                    });
                }
                Err(e) if e.is_timeout() || e.is_connect() => {
                    last_error = Some(e.to_string());
                    if attempt < 2 {
                        continue;
                    }
                }
                Err(e) => return Err(AppError::Gateway(e.to_string())),
            }
        }

        Err(AppError::Gateway(
            last_error.unwrap_or_else(|| "gateway retries exhausted".into()),
        ))
    }

    async fn process_deposit(&self, request: GatewayDepositRequest) -> Result<GatewayResponse, AppError> {
        let payer = build_payer(&request.payment_method)?;

        let payload = PawapayDepositRequest {
            amount: format_pawapay_amount(request.amount),
            currency: request.currency.clone(),
            deposit_id: request.deposit_id.to_string(),
            payer,
            client_reference_id: request.client_reference,
        };

        let url = format!("{}/v2/deposits", self.base_url.trim_end_matches('/'));
        self.post_with_retry(&url, &payload, request.deposit_id.to_string(), |body| {
            body.deposit_id.clone()
        })
        .await
    }
}

#[derive(Serialize)]
struct PawapayDepositRequest {
    amount: String,
    currency: String,
    #[serde(rename = "depositId")]
    deposit_id: String,
    payer: serde_json::Value,
    #[serde(rename = "clientReferenceId")]
    client_reference_id: Option<String>,
}

impl PawapayGateway {
    async fn post_with_retry<T: Serialize>(
        &self,
        url: &str,
        payload: &T,
        fallback_id: String,
        reference: impl Fn(&PawapayResponse) -> Option<String>,
    ) -> Result<GatewayResponse, AppError> {
        let mut last_error = None;

        for attempt in 0..3 {
            if attempt > 0 {
                let delay = Duration::from_millis(100 * 4_u64.pow(attempt - 1));
                tokio::time::sleep(delay).await;
            }

            let response = self
                .client
                .post(url)
                .bearer_auth(&self.api_token)
                .json(payload)
                .send()
                .await;

            match response {
                Ok(resp) => {
                    let status_code = resp.status();
                    let body: PawapayResponse = resp
                        .json()
                        .await
                        .map_err(|e| AppError::Gateway(format!("invalid gateway response: {e}")))?;

                    let success = matches!(
                        body.status.as_str(),
                        "ACCEPTED" | "COMPLETED" | "DUPLICATE_IGNORED"
                    );

                    if success {
                        return Ok(GatewayResponse {
                            reference: reference(&body).or(Some(fallback_id.clone())),
                            status: body.status,
                            success: true,
                            error: None,
                        });
                    }

                    if Self::is_retryable(status_code, &body) && attempt < 2 {
                        last_error = body.failure_reason.map(|f| f.failure_message);
                        continue;
                    }

                    return Ok(GatewayResponse {
                        reference: reference(&body),
                        status: body.status,
                        success: false,
                        error: body
                            .failure_reason
                            .map(|f| format!("{}: {}", f.failure_code, f.failure_message)),
                    });
                }
                Err(e) if e.is_timeout() || e.is_connect() => {
                    last_error = Some(e.to_string());
                    if attempt < 2 {
                        continue;
                    }
                }
                Err(e) => return Err(AppError::Gateway(e.to_string())),
            }
        }

        Err(AppError::Gateway(
            last_error.unwrap_or_else(|| "gateway retries exhausted".into()),
        ))
    }
}

fn build_payer(method: &crate::models::PaymentMethod) -> Result<serde_json::Value, AppError> {
    build_recipient(method)
}

/// Relay stores amounts as integer minor units (cents). PawaPay expects major units as a string
/// (e.g. ZMW 500.00 → `"500"` or `"500.00"`), not `"50000"`.
fn format_pawapay_amount(amount_minor: i64) -> String {
    let whole = amount_minor / 100;
    let cents = amount_minor.rem_euclid(100);
    if cents == 0 {
        whole.to_string()
    } else {
        format!("{whole}.{cents:02}")
    }
}

fn build_recipient(method: &crate::models::PaymentMethod) -> Result<serde_json::Value, AppError> {
    match method.method_type.as_str() {
        "mmo" => {
            // Normalize phone → phoneNumber for PawaPay accountDetails.
            let mut details = method.details.clone();
            if details.get("phoneNumber").and_then(|v| v.as_str()).is_none() {
                if let Some(phone) = details.get("phone").cloned() {
                    details
                        .as_object_mut()
                        .map(|o| o.insert("phoneNumber".into(), phone));
                }
            }
            Ok(serde_json::json!({
                "type": "MMO",
                "accountDetails": details
            }))
        }
        other => Err(AppError::Validation(format!(
            "unsupported payment method type for pawapay: {other}"
        ))),
    }
}

#[cfg(test)]
mod tests {
    use super::format_pawapay_amount;

    #[test]
    fn formats_minor_units_as_major_string() {
        assert_eq!(format_pawapay_amount(50000), "500");
        assert_eq!(format_pawapay_amount(1500), "15");
        assert_eq!(format_pawapay_amount(1505), "15.05");
        assert_eq!(format_pawapay_amount(1), "0.01");
    }
}
