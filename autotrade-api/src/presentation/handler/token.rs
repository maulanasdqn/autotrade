use axum::extract::State;
use axum::http::StatusCode;
use axum::Json;
use serde::{Deserialize, Serialize};

use crate::infrastructure::stockbit::client::StockbitClient;
use crate::presentation::state::AppState;

#[derive(Serialize)]
pub struct TokenStatus {
    pub valid: bool,
    pub expires_at: Option<String>,
    pub ttl_seconds: Option<i64>,
    pub ttl_human: Option<String>,
}

#[derive(Deserialize)]
pub struct UpdateToken {
    pub token: String,
}

pub async fn get_token_status(
    State(state): State<AppState>,
) -> Json<TokenStatus> {
    let token = state.token.read().await;

    if token.is_empty() {
        return Json(TokenStatus {
            valid: false,
            expires_at: None,
            ttl_seconds: None,
            ttl_human: None,
        });
    }

    match StockbitClient::jwt_expiry(&token) {
        Some(exp) => {
            let now = chrono::Utc::now().timestamp();
            let ttl = exp - now;
            let valid = ttl > 0;

            let expires_at = chrono::DateTime::from_timestamp(exp, 0)
                .map(|dt| dt.format("%Y-%m-%d %H:%M:%S UTC").to_string());

            let ttl_human = if ttl > 0 {
                let hours = ttl / 3600;
                let mins = (ttl % 3600) / 60;
                Some(format!("{}h {}m", hours, mins))
            } else {
                Some("expired".to_string())
            };

            Json(TokenStatus {
                valid,
                expires_at,
                ttl_seconds: Some(ttl),
                ttl_human,
            })
        }
        None => Json(TokenStatus {
            valid: false,
            expires_at: None,
            ttl_seconds: None,
            ttl_human: None,
        }),
    }
}

pub async fn update_token(
    State(state): State<AppState>,
    Json(body): Json<UpdateToken>,
) -> (StatusCode, Json<TokenStatus>) {
    let new_token = body.token.trim().to_string();

    if new_token.is_empty() {
        return (
            StatusCode::BAD_REQUEST,
            Json(TokenStatus {
                valid: false,
                expires_at: None,
                ttl_seconds: None,
                ttl_human: None,
            }),
        );
    }

    {
        let mut w = state.token.write().await;
        *w = new_token.clone();
    }

    tracing::info!("[TOKEN] Token updated via API");

    let status = match StockbitClient::jwt_expiry(&new_token) {
        Some(exp) => {
            let now = chrono::Utc::now().timestamp();
            let ttl = exp - now;
            let hours = ttl / 3600;
            let mins = (ttl % 3600) / 60;

            TokenStatus {
                valid: ttl > 0,
                expires_at: chrono::DateTime::from_timestamp(exp, 0)
                    .map(|dt| {
                        dt.format("%Y-%m-%d %H:%M:%S UTC").to_string()
                    }),
                ttl_seconds: Some(ttl),
                ttl_human: Some(format!("{}h {}m", hours, mins)),
            }
        }
        None => TokenStatus {
            valid: false,
            expires_at: None,
            ttl_seconds: None,
            ttl_human: None,
        },
    };

    (StatusCode::OK, Json(status))
}
