use serde::Deserialize;

#[derive(Debug, Deserialize)]
pub struct TradingInfoSSResponse {
    #[serde(rename = "KodeEmiten")]
    pub kode_emiten: Option<String>,
    pub replies: Vec<TradingInfoSSRow>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct TradingInfoSSRow {
    pub date: Option<String>,
    pub stock_code: Option<String>,
    pub stock_name: Option<String>,
    pub previous: Option<f64>,
    pub open_price: Option<f64>,
    pub high: Option<f64>,
    pub low: Option<f64>,
    pub close: Option<f64>,
    pub volume: Option<f64>,
    pub value: Option<f64>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct TradingInfoDaily {
    pub security_code: Option<String>,
    pub previous_price: Option<f64>,
    pub opening_price: Option<f64>,
    pub highest_price: Option<f64>,
    pub lowest_price: Option<f64>,
    pub closing_price: Option<f64>,
    pub change: Option<f64>,
    pub traded_volume: Option<f64>,
    pub traded_value: Option<f64>,
}

#[derive(Debug, Deserialize)]
pub struct FinancialDataResponse {
    pub data: Vec<FinancialRatioRow>,
}

#[derive(Debug, Deserialize)]
pub struct FinancialRatioRow {
    pub code: Option<String>,
    #[serde(alias = "stockName")]
    pub stock_name: Option<String>,
    pub eps: Option<f64>,
    pub per: Option<f64>,
    #[serde(alias = "priceBV")]
    pub price_bv: Option<f64>,
    #[serde(alias = "deRatio")]
    pub de_ratio: Option<f64>,
    pub roe: Option<f64>,
}

#[derive(Debug, Deserialize)]
pub struct CompanyProfilesResponse {
    pub data: Vec<CompanyProfile>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct CompanyProfile {
    pub kode_emiten: Option<String>,
    pub nama_emiten: Option<String>,
}
