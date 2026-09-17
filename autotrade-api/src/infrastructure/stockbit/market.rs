use async_trait::async_trait;

use crate::domain::entity::stock::{Stock, StockFundamental, StockPrice};
use crate::domain::error::DomainError;
use crate::domain::port::market::MarketDataPort;

use super::client::StockbitClient;

#[async_trait]
impl MarketDataPort for StockbitClient {
    async fn get_stock(&self, symbol: &str) -> Result<Stock, DomainError> {
        let _url = format!("{}/v1/stocks/{}", self.base_url, symbol);
        // TODO: implement Stockbit API call
        Err(DomainError::ExternalService(format!(
            "Stockbit API not yet implemented for {symbol}"
        )))
    }

    async fn get_price_history(
        &self,
        symbol: &str,
        _days: u32,
    ) -> Result<Vec<StockPrice>, DomainError> {
        let _url =
            format!("{}/v1/stocks/{}/history", self.base_url, symbol);
        Err(DomainError::ExternalService(format!(
            "Stockbit API not yet implemented for {symbol}"
        )))
    }

    async fn get_fundamentals(
        &self,
        symbol: &str,
    ) -> Result<StockFundamental, DomainError> {
        let _url =
            format!("{}/v1/stocks/{}/fundamental", self.base_url, symbol);
        Err(DomainError::ExternalService(format!(
            "Stockbit API not yet implemented for {symbol}"
        )))
    }

    async fn search_stocks(
        &self,
        _query: &str,
    ) -> Result<Vec<Stock>, DomainError> {
        Err(DomainError::ExternalService(
            "Stockbit search not yet implemented".into(),
        ))
    }
}
