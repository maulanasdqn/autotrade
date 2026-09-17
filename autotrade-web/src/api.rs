use gloo_net::http::Request;
use serde::de::DeserializeOwned;

use crate::dto::{
    AnalysisResponse, ApiResponse, AutoTradeFullResponse,
    PortfolioResponse, RuleResponse, RulesListResponse,
    SuggestionDto, TokenStatus,
};

const BASE: &str = "/api/v1";

async fn get_json<T: DeserializeOwned>(url: &str) -> Result<T, String> {
    let resp = Request::get(url)
        .send()
        .await
        .map_err(|e| e.to_string())?;
    resp.json::<T>().await.map_err(|e| e.to_string())
}

async fn post_empty<T: DeserializeOwned>(url: &str) -> Result<T, String> {
    let resp = Request::post(url)
        .header("Content-Type", "application/json")
        .body("{}")
        .map_err(|e| e.to_string())?
        .send()
        .await
        .map_err(|e| e.to_string())?;
    resp.json::<T>().await.map_err(|e| e.to_string())
}

async fn post_json<T: DeserializeOwned>(
    url: &str,
    body: &str,
) -> Result<T, String> {
    let resp = Request::post(url)
        .header("Content-Type", "application/json")
        .body(body)
        .map_err(|e| e.to_string())?
        .send()
        .await
        .map_err(|e| e.to_string())?;
    resp.json::<T>().await.map_err(|e| e.to_string())
}

async fn delete_req(url: &str) -> Result<(), String> {
    Request::delete(url)
        .send()
        .await
        .map_err(|e| e.to_string())?;
    Ok(())
}

pub async fn fetch_suggestions(
    limit: usize,
) -> Result<Vec<SuggestionDto>, String> {
    let url = format!("{BASE}/suggest?limit={limit}");
    let resp: ApiResponse<Vec<SuggestionDto>> = get_json(&url).await?;
    if resp.success {
        resp.data.ok_or_else(|| "no data".into())
    } else {
        Err(resp.error.unwrap_or_else(|| "unknown error".into()))
    }
}

pub async fn analyze_stock(
    symbol: &str,
) -> Result<AnalysisResponse, String> {
    let url = format!("{BASE}/analyze/{symbol}");
    let resp: ApiResponse<AnalysisResponse> = post_empty(&url).await?;
    if resp.success {
        resp.data.ok_or_else(|| "no data".into())
    } else {
        Err(resp.error.unwrap_or_else(|| "unknown error".into()))
    }
}

pub async fn fetch_portfolio() -> Result<PortfolioResponse, String> {
    get_json(&format!("{BASE}/portfolio")).await
}

pub async fn fetch_rules() -> Result<RulesListResponse, String> {
    get_json(&format!("{BASE}/rules")).await
}

pub async fn create_rule(
    symbol: &str,
    buy_below: f64,
    sell_above: f64,
    stop_loss: f64,
    max_lot: u32,
) -> Result<RuleResponse, String> {
    let body = serde_json::json!({
        "symbol": symbol,
        "buy_below": buy_below,
        "sell_above": sell_above,
        "stop_loss": stop_loss,
        "max_lot": max_lot,
    });
    post_json(&format!("{BASE}/rules"), &body.to_string()).await
}

pub async fn delete_rule(symbol: &str) -> Result<(), String> {
    delete_req(&format!("{BASE}/rules/{symbol}")).await
}

pub async fn fetch_token_status() -> Result<TokenStatus, String> {
    get_json(&format!("{BASE}/token/status")).await
}

pub async fn update_token(token: &str) -> Result<TokenStatus, String> {
    let body = serde_json::json!({ "token": token });
    let resp = Request::put(&format!("{BASE}/token"))
        .header("Content-Type", "application/json")
        .body(body.to_string())
        .map_err(|e| e.to_string())?
        .send()
        .await
        .map_err(|e| e.to_string())?;
    resp.json::<TokenStatus>().await.map_err(|e| e.to_string())
}

pub async fn trigger_autotrade() -> Result<AutoTradeFullResponse, String> {
    let resp: AutoTradeFullResponse =
        post_empty(&format!("{BASE}/autotrade")).await?;
    if resp.success {
        Ok(resp)
    } else {
        Err(resp.error.unwrap_or_else(|| "unknown error".into()))
    }
}
