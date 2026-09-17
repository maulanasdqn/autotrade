use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Position {
    pub symbol: String,
    pub lot: u32,
    pub avg_price: Decimal,
    pub current_price: Decimal,
}

impl Position {
    pub fn total_shares(&self) -> u32 {
        self.lot * 100
    }

    pub fn market_value(&self) -> Decimal {
        self.current_price * Decimal::from(self.total_shares())
    }

    pub fn unrealized_pnl(&self) -> Decimal {
        (self.current_price - self.avg_price) * Decimal::from(self.total_shares())
    }

    pub fn pnl_percent(&self) -> Decimal {
        if self.avg_price.is_zero() {
            return Decimal::ZERO;
        }
        ((self.current_price - self.avg_price) / self.avg_price)
            * Decimal::from(100)
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Portfolio {
    pub balance: Decimal,
    pub equity: Decimal,
    pub allocated: Decimal,
    pub positions: Vec<Position>,
}

impl Portfolio {
    pub fn total_equity(&self) -> Decimal {
        if !self.equity.is_zero() {
            return self.equity;
        }
        let positions_value: Decimal =
            self.positions.iter().map(|p| p.market_value()).sum();
        self.balance + positions_value + self.allocated
    }
}
