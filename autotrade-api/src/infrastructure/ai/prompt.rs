use chrono::Utc;
use rust_decimal::Decimal;
use std::str::FromStr;
use uuid::Uuid;

use crate::domain::entity::analysis::{StockAnalysis, StockSuggestion};
use crate::domain::entity::stock::{StockFundamental, StockPrice};
use crate::domain::error::DomainError;
use crate::domain::value::{PriceTarget, Signal, Trend};

pub const ANALYSIS_SYSTEM: &str = "\
You are an expert Indonesian stock market analyst. \
Analyze stocks comprehensively using: \
1) Technical analysis from price data, \
2) Fundamental analysis from financial ratios, \
3) Corporate governance — are executives trustworthy? Any scandals, lawsuits, or fraud? \
4) Recent news and events — regulatory actions, lawsuits, management changes, \
5) Industry risks and sector outlook. \
If the company has known fraud, governance issues, or ongoing legal problems, \
lower confidence and recommend caution (Hold/Sell). \
Respond ONLY in valid JSON with these fields: \
signal (StrongBuy|Buy|Hold|Sell|StrongSell), \
trend (Bullish|Bearish|Sideways), \
confidence (0-100), \
entry (price), take_profit (price), stop_loss (price), \
risk_factors (array of strings — fraud, governance, legal, market risks), \
reasoning (one paragraph in English covering technicals, fundamentals, and governance).";

pub const SUGGEST_SYSTEM: &str = "\
You are an Indonesian stock market analyst. \
Suggest IDX stocks likely to rise or fall. \
Respond ONLY as a JSON array of objects with: \
symbol, name, signal (StrongBuy|Buy|Hold|Sell|StrongSell), \
trend (Bullish|Bearish|Sideways), \
current_price, target_price, potential_return (percent), \
confidence (0-100), reason (one sentence).";

pub fn build_analysis_prompt(
    symbol: &str,
    prices: &[StockPrice],
    fund: &StockFundamental,
) -> String {
    let recent: Vec<String> = prices
        .iter()
        .rev()
        .take(30)
        .map(|p| {
            format!(
                "{}: O={} H={} L={} C={} V={}",
                p.date.format("%Y-%m-%d"),
                p.open, p.high, p.low, p.close, p.volume,
            )
        })
        .collect();

    format!(
        "Analyze {symbol} on IDX.\n\
         \nRecent prices (newest first):\n{prices}\n\
         \nFundamentals:\n\
         PE={pe} PB={pb} ROE={roe} DER={der} EPS={eps} DivYield={dy}\n\
         \nBased on your knowledge, also evaluate:\n\
         - Is this company involved in any fraud, scandals, or legal issues?\n\
         - Are the executives/management trustworthy and competent?\n\
         - Any recent news that could impact the stock?\n\
         - What are the key risk factors?\n\
         \nProvide your comprehensive analysis as JSON.",
        prices = recent.join("\n"),
        pe = fmt_opt(fund.pe_ratio),
        pb = fmt_opt(fund.pb_ratio),
        roe = fmt_opt(fund.roe),
        der = fmt_opt(fund.der),
        eps = fmt_opt(fund.eps),
        dy = fmt_opt(fund.dividend_yield),
    )
}

pub fn build_suggest_prompt(limit: usize) -> String {
    format!(
        "Suggest the top {limit} IDX stocks right now \
         (mix of potential risers and fallers). \
         Use current market knowledge. Respond as JSON array."
    )
}

pub fn parse_analysis(
    symbol: &str,
    raw: &str,
) -> Result<StockAnalysis, DomainError> {
    let json = extract_json(raw);
    let v: serde_json::Value = serde_json::from_str(json)
        .map_err(|e| DomainError::AnalysisFailed(format!("parse: {e}")))?;

    let risk_factors = v["risk_factors"]
        .as_array()
        .map(|arr| {
            arr.iter()
                .filter_map(|x| x.as_str().map(String::from))
                .collect()
        })
        .unwrap_or_default();

    Ok(StockAnalysis {
        id: Uuid::new_v4(),
        symbol: symbol.to_string(),
        signal: parse_signal(v["signal"].as_str().unwrap_or("Hold")),
        trend: parse_trend(v["trend"].as_str().unwrap_or("Sideways")),
        confidence: parse_dec(&v["confidence"]),
        price_target: PriceTarget {
            entry: parse_dec(&v["entry"]),
            take_profit: parse_dec(&v["take_profit"]),
            stop_loss: parse_dec(&v["stop_loss"]),
        },
        risk_factors,
        reasoning: v["reasoning"].as_str().unwrap_or("").to_string(),
        analyzed_at: Utc::now(),
    })
}

pub fn parse_suggestions(
    raw: &str,
) -> Result<Vec<StockSuggestion>, DomainError> {
    let json = extract_json(raw);
    let arr: Vec<serde_json::Value> = serde_json::from_str(json)
        .map_err(|e| DomainError::AnalysisFailed(format!("parse: {e}")))?;

    Ok(arr
        .iter()
        .map(|v| StockSuggestion {
            symbol: v["symbol"].as_str().unwrap_or("").to_string(),
            name: v["name"].as_str().unwrap_or("").to_string(),
            signal: parse_signal(v["signal"].as_str().unwrap_or("Hold")),
            trend: parse_trend(v["trend"].as_str().unwrap_or("Sideways")),
            current_price: parse_dec(&v["current_price"]),
            target_price: parse_dec(&v["target_price"]),
            potential_return: parse_dec(&v["potential_return"]),
            confidence: parse_dec(&v["confidence"]),
            reason: v["reason"].as_str().unwrap_or("").to_string(),
        })
        .collect())
}

fn extract_json(raw: &str) -> &str {
    let trimmed = raw.trim();
    let first_brace = trimmed.find('{');
    let first_bracket = trimmed.find('[');

    let start = match (first_brace, first_bracket) {
        (Some(b), Some(k)) if k < b => k,
        (Some(b), _) => b,
        (None, Some(k)) => k,
        _ => return trimmed,
    };

    let (open, close) = if trimmed.as_bytes()[start] == b'[' {
        ('[', ']')
    } else {
        ('{', '}')
    };

    if let Some(end) = trimmed.rfind(close) {
        return &trimmed[start..=end];
    }
    trimmed
}

fn parse_signal(s: &str) -> Signal {
    match s {
        "StrongBuy" => Signal::StrongBuy,
        "Buy" => Signal::Buy,
        "Sell" => Signal::Sell,
        "StrongSell" => Signal::StrongSell,
        _ => Signal::Hold,
    }
}

fn parse_trend(s: &str) -> Trend {
    match s {
        "Bullish" => Trend::Bullish,
        "Bearish" => Trend::Bearish,
        _ => Trend::Sideways,
    }
}

fn parse_dec(v: &serde_json::Value) -> Decimal {
    match v {
        serde_json::Value::Number(n) => {
            Decimal::from_str(&n.to_string()).unwrap_or(Decimal::ZERO)
        }
        serde_json::Value::String(s) => {
            Decimal::from_str(s).unwrap_or(Decimal::ZERO)
        }
        _ => Decimal::ZERO,
    }
}

fn fmt_opt(v: Option<Decimal>) -> String {
    v.map(|d| d.to_string()).unwrap_or_else(|| "N/A".into())
}
