use std::env;

#[derive(Debug, Clone)]
pub struct AppConfig {
    pub stockbit_token: String,
    pub ai_api_url: String,
    pub ai_api_key: String,
    pub ai_model: String,
    pub server_port: u16,
}

impl AppConfig {
    pub fn from_env() -> Self {
        Self {
            stockbit_token: env::var("STOCKBIT_TOKEN")
                .unwrap_or_default(),
            ai_api_url: env::var("AI_API_URL")
                .unwrap_or_else(|_| "https://api.deepseek.com".into()),
            ai_api_key: env::var("AI_API_KEY").unwrap_or_default(),
            ai_model: env::var("AI_MODEL")
                .unwrap_or_else(|_| "deepseek-chat".into()),
            server_port: env::var("PORT")
                .ok()
                .and_then(|p| p.parse().ok())
                .unwrap_or(3000),
        }
    }
}
