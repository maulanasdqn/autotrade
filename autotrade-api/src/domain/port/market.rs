use async_trait::async_trait;

use crate::domain::entity::stock::{Stock, StockFundamental, StockPrice};
use crate::domain::error::DomainError;

#[async_trait]
pub trait MarketDataPort: Send + Sync {
    async fn get_stock(&self, symbol: &str) -> Result<Stock, DomainError>;

    async fn get_price_history(
        &self,
        symbol: &str,
        days: u32,
    ) -> Result<Vec<StockPrice>, DomainError>;

    async fn get_fundamentals(
        &self,
        symbol: &str,
    ) -> Result<StockFundamental, DomainError>;

    async fn search_stocks(
        &self,
        query: &str,
    ) -> Result<Vec<Stock>, DomainError>;
}
