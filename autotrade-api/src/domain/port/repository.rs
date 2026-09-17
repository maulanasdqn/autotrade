use async_trait::async_trait;

use crate::domain::entity::analysis::StockAnalysis;
use crate::domain::entity::order::Order;
use crate::domain::error::DomainError;
use crate::domain::value::TradeRule;

#[async_trait]
pub trait AnalysisRepository: Send + Sync {
    async fn save(&self, analysis: &StockAnalysis) -> Result<(), DomainError>;

    async fn find_latest(
        &self,
        symbol: &str,
    ) -> Result<Option<StockAnalysis>, DomainError>;

    async fn find_all_latest(
        &self,
        limit: usize,
    ) -> Result<Vec<StockAnalysis>, DomainError>;
}

#[async_trait]
pub trait OrderRepository: Send + Sync {
    async fn save(&self, order: &Order) -> Result<(), DomainError>;

    async fn find_by_id(
        &self,
        id: &str,
    ) -> Result<Option<Order>, DomainError>;

    async fn find_open_orders(&self) -> Result<Vec<Order>, DomainError>;
}

#[async_trait]
pub trait TradeRuleRepository: Send + Sync {
    async fn save(
        &self,
        symbol: &str,
        rule: &TradeRule,
    ) -> Result<(), DomainError>;

    async fn find(
        &self,
        symbol: &str,
    ) -> Result<Option<TradeRule>, DomainError>;

    async fn find_all(&self) -> Result<Vec<(String, TradeRule)>, DomainError>;

    async fn delete(&self, symbol: &str) -> Result<(), DomainError>;
}
