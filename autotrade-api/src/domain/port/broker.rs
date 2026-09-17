use async_trait::async_trait;
use rust_decimal::Decimal;

use crate::domain::entity::order::Order;
use crate::domain::entity::portfolio::Portfolio;
use crate::domain::error::DomainError;
use crate::domain::value::OrderSide;

#[async_trait]
pub trait BrokerPort: Send + Sync {
    async fn place_order(
        &self,
        symbol: &str,
        side: OrderSide,
        lot: u32,
        price: Decimal,
    ) -> Result<Order, DomainError>;

    async fn cancel_order(&self, order_id: &str) -> Result<(), DomainError>;

    async fn get_order(&self, order_id: &str) -> Result<Order, DomainError>;

    async fn get_open_orders(&self) -> Result<Vec<Order>, DomainError>;

    async fn get_portfolio(&self) -> Result<Portfolio, DomainError>;
}
