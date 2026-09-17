use std::sync::Mutex;

use async_trait::async_trait;
use chrono::Utc;
use rust_decimal::Decimal;
use uuid::Uuid;

use crate::domain::entity::order::{Order};
use crate::domain::entity::portfolio::{Portfolio, Position};
use crate::domain::error::DomainError;
use crate::domain::port::broker::BrokerPort;
use crate::domain::value::{OrderSide, OrderStatus};

pub struct SimulatedBroker {
    balance: Mutex<Decimal>,
    positions: Mutex<Vec<Position>>,
    orders: Mutex<Vec<Order>>,
}

impl SimulatedBroker {
    pub fn new(initial_balance: Decimal) -> Self {
        Self {
            balance: Mutex::new(initial_balance),
            positions: Mutex::new(Vec::new()),
            orders: Mutex::new(Vec::new()),
        }
    }
}

#[async_trait]
impl BrokerPort for SimulatedBroker {
    async fn place_order(
        &self,
        symbol: &str,
        side: OrderSide,
        lot: u32,
        price: Decimal,
    ) -> Result<Order, DomainError> {
        let cost = price * Decimal::from(lot) * Decimal::from(100);
        let now = Utc::now();

        let mut balance = self.balance.lock().unwrap();
        let mut positions = self.positions.lock().unwrap();

        match side {
            OrderSide::Buy => {
                if *balance < cost {
                    return Err(DomainError::ExternalService(
                        format!(
                            "Insufficient balance: need {} but have {}",
                            cost, *balance
                        ),
                    ));
                }
                *balance -= cost;

                if let Some(pos) = positions
                    .iter_mut()
                    .find(|p| p.symbol == symbol)
                {
                    let total_old = pos.avg_price
                        * Decimal::from(pos.lot)
                        * Decimal::from(100);
                    pos.lot += lot;
                    pos.avg_price =
                        (total_old + cost)
                            / (Decimal::from(pos.lot) * Decimal::from(100));
                    pos.current_price = price;
                } else {
                    positions.push(Position {
                        symbol: symbol.to_string(),
                        lot,
                        avg_price: price,
                        current_price: price,
                    });
                }
            }
            OrderSide::Sell => {
                let pos = positions
                    .iter_mut()
                    .find(|p| p.symbol == symbol)
                    .ok_or_else(|| {
                        DomainError::ExternalService(format!(
                            "No position in {symbol} to sell"
                        ))
                    })?;

                if pos.lot < lot {
                    return Err(DomainError::ExternalService(
                        format!(
                            "Insufficient lots: have {} but want to sell {}",
                            pos.lot, lot
                        ),
                    ));
                }

                pos.lot -= lot;
                *balance += cost;

                if pos.lot == 0 {
                    positions.retain(|p| p.symbol != symbol);
                }
            }
        }

        let order = Order {
            id: Uuid::new_v4(),
            symbol: symbol.to_string(),
            side,
            lot,
            price,
            status: OrderStatus::Filled,
            created_at: now,
            filled_at: Some(now),
        };

        self.orders.lock().unwrap().push(order.clone());

        tracing::info!(
            symbol,
            side = ?side,
            lot,
            price = %price,
            balance = %*balance,
            "[SIM] Order filled"
        );

        Ok(order)
    }

    async fn cancel_order(
        &self,
        _order_id: &str,
    ) -> Result<(), DomainError> {
        Ok(())
    }

    async fn get_order(
        &self,
        order_id: &str,
    ) -> Result<Order, DomainError> {
        let id: Uuid = order_id.parse().map_err(|_| {
            DomainError::ExternalService("Invalid order ID".into())
        })?;

        self.orders
            .lock()
            .unwrap()
            .iter()
            .find(|o| o.id == id)
            .cloned()
            .ok_or_else(|| {
                DomainError::ExternalService("Order not found".into())
            })
    }

    async fn get_open_orders(&self) -> Result<Vec<Order>, DomainError> {
        Ok(self
            .orders
            .lock()
            .unwrap()
            .iter()
            .filter(|o| o.status == OrderStatus::Pending)
            .cloned()
            .collect())
    }

    async fn get_portfolio(&self) -> Result<Portfolio, DomainError> {
        Ok(Portfolio {
            balance: *self.balance.lock().unwrap(),
            positions: self.positions.lock().unwrap().clone(),
        })
    }
}
