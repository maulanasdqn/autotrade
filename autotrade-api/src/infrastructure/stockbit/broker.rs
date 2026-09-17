use async_trait::async_trait;
use chrono::Utc;
use rust_decimal::Decimal;
use serde::Deserialize;
use std::str::FromStr;
use uuid::Uuid;

use crate::domain::entity::order::Order;
use crate::domain::entity::portfolio::{Portfolio, Position};
use crate::domain::error::DomainError;
use crate::domain::port::broker::BrokerPort;
use crate::domain::value::{OrderSide, OrderStatus};

use super::client::StockbitClient;

#[derive(Debug, Deserialize)]
struct StockbitResponse<T> {
    data: Option<T>,
    error_type: Option<String>,
    message: Option<String>,
}

#[derive(Debug, Deserialize)]
struct SbPortfolio {
    #[serde(default)]
    summary: Option<SbSummary>,
    #[serde(default)]
    stocks: Option<Vec<SbStock>>,
}

#[derive(Debug, Deserialize)]
struct SbSummary {
    #[serde(default)]
    trading_balance: Option<f64>,
    #[serde(default)]
    total_equity: Option<f64>,
    #[serde(default)]
    cash_balance: Option<f64>,
}

#[derive(Debug, Deserialize)]
struct SbStock {
    #[serde(default)]
    symbol: Option<String>,
    #[serde(default)]
    code: Option<String>,
    #[serde(default)]
    lot: Option<f64>,
    #[serde(default)]
    avg_price: Option<f64>,
    #[serde(default)]
    last_price: Option<f64>,
    #[serde(default)]
    current_price: Option<f64>,
}

#[derive(Debug, Deserialize)]
struct SbOrderResponse {
    #[serde(default)]
    order_id: Option<String>,
    #[serde(default)]
    status: Option<String>,
    #[serde(default)]
    message: Option<String>,
}

#[derive(Debug, Deserialize)]
struct SbOrderDetail {
    #[serde(default)]
    order_id: Option<String>,
    #[serde(default)]
    symbol: Option<String>,
    #[serde(default)]
    code: Option<String>,
    #[serde(default)]
    side: Option<String>,
    #[serde(default)]
    order_side: Option<String>,
    #[serde(default)]
    price: Option<f64>,
    #[serde(default)]
    qty: Option<f64>,
    #[serde(default)]
    shares: Option<f64>,
    #[serde(default)]
    lot: Option<f64>,
    #[serde(default)]
    status: Option<String>,
    #[serde(default)]
    status_text: Option<String>,
}

impl StockbitClient {
    async fn auth_header(&self) -> String {
        let token = self.token.read().await;
        if token.starts_with("Bearer ") {
            token.clone()
        } else {
            format!("Bearer {}", &*token)
        }
    }

    fn map_sb_status(status: &str) -> OrderStatus {
        match status.to_lowercase().as_str() {
            "open" | "pending" | "active" => OrderStatus::Pending,
            "match" | "filled" | "done" => OrderStatus::Filled,
            "partial" | "partially_filled" => {
                OrderStatus::PartiallyFilled
            }
            "withdraw" | "cancelled" | "canceled" | "expired" => {
                OrderStatus::Cancelled
            }
            "reject" | "rejected" => OrderStatus::Rejected,
            _ => OrderStatus::Pending,
        }
    }

    fn parse_order(detail: &SbOrderDetail) -> Order {
        let symbol = detail
            .symbol
            .clone()
            .or_else(|| detail.code.clone())
            .unwrap_or_default();

        let side_str = detail
            .side
            .clone()
            .or_else(|| detail.order_side.clone())
            .unwrap_or_default();
        let side = if side_str.to_lowercase().contains("sell") {
            OrderSide::Sell
        } else {
            OrderSide::Buy
        };

        let shares = detail.shares.or(detail.qty).unwrap_or(0.0);
        let lot = detail.lot.unwrap_or(shares / 100.0) as u32;

        let price =
            Decimal::from_str(&detail.price.unwrap_or(0.0).to_string())
                .unwrap_or_default();

        let status_str = detail
            .status
            .clone()
            .or_else(|| detail.status_text.clone())
            .unwrap_or_default();
        let status = Self::map_sb_status(&status_str);

        let id = detail
            .order_id
            .as_deref()
            .and_then(|s| Uuid::from_str(s).ok())
            .unwrap_or_else(Uuid::new_v4);

        let now = Utc::now();
        Order {
            id,
            symbol,
            side,
            lot,
            price,
            status,
            created_at: now,
            filled_at: if status == OrderStatus::Filled {
                Some(now)
            } else {
                None
            },
        }
    }
}

