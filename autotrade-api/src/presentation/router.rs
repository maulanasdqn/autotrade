use axum::routing::{delete, get, post, put};
use axum::Router;
use tower_http::cors::CorsLayer;
use tower_http::services::{ServeDir, ServeFile};

use super::handler::{analysis, health, portfolio, rules, token, trade};
use super::state::AppState;

pub fn build(state: AppState) -> Router {
    let api = Router::new()
        .route("/health", get(health::health))
        .route("/api/v1/analyze/{symbol}", post(analysis::analyze_stock))
        .route("/api/v1/suggest", get(analysis::suggest_stocks))
        .route("/api/v1/autotrade", post(trade::execute_autotrade))
        .route("/api/v1/rules", get(rules::list_rules))
        .route("/api/v1/rules", post(rules::create_rule))
        .route("/api/v1/rules/{symbol}", delete(rules::delete_rule))
        .route("/api/v1/portfolio", get(portfolio::get_portfolio))
        .route("/api/v1/token/status", get(token::get_token_status))
        .route("/api/v1/token", put(token::update_token))
        .with_state(state)
        .layer(CorsLayer::permissive());

    let web = ServeDir::new("autotrade-web/dist")
        .fallback(ServeFile::new("autotrade-web/dist/index.html"));

    api.fallback_service(web)
}
