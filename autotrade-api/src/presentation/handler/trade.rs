use axum::extract::State;
use axum::Json;
use serde_json::{json, Value};

use crate::presentation::state::AppState;

pub async fn execute_autotrade(
    State(state): State<AppState>,
) -> Json<Value> {
    match state.autotrade.execute().await {
        Ok(orders) => {
            let data: Vec<_> = orders
                .iter()
                .map(|o| {
                    json!({
                        "id": o.id.to_string(),
                        "symbol": o.symbol,
                        "side": o.side,
                        "lot": o.lot,
                        "price": o.price.to_string(),
                        "status": o.status,
                    })
                })
                .collect();

            Json(json!({
                "success": true,
                "executed": data.len(),
                "orders": data,
            }))
        }
        Err(e) => Json(json!({
            "success": false,
            "error": e.to_string(),
        })),
    }
}