#[async_trait]
impl BrokerPort for StockbitClient {
    async fn place_order(
        &self,
        symbol: &str,
        side: OrderSide,
        lot: u32,
        price: Decimal,
    ) -> Result<Order, DomainError> {
        let endpoint = match side {
            OrderSide::Buy => "buy",
            OrderSide::Sell => "sell",
        };
        let url = format!("{}/order/v2/{}", self.base_url, endpoint);
        let shares = lot * 100;

        let price_i64: i64 =
            price.to_string().parse::<f64>().unwrap_or(0.0) as i64;

        let ui_ref = format!(
            "W{}{}",
            chrono::Utc::now().timestamp_millis(),
            &Uuid::new_v4().to_string()[..8]
        );

        let body = serde_json::json!({
            "ui_ref": ui_ref,
            "symbol": symbol,
            "price": price_i64,
            "shares": shares,
            "board_type": "RG",
            "is_gtc": false,
            "time_in_force": "0",
            "platform_order_type": "PLATFORM_ORDER_TYPE_LIMIT_DAY"
        });

        tracing::info!(
            symbol,
            side = ?side,
            lot,
            price = %price,
            "[STOCKBIT] Placing {} order", endpoint
        );

        let auth = self.auth_header().await;
        let resp = self
            .http
            .post(&url)
            .header("Authorization", auth)
            .header("Content-Type", "application/json")
            .json(&body)
            .send()
            .await
            .map_err(|e| {
                DomainError::ExternalService(format!(
                    "Stockbit request failed: {e}"
                ))
            })?;

        let status_code = resp.status();
        let raw: serde_json::Value =
            resp.json().await.map_err(|e| {
                DomainError::ExternalService(format!(
                    "Failed to parse Stockbit response: {e}"
                ))
            })?;

        tracing::info!(
            status = %status_code,
            response = %raw,
            "[STOCKBIT] Order response"
        );

        if !status_code.is_success() {
            let msg =
                raw["message"].as_str().unwrap_or("Order failed");
            return Err(DomainError::ExternalService(format!(
                "Stockbit order failed ({}): {}",
                status_code, msg
            )));
        }

        let data = &raw["data"];
        let order_id = data["order_id"].as_str().unwrap_or("");

        let id = Uuid::from_str(order_id)
            .unwrap_or_else(|_| Uuid::new_v4());

        let now = Utc::now();
        Ok(Order {
            id,
            symbol: symbol.to_string(),
            side,
            lot,
            price,
            status: OrderStatus::Pending,
            created_at: now,
            filled_at: None,
        })
    }

    async fn cancel_order(
        &self,
        order_id: &str,
    ) -> Result<(), DomainError> {
        let url = format!("{}/order/v2/cancel", self.base_url);

        let body = serde_json::json!({
            "order_id": order_id
        });

        let auth = self.auth_header().await;
        let resp = self
            .http
            .post(&url)
            .header("Authorization", auth)
            .json(&body)
            .send()
            .await
            .map_err(|e| {
                DomainError::ExternalService(format!(
                    "Stockbit cancel failed: {e}"
                ))
            })?;

        let status_code = resp.status();
        if !status_code.is_success() {
            let raw: serde_json::Value =
                resp.json().await.unwrap_or_default();
            let msg = raw["message"]
                .as_str()
                .unwrap_or("Cancel failed");
            return Err(DomainError::ExternalService(format!(
                "Stockbit cancel failed ({}): {}",
                status_code, msg
            )));
        }

        tracing::info!(
            order_id,
            "[STOCKBIT] Order cancelled"
        );
        Ok(())
    }

    async fn get_order(
        &self,
        order_id: &str,
    ) -> Result<Order, DomainError> {
        let url = format!(
            "{}/order/v2/detail?order_id={}",
            self.base_url, order_id
        );

        let auth = self.auth_header().await;
        let resp = self
            .http
            .get(&url)
            .header("Authorization", auth)
            .send()
            .await
            .map_err(|e| {
                DomainError::ExternalService(format!(
                    "Stockbit request failed: {e}"
                ))
            })?;

        let status_code = resp.status();
        let raw: serde_json::Value =
            resp.json().await.map_err(|e| {
                DomainError::ExternalService(format!(
                    "Failed to parse response: {e}"
                ))
            })?;

        if !status_code.is_success() {
            let msg = raw["message"]
                .as_str()
                .unwrap_or("Failed to get order");
            return Err(DomainError::ExternalService(format!(
                "Stockbit order detail failed ({}): {}",
                status_code, msg
            )));
        }

        let detail: SbOrderDetail =
            serde_json::from_value(raw["data"].clone()).map_err(
                |e| {
                    DomainError::ExternalService(format!(
                        "Failed to parse order detail: {e}"
                    ))
                },
            )?;

        Ok(Self::parse_order(&detail))
    }

