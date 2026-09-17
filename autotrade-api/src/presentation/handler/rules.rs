use axum::extract::{Path, State};
use axum::http::StatusCode;
use axum::Json;
use paginator_rs::{PaginatorResponse, PaginatorResponseMeta};
use serde::{Deserialize, Serialize};
use serde_json::json;
use zod_rs::prelude::*;

use crate::domain::value::TradeRule;
use crate::presentation::state::AppState;

#[derive(Deserialize)]
pub struct CreateRuleRequest {
    pub symbol: String,
    pub buy_below: f64,
    pub sell_above: f64,
    pub stop_loss: f64,
    pub max_lot: u32,
}

#[derive(Serialize)]
pub struct RuleResponse {
    pub symbol: String,
    pub buy_below: String,
    pub sell_above: String,
    pub stop_loss: String,
    pub max_lot: u32,
}

fn validate_create_rule(body: &serde_json::Value) -> Result<(), String> {
    let schema = object()
        .field("symbol", string().min(1).max(10))
        .field("buy_below", number().positive())
        .field("sell_above", number().positive())
        .field("stop_loss", number().positive())
        .field("max_lot", number().positive().int())
        .strict();

    schema.safe_parse(body).map(|_| ()).map_err(|e| e.to_string())
}

pub async fn list_rules(
    State(state): State<AppState>,
) -> Result<Json<PaginatorResponse<RuleResponse>>, (StatusCode, Json<serde_json::Value>)> {
    let rules = state
        .rules
        .find_all()
        .await
        .map_err(|e| {
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(json!({"error": e.to_string()})),
            )
        })?;

    let total = rules.len() as u32;
    let data: Vec<RuleResponse> = rules
        .into_iter()
        .map(|(symbol, rule)| RuleResponse {
            symbol,
            buy_below: rule.buy_below.to_string(),
            sell_above: rule.sell_above.to_string(),
            stop_loss: rule.stop_loss.to_string(),
            max_lot: rule.max_lot,
        })
        .collect();

    let meta = PaginatorResponseMeta::new(1, total.max(1), total);

    Ok(Json(PaginatorResponse { data, meta }))
}

pub async fn create_rule(
    State(state): State<AppState>,
    Json(body): Json<serde_json::Value>,
) -> Result<(StatusCode, Json<RuleResponse>), (StatusCode, Json<serde_json::Value>)> {
    validate_create_rule(&body).map_err(|e| {
        (StatusCode::BAD_REQUEST, Json(json!({"error": e})))
    })?;

    let req: CreateRuleRequest = serde_json::from_value(body).map_err(|e| {
        (
            StatusCode::BAD_REQUEST,
            Json(json!({"error": e.to_string()})),
        )
    })?;

    let rule = TradeRule {
        buy_below: rust_decimal::Decimal::from_f64_retain(req.buy_below)
            .unwrap_or_default(),
        sell_above: rust_decimal::Decimal::from_f64_retain(req.sell_above)
            .unwrap_or_default(),
        stop_loss: rust_decimal::Decimal::from_f64_retain(req.stop_loss)
            .unwrap_or_default(),
        max_lot: req.max_lot,
    };

    let symbol = req.symbol.to_uppercase();

    state.rules.save(&symbol, &rule).await.map_err(|e| {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(json!({"error": e.to_string()})),
        )
    })?;

    Ok((
        StatusCode::CREATED,
        Json(RuleResponse {
            symbol,
            buy_below: rule.buy_below.to_string(),
            sell_above: rule.sell_above.to_string(),
            stop_loss: rule.stop_loss.to_string(),
            max_lot: rule.max_lot,
        }),
    ))
}

pub async fn delete_rule(
    State(state): State<AppState>,
    Path(symbol): Path<String>,
) -> Result<StatusCode, (StatusCode, Json<serde_json::Value>)> {
    state
        .rules
        .delete(&symbol.to_uppercase())
        .await
        .map_err(|e| {
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(json!({"error": e.to_string()})),
            )
        })?;

    Ok(StatusCode::NO_CONTENT)
}
