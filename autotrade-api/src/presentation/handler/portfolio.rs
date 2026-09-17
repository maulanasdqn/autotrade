use axum::extract::State;
use axum::http::StatusCode;
use axum::Json;
use serde::Serialize;
use serde_json::json;

use crate::presentation::state::AppState;

#[derive(Serialize)]
pub struct PositionResponse {
    pub symbol: String,
    pub lot: u32,
    pub avg_price: String,
    pub current_price: String,
    pub market_value: String,
    pub unrealized_pnl: String,
}

#[derive(Serialize)]
pub struct PortfolioResponse {
    pub cash_balance: String,
    pub positions: Vec<PositionResponse>,
    pub total_equity: String,
}

pub async fn get_portfolio(
    State(state): State<AppState>,
) -> Result<Json<PortfolioResponse>, (StatusCode, Json<serde_json::Value>)> {
    let portfolio = state.broker.get_portfolio().await.map_err(|e| {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(json!({"error": e.to_string()})),
        )
    })?;

    let positions = portfolio
        .positions
        .iter()
        .map(|p| PositionResponse {
            symbol: p.symbol.clone(),
            lot: p.lot,
            avg_price: p.avg_price.to_string(),
            current_price: p.current_price.to_string(),
            market_value: p.market_value().to_string(),
            unrealized_pnl: p.unrealized_pnl().to_string(),
        })
        .collect();

    Ok(Json(PortfolioResponse {
        cash_balance: portfolio.balance.to_string(),
        positions,
        total_equity: portfolio.total_equity().to_string(),
    }))
}