    async fn get_open_orders(
        &self,
    ) -> Result<Vec<Order>, DomainError> {
        let url = format!("{}/order/v2/list", self.base_url);

        let auth = self.auth_header().await;
        let resp = self
            .http
            .get(&url)
            .header("Authorization", auth)
            .send()
            .await
            .map_err(|e| {
                DomainError::ExternalService(format!(
                    "Stockbit request failed: {e}"
                ))
            })?;

        let status_code = resp.status();
        let raw: serde_json::Value =
            resp.json().await.map_err(|e| {
                DomainError::ExternalService(format!(
                    "Failed to parse response: {e}"
                ))
            })?;

        tracing::debug!(
            response = %raw,
            "[STOCKBIT] Order list raw response"
        );

        if !status_code.is_success() {
            let msg = raw["message"]
                .as_str()
                .unwrap_or("Failed to list orders");
            return Err(DomainError::ExternalService(format!(
                "Stockbit order list failed ({}): {}",
                status_code, msg
            )));
        }

        let orders_val = if raw["data"].is_array() {
            &raw["data"]
        } else if raw["data"]["orders"].is_array() {
            &raw["data"]["orders"]
        } else if raw["data"]["list"].is_array() {
            &raw["data"]["list"]
        } else {
            return Ok(Vec::new());
        };

        let details: Vec<SbOrderDetail> =
            serde_json::from_value(orders_val.clone())
                .unwrap_or_default();

        Ok(details
            .iter()
            .map(Self::parse_order)
            .filter(|o| o.status == OrderStatus::Pending)
            .collect())
    }

    async fn get_portfolio(
        &self,
    ) -> Result<Portfolio, DomainError> {
        let url = format!("{}/portfolio/v2/list", self.base_url);

        let auth = self.auth_header().await;
        let resp = self
            .http
            .get(&url)
            .header("Authorization", auth)
            .send()
            .await
            .map_err(|e| {
                DomainError::ExternalService(format!(
                    "Stockbit request failed: {e}"
                ))
            })?;

        let status_code = resp.status();
        let raw: serde_json::Value =
            resp.json().await.map_err(|e| {
                DomainError::ExternalService(format!(
                    "Failed to parse response: {e}"
                ))
            })?;

        tracing::info!(
            status = %status_code,
            "[STOCKBIT] Portfolio response received"
        );
        tracing::debug!(
            response = %raw,
            "[STOCKBIT] Portfolio raw response"
        );

        if !status_code.is_success() {
            let msg = raw["message"]
                .as_str()
                .unwrap_or("Failed to get portfolio");
            return Err(DomainError::ExternalService(format!(
                "Stockbit portfolio failed ({}): {}",
                status_code, msg
            )));
        }

        let data = &raw["data"];

        let balance = data["summary"]["trading"]["balance"]
            .as_f64()
            .or_else(|| data["summary"]["equity"].as_f64())
            .unwrap_or(0.0);

        let balance_dec =
            Decimal::from_str(&balance.to_string())
                .unwrap_or_default();

        let stocks_arr = if data["results"].is_array() {
            data["results"].as_array()
        } else if data["stocks"].is_array() {
            data["stocks"].as_array()
        } else {
            None
        };

        let positions = stocks_arr
            .map(|arr| {
                arr.iter()
                    .filter_map(|s| {
                        let symbol = s["symbol"]
                            .as_str()
                            .or_else(|| s["code"].as_str())?;

                        let lot = s["lot"]
                            .as_f64()
                            .or_else(|| {
                                s["shares"]
                                    .as_f64()
                                    .map(|sh| sh / 100.0)
                            })
                            .unwrap_or(0.0)
                            as u32;

                        if lot == 0 {
                            return None;
                        }

                        let avg = s["avg_price"]
                            .as_f64()
                            .or_else(|| {
                                s["average_price"].as_f64()
                            })
                            .unwrap_or(0.0);

                        let cur = s["last_price"]
                            .as_f64()
                            .or_else(|| {
                                s["current_price"].as_f64()
                            })
                            .or_else(|| s["close"].as_f64())
                            .unwrap_or(avg);

                        Some(Position {
                            symbol: symbol.to_string(),
                            lot,
                            avg_price: Decimal::from_str(
                                &avg.to_string(),
                            )
                            .unwrap_or_default(),
                            current_price: Decimal::from_str(
                                &cur.to_string(),
                            )
                            .unwrap_or_default(),
                        })
                    })
                    .collect()
            })
            .unwrap_or_default();

        Ok(Portfolio {
            balance: balance_dec,
            positions,
        })
    }
}
