use async_trait::async_trait;

use crate::domain::entity::analysis::{StockAnalysis, StockSuggestion};
use crate::domain::entity::stock::{StockFundamental, StockPrice};
use crate::domain::error::DomainError;

#[async_trait]
pub trait AiAnalysisPort: Send + Sync {
    async fn analyze_stock(
        &self,
        symbol: &str,
        prices: &[StockPrice],
        fundamental: &StockFundamental,
    ) -> Result<StockAnalysis, DomainError>;

    async fn suggest_stocks(
        &self,
        limit: usize,
    ) -> Result<Vec<StockSuggestion>, DomainError>;
}
