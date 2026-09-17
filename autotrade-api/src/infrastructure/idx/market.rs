use async_trait::async_trait;
use chrono::Utc;
use rust_decimal::Decimal;
use std::str::FromStr;

use crate::domain::entity::stock::{Stock, StockFundamental, StockPrice};
use crate::domain::error::DomainError;
use crate::domain::port::market::MarketDataPort;

use super::client::IdxClient;
use super::dto::{
    CompanyProfilesResponse, FinancialDataResponse, FinancialRatioRow,
    TradingInfoDaily, TradingInfoSSResponse,
};

fn dec(v: Option<f64>) -> Decimal {
    v.map(|f| Decimal::from_str(&f.to_string()).unwrap_or_default())
        .unwrap_or_default()
}

fn dec_opt(v: Option<f64>) -> Option<Decimal> {
    v.map(|f| Decimal::from_str(&f.to_string()).unwrap_or_default())
}

#[async_trait]
impl MarketDataPort for IdxClient {
    async fn get_stock(&self, symbol: &str) -> Result<Stock, DomainError> {
        let path = format!(
            "/primary/ListedCompany/GetTradingInfoDaily?code={}",
            symbol
        );
        let body = self.get(&path).await?;
        let info: TradingInfoDaily = serde_json::from_str(&body)
            .map_err(|e| DomainError::ExternalService(format!("parse daily: {e}")))?;

        Ok(Stock {
            symbol: symbol.to_string(),
            name: info
                .security_code
                .unwrap_or_else(|| symbol.to_string()),
            sector: String::new(),
            last_price: dec(info.closing_price),
            volume: info.traded_volume.unwrap_or(0.0) as u64,
            market_cap: Decimal::ZERO,
            updated_at: Utc::now(),
        })
    }

    async fn get_price_history(
        &self,
        symbol: &str,
        days: u32,
    ) -> Result<Vec<StockPrice>, DomainError> {
        let path = format!(
            "/primary/ListedCompany/GetTradingInfoSS?code={}&start=0&length={}",
            symbol, days
        );
        let body = self.get(&path).await?;
        let resp: TradingInfoSSResponse = serde_json::from_str(&body)
            .map_err(|e| DomainError::ExternalService(format!("parse history: {e}")))?;

        let prices = resp
            .replies
            .into_iter()
            .map(|row| {
                let date = row
                    .date
                    .as_deref()
                    .and_then(|s| {
                        chrono::NaiveDateTime::parse_from_str(s, "%Y-%m-%dT%H:%M:%S")
                            .ok()
                            .map(|d| d.and_utc())
                    })
                    .unwrap_or_else(Utc::now);

                StockPrice {
                    symbol: symbol.to_string(),
                    open: dec(row.open_price),
                    high: dec(row.high),
                    low: dec(row.low),
                    close: dec(row.close),
                    volume: row.volume.unwrap_or(0.0) as u64,
                    date,
                }
            })
            .collect();

        Ok(prices)
    }

    async fn get_fundamentals(
        &self,
        symbol: &str,
    ) -> Result<StockFundamental, DomainError> {
        let now = Utc::now();
        let (year, month) = find_latest_period(now);
        let path = format!(
            "/primary/DigitalStatistic/GetApiDataPaginated\
             ?urlName=LINK_FINANCIAL_DATA_RATIO\
             &periodYear={year}&periodMonth={month}\
             &periodType=monthly&isPrint=False\
             &cumulative=false&pageSize=1000",
        );
        let body = self.get(&path).await?;
        let resp: FinancialDataResponse = serde_json::from_str(&body)
            .map_err(|e| DomainError::ExternalService(format!("parse fundamental: {e}")))?;

        find_fundamental(symbol, resp.data)
    }

    async fn search_stocks(
        &self,
        query: &str,
    ) -> Result<Vec<Stock>, DomainError> {
        let path =
            "/primary/ListedCompany/GetCompanyProfiles?start=0&length=9999";
        let body = self.get(path).await?;
        let resp: CompanyProfilesResponse =
            serde_json::from_str(&body).map_err(|e| {
                DomainError::ExternalService(format!("parse profiles: {e}"))
            })?;

        let q = query.to_uppercase();
        let results = resp
            .data
            .into_iter()
            .filter(|c| {
                c.kode_emiten
                    .as_deref()
                    .is_some_and(|k| k.contains(&q))
                    || c.nama_emiten
                        .as_deref()
                        .is_some_and(|n| n.to_uppercase().contains(&q))
            })
            .take(20)
            .map(|c| Stock {
                symbol: c.kode_emiten.unwrap_or_default(),
                name: c.nama_emiten.unwrap_or_default(),
                sector: String::new(),
                last_price: Decimal::ZERO,
                volume: 0,
                market_cap: Decimal::ZERO,
                updated_at: Utc::now(),
            })
            .collect();

        Ok(results)
    }
}

fn find_latest_period(now: chrono::DateTime<Utc>) -> (i32, u32) {
    let y = now.format("%Y").to_string().parse::<i32>().unwrap();
    let m = now.format("%-m").to_string().parse::<u32>().unwrap();
    if m <= 3 { (y - 1, 12) } else { (y, m - 3) }
}

fn find_fundamental(
    symbol: &str,
    rows: Vec<FinancialRatioRow>,
) -> Result<StockFundamental, DomainError> {
    let row = rows
        .into_iter()
        .find(|r| {
            r.code
                .as_deref()
                .is_some_and(|c| c.eq_ignore_ascii_case(symbol))
        })
        .ok_or_else(|| DomainError::StockNotFound(symbol.to_string()))?;

    Ok(StockFundamental {
        symbol: symbol.to_string(),
        pe_ratio: dec_opt(row.per),
        pb_ratio: dec_opt(row.price_bv),
        roe: dec_opt(row.roe),
        der: dec_opt(row.de_ratio),
        eps: dec_opt(row.eps),
        dividend_yield: None,
    })
}
