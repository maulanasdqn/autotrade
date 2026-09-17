use std::sync::Arc;

use crate::domain::entity::order::Order;
use crate::domain::error::DomainError;
use crate::domain::port::broker::BrokerPort;
use crate::domain::port::market::MarketDataPort;
use crate::domain::port::repository::TradeRuleRepository;
use crate::domain::value::OrderSide;

pub struct AutoTradeUseCase {
    market: Arc<dyn MarketDataPort>,
    broker: Arc<dyn BrokerPort>,
    rules: Arc<dyn TradeRuleRepository>,
}

impl AutoTradeUseCase {
    pub fn new(
        market: Arc<dyn MarketDataPort>,
        broker: Arc<dyn BrokerPort>,
        rules: Arc<dyn TradeRuleRepository>,
    ) -> Self {
        Self { market, broker, rules }
    }

    pub async fn execute(&self) -> Result<Vec<Order>, DomainError> {
        let all_rules = self.rules.find_all().await?;
        let portfolio = self.broker.get_portfolio().await?;
        let mut executed = Vec::new();

        for (symbol, rule) in &all_rules {
            let stock = self.market.get_stock(symbol).await?;
            let price = stock.last_price;

            let position = portfolio
                .positions
                .iter()
                .find(|p| p.symbol == *symbol);

            if position.is_none() && price <= rule.buy_below {
                tracing::info!(%symbol, %price, "auto-buy triggered");
                let order = self
                    .broker
                    .place_order(symbol, OrderSide::Buy, rule.max_lot, price)
                    .await?;
                executed.push(order);
                continue;
            }

            if let Some(pos) = position {
                let trigger = if price >= rule.sell_above {
                    Some("take-profit")
                } else if price <= rule.stop_loss {
                    Some("stop-loss")
                } else {
                    None
                };

                if let Some(reason) = trigger {
                    tracing::info!(%symbol, %price, reason, "auto-sell triggered");
                    let order = self
                        .broker
                        .place_order(symbol, OrderSide::Sell, pos.lot, price)
                        .await?;
                    executed.push(order);
                }
            }
        }

        Ok(executed)
    }
}
