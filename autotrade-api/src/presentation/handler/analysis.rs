use axum::extract::{Path, Query, State};
use axum::Json;
use serde::Deserialize;
use serde_json::{json, Value};

use crate::presentation::state::AppState;

#[derive(Deserialize)]
pub struct SuggestQuery {
    pub limit: Option<usize>,
}

pub async fn analyze_stock(
    State(state): State<AppState>,
    Path(symbol): Path<String>,
) -> Json<Value> {
    match state.analyze.execute(&symbol.to_uppercase()).await {
        Ok(a) => Json(json!({
            "success": true,
            "data": {
                "symbol": a.symbol,
                "signal": a.signal,
                "trend": a.trend,
                "confidence": a.confidence.to_string(),
                "entry": a.price_target.entry.to_string(),
                "take_profit": a.price_target.take_profit.to_string(),
                "stop_loss": a.price_target.stop_loss.to_string(),
                "risk_factors": a.risk_factors,
                "reasoning": a.reasoning,
            }
        })),
        Err(e) => Json(json!({
            "success": false,
            "error": e.to_string(),
        })),
    }
}

pub async fn suggest_stocks(
    State(state): State<AppState>,
    Query(query): Query<SuggestQuery>,
) -> Json<Value> {
    let limit = query.limit.unwrap_or(10);
    match state.suggest.execute(limit).await {
        Ok(suggestions) => Json(json!({
            "success": true,
            "data": suggestions,
        })),
        Err(e) => Json(json!({
            "success": false,
            "error": e.to_string(),
        })),
    }
}
