use chrono::{DateTime, Utc};
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::domain::value::{PriceTarget, Signal, Trend};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StockAnalysis {
    pub id: Uuid,
    pub symbol: String,
    pub signal: Signal,
    pub trend: Trend,
    pub confidence: Decimal,
    pub price_target: PriceTarget,
    pub risk_factors: Vec<String>,
    pub reasoning: String,
    pub analyzed_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StockSuggestion {
    pub symbol: String,
    pub name: String,
    pub signal: Signal,
    pub trend: Trend,
    pub current_price: Decimal,
    pub target_price: Decimal,
    pub potential_return: Decimal,
    pub confidence: Decimal,
    pub reason: String,
}
