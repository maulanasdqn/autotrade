use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Deserialize)]
pub struct ApiResponse<T> {
    pub success: bool,
    pub data: Option<T>,
    pub error: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SuggestionDto {
    pub symbol: String,
    pub name: String,
    pub signal: String,
    pub trend: String,
    pub current_price: String,
    pub target_price: String,
    pub potential_return: String,
    pub confidence: String,
    pub reason: String,
}

#[derive(Debug, Clone, Deserialize)]
pub struct AnalysisResponse {
    pub symbol: String,
    pub signal: String,
    pub trend: String,
    pub confidence: String,
    pub entry: String,
    pub take_profit: String,
    pub stop_loss: String,
    pub risk_factors: Option<Vec<String>>,
    pub reasoning: String,
}

#[derive(Debug, Clone, Deserialize)]
pub struct PortfolioResponse {
    pub cash_balance: String,
    pub positions: Vec<PositionResponse>,
    pub total_equity: String,
}

#[derive(Debug, Clone, Deserialize)]
pub struct PositionResponse {
    pub symbol: String,
    pub lot: u32,
    pub avg_price: String,
    pub current_price: String,
    pub market_value: String,
    pub unrealized_pnl: String,
}

#[derive(Debug, Clone, Deserialize)]
pub struct RulesListResponse {
    pub data: Vec<RuleResponse>,
    pub meta: PaginatorMeta,
}

#[derive(Debug, Clone, Deserialize)]
pub struct PaginatorMeta {
    pub page: u32,
    pub per_page: u32,
    pub total: u32,
    pub total_pages: u32,
    pub has_next: bool,
    pub has_prev: bool,
}

#[derive(Debug, Clone, Deserialize)]
pub struct RuleResponse {
    pub symbol: String,
    pub buy_below: String,
    pub sell_above: String,
    pub stop_loss: String,
    pub max_lot: u32,
}

#[derive(Debug, Clone, Deserialize)]
pub struct TokenStatus {
    pub valid: bool,
    pub expires_at: Option<String>,
    pub ttl_seconds: Option<i64>,
    pub ttl_human: Option<String>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct AutoTradeFullResponse {
    pub success: bool,
    pub executed: Option<usize>,
    pub orders: Option<Vec<OrderResponse>>,
    pub error: Option<String>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct OrderResponse {
    pub id: String,
    pub symbol: String,
    pub side: String,
    pub lot: u32,
    pub price: String,
    pub status: String,
}
