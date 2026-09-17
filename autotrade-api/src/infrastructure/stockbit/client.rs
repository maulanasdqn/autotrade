use base64::{engine::general_purpose::URL_SAFE_NO_PAD, Engine};
use reqwest::Client;
use std::sync::Arc;
use tokio::sync::RwLock;

#[derive(Clone)]
pub struct StockbitClient {
    pub http: Client,
    pub base_url: String,
    pub token: Arc<RwLock<String>>,
}

impl StockbitClient {
    pub fn new(token: &str) -> Self {
        Self {
            http: Client::new(),
            base_url: "https://carina.stockbit.com".to_string(),
            token: Arc::new(RwLock::new(token.to_string())),
        }
    }

    pub fn jwt_expiry(token: &str) -> Option<i64> {
        let payload = token.split('.').nth(1)?;
        let decoded = URL_SAFE_NO_PAD.decode(payload).ok()?;
        let json: serde_json::Value =
            serde_json::from_slice(&decoded).ok()?;
        json["exp"].as_i64()
    }

    pub async fn token_str(&self) -> String {
        self.token.read().await.clone()
    }

    pub fn spawn_refresh_task(&self) {
        let token = self.token.clone();
        let http = self.http.clone();
        let base_url = self.base_url.clone();

        tokio::spawn(async move {
            loop {
                let current_token = token.read().await.clone();

                let exp = match Self::jwt_expiry(&current_token) {
                    Some(e) => e,
                    None => {
                        tracing::warn!(
                            "[STOCKBIT] Cannot parse JWT expiry, \
                             refresh task stopping"
                        );
                        return;
                    }
                };

                let now = chrono::Utc::now().timestamp();
                let refresh_at = exp - 600;
                let wait_secs = (refresh_at - now).max(0);

                if wait_secs > 0 {
                    tracing::info!(
                        expires_in_min = (exp - now) / 60,
                        refresh_in_min = wait_secs / 60,
                        "[STOCKBIT] Token valid, will refresh in {} min",
                        wait_secs / 60
                    );
                    tokio::time::sleep(
                        std::time::Duration::from_secs(wait_secs as u64),
                    )
                    .await;
                }

                let auth = if current_token.starts_with("Bearer ") {
                    current_token.clone()
                } else {
                    format!("Bearer {}", current_token)
                };

                tracing::info!("[STOCKBIT] Attempting token refresh...");

                match http
                    .post(format!("{}/auth/refresh", base_url))
                    .header("Authorization", &auth)
                    .header("Content-Type", "application/json")
                    .send()
                    .await
                {
                    Ok(resp) => {
                        let status = resp.status();
                        let body: serde_json::Value =
                            resp.json().await.unwrap_or_default();

                        if status.is_success() {
                            let new_token = body["data"]["token"]
                                .as_str()
                                .or_else(|| {
                                    body["data"]["access_token"].as_str()
                                })
                                .or_else(|| body["token"].as_str())
                                .or_else(|| body["access_token"].as_str());

                            if let Some(t) = new_token {
                                let mut w = token.write().await;
                                *w = t.to_string();
                                tracing::info!(
                                    "[STOCKBIT] Token refreshed \
                                     successfully"
                                );
                                continue;
                            }
                            tracing::warn!(
                                response = %body,
                                "[STOCKBIT] Refresh OK but no token \
                                 found in response"
                            );
                        } else {
                            tracing::warn!(
                                status = %status,
                                error = %body,
                                "[STOCKBIT] Token refresh failed"
                            );
                        }
                    }
                    Err(e) => {
                        tracing::error!(
                            error = %e,
                            "[STOCKBIT] Token refresh request failed"
                        );
                    }
                }

                tracing::warn!(
                    "[STOCKBIT] Refresh failed, retrying in 5 min"
                );
                tokio::time::sleep(std::time::Duration::from_secs(300))
                    .await;
            }
        });
    }
}
