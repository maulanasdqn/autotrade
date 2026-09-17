use async_trait::async_trait;
use chrono::{DateTime, TimeZone, Utc};
use reqwest::Client;
use rust_decimal::Decimal;
use serde::Deserialize;
use std::str::FromStr;

use crate::domain::entity::stock::{Stock, StockFundamental, StockPrice};
use crate::domain::error::DomainError;
use crate::domain::port::market::MarketDataPort;

#[derive(Clone)]
pub struct YahooClient {
    http: Client,
}

#[derive(Debug, Deserialize)]
struct ChartResponse {
    chart: ChartResult,
}

#[derive(Debug, Deserialize)]
struct ChartResult {
    result: Option<Vec<ChartData>>,
    error: Option<serde_json::Value>,
}

#[derive(Debug, Deserialize)]
struct ChartData {
    meta: ChartMeta,
    #[serde(default)]
    timestamp: Vec<i64>,
    #[serde(default)]
    indicators: Indicators,
}

#[derive(Debug, Deserialize)]
struct ChartMeta {
    symbol: String,
    #[serde(default, rename = "regularMarketPrice")]
    regular_market_price: Option<f64>,
    #[serde(default, rename = "regularMarketVolume")]
    regular_market_volume: Option<u64>,
    #[serde(default, rename = "longName")]
    long_name: Option<String>,
    #[serde(default, rename = "shortName")]
    short_name: Option<String>,
    #[serde(default, rename = "trailingPegRatio")]
    trailing_peg_ratio: Option<f64>,
}

#[derive(Debug, Default, Deserialize)]
struct Indicators {
    #[serde(default)]
    quote: Vec<QuoteData>,
}

#[derive(Debug, Deserialize)]
struct QuoteData {
    #[serde(default)]
    open: Vec<Option<f64>>,
    #[serde(default)]
    high: Vec<Option<f64>>,
    #[serde(default)]
    low: Vec<Option<f64>>,
    #[serde(default)]
    close: Vec<Option<f64>>,
    #[serde(default)]
    volume: Vec<Option<u64>>,
}

fn to_jk(symbol: &str) -> String {
    let s = symbol.to_uppercase();
    if s.ends_with(".JK") {
        s
    } else {
        format!("{s}.JK")
    }
}

fn dec(v: f64) -> Decimal {
    Decimal::from_str(&v.to_string()).unwrap_or_default()
}

impl YahooClient {
    pub fn new() -> Self {
        Self {
            http: Client::builder()
                .user_agent("Mozilla/5.0 (Macintosh; Intel Mac OS X 10_15_7) AppleWebKit/537.36")
                .build()
                .unwrap(),
        }
    }

    async fn chart(
        &self,
        symbol: &str,
        range: &str,
        interval: &str,
    ) -> Result<ChartData, DomainError> {
        let yf_symbol = to_jk(symbol);
        let url = format!(
            "https://query1.finance.yahoo.com/v8/finance/chart/{yf_symbol}?range={range}&interval={interval}"
        );

        let resp = self
            .http
            .get(&url)
            .send()
            .await
            .map_err(|e| DomainError::ExternalService(format!("Yahoo request failed: {e}")))?;

        let body: ChartResponse = resp
            .json()
            .await
            .map_err(|e| DomainError::ExternalService(format!("Yahoo parse failed: {e}")))?;

        if let Some(err) = body.chart.error {
            return Err(DomainError::ExternalService(format!("Yahoo error: {err}")));
        }

        body.chart
            .result
            .and_then(|mut r| r.pop())
            .ok_or_else(|| {
                DomainError::StockNotFound(symbol.to_string())
            })
    }
}

#[async_trait]
impl MarketDataPort for YahooClient {
    async fn get_stock(&self, symbol: &str) -> Result<Stock, DomainError> {
        let data = self.chart(symbol, "1d", "1d").await?;

        let name = data
            .meta
            .long_name
            .or(data.meta.short_name)
            .unwrap_or_else(|| symbol.to_string());

        Ok(Stock {
            symbol: symbol.to_uppercase(),
            name,
            sector: String::new(),
            last_price: dec(data.meta.regular_market_price.unwrap_or(0.0)),
            volume: data.meta.regular_market_volume.unwrap_or(0),
            market_cap: Decimal::ZERO,
            updated_at: Utc::now(),
        })
    }

    async fn get_price_history(
        &self,
        symbol: &str,
        days: u32,
    ) -> Result<Vec<StockPrice>, DomainError> {
        let range = match days {
            0..=7 => "5d",
            8..=31 => "1mo",
            32..=93 => "3mo",
            94..=186 => "6mo",
            _ => "1y",
        };

        let data = self.chart(symbol, range, "1d").await?;

        let quotes = data.indicators.quote.into_iter().next().unwrap_or(QuoteData {
            open: vec![],
            high: vec![],
            low: vec![],
            close: vec![],
            volume: vec![],
        });

        let sym = symbol.to_uppercase();
        let prices: Vec<StockPrice> = data
            .timestamp
            .iter()
            .enumerate()
            .filter_map(|(i, &ts)| {
                let open = quotes.open.get(i).copied().flatten()?;
                let high = quotes.high.get(i).copied().flatten()?;
                let low = quotes.low.get(i).copied().flatten()?;
                let close = quotes.close.get(i).copied().flatten()?;
                let vol = quotes.volume.get(i).copied().flatten().unwrap_or(0);

                let date: DateTime<Utc> = Utc.timestamp_opt(ts, 0).single()?;

                Some(StockPrice {
                    symbol: sym.clone(),
                    open: dec(open),
                    high: dec(high),
                    low: dec(low),
                    close: dec(close),
                    volume: vol,
                    date,
                })
            })
            .collect();

        if prices.is_empty() {
            return Err(DomainError::ExternalService(format!(
                "No price data for {symbol}"
            )));
        }

        Ok(prices)
    }

    async fn get_fundamentals(
        &self,
        symbol: &str,
    ) -> Result<StockFundamental, DomainError> {
        Ok(StockFundamental {
            symbol: symbol.to_uppercase(),
            pe_ratio: None,
            pb_ratio: None,
            roe: None,
            der: None,
            eps: None,
            dividend_yield: None,
        })
    }

    async fn search_stocks(
        &self,
        _query: &str,
    ) -> Result<Vec<Stock>, DomainError> {
        Ok(vec![])
    }
}
