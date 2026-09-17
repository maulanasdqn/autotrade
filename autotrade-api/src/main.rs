mod application;
mod domain;
mod infrastructure;
mod presentation;

use std::sync::Arc;

use infrastructure::ai::AiService;
use infrastructure::config::AppConfig;
use infrastructure::repository::analysis::InMemoryAnalysisRepo;
use infrastructure::repository::trade_rule::InMemoryTradeRuleRepo;
use infrastructure::stockbit::client::StockbitClient;
use infrastructure::yahoo::client::YahooClient;

use application::analyze::AnalyzeStockUseCase;
use application::autotrade::AutoTradeUseCase;
use application::suggest::SuggestStocksUseCase;

use presentation::state::AppState;

#[tokio::main]
async fn main() {
    dotenvy::dotenv().ok();

    tracing_subscriber::fmt()
        .with_env_filter("info")
        .init();

    let config = AppConfig::from_env();

    if config.stockbit_token.is_empty() {
        tracing::warn!(
            "STOCKBIT_TOKEN not set — portfolio and trading will fail. \
             Get your token from Stockbit web DevTools → Network → \
             any carina.stockbit.com request → Authorization header."
        );
    }

    let market = Arc::new(YahooClient::new());

    let broker = Arc::new(StockbitClient::new(&config.stockbit_token));
    broker.spawn_refresh_task();

    let ai = Arc::new(AiService::new(
        &config.ai_api_url,
        &config.ai_api_key,
        &config.ai_model,
    ));
    let analysis_repo = Arc::new(InMemoryAnalysisRepo::new());
    let trade_rule_repo = Arc::new(InMemoryTradeRuleRepo::new());

    let token = broker.token.clone();

    let state = AppState {
        analyze: Arc::new(AnalyzeStockUseCase::new(
            market.clone(),
            ai.clone(),
            analysis_repo,
        )),
        suggest: Arc::new(SuggestStocksUseCase::new(ai)),
        autotrade: Arc::new(AutoTradeUseCase::new(
            market.clone(),
            broker.clone(),
            trade_rule_repo.clone(),
        )),
        rules: trade_rule_repo,
        broker,
        token,
    };

    let app = presentation::router::build(state);
    let addr = format!("0.0.0.0:{}", config.server_port);
    tracing::info!("API server listening on {addr}");

    let listener = tokio::net::TcpListener::bind(&addr).await.unwrap();
    axum::serve(listener, app).await.unwrap();
}
